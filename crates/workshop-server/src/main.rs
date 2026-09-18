#![cfg_attr(not(test), windows_subsystem = "windows")]

use axum::http::{header, Method};
use axum::{middleware as axum_middleware, routing::get, Router};
use std::net::SocketAddr;
use tower_http::cors::{AllowHeaders, CorsLayer};
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::prelude::*;

mod audit;
mod auth;
mod backup;
mod barcode;
mod certificate;
mod config;
mod crypto;
mod db_manager;
mod device_key;
mod error;
mod license;
mod middleware;
mod rate_limiter;
mod routes;
mod schema;
mod secrets;
#[cfg(target_os = "windows")]
mod splash;
mod state;
mod tls;
#[cfg(target_os = "windows")]
mod tray;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 0. Splash screen (only on Windows, non-test builds)
    #[cfg(all(target_os = "windows", not(test)))]
    let (ready_tx, ready_rx) = tokio::sync::oneshot::channel();
    #[cfg(all(target_os = "windows", not(test)))]
    let _splash_handle = std::thread::spawn(move || splash::show(ready_rx));

    // 0. Init crypto provider for rustls before anything else uses TLS
    tls::init_crypto_provider();

    // 0.1. Compute data_dir early (needed for log file)
    let data_dir = dirs::data_local_dir()
        .unwrap_or_else(|| {
            std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."))
        })
        .join("WorkshopManager")
        .join("data");
    std::fs::create_dir_all(&data_dir)?;

    // 0.2. Init tracing with file appender
    let log_dir = data_dir.join("logs");
    std::fs::create_dir_all(&log_dir)?;
    let file_appender = tracing_appender::rolling::daily(&log_dir, "server.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    let stdout_layer = tracing_subscriber::fmt::layer()
        .with_target(false)
        .with_thread_ids(true)
        .with_writer(std::io::stdout);

    let file_layer = tracing_subscriber::fmt::layer()
        .with_target(false)
        .with_thread_ids(true)
        .with_ansi(false)
        .with_writer(non_blocking);

    tracing_subscriber::registry()
        .with(stdout_layer)
        .with(file_layer)
        .init();

    tracing::info!("Starting WorkshopManager Server...");
    tracing::info!("Log file: {}", log_dir.join("server.log").display());

    // 1. Load config
    let config = config::load_config(&data_dir)?;
    tracing::info!("Config loaded");

    // 1.1. Configure IVA rate from config
    workshop_common::money::set_iva_rate(
        rust_decimal::Decimal::try_from(config.iva_rate)
            .unwrap_or(rust_decimal::Decimal::new(19, 2)),
    );
    tracing::info!("IVA rate configured: {}", config.iva_rate);

    // 2. Init secrets (data_dir already computed above)
    let secrets = secrets::init_secrets(&data_dir)?;
    tracing::info!("Secrets initialized");

    // 3. Init crypto
    crypto::init(&data_dir)?;
    tracing::info!("Crypto initialized");

    // 4. License validation
    let license = match license::load_license(&data_dir) {
        license::LicenseStatus::Valid(lic) => {
            if let Err(e) = license::validate_license(&lic) {
                tracing::error!("Licencia inválida: {}", e);
                tracing::info!("El servidor funcionará en modo trial por 7 días");
                None
            } else {
                tracing::info!("Licencia válida: {} ({})", lic.license_key, lic.tier);
                Some(lic)
            }
        }
        license::LicenseStatus::FirstRun(hw_hash) => {
            tracing::info!("Primer uso — Hardware ID: {}", hw_hash);
            tracing::info!("Código de activación: {}", &hw_hash[..16]);
            tracing::info!("Contacte al proveedor para activar la licencia");

            // Intentar validación online
            match license::validate_online("trial", &hw_hash).await {
                Some(lic) => {
                    tracing::info!("Licencia trial activada online");
                    let _ = license::save_license(&lic, &data_dir);
                    Some(lic)
                }
                None => {
                    tracing::info!("Sin conexión — funcionando en modo trial (7 días)");
                    Some(workshop_common::license::create_trial_license(&hw_hash))
                }
            }
        }
        license::LicenseStatus::Invalid(e) => {
            tracing::error!("Error de licencia: {}", e);
            tracing::info!("El servidor funcionará en modo trial por 7 días");
            None
        }
    };

    // 5. Start embedded PostgreSQL
    let mut db_manager = db_manager::DbManager::new(&data_dir)?;
    let database_url = db_manager.start().await.map_err(|e| {
        tracing::error!(error = %e, "Failed to start embedded PostgreSQL");
        e
    })?;
    tracing::info!("PostgreSQL embedded started");

    // 6. Create connection pool and run migrations
    let pool = db_manager::create_pool(&database_url).await?;
    schema::run_migrations(&pool).await?;
    tracing::info!("Database pool and migrations ready");

    // 7. Start automatic backup scheduler
    let pg_dump_path = db_manager.binary_dir().join("pg_dump");
    let backups_dir = data_dir.join("backups");
    tokio::spawn(backup_scheduler(pg_dump_path, database_url, backups_dir));

    // 8. Create AppState
    let state = state::AppState::new(secrets, config, pool, license.clone());

    // 9. Build router — CORS origins from config or auto-detect
    let cors_origins: Vec<axum::http::HeaderValue> = if state.config.cors_origins.is_empty() {
        // Default: localhost + auto-detect LAN IP
        let mut origins = vec![
            "http://localhost".parse().expect("valid"),
            "https://localhost".parse().expect("valid"),
            "http://127.0.0.1".parse().expect("valid"),
            "https://127.0.0.1".parse().expect("valid"),
        ];
        // Auto-detect LAN IP and add it
        if let Ok(ip) = local_ip_address::local_ip() {
            let ip_str = ip.to_string();
            if let Ok(origin) = format!("https://{}", ip_str).parse() {
                tracing::info!("CORS: adding auto-detected LAN origin https://{}", ip_str);
                origins.push(origin);
            }
        }
        origins
    } else {
        state
            .config
            .cors_origins
            .iter()
            .filter_map(|s| s.parse().ok())
            .collect()
    };

    let protected_api = routes::protected_routes()
        .route_layer(axum_middleware::from_fn_with_state(
            state.clone(),
            middleware::authenticate_middleware,
        ))
        .route_layer(axum_middleware::from_fn_with_state(
            state.clone(),
            device_key::require_device_key,
        ))
        .route_layer(axum_middleware::from_fn_with_state(
            state.clone(),
            middleware::api_key_middleware,
        ));

    let admin_api = routes::admin_routes()
        .route_layer(axum_middleware::from_fn(
            middleware::require_admin_middleware,
        ))
        .route_layer(axum_middleware::from_fn_with_state(
            state.clone(),
            middleware::authenticate_middleware,
        ))
        .route_layer(axum_middleware::from_fn_with_state(
            state.clone(),
            device_key::require_device_key,
        ))
        .route_layer(axum_middleware::from_fn_with_state(
            state.clone(),
            middleware::api_key_middleware,
        ));

    let api = Router::new()
        .merge(routes::public_routes())
        .merge(protected_api)
        .merge(admin_api);

    let app = Router::new()
        .route("/health", get(health))
        .nest("/api", api)
        .layer(RequestBodyLimitLayer::new(10 * 1024 * 1024)) // 10MB max body
        .layer(
            CorsLayer::new()
                .allow_origin(cors_origins)
                .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
                .allow_headers(AllowHeaders::list([
                    header::AUTHORIZATION,
                    header::CONTENT_TYPE,
                ])),
        )
        .layer(SetResponseHeaderLayer::overriding(
            header::STRICT_TRANSPORT_SECURITY,
            axum::http::HeaderValue::from_static("max-age=31536000; includeSubDomains"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::X_CONTENT_TYPE_OPTIONS,
            axum::http::HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::X_FRAME_OPTIONS,
            axum::http::HeaderValue::from_static("DENY"),
        ))
        .layer(TraceLayer::new_for_http())
        .with_state(state.clone());

    // 10. Start HTTPS server — include bind address as TLS SAN
    let host: std::net::IpAddr = state.config.host.parse()?;
    let addr = SocketAddr::from((host, state.config.port));
    let san = if host.is_unspecified() {
        // If binding 0.0.0.0, auto-detect LAN IP for the cert SAN
        local_ip_address::local_ip()
            .map(|ip| ip.to_string())
            .unwrap_or_default()
    } else {
        host.to_string()
    };
    let extra_sans = if san.is_empty() { vec![] } else { vec![san] };
    let (certs, key) = tls::load_or_generate_tls_config(&data_dir, &extra_sans)?;
    let rustls_config = tls::create_axum_rustls_config(certs, key)?;

    tracing::info!("Server listening on https://{}", addr);

    // Close splash screen — server is ready
    #[cfg(all(target_os = "windows", not(test)))]
    {
        let _ = ready_tx.send(Ok(format!("https://{}", addr)));
    }

    let handle = axum_server::Handle::new();
    let server_task = tokio::spawn(
        axum_server::bind_rustls(addr, rustls_config)
            .handle(handle.clone())
            .serve(app.into_make_service()),
    );

    #[cfg(target_os = "windows")]
    {
        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
        let tray_pool = state.pool.clone();
        let tray_config = state.config.clone();
        let tray_license = license
            .as_ref()
            .map(|l| format!("{} ({})", l.license_key, l.tier));
        std::thread::spawn(move || tray::run(shutdown_tx, tray_pool, tray_config, tray_license));
        let ctrl_c = tokio::signal::ctrl_c();
        tokio::select! {
            _ = shutdown_rx => {}
            _ = ctrl_c => {}
        }
    }

    #[cfg(not(target_os = "windows"))]
    tokio::signal::ctrl_c().await?;

    tracing::info!("Shutdown requested");
    handle.graceful_shutdown(Some(std::time::Duration::from_secs(10)));
    server_task.await??;

    tracing::info!("Stopping embedded PostgreSQL...");
    if let Err(e) = db_manager.stop().await {
        tracing::error!(error = %e, "Failed to stop PostgreSQL gracefully");
    }
    tracing::info!("Shutdown complete");

    Ok(())
}

async fn health() -> &'static str {
    "OK"
}

async fn backup_scheduler(
    pg_dump_path: std::path::PathBuf,
    database_url: String,
    backups_dir: std::path::PathBuf,
) {
    const BACKUP_INTERVAL_HOURS: u64 = 24;
    const KEEP_BACKUP_COUNT: usize = 7;

    // First backup immediately at boot
    if let Err(e) = backup::create_backup(&pg_dump_path, &database_url, &backups_dir).await {
        tracing::error!(error = %e, "Initial backup failed");
    }
    if let Err(e) = backup::prune_old_backups(&backups_dir, KEEP_BACKUP_COUNT) {
        tracing::error!(error = %e, "Backup pruning failed");
    }

    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(
            BACKUP_INTERVAL_HOURS * 3600,
        ))
        .await;

        if let Err(e) = backup::create_backup(&pg_dump_path, &database_url, &backups_dir).await {
            tracing::error!(error = %e, "Backup failed");
            continue;
        }

        if let Err(e) = backup::prune_old_backups(&backups_dir, KEEP_BACKUP_COUNT) {
            tracing::error!(error = %e, "Backup pruning failed");
        }
    }
}

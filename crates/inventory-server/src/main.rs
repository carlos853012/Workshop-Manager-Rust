use axum::{middleware as axum_middleware, routing::get, Router};
use std::net::SocketAddr;
use tokio::sync::oneshot;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

mod audit;
mod auth;
mod backup;
mod barcode;
mod config;
mod crypto;
mod db_manager;
mod device_key;
mod error;
mod middleware;
mod rate_limiter;
mod routes;
mod schema;
mod secrets;
mod state;
mod tls;
#[cfg(target_os = "windows")]
mod tray;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 0. Init crypto provider for rustls before anything else uses TLS
    tls::init_crypto_provider();

    // 1. Init tracing
    tracing_subscriber::fmt()
        .with_target(false)
        .with_thread_ids(true)
        .init();

    tracing::info!("Starting WorkshopManager Server...");

    // 2. Load config
    let config = config::load_config()?;
    tracing::info!("Config loaded");

    // 3. Init secrets
    let data_dir = dirs::data_local_dir()
        .unwrap_or_else(|| std::env::current_dir().unwrap())
        .join("WorkshopManager")
        .join("data");
    std::fs::create_dir_all(&data_dir)?;
    let secrets = secrets::init_secrets(&data_dir)?;
    tracing::info!("Secrets initialized");

    // 4. Init crypto
    crypto::init(&data_dir)?;
    tracing::info!("Crypto initialized");

    // 5. Start embedded PostgreSQL
    let mut db_manager = db_manager::DbManager::new(&data_dir)?;
    let database_url = db_manager.start().await?;
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
    let state = state::AppState::new(secrets, config, pool);

    // 9. Build router
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
        .route_layer(axum_middleware::from_fn_with_state(
            state.clone(),
            middleware::authenticate_middleware,
        ))
        .route_layer(axum_middleware::from_fn_with_state(
            state.clone(),
            device_key::require_device_key,
        ))
        .route_layer(axum_middleware::from_fn(
            middleware::require_admin_middleware,
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
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state.clone());

    // 10. Start HTTPS server
    let host: std::net::IpAddr = state.config.host.parse()?;
    let addr = SocketAddr::from((host, state.config.port));
    let (certs, key) = tls::load_or_generate_tls_config(&data_dir)?;
    let rustls_config = tls::create_axum_rustls_config(certs, key)?;

    tracing::info!("Server listening on https://{}", addr);

    let handle = axum_server::Handle::new();
    let server_task = tokio::spawn(
        axum_server::bind_rustls(addr, rustls_config)
            .handle(handle.clone())
            .serve(app.into_make_service()),
    );

    #[cfg(target_os = "windows")]
    {
        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        let tray_pool = state.pool.clone();
        std::thread::spawn(move || tray::run(shutdown_tx, tray_pool));
        shutdown_rx
            .await
            .map_err(|_| anyhow::anyhow!("Tray shutdown signal lost"))?;
    }

    #[cfg(not(target_os = "windows"))]
    tokio::signal::ctrl_c().await?;

    tracing::info!("Shutdown requested");
    handle.graceful_shutdown(Some(std::time::Duration::from_secs(10)));
    server_task.await??;

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

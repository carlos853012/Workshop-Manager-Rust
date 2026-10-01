#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

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
mod tray;
mod validation;

/// En release (MSI) el exe no tiene consola. Sin consola, cada proceso hijo
/// (initdb, pg_ctl, postgres) abre su propia ventana negra. Creamos una consola
/// oculta para que los hijos la hereden.
#[cfg(all(windows, not(debug_assertions)))]
fn hide_console_for_children() {
    use windows_sys::Win32::System::Console::{AllocConsole, GetConsoleWindow};
    use windows_sys::Win32::UI::WindowsAndMessaging::{ShowWindow, SW_HIDE};
    unsafe {
        if GetConsoleWindow().is_null() {
            AllocConsole();
        }
        let h = GetConsoleWindow();
        if !h.is_null() {
            ShowWindow(h, SW_HIDE);
        }
    }
}

/// En release no hay consola: mostrar el error de licencia en un diálogo
/// modal (Windows) o por stderr (otros SO) antes de salir (H15).
fn fatal_license_dialog(title: &str, msg: &str) {
    #[cfg(windows)]
    unsafe {
        use windows_sys::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR, MB_OK};
        let title_w: Vec<u16> = title.encode_utf16().chain(std::iter::once(0)).collect();
        let msg_w: Vec<u16> = msg.encode_utf16().chain(std::iter::once(0)).collect();
        MessageBoxW(
            std::ptr::null_mut(),
            msg_w.as_ptr(),
            title_w.as_ptr(),
            MB_OK | MB_ICONERROR,
        );
    }
    #[cfg(not(windows))]
    {
        eprintln!("{} — {}", title, msg);
    }
}

/// El Worker rechazó la licencia durante la revalidación oportunista:
/// registrar, mostrar diálogo visible (H15) y apagar el servidor.
fn revalidation_fatal(handle: &axum_server::Handle, title: &str, detail: &str) -> ! {
    tracing::error!("═══════════════════════════════════════════════════════");
    tracing::error!("  LICENCIA RECHAZADA POR EL SERVIDOR DE LICENCIAS");
    tracing::error!("  {}", detail);
    tracing::error!("  El servidor se apagará automáticamente");
    tracing::error!("═══════════════════════════════════════════════════════");
    handle.graceful_shutdown(Some(std::time::Duration::from_secs(5)));
    fatal_license_dialog(
        &format!("WorkshopManager — {}", title),
        &format!(
            "{}\n\nEl servidor se apagará.\n\nContacte al proveedor para obtener una nueva licencia.",
            detail
        ),
    );
    std::process::exit(1);
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    #[cfg(all(windows, not(debug_assertions)))]
    hide_console_for_children();

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
    if license::is_placeholder_key() {
        tracing::error!("═══════════════════════════════════════════════════════");
        tracing::error!("  Clave pública del vendor no configurada");
        tracing::error!("  Ejecuta: license-tool generate-keypair");
        tracing::error!("═══════════════════════════════════════════════════════");
        #[cfg(all(target_os = "windows", not(test)))]
        {
            let _ = ready_tx.send(Err("Clave pública del vendor no configurada".to_string()));
            std::thread::sleep(std::time::Duration::from_millis(250));
        }
        fatal_license_dialog(
            "WorkshopManager — Error de licencia",
            "Clave pública del vendor no configurada.\n\nEjecuta: license-tool generate-keypair",
        );
        std::process::exit(1);
    }
    let license = match license::load_license(&data_dir) {
        license::LicenseStatus::Valid(lic) => {
            if let Err(e) = license::validate_license(&lic) {
                tracing::error!("═══════════════════════════════════════════════════════");
                tracing::error!("  LICENCIA EXPIRADA O INVÁLIDA");
                tracing::error!("  {}", e);
                tracing::error!("  Contacte al proveedor para obtener una nueva licencia");
                tracing::error!("═══════════════════════════════════════════════════════");
                #[cfg(all(target_os = "windows", not(test)))]
                {
                    let _ = ready_tx.send(Err(format!("Licencia expirada: {}", e)));
                    std::thread::sleep(std::time::Duration::from_millis(250));
                }
                fatal_license_dialog(
                    "WorkshopManager — Licencia expirada",
                    &format!(
                        "La licencia ha expirado o es inválida.\n\n{}\n\nContacte al proveedor para obtener una nueva licencia.",
                        e
                    ),
                );
                std::process::exit(1);
            } else {
                tracing::info!("Licencia válida: {} ({})", lic.license_key, lic.tier);
                Some(lic)
            }
        }
        license::LicenseStatus::FirstRun(hw_hash) => {
            tracing::info!("Activando licencia de prueba...");
            if hw_hash.len() >= 16 {
                tracing::info!("Código de activación: {}", &hw_hash[..16]);
            }

            match license::validate_online(&config.license_api_url, "trial", &hw_hash).await {
                license::OnlineResult::Ok {
                    signed,
                    license: lic,
                } => {
                    tracing::info!("Licencia trial activada online");
                    if signed.is_empty() {
                        tracing::warn!("Worker no envió signed_license — licencia no persistida");
                    } else if let Err(e) = license::save_signed_license(&signed, &data_dir) {
                        tracing::error!("Error guardando licencia: {}", e);
                    }
                    Some(lic)
                }
                license::OnlineResult::Rejected(reason) => {
                    tracing::error!("═══════════════════════════════════════════════════════");
                    tracing::error!("  ACTIVACIÓN RECHAZADA: {}", reason);
                    tracing::error!("  No se puede iniciar sin una licencia válida");
                    tracing::error!("═══════════════════════════════════════════════════════");
                    #[cfg(all(target_os = "windows", not(test)))]
                    {
                        let _ = ready_tx.send(Err(format!("Activación rechazada: {}", reason)));
                        std::thread::sleep(std::time::Duration::from_millis(250));
                    }
                    fatal_license_dialog(
                        "WorkshopManager — Activación rechazada",
                        &format!(
                            "La activación de la licencia fue rechazada.\n\n{}\n\nNo se puede iniciar sin una licencia válida.",
                            reason
                        ),
                    );
                    std::process::exit(1);
                }
                license::OnlineResult::Unreachable(reason) => {
                    tracing::error!("═══════════════════════════════════════════════════════");
                    tracing::error!("  SIN CONEXIÓN: {}", reason);
                    tracing::error!("  Se requiere conexión a internet para activar");
                    tracing::error!("  la licencia por primera vez.");
                    tracing::error!("═══════════════════════════════════════════════════════");
                    #[cfg(all(target_os = "windows", not(test)))]
                    {
                        let _ = ready_tx.send(Err(format!("Sin conexión: {}", reason)));
                        std::thread::sleep(std::time::Duration::from_millis(250));
                    }
                    fatal_license_dialog(
                        "WorkshopManager — Sin conexión",
                        &format!(
                            "No se pudo contactar al servidor de licencias.\n\n{}\n\nSe requiere conexión a internet para activar la licencia por primera vez.",
                            reason
                        ),
                    );
                    std::process::exit(1);
                }
            }
        }
        license::LicenseStatus::Invalid(e) => {
            tracing::error!("═══════════════════════════════════════════════════════");
            tracing::error!("  ERROR DE LICENCIA");
            tracing::error!("  {}", e);
            tracing::error!("  Contacte al proveedor para obtener una nueva licencia");
            tracing::error!("═══════════════════════════════════════════════════════");
            #[cfg(all(target_os = "windows", not(test)))]
            {
                let _ = ready_tx.send(Err(format!("Error de licencia: {}", e)));
                std::thread::sleep(std::time::Duration::from_millis(250));
            }
            fatal_license_dialog(
                "WorkshopManager — Error de licencia",
                &format!(
                    "No se pudo cargar la licencia.\n\n{}\n\nContacte al proveedor para obtener una nueva licencia.",
                    e
                ),
            );
            std::process::exit(1);
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

    let protected_api = routes::protected_routes(state.clone())
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
        .layer(SetResponseHeaderLayer::overriding(
            axum::http::header::CONTENT_SECURITY_POLICY,
            axum::http::HeaderValue::from_static("default-src 'none'; frame-ancestors 'none'"),
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

    // 7b. License expiry watcher — shuts down server if license expires at runtime
    {
        let data_dir_clone = data_dir.clone();
        let handle_clone = handle.clone();
        // Si license.dat existía al arrancar y desaparece en ejecución, fue
        // borrado a mano: tratarlo como manipulación (5.4). Solo se ignora
        // FirstRun si nunca hubo archivo (compat con Workers sin blob).
        let had_license_file = data_dir.join("license.dat").exists();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(300)).await;
                match license::load_license(&data_dir_clone) {
                    license::LicenseStatus::Valid(lic) => {
                        if let Err(e) = license::validate_license(&lic) {
                            tracing::error!(
                                "═══════════════════════════════════════════════════════"
                            );
                            tracing::error!("  LICENCIA EXPIRADA (detectado durante ejecución)");
                            tracing::error!("  {}", e);
                            tracing::error!("  El servidor se apagará automáticamente");
                            tracing::error!(
                                "  Contacte al proveedor para obtener una nueva licencia"
                            );
                            tracing::error!(
                                "═══════════════════════════════════════════════════════"
                            );
                            handle_clone.graceful_shutdown(Some(std::time::Duration::from_secs(5)));
                            break;
                        }
                    }
                    license::LicenseStatus::Invalid(e) => {
                        tracing::error!("═══════════════════════════════════════════════════════");
                        tracing::error!("  LICENCIA INVÁLIDA (detectado durante ejecución)");
                        tracing::error!("  {}", e);
                        tracing::error!("  El servidor se apagará automáticamente");
                        tracing::error!("═══════════════════════════════════════════════════════");
                        handle_clone.graceful_shutdown(Some(std::time::Duration::from_secs(5)));
                        break;
                    }
                    license::LicenseStatus::FirstRun(_) => {
                        if had_license_file {
                            tracing::error!(
                                "═══════════════════════════════════════════════════════"
                            );
                            tracing::error!("  LICENCIA ELIMINADA (detectado durante ejecución)");
                            tracing::error!("  El archivo license.dat fue borrado a mano");
                            tracing::error!("  El servidor se apagará automáticamente");
                            tracing::error!(
                                "═══════════════════════════════════════════════════════"
                            );
                            handle_clone.graceful_shutdown(Some(std::time::Duration::from_secs(5)));
                            break;
                        }
                    }
                }
            }
        });
    }

    // 7c. Revalidación oportunista — verifica la licencia con el Worker
    // cada 24 h (primer intento a los 5 min). Diseño offline-first: si no
    // hay internet, se omite el intento silenciosamente y se reintenta en
    // el próximo ciclo; solo una respuesta explícita de revocación/
    // expiración/invalidación apaga el servidor.
    {
        let data_dir_clone = data_dir.clone();
        let handle_clone = handle.clone();
        let api_url = state.config.license_api_url.clone();
        let license_key = license.as_ref().map(|l| l.license_key.clone());
        tokio::spawn(async move {
            let Some(license_key) = license_key else {
                tracing::info!("Sin licencia cargada — revalidación oportunista inactiva");
                return;
            };
            const FIRST_DELAY_SECS: u64 = 300;
            const INTERVAL_SECS: u64 = 24 * 60 * 60;
            let mut first = true;
            loop {
                if first {
                    tokio::time::sleep(tokio::time::Duration::from_secs(FIRST_DELAY_SECS)).await;
                    first = false;
                } else {
                    tokio::time::sleep(tokio::time::Duration::from_secs(INTERVAL_SECS)).await;
                }

                let hw_hash = match license::get_current_hardware_hash() {
                    Ok(h) => h,
                    Err(e) => {
                        tracing::warn!("Revalidación omitida (hardware no disponible): {}", e);
                        continue;
                    }
                };

                match license::revalidate_online(&api_url, &license_key, &hw_hash).await {
                    license::RevalidateResult::Active {
                        signed: Some(signed),
                    } => match license::save_signed_license(&signed, &data_dir_clone) {
                        Ok(()) => {
                            tracing::info!("Licencia revalidada online (blob renovado)")
                        }
                        Err(e) => {
                            tracing::error!("Licencia revalidada, pero no se pudo guardar: {}", e)
                        }
                    },
                    license::RevalidateResult::Active { signed: None } => {
                        tracing::info!("Licencia revalidada online (vigente)");
                    }
                    license::RevalidateResult::Unreachable(reason) => {
                        // Offline-first: no hay castigo por estar sin internet.
                        tracing::debug!("Revalidación omitida (sin conexión): {}", reason);
                    }
                    license::RevalidateResult::Revoked => {
                        revalidation_fatal(
                            &handle_clone,
                            "Licencia revocada",
                            &format!("La licencia {} fue revocada desde el panel.", license_key),
                        );
                    }
                    license::RevalidateResult::Expired => {
                        revalidation_fatal(
                            &handle_clone,
                            "Licencia expirada",
                            &format!("La licencia {} expiró en el servidor.", license_key),
                        );
                    }
                    license::RevalidateResult::Invalid(reason) => {
                        let detail = match reason.as_str() {
                            "hardware_mismatch" => {
                                format!("La licencia {} no corresponde a este equipo.", license_key)
                            }
                            "not_found" => {
                                format!("La licencia {} no existe en el servidor.", license_key)
                            }
                            other => {
                                format!("La licencia {} es inválida ({}).", license_key, other)
                            }
                        };
                        revalidation_fatal(&handle_clone, "Licencia inválida", &detail);
                    }
                }
            }
        });
    }

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

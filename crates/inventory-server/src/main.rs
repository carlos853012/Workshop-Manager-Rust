use axum::{routing::get, Router};
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

mod state;
mod config;
mod secrets;
mod crypto;
mod auth;
mod error;
mod middleware;
mod rate_limiter;
mod audit;
mod schema;
mod routes;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
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

    // 5. Create AppState (sin PostgreSQL embebido por ahora)
    let state = state::AppState::new(secrets, config);

    // 6. Build router
    let app = Router::new()
        .route("/health", get(health))
        .nest("/api", routes::api_routes())
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    // 7. Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], 8443));
    tracing::info!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}

async fn health() -> &'static str {
    "OK"
}

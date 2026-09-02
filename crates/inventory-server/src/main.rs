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
mod db_manager;
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

    // 5. Start embedded PostgreSQL
    let mut db_manager = db_manager::DbManager::new(&data_dir)?;
    let database_url = db_manager.start().await?;
    tracing::info!("PostgreSQL embedded started");

    // 6. Create connection pool and run migrations
    let pool = db_manager::create_pool(&database_url).await?;
    schema::run_migrations(&pool).await?;
    tracing::info!("Database pool and migrations ready");

    // 7. Create AppState
    let state = state::AppState::new(secrets, config, pool);

    // 8. Build router
    let app = Router::new()
        .route("/health", get(health))
        .nest("/api", routes::api_routes())
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    // 9. Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], 8443));
    tracing::info!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}

async fn health() -> &'static str {
    "OK"
}

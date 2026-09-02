use axum::Router;

use crate::state::AppState;

mod auth;
mod products;
mod sales;
mod repairs;
mod suppliers;
mod analytics;
mod users;
mod reports;

/// Rutas públicas de /api (no requieren autenticación).
pub fn public_routes() -> Router<AppState> {
    Router::new().nest("/auth", auth::public_routes())
}

/// Rutas protegidas de /api (requieren JWT).
pub fn protected_routes() -> Router<AppState> {
    Router::new()
        .nest("/auth/status", auth::protected_routes())
        .nest("/products", products::routes())
        .nest("/sales", sales::routes())
        .nest("/repairs", repairs::routes())
        .nest("/suppliers", suppliers::routes())
        .nest("/analytics", analytics::routes())
        .nest("/reports", reports::routes())
}

/// Rutas de administración de /api (requieren JWT + rol admin).
pub fn admin_routes() -> Router<AppState> {
    Router::new().nest("/users", users::routes())
}



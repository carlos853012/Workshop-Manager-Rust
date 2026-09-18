use axum::Router;

use crate::state::AppState;

mod analytics;
mod audit;
mod auth;
mod device_keys;
pub(crate) mod pagination;
mod products;
mod repairs;
pub(crate) mod reports;
mod sales;
mod seed;
mod suppliers;
mod users;

/// Rutas públicas de /api (no requieren autenticación).
pub fn public_routes() -> Router<AppState> {
    Router::new().nest("/auth", auth::public_routes())
}

/// Rutas protegidas de /api (requieren JWT).
pub fn protected_routes() -> Router<AppState> {
    Router::new()
        .nest("/auth", auth::protected_routes())
        .nest("/products", products::routes())
        .nest("/sales", sales::routes())
        .nest("/repairs", repairs::routes())
        .nest("/suppliers", suppliers::routes())
        .nest("/analytics", analytics::routes())
        .nest("/reports", reports::routes())
}

/// Rutas de administración de /api (requieren JWT + rol admin).
pub fn admin_routes() -> Router<AppState> {
    Router::new()
        .nest("/users", users::routes())
        .nest("/device-keys", device_keys::routes())
        .nest("/audit", audit::routes())
        .nest("/seed", seed::routes())
}

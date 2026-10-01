use axum::{middleware as axum_middleware, Router};
use workshop_common::features::Feature;

use crate::middleware;
use crate::state::AppState;

mod analytics;
mod audit;
mod auth;
mod config;
mod device_keys;
pub(crate) mod pagination;
mod products;
mod repairs;
pub(crate) mod reports;
mod sales;
#[cfg(debug_assertions)]
mod seed;
mod suppliers;
mod users;

/// Rutas públicas de /api (no requieren autenticación).
pub fn public_routes() -> Router<AppState> {
    Router::new().nest("/auth", auth::public_routes())
}

/// Rutas protegidas de /api (requieren JWT).
///
/// El grupo `/reports` exige la feature `ClientHistory` (sentinel del tier
/// Reports, que desbloquea Reportes + Certificado de Servicios). Los demás
/// grupos son base y no se gatean por licencia.
pub fn protected_routes(state: AppState) -> Router<AppState> {
    let reports = reports::routes().route_layer(axum_middleware::from_fn_with_state(
        (state.clone(), Feature::ClientHistory),
        middleware::require_feature,
    ));

    Router::new()
        .nest("/auth", auth::protected_routes())
        .nest("/config", config::routes())
        .nest("/products", products::routes())
        .nest("/sales", sales::routes())
        .nest("/repairs", repairs::routes())
        .nest("/suppliers", suppliers::routes())
        .nest("/analytics", analytics::routes())
        .nest("/reports", reports)
}

/// Rutas de administración de /api (requieren JWT + rol admin).
pub fn admin_routes() -> Router<AppState> {
    let router = Router::new()
        .nest("/users", users::routes())
        .nest("/device-keys", device_keys::routes())
        .nest("/audit", audit::routes());

    #[cfg(debug_assertions)]
    let router = router.nest("/seed", seed::routes());

    router
}

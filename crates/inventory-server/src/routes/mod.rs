use axum::Router;
use axum::routing::get;

use crate::state::AppState;

mod auth;
mod products;
mod sales;
mod repairs;
mod suppliers;
mod analytics;
mod users;

pub fn api_routes() -> Router<AppState> {
    Router::new()
        .nest("/auth", auth::routes())
        .nest("/products", products::routes())
        .nest("/sales", sales::routes())
        .nest("/repairs", repairs::routes())
        .nest("/suppliers", suppliers::routes())
        .nest("/analytics", analytics::routes())
        .nest("/users", users::routes())
}

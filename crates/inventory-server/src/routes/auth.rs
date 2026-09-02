use axum::{Router, routing::get, Json};
use serde_json::json;

use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/login", get(login))
        .route("/register", get(register))
        .route("/status", get(status))
}

async fn login() -> Json<serde_json::Value> {
    // TODO: Implementar login
    Json(json!({
        "success": false,
        "error": "Not implemented"
    }))
}

async fn register() -> Json<serde_json::Value> {
    // TODO: Implementar register
    Json(json!({
        "success": false,
        "error": "Not implemented"
    }))
}

async fn status() -> Json<serde_json::Value> {
    // TODO: Implementar status
    Json(json!({
        "success": false,
        "error": "Not implemented"
    }))
}

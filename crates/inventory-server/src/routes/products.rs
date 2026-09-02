use axum::{Router, routing::get, Json};
use serde_json::json;

use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_products))
        .route("/:id", get(get_product))
}

async fn list_products() -> Json<serde_json::Value> {
    Json(json!({
        "success": true,
        "data": [],
        "total": 0
    }))
}

async fn get_product() -> Json<serde_json::Value> {
    Json(json!({
        "success": false,
        "error": "Not implemented"
    }))
}

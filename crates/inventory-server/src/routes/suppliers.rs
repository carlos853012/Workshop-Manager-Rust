use axum::{Router, routing::get, Json};
use serde_json::json;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new().route("/", get(list_suppliers))
}

async fn list_suppliers() -> Json<serde_json::Value> {
    Json(json!({"success": true, "data": [], "total": 0}))
}

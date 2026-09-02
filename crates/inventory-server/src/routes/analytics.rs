use axum::{Router, routing::get, Json};
use serde_json::json;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new().route("/dashboard", get(dashboard))
}

async fn dashboard() -> Json<serde_json::Value> {
    Json(json!({
        "success": true,
        "data": {
            "total_products": 0,
            "low_stock": 0,
            "total_sales": 0,
            "total_revenue": 0.0,
            "pending_repairs": 0,
            "completed_repairs": 0,
            "average_sale": 0.0
        }
    }))
}

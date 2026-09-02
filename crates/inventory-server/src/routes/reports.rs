use axum::{Router, extract::State, routing::get, Json};
use inventory_common::dto::ApiResponse;
use sqlx::Row;
use rust_decimal::Decimal;
use serde::Serialize;

use crate::error::AppError;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/clients", get(report_clients))
        .route("/history", get(report_history))
}

#[derive(Debug, Serialize)]
struct ClientReport {
    customer_email: String,
    customer_name: Option<String>,
    total_purchases: i64,
    total_spent: Decimal,
    last_purchase: Option<chrono::DateTime<chrono::Utc>>,
}

async fn report_clients(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<ClientReport>>>, AppError> {
    let rows = sqlx::query(
        "SELECT \
            customer_email, \
            MAX(customer_name) as customer_name, \
            COUNT(*) as total_purchases, \
            COALESCE(SUM(total), 0) as total_spent, \
            MAX(created_at) as last_purchase \
         FROM sales \
         WHERE customer_email IS NOT NULL AND status = 'completed' \
         GROUP BY customer_email \
         ORDER BY total_spent DESC"
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    let reports: Vec<ClientReport> = rows
        .into_iter()
        .map(|row| ClientReport {
            customer_email: row.try_get("customer_email").unwrap_or_default(),
            customer_name: row.try_get("customer_name").ok(),
            total_purchases: row.try_get("total_purchases").unwrap_or(0),
            total_spent: row.try_get("total_spent").unwrap_or(Decimal::ZERO),
            last_purchase: row.try_get("last_purchase").ok(),
        })
        .collect();

    Ok(Json(ApiResponse::success(reports)))
}

#[derive(Debug, Serialize)]
struct HistoryReport {
    id: uuid::Uuid,
    entity_type: String,
    entity_id: uuid::Uuid,
    action: String,
    old_values: Option<serde_json::Value>,
    new_values: Option<serde_json::Value>,
    reason: Option<String>,
    performed_by: Option<uuid::Uuid>,
    created_at: chrono::DateTime<chrono::Utc>,
}

async fn report_history(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<HistoryReport>>>, AppError> {
    let rows = sqlx::query(
        "SELECT \
            id, entity_type, entity_id, action, old_values, new_values, reason, performed_by, created_at \
         FROM audit_log \
         ORDER BY created_at DESC \
         LIMIT 500"
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    let reports: Result<Vec<HistoryReport>, AppError> = rows
        .into_iter()
        .map(|row| {
            Ok(HistoryReport {
                id: row.try_get("id").map_err(|e| AppError::Internal(format!("Database error: {}", e)))?,
                entity_type: row.try_get("entity_type").map_err(|e| AppError::Internal(format!("Database error: {}", e)))?,
                entity_id: row.try_get("entity_id").map_err(|e| AppError::Internal(format!("Database error: {}", e)))?,
                action: row.try_get("action").map_err(|e| AppError::Internal(format!("Database error: {}", e)))?,
                old_values: row.try_get("old_values").ok(),
                new_values: row.try_get("new_values").ok(),
                reason: row.try_get("reason").ok(),
                performed_by: row.try_get("performed_by").ok(),
                created_at: row.try_get("created_at").map_err(|e| AppError::Internal(format!("Database error: {}", e)))?,
            })
        })
        .collect();

    Ok(Json(ApiResponse::success(reports?)))
}

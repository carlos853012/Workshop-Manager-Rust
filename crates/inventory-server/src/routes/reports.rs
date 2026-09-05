use axum::{
    extract::{Query, State},
    routing::get,
    Extension, Json, Router,
};
use inventory_common::dto::ApiResponse;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::Row;

use crate::error::AppError;
use crate::middleware::AuthenticatedUser;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/clients", get(report_clients))
        .route("/client-history", get(client_history))
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
    Extension(user): Extension<AuthenticatedUser>,
) -> Result<Json<ApiResponse<Vec<ClientReport>>>, AppError> {
    let wid = user.workshop_id;

    let rows = sqlx::query(
        "SELECT \
            customer_email, \
            MAX(customer_name) as customer_name, \
            COUNT(*) as total_purchases, \
            COALESCE(SUM(total), 0) as total_spent, \
            MAX(created_at) as last_purchase \
         FROM sales \
         WHERE customer_email IS NOT NULL AND status = 'completed' AND workshop_id = $1 \
         GROUP BY customer_email \
         ORDER BY total_spent DESC",
    )
    .bind(wid)
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

#[derive(Debug, Deserialize)]
struct ClientHistoryQuery {
    email: String,
}

#[derive(Debug, Serialize)]
struct ClientSaleRecord {
    id: uuid::Uuid,
    total: Decimal,
    payment_method: String,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
struct ClientRepairRecord {
    id: uuid::Uuid,
    description: Option<String>,
    status: String,
    total: Option<Decimal>,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
struct ClientHistoryResponse {
    email: String,
    name: Option<String>,
    sales: Vec<ClientSaleRecord>,
    repairs: Vec<ClientRepairRecord>,
    total_spent: Decimal,
    total_repairs: i64,
}

async fn client_history(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Query(params): Query<ClientHistoryQuery>,
) -> Result<Json<ApiResponse<ClientHistoryResponse>>, AppError> {
    let wid = user.workshop_id;

    let sales_rows = sqlx::query(
        "SELECT id, total, payment_method, created_at \
         FROM sales \
         WHERE customer_email = $1 AND status = 'completed' AND workshop_id = $2 \
         ORDER BY created_at DESC",
    )
    .bind(&params.email)
    .bind(wid)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    let sales: Vec<ClientSaleRecord> = sales_rows
        .into_iter()
        .map(|row| ClientSaleRecord {
            id: row.try_get("id").unwrap_or_default(),
            total: row.try_get("total").unwrap_or(Decimal::ZERO),
            payment_method: row.try_get("payment_method").unwrap_or_default(),
            created_at: row.try_get("created_at").unwrap_or_default(),
        })
        .collect();

    let repairs_rows = sqlx::query(
        "SELECT id, description, status, total, created_at \
         FROM repairs \
         WHERE customer_email = $1 AND workshop_id = $2 \
         ORDER BY created_at DESC",
    )
    .bind(&params.email)
    .bind(wid)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    let repairs: Vec<ClientRepairRecord> = repairs_rows
        .into_iter()
        .map(|row| ClientRepairRecord {
            id: row.try_get("id").unwrap_or_default(),
            description: row.try_get("description").ok(),
            status: row.try_get("status").unwrap_or_default(),
            total: row.try_get("total").ok(),
            created_at: row.try_get("created_at").unwrap_or_default(),
        })
        .collect();

    let total_spent: Decimal = sales.iter().map(|s| s.total).sum();
    let total_repairs = repairs.len() as i64;

    let name: Option<String> = sqlx::query_scalar(
        "SELECT MAX(customer_name) FROM sales WHERE customer_email = $1 AND workshop_id = $2",
    )
    .bind(&params.email)
    .bind(wid)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?
    .flatten();

    Ok(Json(ApiResponse::success(ClientHistoryResponse {
        email: params.email,
        name,
        sales,
        repairs,
        total_spent,
        total_repairs,
    })))
}

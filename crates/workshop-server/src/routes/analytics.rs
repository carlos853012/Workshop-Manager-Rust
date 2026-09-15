use axum::{extract::State, routing::get, Extension, Json, Router};
use rust_decimal::Decimal;
use workshop_common::dto::{ApiResponse, DashboardResponse};

use crate::error::AppError;
use crate::middleware::AuthenticatedUser;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/dashboard", get(dashboard))
        .route("/kpis", get(kpis))
}

async fn dashboard(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
) -> Result<Json<ApiResponse<DashboardResponse>>, AppError> {
    let wid = user.workshop_id;

    let total_products: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM products WHERE status = 'active' AND workshop_id = $1",
    )
    .bind(wid)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    let low_stock: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM products WHERE status = 'active' AND stock <= min_stock AND workshop_id = $1",
    )
    .bind(wid)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    let total_sales: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sales WHERE status = 'completed' AND workshop_id = $1",
    )
    .bind(wid)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    let total_revenue: Decimal = sqlx::query_scalar(
        "SELECT COALESCE(SUM(total), 0) FROM sales WHERE status = 'completed' AND workshop_id = $1",
    )
    .bind(wid)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    let pending_repairs: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM repairs WHERE status = 'pending' AND workshop_id = $1",
    )
    .bind(wid)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    let completed_repairs: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM repairs WHERE status = 'completed' AND workshop_id = $1",
    )
    .bind(wid)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    let average_sale: Decimal = if total_sales > 0 {
        total_revenue / Decimal::from(total_sales)
    } else {
        Decimal::ZERO
    };

    let response = DashboardResponse {
        total_products,
        low_stock,
        total_sales,
        total_revenue,
        pending_repairs,
        completed_repairs,
        average_sale,
    };

    Ok(Json(ApiResponse::success(response)))
}

#[derive(Debug, serde::Serialize)]
struct KpisResponse {
    total_suppliers: i64,
    total_customers: i64,
    in_progress_repairs: i64,
    cancelled_repairs: i64,
}

async fn kpis(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
) -> Result<Json<ApiResponse<KpisResponse>>, AppError> {
    let wid = user.workshop_id;

    let total_suppliers: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM suppliers WHERE status = 'active' AND workshop_id = $1",
    )
    .bind(wid)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    let total_customers: i64 = sqlx::query_scalar(
        "SELECT COUNT(DISTINCT customer_email) FROM sales WHERE customer_email IS NOT NULL AND status = 'completed' AND workshop_id = $1"
    )
    .bind(wid)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    let in_progress_repairs: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM repairs WHERE status = 'in_progress' AND workshop_id = $1",
    )
    .bind(wid)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    let cancelled_repairs: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM repairs WHERE status = 'cancelled' AND workshop_id = $1",
    )
    .bind(wid)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    Ok(Json(ApiResponse::success(KpisResponse {
        total_suppliers,
        total_customers,
        in_progress_repairs,
        cancelled_repairs,
    })))
}

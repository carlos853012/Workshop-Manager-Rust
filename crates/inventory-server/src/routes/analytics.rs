use axum::{extract::State, routing::get, Json, Router};
use inventory_common::dto::{ApiResponse, DashboardResponse};
use rust_decimal::Decimal;

use crate::error::AppError;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/dashboard", get(dashboard))
        .route("/kpis", get(kpis))
}

async fn dashboard(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<DashboardResponse>>, AppError> {
    let total_products: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM products WHERE status = 'active'")
            .fetch_one(&state.pool)
            .await
            .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    let low_stock: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM products WHERE status = 'active' AND stock <= min_stock",
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    let total_sales: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM sales WHERE status = 'completed'")
            .fetch_one(&state.pool)
            .await
            .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    let total_revenue: Decimal =
        sqlx::query_scalar("SELECT COALESCE(SUM(total), 0) FROM sales WHERE status = 'completed'")
            .fetch_one(&state.pool)
            .await
            .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    let pending_repairs: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM repairs WHERE status = 'pending'")
            .fetch_one(&state.pool)
            .await
            .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    let completed_repairs: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM repairs WHERE status = 'completed'")
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

async fn kpis(State(state): State<AppState>) -> Result<Json<ApiResponse<KpisResponse>>, AppError> {
    let total_suppliers: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM suppliers WHERE status = 'active'")
            .fetch_one(&state.pool)
            .await
            .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    let total_customers: i64 = sqlx::query_scalar(
        "SELECT COUNT(DISTINCT customer_email) FROM sales WHERE customer_email IS NOT NULL AND status = 'completed'"
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    let in_progress_repairs: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM repairs WHERE status = 'in_progress'")
            .fetch_one(&state.pool)
            .await
            .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    let cancelled_repairs: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM repairs WHERE status = 'cancelled'")
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

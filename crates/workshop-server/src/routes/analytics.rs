use axum::{
    extract::{Query, State},
    routing::get,
    Extension, Json, Router,
};
use rust_decimal::Decimal;
use serde::Deserialize;
use workshop_common::dto::{ApiResponse, DashboardResponse, RevenueDataPoint, RevenueResponse};

use crate::error::AppError;
use crate::middleware::AuthenticatedUser;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/dashboard", get(dashboard))
        .route("/kpis", get(kpis))
        .route("/revenue", get(revenue))
}

#[derive(Debug, Deserialize, Default)]
pub struct RevenueParams {
    pub start_date: Option<String>,
    pub end_date: Option<String>,
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

async fn revenue(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Query(params): Query<RevenueParams>,
) -> Result<Json<ApiResponse<RevenueResponse>>, AppError> {
    let wid = user.workshop_id;

    let today = chrono::Local::now().date_naive();
    let default_start = (today - chrono::Duration::days(180))
        .format("%Y-%m-%d")
        .to_string();
    let default_end = today.format("%Y-%m-%d").to_string();

    let start_date = params.start_date.as_deref().unwrap_or(&default_start);
    let end_date = params.end_date.as_deref().unwrap_or(&default_end);

    let start_naive = chrono::NaiveDate::parse_from_str(start_date, "%Y-%m-%d").map_err(|_| {
        AppError::Validation("Fecha de inicio inválida (use YYYY-MM-DD)".to_string())
    })?;
    let end_naive = chrono::NaiveDate::parse_from_str(end_date, "%Y-%m-%d")
        .map_err(|_| AppError::Validation("Fecha de fin inválida (use YYYY-MM-DD)".to_string()))?;

    let days_diff = (end_naive - start_naive).num_days();

    let (group_expr, grouping_label) = if days_diff <= 31 {
        ("TO_CHAR(s.created_at, 'YYYY-MM-DD')", "day".to_string())
    } else if days_diff <= 365 {
        (
            "TO_CHAR(DATE_TRUNC('week', s.created_at), 'YYYY-MM-DD')",
            "week".to_string(),
        )
    } else {
        (
            "TO_CHAR(DATE_TRUNC('month', s.created_at), 'YYYY-MM')",
            "month".to_string(),
        )
    };

    let start_dt = start_naive.and_hms_opt(0, 0, 0).unwrap();
    let end_dt = (end_naive + chrono::Duration::days(1))
        .and_hms_opt(0, 0, 0)
        .unwrap();

    let sales_query = format!(
        "SELECT {group_expr} AS period, COALESCE(SUM(s.total), 0) AS amount \
         FROM sales s \
         WHERE s.status = 'completed' AND s.workshop_id = $1 \
           AND s.created_at >= $2 AND s.created_at < $3 \
         GROUP BY period ORDER BY period"
    );

    let repairs_query = format!(
        "SELECT {group_expr} AS period, COALESCE(SUM(r.labor_cost), 0) AS amount \
         FROM repairs r \
         WHERE r.status = 'completed' AND r.workshop_id = $1 \
           AND r.created_at >= $2 AND r.created_at < $3 \
         GROUP BY period ORDER BY period"
    );

    let sales_rows: Vec<(String, Decimal)> = sqlx::query_as(&sales_query)
        .bind(wid)
        .bind(start_dt)
        .bind(end_dt)
        .fetch_all(&state.pool)
        .await
        .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    let repairs_rows: Vec<(String, Decimal)> = sqlx::query_as(&repairs_query)
        .bind(wid)
        .bind(start_dt)
        .bind(end_dt)
        .fetch_all(&state.pool)
        .await
        .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    let mut all_periods: Vec<String> = Vec::new();
    let mut sales_map: std::collections::HashMap<String, Decimal> =
        std::collections::HashMap::new();
    let mut repairs_map: std::collections::HashMap<String, Decimal> =
        std::collections::HashMap::new();

    for (period, amount) in &sales_rows {
        sales_map.insert(period.clone(), *amount);
        if !all_periods.contains(period) {
            all_periods.push(period.clone());
        }
    }
    for (period, amount) in &repairs_rows {
        repairs_map.insert(period.clone(), *amount);
        if !all_periods.contains(period) {
            all_periods.push(period.clone());
        }
    }
    all_periods.sort();

    let data: Vec<RevenueDataPoint> = all_periods
        .into_iter()
        .map(|period| RevenueDataPoint {
            period: period.clone(),
            sales: *sales_map.get(&period).unwrap_or(&Decimal::ZERO),
            repairs: *repairs_map.get(&period).unwrap_or(&Decimal::ZERO),
        })
        .collect();

    Ok(Json(ApiResponse::success(RevenueResponse {
        data,
        grouping: grouping_label,
    })))
}

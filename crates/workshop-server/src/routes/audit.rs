use axum::{
    extract::{Query, State},
    routing::get,
    Extension, Json, Router,
};
use workshop_common::dto::{ApiResponse, PaginatedResponse};
use workshop_common::AuditLog;

use crate::error::AppError;
use crate::middleware::AuthenticatedUser;
use crate::state::AppState;

use super::pagination::PaginationParams;

pub fn routes() -> Router<AppState> {
    Router::new().route("/", get(list_audit_logs))
}

async fn list_audit_logs(
    State(state): State<AppState>,
    Extension(_user): Extension<AuthenticatedUser>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<ApiResponse<PaginatedResponse<AuditLog>>>, AppError> {
    params.validate().map_err(AppError::Validation)?;

    let offset = params.offset();

    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM audit_log")
        .fetch_one(&state.pool)
        .await
        .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    let items: Vec<AuditLog> = sqlx::query_as(
        "SELECT id, user_id, action, entity_type, entity_id, old_values, new_values, ip_address, user_agent, created_at \
         FROM audit_log ORDER BY created_at DESC LIMIT $1 OFFSET $2",
    )
    .bind(params.per_page)
    .bind(offset)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    let response = PaginatedResponse {
        items,
        total,
        page: params.page,
        per_page: params.per_page,
    };

    Ok(Json(ApiResponse::success(response)))
}

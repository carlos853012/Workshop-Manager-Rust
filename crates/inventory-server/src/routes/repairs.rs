use axum::{
    extract::{Path, Query, State},
    routing::{get, post, put},
    Extension, Json, Router,
};
use chrono::Utc;
use inventory_common::dto::{ApiResponse, CreateRepairRequest, PaginatedResponse};
use inventory_common::{Repair, RepairStatus, RepairUpdate};
use serde::Deserialize;
use uuid::Uuid;

use crate::audit::{self, redact_sensitive};
use crate::error::AppError;
use crate::middleware::AuthenticatedUser;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_repairs))
        .route("/", post(create_repair))
        .route("/:id", get(get_repair))
        .route("/:id", put(update_repair))
}

#[derive(Debug, Deserialize)]
struct PaginationParams {
    #[serde(default = "super::products::default_page")]
    page: i32,
    #[serde(default = "super::products::default_per_page")]
    per_page: i32,
}

async fn list_repairs(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<ApiResponse<PaginatedResponse<Repair>>>, AppError> {
    if params.page < 1 {
        return Err(AppError::Validation("page must be >= 1".to_string()));
    }
    if params.per_page < 1 || params.per_page > 100 {
        return Err(AppError::Validation(
            "per_page must be between 1 and 100".to_string(),
        ));
    }

    let offset = (params.page - 1) * params.per_page;

    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM repairs WHERE status != 'deleted'")
        .fetch_one(&state.pool)
        .await
        .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    let items: Vec<Repair> = sqlx::query_as(
        "SELECT id, customer_name, customer_email, customer_phone, motorcycle, license_plate, \
         description, diagnosis, technician_id, estimated_delivery, priority as \"priority!: Priority\", \
         status as \"status!: RepairStatus\", estimated_cost, final_cost, created_at, updated_at \
         FROM repairs WHERE status != 'deleted' \
         ORDER BY created_at DESC LIMIT $1 OFFSET $2"
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

async fn get_repair(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<RepairDetail>>, AppError> {
    let repair: Option<Repair> = sqlx::query_as(
        "SELECT id, customer_name, customer_email, customer_phone, motorcycle, license_plate, \
         description, diagnosis, technician_id, estimated_delivery, priority as \"priority!: Priority\", \
         status as \"status!: RepairStatus\", estimated_cost, final_cost, created_at, updated_at \
         FROM repairs WHERE id = $1 AND status != 'deleted'"
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    let repair = repair.ok_or(AppError::NotFound("Repair not found".to_string()))?;

    let updates: Vec<RepairUpdate> = sqlx::query_as(
        "SELECT id, repair_id, status as \"status!: RepairStatus\", description, created_by, created_at \
         FROM repair_updates WHERE repair_id = $1 ORDER BY created_at ASC"
    )
    .bind(id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    Ok(Json(ApiResponse::success(RepairDetail { repair, updates })))
}

#[derive(Debug, serde::Serialize)]
struct RepairDetail {
    #[serde(flatten)]
    repair: Repair,
    updates: Vec<RepairUpdate>,
}

async fn create_repair(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(req): Json<CreateRepairRequest>,
) -> Result<Json<ApiResponse<RepairDetail>>, AppError> {
    validate_create_repair_request(&req)?;

    let id = Uuid::new_v4();
    let now = Utc::now();

    sqlx::query(
        "INSERT INTO repairs \
         (id, customer_name, customer_email, customer_phone, motorcycle, license_plate, description, \
          priority, status, estimated_cost, estimated_delivery, created_at, updated_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'pending', $9, $10, $11, $12)"
    )
    .bind(id)
    .bind(&req.customer_name)
    .bind(&req.customer_email)
    .bind(&req.customer_phone)
    .bind(&req.motorcycle)
    .bind(&req.license_plate)
    .bind(&req.description)
    .bind(&req.priority)
    .bind(req.estimated_cost)
    .bind(req.estimated_delivery)
    .bind(now)
    .bind(now)
    .execute(&state.pool)
    .await
    .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    let initial_update = RepairUpdate {
        id: Uuid::new_v4(),
        repair_id: id,
        status: Some(RepairStatus::Pending.to_string()),
        description: Some("Repair created".to_string()),
        created_by: Some(user.id),
        created_at: now,
    };

    sqlx::query(
        "INSERT INTO repair_updates \
         (id, repair_id, status, description, created_by, created_at) \
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(initial_update.id)
    .bind(initial_update.repair_id)
    .bind(&initial_update.status)
    .bind(&initial_update.description)
    .bind(initial_update.created_by)
    .bind(initial_update.created_at)
    .execute(&state.pool)
    .await
    .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    let repair = Repair {
        id,
        customer_name: req.customer_name,
        customer_email: req.customer_email,
        customer_phone: req.customer_phone,
        motorcycle: req.motorcycle,
        license_plate: req.license_plate,
        description: req.description,
        diagnosis: None,
        technician_id: None,
        estimated_delivery: req.estimated_delivery,
        priority: req.priority,
        status: RepairStatus::Pending,
        estimated_cost: req.estimated_cost,
        final_cost: None,
        created_at: now,
        updated_at: now,
    };

    let detail = RepairDetail {
        repair: repair.clone(),
        updates: vec![initial_update],
    };

    let mut new_values = serde_json::to_value(&detail).unwrap_or_default();
    redact_sensitive(&mut new_values);

    audit::log_change(
        &state.pool,
        Some(user.id),
        "create",
        "repair",
        id,
        None,
        Some(new_values),
        None,
        None,
    )
    .await
    .map_err(|e| AppError::Internal(format!("Audit error: {}", e)))?;

    Ok(Json(ApiResponse::success(detail)))
}

#[derive(Debug, Deserialize)]
struct UpdateRepairRequest {
    pub status: Option<RepairStatus>,
    pub diagnosis: Option<String>,
    pub technician_id: Option<Uuid>,
    pub estimated_cost: Option<rust_decimal::Decimal>,
    pub final_cost: Option<rust_decimal::Decimal>,
    pub estimated_delivery: Option<chrono::NaiveDate>,
}

async fn update_repair(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateRepairRequest>,
) -> Result<Json<ApiResponse<RepairDetail>>, AppError> {
    let old_repair: Option<Repair> = sqlx::query_as(
        "SELECT id, customer_name, customer_email, customer_phone, motorcycle, license_plate, \
         description, diagnosis, technician_id, estimated_delivery, priority as \"priority!: Priority\", \
         status as \"status!: RepairStatus\", estimated_cost, final_cost, created_at, updated_at \
         FROM repairs WHERE id = $1 AND status != 'deleted'"
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    let old_repair = old_repair.ok_or(AppError::NotFound("Repair not found".to_string()))?;

    let old_status = old_repair.status.clone();
    let old_priority = old_repair.priority.clone();
    let new_status = req.status.unwrap_or(old_status.clone());
    let new_diagnosis = req.diagnosis.or(old_repair.diagnosis.clone());
    let new_technician_id = req.technician_id.or(old_repair.technician_id);
    let new_estimated_cost = req.estimated_cost.or(old_repair.estimated_cost);
    let new_final_cost = req.final_cost.or(old_repair.final_cost);
    let new_estimated_delivery = req.estimated_delivery.or(old_repair.estimated_delivery);
    let now = Utc::now();

    sqlx::query(
        "UPDATE repairs SET \
         status = $2, diagnosis = $3, technician_id = $4, estimated_cost = $5, final_cost = $6, \
         estimated_delivery = $7, updated_at = $8 \
         WHERE id = $1 AND status != 'deleted'",
    )
    .bind(id)
    .bind(&new_status)
    .bind(&new_diagnosis)
    .bind(new_technician_id)
    .bind(new_estimated_cost)
    .bind(new_final_cost)
    .bind(new_estimated_delivery)
    .bind(now)
    .execute(&state.pool)
    .await
    .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    let status_changed = new_status != old_status;
    if status_changed {
        sqlx::query(
            "INSERT INTO repair_updates \
             (id, repair_id, status, description, created_by, created_at) \
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(Uuid::new_v4())
        .bind(id)
        .bind(&new_status)
        .bind(Some(format!(
            "Status changed from {} to {}",
            old_status, new_status
        )))
        .bind(Some(user.id))
        .bind(now)
        .execute(&state.pool)
        .await
        .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;
    }

    let repair = Repair {
        id,
        customer_name: old_repair.customer_name.clone(),
        customer_email: old_repair.customer_email.clone(),
        customer_phone: old_repair.customer_phone.clone(),
        motorcycle: old_repair.motorcycle.clone(),
        license_plate: old_repair.license_plate.clone(),
        description: old_repair.description.clone(),
        diagnosis: new_diagnosis,
        technician_id: new_technician_id,
        estimated_delivery: new_estimated_delivery,
        priority: old_priority,
        status: new_status,
        estimated_cost: new_estimated_cost,
        final_cost: new_final_cost,
        created_at: old_repair.created_at,
        updated_at: now,
    };

    let updates: Vec<RepairUpdate> = sqlx::query_as(
        "SELECT id, repair_id, status as \"status!: RepairStatus\", description, created_by, created_at \
         FROM repair_updates WHERE repair_id = $1 ORDER BY created_at ASC"
    )
    .bind(id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    let detail = RepairDetail { repair, updates };

    let mut old_values = serde_json::to_value(&old_repair).unwrap_or_default();
    redact_sensitive(&mut old_values);
    let mut new_values = serde_json::to_value(&detail).unwrap_or_default();
    redact_sensitive(&mut new_values);

    audit::log_change(
        &state.pool,
        Some(user.id),
        "update",
        "repair",
        id,
        Some(old_values),
        Some(new_values),
        None,
        None,
    )
    .await
    .map_err(|e| AppError::Internal(format!("Audit error: {}", e)))?;

    Ok(Json(ApiResponse::success(detail)))
}

fn validate_create_repair_request(req: &CreateRepairRequest) -> Result<(), AppError> {
    if let Some(ref email) = req.customer_email {
        if !email.contains('@') || !email.contains('.') {
            return Err(AppError::Validation("Invalid customer email".to_string()));
        }
    }

    if let Some(ref phone) = req.customer_phone {
        if phone.len() > 20 {
            return Err(AppError::Validation(
                "customer_phone must be <= 20 characters".to_string(),
            ));
        }
    }

    if let Some(ref license_plate) = req.license_plate {
        if license_plate.len() > 50 {
            return Err(AppError::Validation(
                "license_plate must be <= 50 characters".to_string(),
            ));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use inventory_common::Priority;

    #[test]
    fn test_validate_create_repair_request_valid() {
        let req = CreateRepairRequest {
            customer_name: Some("John Doe".to_string()),
            customer_email: Some("john@example.com".to_string()),
            customer_phone: Some("1234567890".to_string()),
            motorcycle: Some("Honda CB500".to_string()),
            license_plate: Some("ABC123".to_string()),
            description: Some("Oil change".to_string()),
            priority: Priority::Medium,
            estimated_cost: None,
            estimated_delivery: None,
        };

        assert!(validate_create_repair_request(&req).is_ok());
    }

    #[test]
    fn test_validate_create_repair_request_invalid_email() {
        let req = CreateRepairRequest {
            customer_name: None,
            customer_email: Some("invalid".to_string()),
            customer_phone: None,
            motorcycle: None,
            license_plate: None,
            description: None,
            priority: Priority::Low,
            estimated_cost: None,
            estimated_delivery: None,
        };

        assert!(validate_create_repair_request(&req).is_err());
    }
}

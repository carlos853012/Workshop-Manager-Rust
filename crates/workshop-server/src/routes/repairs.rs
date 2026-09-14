use axum::{
    extract::{Path, Query, State},
    routing::{delete, get, post, put},
    Extension, Json, Router,
};
use chrono::Utc;
use workshop_common::dto::{
    AddRepairPartRequest, ApiResponse, CreateRepairRequest, PaginatedResponse, RepairPartResponse,
};
use workshop_common::patente;
use workshop_common::{Repair, RepairPart, RepairStatus, RepairUpdate};
use rust_decimal::Decimal;
use serde::Deserialize;
use uuid::Uuid;

use crate::audit::{self, redact_sensitive};
use crate::error::AppError;
use crate::middleware::AuthenticatedUser;
use crate::state::AppState;

use super::pagination::PaginationParams;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_repairs))
        .route("/", post(create_repair))
        .route("/:id", get(get_repair))
        .route("/:id", put(update_repair))
        .route("/:id/parts", get(list_repair_parts))
        .route("/:id/parts", post(add_repair_part))
        .route("/:id/parts/:part_id", delete(remove_repair_part))
}

async fn list_repairs(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<ApiResponse<PaginatedResponse<Repair>>>, AppError> {
    params.validate().map_err(AppError::Validation)?;

    let offset = params.offset();

    let total: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM repairs WHERE status != 'deleted' AND workshop_id = $1",
    )
    .bind(user.workshop_id)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    let items: Vec<Repair> = sqlx::query_as(
        "SELECT id, workshop_id, customer_name, customer_email, customer_phone, vehicle, license_plate, \
         description, diagnosis, technician_id, estimated_delivery, priority, \
         status, estimated_cost, final_cost, labor_cost, created_at, updated_at \
         FROM repairs WHERE status != 'deleted' AND workshop_id = $3 \
         ORDER BY created_at DESC LIMIT $1 OFFSET $2",
    )
    .bind(params.per_page)
    .bind(offset)
    .bind(user.workshop_id)
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
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<RepairDetail>>, AppError> {
    let repair: Option<Repair> = sqlx::query_as(
        "SELECT id, workshop_id, customer_name, customer_email, customer_phone, vehicle, license_plate, \
         description, diagnosis, technician_id, estimated_delivery, priority, \
         status, estimated_cost, final_cost, labor_cost, created_at, updated_at \
         FROM repairs WHERE id = $1 AND status != 'deleted' AND workshop_id = $2"
    )
    .bind(id)
    .bind(user.workshop_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    let repair = repair.ok_or(AppError::NotFound("Repair not found".to_string()))?;

    let updates: Vec<RepairUpdate> = sqlx::query_as(
        "SELECT id, repair_id, status, description, created_by, created_at \
         FROM repair_updates WHERE repair_id = $1 ORDER BY created_at ASC",
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

    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    let id = Uuid::new_v4();
    let now = Utc::now();

    sqlx::query(
        "INSERT INTO repairs \
         (id, workshop_id, customer_name, customer_email, customer_phone, vehicle, license_plate, description, \
           priority, status, estimated_cost, estimated_delivery, created_at, updated_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 'pending', $10, $11, $12, $13)"
    )
    .bind(id)
    .bind(user.workshop_id)
    .bind(&req.customer_name)
    .bind(&req.customer_email)
    .bind(&req.customer_phone)
    .bind(&req.vehicle)
    .bind(&req.license_plate)
    .bind(&req.description)
    .bind(&req.priority)
    .bind(req.estimated_cost)
    .bind(req.estimated_delivery)
    .bind(now)
    .bind(now)
    .execute(&mut *tx)
    .await
    .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    let initial_update = RepairUpdate {
        id: Uuid::new_v4(),
        repair_id: id,
        status: Some(RepairStatus::Pending),
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
    .execute(&mut *tx)
    .await
    .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    tx.commit()
        .await
        .map_err(|e| AppError::Internal(format!("Transaction commit error: {}", e)))?;

    let repair = Repair {
        id,
        workshop_id: user.workshop_id,
        customer_name: req.customer_name,
        customer_email: req.customer_email,
        customer_phone: req.customer_phone,
        vehicle: req.vehicle,
        license_plate: req.license_plate,
        description: req.description,
        diagnosis: None,
        technician_id: None,
        estimated_delivery: req.estimated_delivery,
        priority: req.priority,
        status: RepairStatus::Pending,
        estimated_cost: req.estimated_cost,
        final_cost: None,
        labor_cost: None,
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
    pub labor_cost: Option<rust_decimal::Decimal>,
    pub estimated_delivery: Option<chrono::NaiveDate>,
}

async fn update_repair(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateRepairRequest>,
) -> Result<Json<ApiResponse<RepairDetail>>, AppError> {
    let old_repair: Option<Repair> = sqlx::query_as(
        "SELECT id, workshop_id, customer_name, customer_email, customer_phone, vehicle, license_plate, \
         description, diagnosis, technician_id, estimated_delivery, priority, \
         status, estimated_cost, final_cost, labor_cost, created_at, updated_at \
         FROM repairs WHERE id = $1 AND status != 'deleted' AND workshop_id = $2"
    )
    .bind(id)
    .bind(user.workshop_id)
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
    let new_labor_cost = req.labor_cost.or(old_repair.labor_cost);
    let new_estimated_delivery = req.estimated_delivery.or(old_repair.estimated_delivery);
    let now = Utc::now();

    sqlx::query(
        "UPDATE repairs SET \
         status = $2, diagnosis = $3, technician_id = $4, estimated_cost = $5, final_cost = $6, \
         labor_cost = $7, estimated_delivery = $8, updated_at = $9 \
         WHERE id = $1 AND status != 'deleted' AND workshop_id = $10",
    )
    .bind(id)
    .bind(&new_status)
    .bind(&new_diagnosis)
    .bind(new_technician_id)
    .bind(new_estimated_cost)
    .bind(new_final_cost)
    .bind(new_labor_cost)
    .bind(new_estimated_delivery)
    .bind(now)
    .bind(user.workshop_id)
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
        workshop_id: old_repair.workshop_id,
        customer_name: old_repair.customer_name.clone(),
        customer_email: old_repair.customer_email.clone(),
        customer_phone: old_repair.customer_phone.clone(),
        vehicle: old_repair.vehicle.clone(),
        license_plate: old_repair.license_plate.clone(),
        description: old_repair.description.clone(),
        diagnosis: new_diagnosis,
        technician_id: new_technician_id,
        estimated_delivery: new_estimated_delivery,
        priority: old_priority,
        status: new_status,
        estimated_cost: new_estimated_cost,
        final_cost: new_final_cost,
        labor_cost: old_repair.labor_cost,
        created_at: old_repair.created_at,
        updated_at: now,
    };

    let updates: Vec<RepairUpdate> = sqlx::query_as(
        "SELECT id, repair_id, status, description, created_by, created_at \
         FROM repair_updates WHERE repair_id = $1 ORDER BY created_at ASC",
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
        if !license_plate.is_empty() {
            if let Err(e) = patente::validar_patente_con_error(license_plate) {
                return Err(AppError::Validation(e.to_string()));
            }
        }
    }

    Ok(())
}

async fn list_repair_parts(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(repair_id): Path<Uuid>,
) -> Result<Json<ApiResponse<Vec<RepairPartResponse>>>, AppError> {
    let repair_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM repairs WHERE id = $1 AND workshop_id = $2 AND status != 'deleted')",
    )
    .bind(repair_id)
    .bind(user.workshop_id)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    if !repair_exists {
        return Err(AppError::NotFound("Repair not found".to_string()));
    }

    let parts: Vec<RepairPart> = sqlx::query_as(
        "SELECT id, repair_id, name, quantity, unit_cost, total_cost, product_id, created_at \
         FROM repair_parts WHERE repair_id = $1 ORDER BY created_at ASC",
    )
    .bind(repair_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    let response: Vec<RepairPartResponse> = parts
        .into_iter()
        .map(|p| RepairPartResponse {
            id: p.id,
            repair_id: p.repair_id,
            name: p.name,
            quantity: p.quantity,
            unit_cost: p.unit_cost,
            total_cost: p.total_cost,
            product_id: p.product_id,
            created_at: p.created_at,
        })
        .collect();

    Ok(Json(ApiResponse::success(response)))
}

async fn add_repair_part(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(repair_id): Path<Uuid>,
    Json(req): Json<AddRepairPartRequest>,
) -> Result<Json<ApiResponse<RepairPartResponse>>, AppError> {
    if req.name.trim().is_empty() {
        return Err(AppError::Validation("name is required".to_string()));
    }
    if req.quantity <= Decimal::ZERO {
        return Err(AppError::Validation("quantity must be > 0".to_string()));
    }

    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    let repair: Option<Repair> = sqlx::query_as(
        "SELECT id, workshop_id, customer_name, customer_email, customer_phone, vehicle, license_plate, description, \
         diagnosis, technician_id, estimated_delivery, priority, status, estimated_cost, final_cost, labor_cost, created_at, updated_at \
         FROM repairs WHERE id = $1 AND workshop_id = $2",
    )
    .bind(repair_id)
    .bind(user.workshop_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    let _repair = repair.ok_or(AppError::NotFound("Repair not found".to_string()))?;

    if let Some(pid) = req.product_id {
        let rows = sqlx::query(
            "UPDATE products SET stock = stock - $1, updated_at = NOW() \
             WHERE id = $2 AND workshop_id = $3 AND stock >= $1",
        )
        .bind(req.quantity)
        .bind(pid)
        .bind(user.workshop_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

        if rows.rows_affected() == 0 {
            return Err(AppError::Validation(
                "Stock insuficiente para este producto".to_string(),
            ));
        }
    }

    let total_cost = req.unit_cost.map(|uc| uc * req.quantity);
    let id = Uuid::new_v4();
    let now = Utc::now();

    sqlx::query(
        "INSERT INTO repair_parts (id, repair_id, name, quantity, unit_cost, total_cost, product_id, created_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
    )
    .bind(id)
    .bind(repair_id)
    .bind(&req.name)
    .bind(req.quantity)
    .bind(req.unit_cost)
    .bind(total_cost)
    .bind(req.product_id)
    .bind(now)
    .execute(&mut *tx)
    .await
    .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    tx.commit()
        .await
        .map_err(|e| AppError::Internal(format!("Transaction commit error: {}", e)))?;

    let part = RepairPartResponse {
        id,
        repair_id,
        name: req.name,
        quantity: req.quantity,
        unit_cost: req.unit_cost,
        total_cost,
        product_id: req.product_id,
        created_at: now,
    };

    let mut new_values = serde_json::to_value(&part).unwrap_or_default();
    redact_sensitive(&mut new_values);

    if let Err(e) = audit::log_change(
        &state.pool,
        Some(user.id),
        "add_part",
        "repair",
        repair_id,
        None,
        Some(new_values),
        None,
        None,
    )
    .await
    {
        tracing::warn!("Audit log failed for add_repair_part: {}", e);
    }

    Ok(Json(ApiResponse::success(part)))
}

async fn remove_repair_part(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path((repair_id, part_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    let repair: Option<Repair> = sqlx::query_as(
        "SELECT id, workshop_id, customer_name, customer_email, customer_phone, vehicle, license_plate, description, \
         diagnosis, technician_id, estimated_delivery, priority, status, estimated_cost, final_cost, labor_cost, created_at, updated_at \
         FROM repairs WHERE id = $1 AND workshop_id = $2",
    )
    .bind(repair_id)
    .bind(user.workshop_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    if repair.is_none() {
        return Err(AppError::NotFound("Repair not found".to_string()));
    }

    let part: Option<RepairPart> = sqlx::query_as(
        "SELECT id, repair_id, name, quantity, unit_cost, total_cost, product_id, created_at \
         FROM repair_parts WHERE id = $1 AND repair_id = $2",
    )
    .bind(part_id)
    .bind(repair_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    let part = part.ok_or(AppError::NotFound("Part not found".to_string()))?;

    sqlx::query("DELETE FROM repair_parts WHERE id = $1 AND repair_id = $2")
        .bind(part_id)
        .bind(repair_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    if let Some(pid) = part.product_id {
        sqlx::query(
            "UPDATE products SET stock = stock + $1, updated_at = NOW() \
             WHERE id = $2 AND workshop_id = $3",
        )
        .bind(part.quantity)
        .bind(pid)
        .bind(user.workshop_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;
    }

    tx.commit()
        .await
        .map_err(|e| AppError::Internal(format!("Transaction commit error: {}", e)))?;

    if let Err(e) = audit::log_change(
        &state.pool,
        Some(user.id),
        "remove_part",
        "repair",
        repair_id,
        None,
        None,
        None,
        None,
    )
    .await
    {
        tracing::warn!("Audit log failed for remove_repair_part: {}", e);
    }

    Ok(Json(ApiResponse::success(())))
}

#[cfg(test)]
mod tests {
    use super::*;
    use workshop_common::Priority;

    #[test]
    fn test_validate_create_repair_request_valid() {
        let req = CreateRepairRequest {
            customer_name: Some("John Doe".to_string()),
            customer_email: Some("john@example.com".to_string()),
            customer_phone: Some("1234567890".to_string()),
            vehicle: Some("Honda CB500".to_string()),
            license_plate: Some("BCDF12".to_string()),
            description: Some("Oil change".to_string()),
            priority: Priority::Medium,
            estimated_cost: None,
            estimated_delivery: None,
        };

        assert!(validate_create_repair_request(&req).is_ok());
    }

    #[test]
    fn test_validate_create_repair_request_invalid_plate() {
        let req = CreateRepairRequest {
            customer_name: Some("Jane Doe".to_string()),
            customer_email: None,
            customer_phone: None,
            vehicle: None,
            license_plate: Some("INVALID".to_string()),
            description: None,
            priority: Priority::Low,
            estimated_cost: None,
            estimated_delivery: None,
        };

        assert!(validate_create_repair_request(&req).is_err());
    }

    #[test]
    fn test_validate_create_repair_request_valid_plate_formats() {
        // Formato nuevo: 4 letras + 2 dígitos
        let req = CreateRepairRequest {
            customer_name: Some("Test".to_string()),
            customer_email: None,
            customer_phone: None,
            vehicle: None,
            license_plate: Some("BCDF12".to_string()),
            description: None,
            priority: Priority::Low,
            estimated_cost: None,
            estimated_delivery: None,
        };
        assert!(validate_create_repair_request(&req).is_ok());

        // Formato antiguo: 2 letras + 4 dígitos
        let req = CreateRepairRequest {
            customer_name: Some("Test".to_string()),
            customer_email: None,
            customer_phone: None,
            vehicle: None,
            license_plate: Some("AR1240".to_string()),
            description: None,
            priority: Priority::Low,
            estimated_cost: None,
            estimated_delivery: None,
        };
        assert!(validate_create_repair_request(&req).is_ok());

        // Motos: 3 letras + 2 dígitos
        let req = CreateRepairRequest {
            customer_name: Some("Test".to_string()),
            customer_email: None,
            customer_phone: None,
            vehicle: None,
            license_plate: Some("BJH61".to_string()),
            description: None,
            priority: Priority::Low,
            estimated_cost: None,
            estimated_delivery: None,
        };
        assert!(validate_create_repair_request(&req).is_ok());
    }

    #[test]
    fn test_validate_create_repair_request_empty_plate_ok() {
        let req = CreateRepairRequest {
            customer_name: Some("Test".to_string()),
            customer_email: None,
            customer_phone: None,
            vehicle: None,
            license_plate: None,
            description: None,
            priority: Priority::Low,
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
            vehicle: None,
            license_plate: None,
            description: None,
            priority: Priority::Low,
            estimated_cost: None,
            estimated_delivery: None,
        };

        assert!(validate_create_repair_request(&req).is_err());
    }
}

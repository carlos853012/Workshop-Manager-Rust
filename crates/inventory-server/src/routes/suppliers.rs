use axum::{
    extract::{Path, Query, State},
    routing::{get, post, put},
    Extension, Json, Router,
};
use chrono::Utc;
use inventory_common::dto::{ApiResponse, CreateSupplierRequest, PaginatedResponse};
use inventory_common::Supplier;
use uuid::Uuid;

use crate::audit::{self, redact_sensitive};
use crate::error::AppError;
use crate::middleware::AuthenticatedUser;
use crate::state::AppState;

use super::pagination::PaginationParams;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_suppliers))
        .route("/", post(create_supplier))
        .route("/:id", get(get_supplier))
        .route("/:id", put(update_supplier))
}

async fn list_suppliers(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<ApiResponse<PaginatedResponse<Supplier>>>, AppError> {
    params.validate().map_err(AppError::Validation)?;

    let offset = params.offset();

    let total: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM suppliers WHERE status = 'active' AND workshop_id = $1",
    )
    .bind(user.workshop_id)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    let items: Vec<Supplier> = sqlx::query_as(
        "SELECT id, workshop_id, name, contact_person, email, phone, address, tax_id, payment_terms, status, created_at, updated_at \
         FROM suppliers WHERE status = 'active' AND workshop_id = $1 \
         ORDER BY name ASC LIMIT $2 OFFSET $3"
    )
    .bind(user.workshop_id)
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

async fn get_supplier(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<Supplier>>, AppError> {
    let supplier: Option<Supplier> = sqlx::query_as(
        "SELECT id, workshop_id, name, contact_person, email, phone, address, tax_id, payment_terms, status, created_at, updated_at \
         FROM suppliers WHERE id = $1 AND status = 'active' AND workshop_id = $2"
    )
    .bind(id)
    .bind(user.workshop_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    match supplier {
        Some(supplier) => Ok(Json(ApiResponse::success(supplier))),
        None => Err(AppError::NotFound("Supplier not found".to_string())),
    }
}

async fn create_supplier(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(req): Json<CreateSupplierRequest>,
) -> Result<Json<ApiResponse<Supplier>>, AppError> {
    validate_create_supplier_request(&req)?;

    let id = Uuid::new_v4();
    let now = Utc::now();

    sqlx::query(
        "INSERT INTO suppliers \
         (id, workshop_id, name, contact_person, email, phone, address, tax_id, payment_terms, status, created_at, updated_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'active', $9, $10)"
    )
    .bind(id)
    .bind(user.workshop_id)
    .bind(&req.name)
    .bind(&req.contact_person)
    .bind(&req.email)
    .bind(&req.phone)
    .bind(&req.address)
    .bind(&req.tax_id)
    .bind(&req.payment_terms)
    .bind(now)
    .bind(now)
    .execute(&state.pool)
    .await
    .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    let supplier = Supplier {
        id,
        workshop_id: user.workshop_id,
        name: req.name,
        contact_person: req.contact_person,
        email: req.email,
        phone: req.phone,
        address: req.address,
        tax_id: req.tax_id,
        payment_terms: req.payment_terms,
        status: "active".to_string(),
        created_at: now,
        updated_at: now,
    };

    let mut new_values = serde_json::to_value(&supplier).unwrap_or_default();
    redact_sensitive(&mut new_values);

    audit::log_change(
        &state.pool,
        Some(user.id),
        "create",
        "supplier",
        id,
        None,
        Some(new_values),
        None,
        None,
    )
    .await
    .map_err(|e| AppError::Internal(format!("Audit error: {}", e)))?;

    Ok(Json(ApiResponse::success(supplier)))
}

async fn update_supplier(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<CreateSupplierRequest>,
) -> Result<Json<ApiResponse<Supplier>>, AppError> {
    validate_create_supplier_request(&req)?;

    let old_supplier: Option<Supplier> = sqlx::query_as(
        "SELECT id, workshop_id, name, contact_person, email, phone, address, tax_id, payment_terms, status, created_at, updated_at \
         FROM suppliers WHERE id = $1 AND status = 'active' AND workshop_id = $2"
    )
    .bind(id)
    .bind(user.workshop_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    let old_supplier = old_supplier.ok_or(AppError::NotFound("Supplier not found".to_string()))?;
    let now = Utc::now();

    sqlx::query(
        "UPDATE suppliers SET \
         name = $2, contact_person = $3, email = $4, phone = $5, address = $6, tax_id = $7, \
         payment_terms = $8, updated_at = $9 \
         WHERE id = $1 AND status = 'active' AND workshop_id = $10",
    )
    .bind(id)
    .bind(&req.name)
    .bind(&req.contact_person)
    .bind(&req.email)
    .bind(&req.phone)
    .bind(&req.address)
    .bind(&req.tax_id)
    .bind(&req.payment_terms)
    .bind(now)
    .bind(user.workshop_id)
    .execute(&state.pool)
    .await
    .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    let supplier = Supplier {
        id,
        workshop_id: old_supplier.workshop_id,
        name: req.name,
        contact_person: req.contact_person,
        email: req.email,
        phone: req.phone,
        address: req.address,
        tax_id: req.tax_id,
        payment_terms: req.payment_terms,
        status: old_supplier.status.clone(),
        created_at: old_supplier.created_at,
        updated_at: now,
    };

    let mut old_values = serde_json::to_value(&old_supplier).unwrap_or_default();
    redact_sensitive(&mut old_values);
    let mut new_values = serde_json::to_value(&supplier).unwrap_or_default();
    redact_sensitive(&mut new_values);

    audit::log_change(
        &state.pool,
        Some(user.id),
        "update",
        "supplier",
        id,
        Some(old_values),
        Some(new_values),
        None,
        None,
    )
    .await
    .map_err(|e| AppError::Internal(format!("Audit error: {}", e)))?;

    Ok(Json(ApiResponse::success(supplier)))
}

fn validate_create_supplier_request(req: &CreateSupplierRequest) -> Result<(), AppError> {
    if req.name.trim().is_empty() || req.name.len() > 200 {
        return Err(AppError::Validation(
            "name must be between 1 and 200 characters".to_string(),
        ));
    }

    if let Some(ref email) = req.email {
        if !email.is_empty() && (!email.contains('@') || !email.contains('.')) {
            return Err(AppError::Validation("Invalid email format".to_string()));
        }
    }

    if let Some(ref phone) = req.phone {
        if phone.len() > 20 {
            return Err(AppError::Validation(
                "phone must be <= 20 characters".to_string(),
            ));
        }
    }

    if let Some(ref tax_id) = req.tax_id {
        if tax_id.len() > 50 {
            return Err(AppError::Validation(
                "tax_id must be <= 50 characters".to_string(),
            ));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_create_supplier_request_valid() {
        let req = CreateSupplierRequest {
            name: "Supplier Co".to_string(),
            contact_person: Some("John".to_string()),
            email: Some("supplier@example.com".to_string()),
            phone: Some("1234567890".to_string()),
            address: None,
            tax_id: Some("TAX123".to_string()),
            payment_terms: Some("Net 30".to_string()),
        };

        assert!(validate_create_supplier_request(&req).is_ok());
    }

    #[test]
    fn test_validate_create_supplier_request_invalid_name() {
        let req = CreateSupplierRequest {
            name: "".to_string(),
            contact_person: None,
            email: None,
            phone: None,
            address: None,
            tax_id: None,
            payment_terms: None,
        };

        assert!(validate_create_supplier_request(&req).is_err());
    }
}

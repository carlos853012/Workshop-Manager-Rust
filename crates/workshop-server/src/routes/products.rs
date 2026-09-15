use axum::{
    extract::{Path, Query, State},
    routing::{delete, get, post, put},
    Extension, Json, Router,
};
use chrono::Utc;
use rust_decimal::Decimal;
use workshop_common::dto::{
    ApiResponse, CreateProductRequest, PaginatedResponse, PosLookupRequest, PosProductResponse,
};
use workshop_common::{Product, UserRole};

use uuid::Uuid;

use crate::audit::{self, redact_sensitive};
use crate::barcode::{generate_ean13, get_workshop_prefix};
use crate::error::AppError;
use crate::middleware::AuthenticatedUser;

use crate::state::AppState;

use super::pagination::PaginationParams;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_products))
        .route("/", post(create_product))
        .route("/:id", get(get_product))
        .route("/:id", put(update_product))
        .route("/:id", delete(delete_product))
        .route("/lookup", get(lookup_product_by_barcode))
}

async fn list_products(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<ApiResponse<PaginatedResponse<Product>>>, AppError> {
    params.validate().map_err(AppError::Validation)?;

    let offset = params.offset();

    let total: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM products WHERE status = 'active' AND workshop_id = $1",
    )
    .bind(user.workshop_id)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    let items: Vec<Product> = sqlx::query_as(
        "SELECT id, workshop_id, name, description, category, brand, model, sku, barcode, price, cost, stock, min_stock, \
         location, supplier_id, status, created_at, updated_at \
         FROM products WHERE status = 'active' AND workshop_id = $1 \
         ORDER BY created_at DESC LIMIT $2 OFFSET $3"
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

async fn get_product(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<Product>>, AppError> {
    let product: Option<Product> = sqlx::query_as(
        "SELECT id, workshop_id, name, description, category, brand, model, sku, barcode, price, cost, stock, min_stock, \
         location, supplier_id, status, created_at, updated_at \
         FROM products WHERE id = $1 AND status = 'active' AND workshop_id = $2"
    )
    .bind(id)
    .bind(user.workshop_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    match product {
        Some(product) => Ok(Json(ApiResponse::success(product))),
        None => Err(AppError::NotFound("Product not found".to_string())),
    }
}

async fn create_product(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(req): Json<CreateProductRequest>,
) -> Result<Json<ApiResponse<Product>>, AppError> {
    if !matches!(user.role, UserRole::Admin | UserRole::Seller) {
        return Err(AppError::Forbidden("Acceso denegado".to_string()));
    }
    validate_create_product_request(&req)?;

    let id = Uuid::new_v4();
    let now = Utc::now();
    let workshop_id = user.workshop_id;

    // Generate barcode if not provided
    let barcode = if let Some(bc) = req.barcode {
        if bc.trim().is_empty() {
            None
        } else {
            Some(bc.trim().to_string())
        }
    } else {
        // Generate EAN-13 with workshop prefix
        let prefix = get_workshop_prefix(&state.pool, workshop_id)
            .await
            .map_err(|e| AppError::Internal(format!("Barcode error: {}", e)))?;
        let barcode = generate_ean13(&prefix);
        Some(barcode)
    };

    sqlx::query(
        "INSERT INTO products \
         (id, workshop_id, name, description, category, brand, model, sku, barcode, price, cost, stock, min_stock, location, supplier_id, status, created_at, updated_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, 'active', $16, $17)"
    )
    .bind(id)
    .bind(workshop_id)
    .bind(&req.name)
    .bind(&req.description)
    .bind(&req.category)
    .bind(&req.brand)
    .bind(&req.model)
    .bind(&req.sku)
    .bind(&barcode)
    .bind(req.price)
    .bind(req.cost)
    .bind(req.stock)
    .bind(req.min_stock)
    .bind(&req.location)
    .bind(req.supplier_id)
    .bind(now)
    .bind(now)
    .execute(&state.pool)
    .await
    .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    let product = Product {
        id,
        workshop_id,
        name: req.name,
        description: req.description,
        category: req.category,
        brand: req.brand,
        model: req.model,
        sku: req.sku,
        barcode,
        price: req.price,
        cost: req.cost,
        stock: req.stock,
        min_stock: req.min_stock,
        location: req.location,
        supplier_id: req.supplier_id,
        status: "active".to_string(),
        created_at: now,
        updated_at: now,
    };

    let mut new_values = serde_json::to_value(&product).unwrap_or_default();
    redact_sensitive(&mut new_values);

    audit::log_change(
        &state.pool,
        Some(user.id),
        "create",
        "product",
        id,
        None,
        Some(new_values),
        None,
        None,
    )
    .await
    .map_err(|e| AppError::Internal(format!("Audit error: {}", e)))?;

    Ok(Json(ApiResponse::success(product)))
}

async fn update_product(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<CreateProductRequest>,
) -> Result<Json<ApiResponse<Product>>, AppError> {
    if !matches!(user.role, UserRole::Admin | UserRole::Seller) {
        return Err(AppError::Forbidden("Acceso denegado".to_string()));
    }
    validate_create_product_request(&req)?;

    let old_product: Option<Product> = sqlx::query_as(
        "SELECT id, workshop_id, name, description, category, brand, model, sku, barcode, price, cost, \
         stock, min_stock, location, supplier_id, status, created_at, updated_at \
         FROM products WHERE id = $1 AND workshop_id = $2 AND status = 'active'"
    )
    .bind(id)
    .bind(user.workshop_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    let old_product = old_product.ok_or(AppError::NotFound("Product not found".to_string()))?;

    let now = Utc::now();

    sqlx::query(
        "UPDATE products SET \
         name = $2, description = $3, category = $4, brand = $5, model = $6, sku = $7, \
         price = $8, cost = $9, stock = $10, min_stock = $11, location = $12, supplier_id = $13, updated_at = $14 \
         WHERE id = $1 AND workshop_id = $15 AND status = 'active'"
    )
    .bind(id)
    .bind(&req.name)
    .bind(&req.description)
    .bind(&req.category)
    .bind(&req.brand)
    .bind(&req.model)
    .bind(&req.sku)
    .bind(req.price)
    .bind(req.cost)
    .bind(req.stock)
    .bind(req.min_stock)
    .bind(&req.location)
    .bind(req.supplier_id)
    .bind(now)
    .bind(user.workshop_id)
    .execute(&state.pool)
    .await
    .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    let product = Product {
        id,
        workshop_id: old_product.workshop_id,
        name: req.name,
        description: req.description,
        category: req.category,
        brand: req.brand,
        model: req.model,
        sku: req.sku,
        barcode: old_product.barcode.clone(),
        price: req.price,
        cost: req.cost,
        stock: req.stock,
        min_stock: req.min_stock,
        location: req.location,
        supplier_id: req.supplier_id,
        status: old_product.status.clone(),
        created_at: old_product.created_at,
        updated_at: now,
    };

    let mut old_values = serde_json::to_value(&old_product).unwrap_or_default();
    redact_sensitive(&mut old_values);
    let mut new_values = serde_json::to_value(&product).unwrap_or_default();
    redact_sensitive(&mut new_values);

    audit::log_change(
        &state.pool,
        Some(user.id),
        "update",
        "product",
        id,
        Some(old_values),
        Some(new_values),
        None,
        None,
    )
    .await
    .map_err(|e| AppError::Internal(format!("Audit error: {}", e)))?;

    Ok(Json(ApiResponse::success(product)))
}

async fn delete_product(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    if !user.is_admin() {
        return Err(AppError::Forbidden("Acceso denegado".to_string()));
    }
    let old_product: Option<Product> = sqlx::query_as(
        "SELECT id, workshop_id, name, description, category, brand, model, sku, barcode, price, cost, stock, min_stock, \
         location, supplier_id, status, created_at, updated_at \
         FROM products WHERE id = $1 AND status = 'active' AND workshop_id = $2"
    )
    .bind(id)
    .bind(user.workshop_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    let old_product = old_product.ok_or(AppError::NotFound("Product not found".to_string()))?;

    sqlx::query("UPDATE products SET status = 'deleted', updated_at = $2 WHERE id = $1 AND status = 'active' AND workshop_id = $3")
        .bind(id)
        .bind(Utc::now())
        .bind(user.workshop_id)
        .execute(&state.pool)
        .await
        .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    let mut old_values = serde_json::to_value(&old_product).unwrap_or_default();
    redact_sensitive(&mut old_values);

    audit::log_change(
        &state.pool,
        Some(user.id),
        "delete",
        "product",
        id,
        Some(old_values),
        None,
        None,
        None,
    )
    .await
    .map_err(|e| AppError::Internal(format!("Audit error: {}", e)))?;

    Ok(Json(ApiResponse::success(())))
}

async fn lookup_product_by_barcode(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Query(params): Query<PosLookupRequest>,
) -> Result<Json<ApiResponse<PosProductResponse>>, AppError> {
    let barcode = params.barcode.trim();
    if barcode.is_empty() {
        return Err(AppError::Validation("Barcode is required".to_string()));
    }

    type ProductRow = (
        uuid::Uuid,
        String,
        Decimal,
        Decimal,
        i32,
        i32,
        Option<String>,
        Option<String>,
        Option<uuid::Uuid>,
    );

    let product: Option<ProductRow> = sqlx::query_as(
        "SELECT id, name, price, cost, stock, min_stock, barcode, sku, supplier_id \
         FROM products WHERE barcode = $1 AND status = 'active' AND workshop_id = $2",
    )
    .bind(barcode)
    .bind(user.workshop_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    match product {
        Some((id, name, price, cost, stock, min_stock, barcode, sku, supplier_id)) => {
            let response = PosProductResponse {
                product_id: id,
                name,
                price,
                cost,
                stock,
                min_stock,
                barcode,
                sku,
                supplier_id,
            };
            Ok(Json(ApiResponse::success(response)))
        }
        None => Err(AppError::NotFound("Product not found".to_string())),
    }
}

fn validate_create_product_request(req: &CreateProductRequest) -> Result<(), AppError> {
    if req.name.trim().is_empty() || req.name.len() > 200 {
        return Err(AppError::Validation(
            "name must be between 1 and 200 characters".to_string(),
        ));
    }

    if req.price < Decimal::ZERO {
        return Err(AppError::Validation("price must be >= 0".to_string()));
    }

    if req.cost < Decimal::ZERO {
        return Err(AppError::Validation("cost must be >= 0".to_string()));
    }

    if req.stock < 0 {
        return Err(AppError::Validation("stock must be >= 0".to_string()));
    }

    if req.min_stock < 0 {
        return Err(AppError::Validation("min_stock must be >= 0".to_string()));
    }

    if let Some(ref sku) = req.sku {
        if sku.len() > 100 {
            return Err(AppError::Validation(
                "sku must be <= 100 characters".to_string(),
            ));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_create_product_request_valid() {
        let req = CreateProductRequest {
            name: "Oil Filter".to_string(),
            description: None,
            category: None,
            brand: None,
            model: None,
            sku: Some("OIL-123".to_string()),
            barcode: None,
            price: Decimal::new(1500, 2), // 15.00
            cost: Decimal::new(1000, 2),  // 10.00
            stock: 10,
            min_stock: 2,
            location: None,
            supplier_id: None,
        };

        assert!(validate_create_product_request(&req).is_ok());
    }

    #[test]
    fn test_validate_create_product_request_invalid_name() {
        let req = CreateProductRequest {
            name: "".to_string(),
            description: None,
            category: None,
            brand: None,
            model: None,
            sku: None,
            barcode: None,
            price: Decimal::ZERO,
            cost: Decimal::ZERO,
            stock: 0,
            min_stock: 0,
            location: None,
            supplier_id: None,
        };

        assert!(validate_create_product_request(&req).is_err());
    }

    #[test]
    fn test_validate_create_product_request_negative_price() {
        let req = CreateProductRequest {
            name: "Product".to_string(),
            description: None,
            category: None,
            brand: None,
            model: None,
            sku: None,
            barcode: None,
            price: Decimal::new(-1, 0),
            cost: Decimal::ZERO,
            stock: 0,
            min_stock: 0,
            location: None,
            supplier_id: None,
        };

        assert!(validate_create_product_request(&req).is_err());
    }
}

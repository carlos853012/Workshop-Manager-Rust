use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Extension, Json, Router,
};
use chrono::Utc;
use inventory_common::dto::{ApiResponse, CreateSaleRequest, PaginatedResponse};
use inventory_common::{PaymentMethod, Sale, SaleItem};
use rust_decimal::Decimal;
use serde::Deserialize;
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

use crate::audit::{self, redact_sensitive};
use crate::error::AppError;
use crate::middleware::AuthenticatedUser;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_sales))
        .route("/", post(create_sale))
        .route("/:id", get(get_sale))
}

#[derive(Debug, Deserialize)]
struct PaginationParams {
    #[serde(default = "super::products::default_page")]
    page: i32,
    #[serde(default = "super::products::default_per_page")]
    per_page: i32,
}

async fn list_sales(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<ApiResponse<PaginatedResponse<Sale>>>, AppError> {
    if params.page < 1 {
        return Err(AppError::Validation("page must be >= 1".to_string()));
    }
    if params.per_page < 1 || params.per_page > 100 {
        return Err(AppError::Validation(
            "per_page must be between 1 and 100".to_string(),
        ));
    }

    let offset = (params.page - 1) * params.per_page;

    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM sales WHERE status = 'completed'")
        .fetch_one(&state.pool)
        .await
        .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    let items: Vec<Sale> = sqlx::query_as(
        "SELECT id, customer_name, customer_email, customer_phone, total, payment_method, status, created_at \
         FROM sales WHERE status = 'completed' \
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

async fn get_sale(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<SaleDetail>>, AppError> {
    let sale: Option<Sale> = sqlx::query_as(
        "SELECT id, customer_name, customer_email, customer_phone, total, payment_method, status, created_at \
         FROM sales WHERE id = $1 AND status = 'completed'"
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    let sale = sale.ok_or(AppError::NotFound("Sale not found".to_string()))?;

    let items: Vec<SaleItem> = sqlx::query_as(
        "SELECT id, sale_id, product_id, product_name, quantity, unit_price, total \
         FROM sale_items WHERE sale_id = $1",
    )
    .bind(id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    Ok(Json(ApiResponse::success(SaleDetail { sale, items })))
}

#[derive(Debug, serde::Serialize)]
struct SaleDetail {
    #[serde(flatten)]
    sale: Sale,
    items: Vec<SaleItem>,
}

async fn create_sale(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(req): Json<CreateSaleRequest>,
) -> Result<Json<ApiResponse<SaleDetail>>, AppError> {
    validate_create_sale_request(&req)?;

    let payment_method = req.payment_method.clone();

    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    let result = create_sale_in_transaction(&mut tx, &req, payment_method, user.id).await;

    match result {
        Ok(sale_detail) => {
            tx.commit()
                .await
                .map_err(|e| AppError::Internal(format!("Transaction commit error: {}", e)))?;

            let mut new_values = serde_json::to_value(&sale_detail).unwrap_or_default();
            redact_sensitive(&mut new_values);

            audit::log_change(
                &state.pool,
                Some(user.id),
                "create",
                "sale",
                sale_detail.sale.id,
                None,
                Some(new_values),
                None,
                None,
            )
            .await
            .map_err(|e| AppError::Internal(format!("Audit error: {}", e)))?;

            Ok(Json(ApiResponse::success(sale_detail)))
        }
        Err(e) => {
            tx.rollback().await.map_err(|rollback_err| {
                AppError::Internal(format!(
                    "Transaction rollback error: {} (original error: {})",
                    rollback_err, e
                ))
            })?;
            Err(e)
        }
    }
}

async fn create_sale_in_transaction(
    tx: &mut Transaction<'_, Postgres>,
    req: &CreateSaleRequest,
    payment_method: PaymentMethod,
    _user_id: Uuid,
) -> Result<SaleDetail, AppError> {
    if req.items.is_empty() {
        return Err(AppError::Validation(
            "Sale must have at least one item".to_string(),
        ));
    }

    let sale_id = Uuid::new_v4();
    let now = Utc::now();
    let mut sale_total = Decimal::ZERO;
    let mut sale_items = Vec::with_capacity(req.items.len());

    for item_req in &req.items {
        if item_req.quantity <= 0 {
            return Err(AppError::Validation(
                "Item quantity must be positive".to_string(),
            ));
        }

        let product: Option<(String, Decimal, i32)> = sqlx::query_as(
            "SELECT name, price, stock FROM products WHERE id = $1 AND status = 'active' FOR UPDATE"
        )
        .bind(item_req.product_id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

        let (product_name, unit_price, current_stock) = product.ok_or(AppError::NotFound(
            format!("Product {} not found", item_req.product_id),
        ))?;

        if current_stock < item_req.quantity {
            return Err(AppError::Conflict(format!(
                "Insufficient stock for product {}. Available: {}, requested: {}",
                product_name, current_stock, item_req.quantity
            )));
        }

        let item_total = unit_price * Decimal::from(item_req.quantity);
        sale_total += item_total;

        sqlx::query("UPDATE products SET stock = stock - $2, updated_at = $3 WHERE id = $1")
            .bind(item_req.product_id)
            .bind(item_req.quantity)
            .bind(now)
            .execute(&mut **tx)
            .await
            .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

        let sale_item = SaleItem {
            id: Uuid::new_v4(),
            sale_id,
            product_id: item_req.product_id,
            product_name: Some(product_name.clone()),
            quantity: item_req.quantity,
            unit_price,
            total: item_total,
        };

        sale_items.push(sale_item);
    }

    sqlx::query(
        "INSERT INTO sales \
         (id, customer_name, customer_email, customer_phone, total, payment_method, status, created_at) \
         VALUES ($1, $2, $3, $4, $5, $6, 'completed', $7)"
    )
    .bind(sale_id)
    .bind(&req.customer_name)
    .bind(&req.customer_email)
    .bind(&req.customer_phone)
    .bind(sale_total)
    .bind(&payment_method)
    .bind(now)
    .execute(&mut **tx)
    .await
    .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    for item in &sale_items {
        sqlx::query(
            "INSERT INTO sale_items \
             (id, sale_id, product_id, product_name, quantity, unit_price, total) \
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind(item.id)
        .bind(item.sale_id)
        .bind(item.product_id)
        .bind(&item.product_name)
        .bind(item.quantity)
        .bind(item.unit_price)
        .bind(item.total)
        .execute(&mut **tx)
        .await
        .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;
    }

    let sale = Sale {
        id: sale_id,
        customer_name: req.customer_name.clone(),
        customer_email: req.customer_email.clone(),
        customer_phone: req.customer_phone.clone(),
        total: sale_total,
        payment_method,
        status: "completed".to_string(),
        created_at: now,
    };

    Ok(SaleDetail {
        sale,
        items: sale_items,
    })
}

fn validate_create_sale_request(req: &CreateSaleRequest) -> Result<(), AppError> {
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

    if req.items.is_empty() {
        return Err(AppError::Validation(
            "Sale must have at least one item".to_string(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use inventory_common::dto::SaleItemRequest;

    #[test]
    fn test_validate_create_sale_request_valid() {
        let req = CreateSaleRequest {
            customer_name: Some("John Doe".to_string()),
            customer_email: Some("john@example.com".to_string()),
            customer_phone: Some("1234567890".to_string()),
            payment_method: PaymentMethod::Cash,
            items: vec![SaleItemRequest {
                product_id: Uuid::new_v4(),
                quantity: 2,
                unit_price: Decimal::ZERO, // ignorado en validación
            }],
        };

        assert!(validate_create_sale_request(&req).is_ok());
    }

    #[test]
    fn test_validate_create_sale_request_invalid_email() {
        let req = CreateSaleRequest {
            customer_name: None,
            customer_email: Some("invalid".to_string()),
            customer_phone: None,
            payment_method: PaymentMethod::Cash,
            items: vec![SaleItemRequest {
                product_id: Uuid::new_v4(),
                quantity: 1,
                unit_price: Decimal::ZERO,
            }],
        };

        assert!(validate_create_sale_request(&req).is_err());
    }

    #[test]
    fn test_validate_create_sale_request_empty_items() {
        let req = CreateSaleRequest {
            customer_name: None,
            customer_email: None,
            customer_phone: None,
            payment_method: PaymentMethod::Cash,
            items: vec![],
        };

        assert!(validate_create_sale_request(&req).is_err());
    }
}

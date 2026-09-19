use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Extension, Json, Router,
};
use chrono::Utc;
use rust_decimal::Decimal;
use sqlx::{Postgres, Transaction};
use uuid::Uuid;
use workshop_common::dto::{ApiResponse, CreateSaleRequest, PaginatedResponse};
use workshop_common::{PaymentMethod, Sale, SaleItem, UserRole};

use crate::audit::{self, redact_sensitive};
use crate::error::AppError;
use crate::middleware::AuthenticatedUser;
use crate::validation::validate_email;
use crate::state::AppState;

use super::pagination::PaginationParams;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_sales))
        .route("/", post(create_sale))
        .route("/:id", get(get_sale))
        .route("/:id/cancel", post(cancel_sale))
}

async fn list_sales(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<ApiResponse<PaginatedResponse<Sale>>>, AppError> {
    params.validate().map_err(AppError::Validation)?;

    let offset = params.offset();

    let total: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sales WHERE status = 'completed' AND workshop_id = $1",
    )
    .bind(user.workshop_id)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    let items: Vec<Sale> = sqlx::query_as(
        "SELECT id, workshop_id, customer_name, customer_email, customer_phone, subtotal, discount_amount, taxable_amount, tax_amount, total, payment_method, status, created_at \
         FROM sales WHERE status = 'completed' AND workshop_id = $1 \
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

async fn get_sale(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<SaleDetail>>, AppError> {
    let sale: Option<Sale> = sqlx::query_as(
        "SELECT id, workshop_id, customer_name, customer_email, customer_phone, subtotal, discount_amount, taxable_amount, tax_amount, total, payment_method, status, created_at \
         FROM sales WHERE id = $1 AND status = 'completed' AND workshop_id = $2"
    )
    .bind(id)
    .bind(user.workshop_id)
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

async fn cancel_sale(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<Sale>>, AppError> {
    if !matches!(user.role, UserRole::Admin | UserRole::Seller) {
        return Err(AppError::Forbidden("Acceso denegado".to_string()));
    }

    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    let sale: Option<Sale> = sqlx::query_as(
        "SELECT id, workshop_id, customer_name, customer_email, customer_phone, subtotal, discount_amount, taxable_amount, tax_amount, total, payment_method, status, created_at \
         FROM sales WHERE id = $1 AND workshop_id = $2 FOR UPDATE"
    )
    .bind(id)
    .bind(user.workshop_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    let sale = sale.ok_or(AppError::NotFound("Sale not found".to_string()))?;

    if sale.status == "cancelled" {
        return Err(AppError::Conflict("Sale is already cancelled".to_string()));
    }

    // Restore stock for each item
    let items: Vec<SaleItem> = sqlx::query_as(
        "SELECT id, sale_id, product_id, product_name, quantity, unit_price, total \
         FROM sale_items WHERE sale_id = $1",
    )
    .bind(id)
    .fetch_all(&mut *tx)
    .await
    .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    let now = Utc::now();
    for item in &items {
        sqlx::query(
            "UPDATE products SET stock = stock + $2, updated_at = $3 \
             WHERE id = $1 AND workshop_id = $4",
        )
        .bind(item.product_id)
        .bind(item.quantity)
        .bind(now)
        .bind(user.workshop_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;
    }

    sqlx::query("UPDATE sales SET status = 'cancelled' WHERE id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    tx.commit()
        .await
        .map_err(|e| AppError::Internal(format!("Transaction commit error: {}", e)))?;

    let mut old_values = serde_json::to_value(&sale).unwrap_or_default();
    redact_sensitive(&mut old_values);

    if let Err(e) = audit::log_change(
        &state.pool,
        Some(user.id),
        "cancel",
        "sale",
        id,
        Some(old_values),
        None,
        None,
        None,
    )
    .await
    {
        tracing::warn!("Audit log failed for sale cancel {}: {}", id, e);
    }

    let cancelled = Sale {
        status: "cancelled".to_string(),
        ..sale
    };

    Ok(Json(ApiResponse::success(cancelled)))
}

async fn create_sale(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(req): Json<CreateSaleRequest>,
) -> Result<Json<ApiResponse<SaleDetail>>, AppError> {
    if !matches!(user.role, UserRole::Admin | UserRole::Seller) {
        return Err(AppError::Forbidden("Acceso denegado".to_string()));
    }
    validate_create_sale_request(&req)?;

    let payment_method = req.payment_method.clone();

    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    let result =
        create_sale_in_transaction(&mut tx, &req, payment_method, user.id, user.workshop_id).await;

    match result {
        Ok(sale_detail) => {
            tx.commit()
                .await
                .map_err(|e| AppError::Internal(format!("Transaction commit error: {}", e)))?;

            let mut new_values = serde_json::to_value(&sale_detail).unwrap_or_default();
            redact_sensitive(&mut new_values);

            // Audit is best-effort: don't fail the request if audit fails
            if let Err(e) = audit::log_change(
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
            {
                tracing::warn!("Audit log failed for sale {}: {}", sale_detail.sale.id, e);
            }

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
    workshop_id: Uuid,
) -> Result<SaleDetail, AppError> {
    if req.items.is_empty() {
        return Err(AppError::Validation(
            "Sale must have at least one item".to_string(),
        ));
    }

    let sale_id = Uuid::new_v4();
    let now = Utc::now();
    let mut sale_subtotal = Decimal::ZERO;
    let mut sale_items = Vec::with_capacity(req.items.len());

    let discount_amount = req.discount_amount.unwrap_or(Decimal::ZERO);

    // Sort items by product_id to prevent deadlocks (consistent lock ordering)
    let mut sorted_items = req.items.clone();
    sorted_items.sort_by_key(|a| a.product_id);

    for item_req in &sorted_items {
        if item_req.quantity <= 0 {
            return Err(AppError::Validation(
                "Item quantity must be positive".to_string(),
            ));
        }

        let product: Option<(String, Decimal, i32)> = sqlx::query_as(
            "SELECT name, price, stock FROM products WHERE id = $1 AND status = 'active' AND workshop_id = $2 FOR UPDATE"
        )
        .bind(item_req.product_id)
        .bind(workshop_id)
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

        let item_discount = item_req.discount.unwrap_or(Decimal::ZERO);
        let item_subtotal = unit_price * Decimal::from(item_req.quantity);
        let item_discount_amount = item_subtotal * item_discount / Decimal::from(100);
        let item_total = item_subtotal - item_discount_amount;
        // En Chile el precio ya incluye IVA — extraer base e IVA (solo para registro)
        let (_item_base, _item_tax) = workshop_common::money::extract_iva(item_total);

        sale_subtotal += item_subtotal;

        sqlx::query("UPDATE products SET stock = stock - $2, updated_at = $4 WHERE id = $1 AND workshop_id = $3")
            .bind(item_req.product_id)
            .bind(item_req.quantity)
            .bind(workshop_id)
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

    let discount_amount_total = (sale_subtotal * discount_amount) / Decimal::from(100);
    let total = sale_subtotal - discount_amount_total;
    // En Chile el precio ya incluye IVA — extraer base e IVA del total
    let (taxable_amount, tax_amount) = workshop_common::money::extract_iva(total);

    sqlx::query(
        "INSERT INTO sales \
         (id, workshop_id, customer_name, customer_email, customer_phone, subtotal, discount_amount, taxable_amount, tax_amount, total, payment_method, status, created_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, 'completed', $12)"
    )
    .bind(sale_id)
    .bind(workshop_id)
    .bind(&req.customer_name)
    .bind(&req.customer_email)
    .bind(&req.customer_phone)
    .bind(sale_subtotal)
    .bind(discount_amount_total)
    .bind(taxable_amount)
    .bind(tax_amount)
    .bind(total)
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
        workshop_id,
        customer_name: req.customer_name.clone(),
        customer_email: req.customer_email.clone(),
        customer_phone: req.customer_phone.clone(),
        subtotal: sale_subtotal,
        discount_amount: discount_amount_total,
        taxable_amount,
        tax_amount,
        total,
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
        validate_email(email)?;
    }

    if let Some(ref phone) = req.customer_phone {
        if phone.len() > 20 {
            return Err(AppError::Validation(
                "customer_phone must be <= 20 characters".to_string(),
            ));
        }
    }

    if let Some(discount) = req.discount_amount {
        if discount < Decimal::ZERO || discount > Decimal::from(100) {
            return Err(AppError::Validation(
                "discount_amount must be between 0 and 100".to_string(),
            ));
        }
    }

    for (i, item) in req.items.iter().enumerate() {
        if item.quantity == 0 {
            return Err(AppError::Validation(format!(
                "Item {} quantity must be greater than 0",
                i + 1
            )));
        }
        if item.unit_price < Decimal::ZERO {
            return Err(AppError::Validation(format!(
                "Item {} unit_price cannot be negative",
                i + 1
            )));
        }
        if let Some(discount) = item.discount {
            if discount < Decimal::ZERO || discount > Decimal::from(100) {
                return Err(AppError::Validation(format!(
                    "Item {} discount must be between 0 and 100",
                    i + 1
                )));
            }
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
    use workshop_common::dto::SaleItemRequest;

    #[test]
    fn test_validate_create_sale_request_valid() {
        let req = CreateSaleRequest {
            customer_name: Some("John Doe".to_string()),
            customer_email: Some("john@example.com".to_string()),
            customer_phone: Some("1234567890".to_string()),
            payment_method: PaymentMethod::Cash,
            discount_amount: None,
            items: vec![SaleItemRequest {
                product_id: Uuid::new_v4(),
                quantity: 2,
                unit_price: Decimal::ZERO,
                discount: None,
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
            discount_amount: None,
            items: vec![SaleItemRequest {
                product_id: Uuid::new_v4(),
                quantity: 1,
                unit_price: Decimal::ZERO,
                discount: None,
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
            discount_amount: None,
            items: vec![],
        };

        assert!(validate_create_sale_request(&req).is_err());
    }

    #[test]
    fn test_validate_sale_discount_out_of_range() {
        let req = CreateSaleRequest {
            customer_name: None,
            customer_email: None,
            customer_phone: None,
            payment_method: PaymentMethod::Cash,
            discount_amount: Some(Decimal::from(150)),
            items: vec![SaleItemRequest {
                product_id: Uuid::new_v4(),
                quantity: 1,
                unit_price: Decimal::from(10000),
                discount: None,
            }],
        };
        assert!(validate_create_sale_request(&req).is_err());
    }

    #[test]
    fn test_validate_sale_item_discount_out_of_range() {
        let req = CreateSaleRequest {
            customer_name: None,
            customer_email: None,
            customer_phone: None,
            payment_method: PaymentMethod::Cash,
            discount_amount: None,
            items: vec![SaleItemRequest {
                product_id: Uuid::new_v4(),
                quantity: 1,
                unit_price: Decimal::from(10000),
                discount: Some(Decimal::from(-10)),
            }],
        };
        assert!(validate_create_sale_request(&req).is_err());
    }

    #[test]
    fn test_validate_sale_zero_quantity() {
        let req = CreateSaleRequest {
            customer_name: None,
            customer_email: None,
            customer_phone: None,
            payment_method: PaymentMethod::Cash,
            discount_amount: None,
            items: vec![SaleItemRequest {
                product_id: Uuid::new_v4(),
                quantity: 0,
                unit_price: Decimal::from(10000),
                discount: None,
            }],
        };
        assert!(validate_create_sale_request(&req).is_err());
    }

    #[test]
    fn test_validate_sale_negative_unit_price() {
        let req = CreateSaleRequest {
            customer_name: None,
            customer_email: None,
            customer_phone: None,
            payment_method: PaymentMethod::Cash,
            discount_amount: None,
            items: vec![SaleItemRequest {
                product_id: Uuid::new_v4(),
                quantity: 1,
                unit_price: Decimal::from(-5000),
                discount: None,
            }],
        };
        assert!(validate_create_sale_request(&req).is_err());
    }

    #[test]
    fn test_validate_sale_valid_with_discounts() {
        let req = CreateSaleRequest {
            customer_name: None,
            customer_email: None,
            customer_phone: None,
            payment_method: PaymentMethod::Cash,
            discount_amount: Some(Decimal::from(10)),
            items: vec![SaleItemRequest {
                product_id: Uuid::new_v4(),
                quantity: 2,
                unit_price: Decimal::from(15000),
                discount: Some(Decimal::from(5)),
            }],
        };
        assert!(validate_create_sale_request(&req).is_ok());
    }
}

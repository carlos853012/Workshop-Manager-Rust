use axum::{
    extract::{Query, State},
    http::header,
    response::IntoResponse,
    routing::get,
    Extension, Json, Router,
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use workshop_common::dto::ApiResponse;
use workshop_common::{PaymentMethod, RepairStatus};

use crate::error::AppError;
use crate::middleware::AuthenticatedUser;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/clients", get(report_clients))
        .route("/client-history", get(client_history))
        .route("/client-search", get(client_search))
        .route("/client-certificate", get(client_certificate))
        .route("/client-certificate.pdf", get(download_certificate))
}

#[derive(Debug, Serialize)]
struct ClientReport {
    customer_email: String,
    customer_name: Option<String>,
    total_purchases: i64,
    total_spent: Decimal,
    last_purchase: Option<chrono::DateTime<chrono::Utc>>,
}

async fn report_clients(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
) -> Result<Json<ApiResponse<Vec<ClientReport>>>, AppError> {
    let wid = user.workshop_id;

    let rows = sqlx::query(
        "SELECT \
            customer_email, \
            MAX(customer_name) as customer_name, \
            COUNT(*) as total_purchases, \
            COALESCE(SUM(total), 0) as total_spent, \
            MAX(created_at) as last_purchase \
         FROM sales \
         WHERE customer_email IS NOT NULL AND status = 'completed' AND workshop_id = $1 \
         GROUP BY customer_email \
         ORDER BY total_spent DESC",
    )
    .bind(wid)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    let reports: Vec<ClientReport> = rows
        .into_iter()
        .map(|row| ClientReport {
            customer_email: row.try_get("customer_email").unwrap_or_default(),
            customer_name: row.try_get("customer_name").ok(),
            total_purchases: row.try_get("total_purchases").unwrap_or(0),
            total_spent: row.try_get("total_spent").unwrap_or(Decimal::ZERO),
            last_purchase: row.try_get("last_purchase").ok(),
        })
        .collect();

    Ok(Json(ApiResponse::success(reports)))
}

#[derive(Debug, Deserialize)]
struct ClientHistoryQuery {
    email: String,
}

#[derive(Debug, Serialize)]
struct ClientSaleRecord {
    id: uuid::Uuid,
    total: Decimal,
    payment_method: PaymentMethod,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
struct ClientRepairRecord {
    id: uuid::Uuid,
    description: Option<String>,
    status: RepairStatus,
    total: Option<Decimal>,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
struct ClientHistoryResponse {
    email: String,
    name: Option<String>,
    sales: Vec<ClientSaleRecord>,
    repairs: Vec<ClientRepairRecord>,
    total_spent: Decimal,
    total_repairs: i64,
}

async fn client_history(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Query(params): Query<ClientHistoryQuery>,
) -> Result<Json<ApiResponse<ClientHistoryResponse>>, AppError> {
    let wid = user.workshop_id;

    let sales_rows = sqlx::query(
        "SELECT id, total, payment_method, created_at \
         FROM sales \
         WHERE customer_email = $1 AND status = 'completed' AND workshop_id = $2 \
         ORDER BY created_at DESC",
    )
    .bind(&params.email)
    .bind(wid)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    let sales: Vec<ClientSaleRecord> = sales_rows
        .into_iter()
        .map(|row| ClientSaleRecord {
            id: row.try_get("id").unwrap_or_default(),
            total: row.try_get("total").unwrap_or(Decimal::ZERO),
            payment_method: row.try_get("payment_method").unwrap_or_default(),
            created_at: row.try_get("created_at").unwrap_or_default(),
        })
        .collect();

    let repairs_rows = sqlx::query(
        "SELECT id, description, status, final_cost AS total, created_at \
         FROM repairs \
         WHERE customer_email = $1 AND workshop_id = $2 \
         ORDER BY created_at DESC",
    )
    .bind(&params.email)
    .bind(wid)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    let repairs: Vec<ClientRepairRecord> = repairs_rows
        .into_iter()
        .map(|row| ClientRepairRecord {
            id: row.try_get("id").unwrap_or_default(),
            description: row.try_get("description").ok(),
            status: row.try_get("status").unwrap_or_default(),
            total: row.try_get("total").ok(),
            created_at: row.try_get("created_at").unwrap_or_default(),
        })
        .collect();

    let total_spent: Decimal = sales.iter().map(|s| s.total).sum();
    let total_repairs = repairs.len() as i64;

    let name: Option<String> = sqlx::query_scalar(
        "SELECT MAX(customer_name) FROM sales WHERE customer_email = $1 AND workshop_id = $2",
    )
    .bind(&params.email)
    .bind(wid)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?
    .flatten();

    Ok(Json(ApiResponse::success(ClientHistoryResponse {
        email: params.email,
        name,
        sales,
        repairs,
        total_spent,
        total_repairs,
    })))
}

#[derive(Debug, Deserialize)]
pub struct ClientSearchQuery {
    pub query: String,
}

#[derive(Debug, Serialize)]
pub struct VehicleSummary {
    pub license_plate: Option<String>,
    pub vehicle: Option<String>,
    pub total_repairs: i64,
    pub last_repair_date: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Serialize)]
pub struct ClientSearchResult {
    pub customer_name: Option<String>,
    pub customer_email: Option<String>,
    pub customer_phone: Option<String>,
    pub vehicles: Vec<VehicleSummary>,
}

async fn client_search(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Query(params): Query<ClientSearchQuery>,
) -> Result<Json<ApiResponse<Vec<ClientSearchResult>>>, AppError> {
    let wid = user.workshop_id;
    let q = format!("%{}%", params.query);
    tracing::info!(
        "client_search: query='{}', workshop_id={}",
        params.query,
        wid
    );

    let customer_rows = sqlx::query(
        "SELECT DISTINCT customer_name, customer_email, customer_phone \
         FROM repairs \
         WHERE (license_plate ILIKE $1 OR customer_name ILIKE $1 OR customer_email ILIKE $1) \
           AND workshop_id = $2 AND status != 'deleted' \
         ORDER BY customer_name",
    )
    .bind(&q)
    .bind(wid)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    let mut results = Vec::new();

    for row in customer_rows {
        let email: Option<String> = row.try_get("customer_email").ok().flatten();
        let name: Option<String> = row.try_get("customer_name").ok().flatten();
        let phone: Option<String> = row.try_get("customer_phone").ok().flatten();

        let vehicles = if let Some(ref email_val) = email {
            let vehicle_rows = sqlx::query(
                "SELECT DISTINCT vehicle, license_plate, \
                        COUNT(*) as total_repairs, \
                        MAX(created_at) as last_repair_date \
                 FROM repairs \
                 WHERE customer_email = $1 AND workshop_id = $2 AND status != 'deleted' \
                 GROUP BY vehicle, license_plate \
                 ORDER BY last_repair_date DESC",
            )
            .bind(email_val)
            .bind(wid)
            .fetch_all(&state.pool)
            .await
            .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

            vehicle_rows
                .into_iter()
                .map(|v| VehicleSummary {
                    license_plate: v.try_get("license_plate").ok().flatten(),
                    vehicle: v.try_get("vehicle").ok().flatten(),
                    total_repairs: v.try_get("total_repairs").unwrap_or(0),
                    last_repair_date: v.try_get("last_repair_date").ok(),
                })
                .collect()
        } else if let Some(ref name_val) = name {
            let vehicle_rows = sqlx::query(
                "SELECT DISTINCT vehicle, license_plate, \
                        COUNT(*) as total_repairs, \
                        MAX(created_at) as last_repair_date \
                 FROM repairs \
                 WHERE customer_name = $1 AND workshop_id = $2 AND status != 'deleted' \
                 GROUP BY vehicle, license_plate \
                 ORDER BY last_repair_date DESC",
            )
            .bind(name_val)
            .bind(wid)
            .fetch_all(&state.pool)
            .await
            .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

            vehicle_rows
                .into_iter()
                .map(|v| VehicleSummary {
                    license_plate: v.try_get("license_plate").ok().flatten(),
                    vehicle: v.try_get("vehicle").ok().flatten(),
                    total_repairs: v.try_get("total_repairs").unwrap_or(0),
                    last_repair_date: v.try_get("last_repair_date").ok(),
                })
                .collect()
        } else {
            Vec::new()
        };

        results.push(ClientSearchResult {
            customer_name: name,
            customer_email: email,
            customer_phone: phone,
            vehicles,
        });
    }

    Ok(Json(ApiResponse::success(results)))
}

#[derive(Debug, Deserialize)]
pub struct CertificateQuery {
    pub email: Option<String>,
    pub name: Option<String>,
    pub plate: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct WorkshopInfo {
    pub name: String,
    pub address: String,
    pub city: String,
}

#[derive(Debug, Serialize)]
pub struct ServiceEntry {
    pub date: chrono::DateTime<chrono::Utc>,
    pub description: Option<String>,
    pub diagnosis: Option<String>,
    pub status: RepairStatus,
}

#[derive(Debug, Serialize)]
pub struct PartEntry {
    pub name: String,
    pub quantity: Decimal,
    pub product_brand: Option<String>,
    pub product_model: Option<String>,
    pub product_sku: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ServiceCertificateResponse {
    pub workshop: WorkshopInfo,
    pub client: ClientInfo,
    pub vehicle: VehicleInfo,
    pub services: Vec<ServiceEntry>,
    pub parts_used: Vec<PartEntry>,
    pub generated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct ClientInfo {
    pub name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct VehicleInfo {
    pub description: Option<String>,
    pub license_plate: Option<String>,
}

async fn fetch_repairs_for_certificate(
    pool: &sqlx::PgPool,
    email: &Option<String>,
    name: &Option<String>,
    plate: &Option<String>,
    wid: uuid::Uuid,
) -> Result<Vec<sqlx::postgres::PgRow>, sqlx::Error> {
    let query_sql = match (email, name, plate) {
        (Some(e), _, Some(p)) if !e.is_empty() => sqlx::query(
            "SELECT r.customer_name, r.customer_email, r.customer_phone, \
                        r.vehicle, r.license_plate, r.description, r.diagnosis, \
                        r.status, r.created_at \
                 FROM repairs r \
                 WHERE r.customer_email = $1 AND r.license_plate = $2 \
                   AND r.workshop_id = $3 AND r.status != 'deleted' \
                 ORDER BY r.created_at DESC",
        )
        .bind(e)
        .bind(p)
        .bind(wid),
        (Some(e), _, _) if !e.is_empty() => sqlx::query(
            "SELECT r.customer_name, r.customer_email, r.customer_phone, \
                        r.vehicle, r.license_plate, r.description, r.diagnosis, \
                        r.status, r.created_at \
                 FROM repairs r \
                 WHERE r.customer_email = $1 AND r.workshop_id = $2 \
                   AND r.status != 'deleted' \
                 ORDER BY r.created_at DESC",
        )
        .bind(e)
        .bind(wid),
        (_, Some(n), Some(p)) => sqlx::query(
            "SELECT r.customer_name, r.customer_email, r.customer_phone, \
                        r.vehicle, r.license_plate, r.description, r.diagnosis, \
                        r.status, r.created_at \
                 FROM repairs r \
                 WHERE r.customer_name = $1 AND r.license_plate = $2 \
                   AND r.workshop_id = $3 AND r.status != 'deleted' \
                 ORDER BY r.created_at DESC",
        )
        .bind(n)
        .bind(p)
        .bind(wid),
        (_, Some(n), _) => sqlx::query(
            "SELECT r.customer_name, r.customer_email, r.customer_phone, \
                        r.vehicle, r.license_plate, r.description, r.diagnosis, \
                        r.status, r.created_at \
                 FROM repairs r \
                 WHERE r.customer_name = $1 AND r.workshop_id = $2 \
                   AND r.status != 'deleted' \
                 ORDER BY r.created_at DESC",
        )
        .bind(n)
        .bind(wid),
        (_, _, Some(p)) => sqlx::query(
            "SELECT r.customer_name, r.customer_email, r.customer_phone, \
                        r.vehicle, r.license_plate, r.description, r.diagnosis, \
                        r.status, r.created_at \
                 FROM repairs r \
                 WHERE r.license_plate = $1 AND r.workshop_id = $2 \
                   AND r.status != 'deleted' \
                 ORDER BY r.created_at DESC",
        )
        .bind(p)
        .bind(wid),
        _ => {
            return Ok(Vec::new());
        }
    };
    query_sql.fetch_all(pool).await
}

async fn client_certificate(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Query(params): Query<CertificateQuery>,
) -> Result<Json<ApiResponse<ServiceCertificateResponse>>, AppError> {
    let wid = user.workshop_id;

    let workshop_row = sqlx::query("SELECT name, address, city FROM workshops WHERE id = $1")
        .bind(wid)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?
        .ok_or_else(|| AppError::Internal("Workshop not found".to_string()))?;

    let workshop = WorkshopInfo {
        name: workshop_row.try_get("name").unwrap_or_default(),
        address: workshop_row.try_get("address").unwrap_or_default(),
        city: workshop_row.try_get("city").unwrap_or_default(),
    };

    let repair_rows =
        fetch_repairs_for_certificate(&state.pool, &params.email, &params.name, &params.plate, wid)
            .await
            .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    if repair_rows.is_empty() {
        return Err(AppError::Internal(
            "No repairs found for this client".to_string(),
        ));
    }

    let first = &repair_rows[0];
    let client_info = ClientInfo {
        name: first.try_get("customer_name").ok().flatten(),
        email: first.try_get("customer_email").ok().flatten(),
        phone: first.try_get("customer_phone").ok().flatten(),
    };
    let vehicle = VehicleInfo {
        description: first.try_get("vehicle").ok().flatten(),
        license_plate: first.try_get("license_plate").ok().flatten(),
    };

    let mut services = Vec::new();
    let mut repair_ids = Vec::new();

    for row in &repair_rows {
        let repair_id: uuid::Uuid = row.try_get("id").unwrap_or_default();
        repair_ids.push(repair_id);
        services.push(ServiceEntry {
            date: row.try_get("created_at").unwrap_or_default(),
            description: row.try_get("description").ok().flatten(),
            diagnosis: row.try_get("diagnosis").ok().flatten(),
            status: row.try_get("status").unwrap_or_default(),
        });
    }

    let parts_used = if repair_ids.is_empty() {
        Vec::new()
    } else {
        let parts_rows = sqlx::query(
            "SELECT rp.name, rp.quantity, \
                    p.brand as product_brand, \
                    p.model as product_model, \
                    p.sku as product_sku \
             FROM repair_parts rp \
             LEFT JOIN products p ON rp.product_id = p.id \
             WHERE rp.repair_id = ANY($1) \
             ORDER BY rp.name",
        )
        .bind(&repair_ids)
        .fetch_all(&state.pool)
        .await
        .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

        parts_rows
            .into_iter()
            .map(|r| PartEntry {
                name: r.try_get("name").unwrap_or_default(),
                quantity: r.try_get("quantity").unwrap_or(Decimal::ONE),
                product_brand: r.try_get("product_brand").ok().flatten(),
                product_model: r.try_get("product_model").ok().flatten(),
                product_sku: r.try_get("product_sku").ok().flatten(),
            })
            .collect()
    };

    Ok(Json(ApiResponse::success(ServiceCertificateResponse {
        workshop,
        client: client_info,
        vehicle,
        services,
        parts_used,
        generated_at: chrono::Utc::now(),
    })))
}

async fn download_certificate(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Query(params): Query<CertificateQuery>,
) -> Result<impl IntoResponse, AppError> {
    let wid = user.workshop_id;

    let workshop_row = sqlx::query("SELECT name, address, city FROM workshops WHERE id = $1")
        .bind(wid)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?
        .ok_or_else(|| AppError::Internal("Workshop not found".to_string()))?;

    let workshop = WorkshopInfo {
        name: workshop_row.try_get("name").unwrap_or_default(),
        address: workshop_row.try_get("address").unwrap_or_default(),
        city: workshop_row.try_get("city").unwrap_or_default(),
    };

    let repair_rows =
        fetch_repairs_for_certificate(&state.pool, &params.email, &params.name, &params.plate, wid)
            .await
            .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    if repair_rows.is_empty() {
        return Err(AppError::Internal(
            "No repairs found for this client".to_string(),
        ));
    }

    let first = &repair_rows[0];
    let client_info = ClientInfo {
        name: first.try_get("customer_name").ok().flatten(),
        email: first.try_get("customer_email").ok().flatten(),
        phone: first.try_get("customer_phone").ok().flatten(),
    };
    let vehicle = VehicleInfo {
        description: first.try_get("vehicle").ok().flatten(),
        license_plate: first.try_get("license_plate").ok().flatten(),
    };

    let mut services = Vec::new();
    let mut repair_ids = Vec::new();

    for row in &repair_rows {
        let repair_id: uuid::Uuid = row.try_get("id").unwrap_or_default();
        repair_ids.push(repair_id);
        services.push(ServiceEntry {
            date: row.try_get("created_at").unwrap_or_default(),
            description: row.try_get("description").ok().flatten(),
            diagnosis: row.try_get("diagnosis").ok().flatten(),
            status: row.try_get("status").unwrap_or_default(),
        });
    }

    let parts_used = if repair_ids.is_empty() {
        Vec::new()
    } else {
        let parts_rows = sqlx::query(
            "SELECT rp.name, rp.quantity, \
                    p.brand as product_brand, \
                    p.model as product_model, \
                    p.sku as product_sku \
             FROM repair_parts rp \
             LEFT JOIN products p ON rp.product_id = p.id \
             WHERE rp.repair_id = ANY($1) \
             ORDER BY rp.name",
        )
        .bind(&repair_ids)
        .fetch_all(&state.pool)
        .await
        .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

        parts_rows
            .into_iter()
            .map(|r| PartEntry {
                name: r.try_get("name").unwrap_or_default(),
                quantity: r.try_get("quantity").unwrap_or(Decimal::ONE),
                product_brand: r.try_get("product_brand").ok().flatten(),
                product_model: r.try_get("product_model").ok().flatten(),
                product_sku: r.try_get("product_sku").ok().flatten(),
            })
            .collect()
    };

    let cert = ServiceCertificateResponse {
        workshop,
        client: client_info,
        vehicle,
        services,
        parts_used,
        generated_at: chrono::Utc::now(),
    };

    let pdf_bytes =
        crate::certificate::generate_certificate_pdf(&cert).map_err(AppError::Internal)?;

    let filename = format!(
        "certificado_{}.pdf",
        cert.vehicle.license_plate.as_deref().unwrap_or("servicio")
    );

    let headers = [
        (header::CONTENT_TYPE, "application/pdf".to_string()),
        (
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", filename),
        ),
    ];

    Ok((headers, pdf_bytes))
}

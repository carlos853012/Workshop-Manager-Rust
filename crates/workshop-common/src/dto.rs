use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

// ==================== REQUESTS ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub workshop_name: String,
    pub workshop_address: String,
    pub workshop_city: String,
    pub admin_name: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProductRequest {
    pub name: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub brand: Option<String>,
    pub model: Option<String>,
    pub sku: Option<String>,
    pub barcode: Option<String>,
    pub price: Decimal,
    pub cost: Decimal,
    pub stock: i32,
    pub min_stock: i32,
    pub location: Option<String>,
    pub supplier_id: Option<uuid::Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSaleRequest {
    pub customer_name: Option<String>,
    pub customer_email: Option<String>,
    pub customer_phone: Option<String>,
    pub payment_method: crate::PaymentMethod,
    pub discount_amount: Option<Decimal>,
    pub items: Vec<SaleItemRequest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaleItemRequest {
    pub product_id: uuid::Uuid,
    pub quantity: i32,
    pub unit_price: Decimal,
    pub discount: Option<Decimal>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SaleDetailResponse {
    #[serde(flatten)]
    pub sale: crate::Sale,
    pub items: Vec<crate::SaleItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PosLookupRequest {
    pub barcode: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PosProductResponse {
    pub product_id: uuid::Uuid,
    pub name: String,
    pub price: Decimal,
    pub cost: Decimal,
    pub stock: i32,
    pub min_stock: i32,
    pub barcode: Option<String>,
    pub sku: Option<String>,
    pub supplier_id: Option<uuid::Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PosSaleDetail {
    pub product_id: uuid::Uuid,
    pub name: String,
    pub quantity: i32,
    pub unit_price: Decimal,
    pub discount: Decimal,
    pub subtotal: Decimal,
    pub tax_amount: Decimal,
    pub total: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PosSaleResponse {
    pub items: Vec<PosSaleDetail>,
    pub subtotal: Decimal,
    pub discount_amount: Decimal,
    pub taxable_amount: Decimal,
    pub tax_amount: Decimal,
    pub total: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRepairRequest {
    pub customer_name: Option<String>,
    pub customer_email: Option<String>,
    pub customer_phone: Option<String>,
    pub vehicle: Option<String>,
    pub license_plate: Option<String>,
    pub description: Option<String>,
    pub priority: crate::Priority,
    pub estimated_cost: Option<Decimal>,
    pub estimated_delivery: Option<chrono::NaiveDate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddRepairPartRequest {
    pub name: String,
    pub quantity: Decimal,
    pub unit_cost: Option<Decimal>,
    pub product_id: Option<uuid::Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepairPartResponse {
    pub id: uuid::Uuid,
    pub repair_id: uuid::Uuid,
    pub name: String,
    pub quantity: Decimal,
    pub unit_cost: Option<Decimal>,
    pub total_cost: Option<Decimal>,
    pub product_id: Option<uuid::Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RepairDetail {
    #[serde(flatten)]
    pub repair: crate::Repair,
    pub updates: Vec<crate::RepairUpdate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateRepairRequest {
    pub status: Option<crate::RepairStatus>,
    pub diagnosis: Option<String>,
    pub technician_id: Option<uuid::Uuid>,
    pub estimated_cost: Option<Decimal>,
    pub final_cost: Option<Decimal>,
    pub labor_cost: Option<Decimal>,
    pub estimated_delivery: Option<chrono::NaiveDate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSupplierRequest {
    pub name: String,
    pub contact_person: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub tax_id: Option<String>,
    pub payment_terms: Option<String>,
}

// ==================== RESPONSES ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(error: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: crate::User,
    pub workshop: Option<crate::Workshop>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardResponse {
    pub total_products: i64,
    pub low_stock: i64,
    pub total_sales: i64,
    pub total_revenue: Decimal,
    pub pending_repairs: i64,
    pub completed_repairs: i64,
    pub average_sale: Decimal,
}

// ==================== ANALYTICS / KPIs ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KpisResponse {
    pub total_suppliers: i64,
    pub total_customers: i64,
    pub in_progress_repairs: i64,
    pub cancelled_repairs: i64,
}

// ==================== REPORTS ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientReport {
    pub customer_email: String,
    pub customer_name: Option<String>,
    pub total_purchases: i64,
    pub total_spent: Decimal,
    pub last_purchase: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientSaleRecord {
    pub id: uuid::Uuid,
    pub total: Decimal,
    pub payment_method: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientRepairRecord {
    pub id: uuid::Uuid,
    pub description: Option<String>,
    pub status: String,
    pub total: Option<Decimal>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientHistoryResponse {
    pub email: String,
    pub name: Option<String>,
    pub sales: Vec<ClientSaleRecord>,
    pub repairs: Vec<ClientRepairRecord>,
    pub total_spent: Decimal,
    pub total_repairs: i64,
}

// ==================== USER MANAGEMENT ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub email: String,
    pub display_name: Option<String>,
    pub password: String,
    pub role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUserRequest {
    pub display_name: Option<String>,
    pub role: Option<String>,
    pub status: Option<String>,
}

// ==================== SERVICE CERTIFICATE ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientSearchResult {
    pub customer_name: Option<String>,
    pub customer_email: Option<String>,
    pub customer_phone: Option<String>,
    pub vehicles: Vec<VehicleSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleSummary {
    pub license_plate: Option<String>,
    pub vehicle: Option<String>,
    pub total_repairs: i64,
    pub last_repair_date: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceCertificate {
    pub workshop: WorkshopInfo,
    pub client: ClientInfo,
    pub vehicle: VehicleInfo,
    pub services: Vec<ServiceEntry>,
    pub parts_used: Vec<PartEntry>,
    pub generated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkshopInfo {
    pub name: String,
    pub address: String,
    pub city: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInfo {
    pub name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleInfo {
    pub description: Option<String>,
    pub license_plate: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEntry {
    pub date: chrono::DateTime<chrono::Utc>,
    pub description: Option<String>,
    pub diagnosis: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartEntry {
    pub name: String,
    pub quantity: Decimal,
    pub product_brand: Option<String>,
    pub product_model: Option<String>,
    pub product_sku: Option<String>,
}

// ==================== DEVICE KEYS ====================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeviceKeySummary {
    pub id: uuid::Uuid,
    pub bound_ip: Option<String>,
    pub active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_seen_at: Option<chrono::DateTime<chrono::Utc>>,
}

// ==================== ANALYTICS / REVENUE CHART ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevenueDataPoint {
    pub period: String,
    pub sales: Decimal,
    pub repairs: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevenueResponse {
    pub data: Vec<RevenueDataPoint>,
    pub grouping: String,
}

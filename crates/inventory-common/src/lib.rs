pub mod dto;
pub mod features;
pub mod license;

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// ==================== PRODUCTOS ====================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromRow)]
pub struct Product {
    pub id: uuid::Uuid,
    pub name: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub brand: Option<String>,
    pub model: Option<String>,
    pub sku: Option<String>,
    pub price: Decimal,
    pub cost: Decimal,
    pub stock: i32,
    pub min_stock: i32,
    pub location: Option<String>,
    pub supplier_id: Option<uuid::Uuid>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ==================== VENTAS ====================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromRow)]
pub struct Sale {
    pub id: uuid::Uuid,
    pub customer_name: Option<String>,
    pub customer_email: Option<String>,
    pub customer_phone: Option<String>,
    pub total: Decimal,
    pub payment_method: PaymentMethod,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromRow)]
pub struct SaleItem {
    pub id: uuid::Uuid,
    pub sale_id: uuid::Uuid,
    pub product_id: uuid::Uuid,
    pub product_name: Option<String>,
    pub quantity: i32,
    pub unit_price: Decimal,
    pub total: Decimal,
}

// ==================== REPARACIONES ====================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromRow)]
pub struct Repair {
    pub id: uuid::Uuid,
    pub customer_name: Option<String>,
    pub customer_email: Option<String>,
    pub customer_phone: Option<String>,
    pub motorcycle: Option<String>,
    pub license_plate: Option<String>,
    pub description: Option<String>,
    pub diagnosis: Option<String>,
    pub technician_id: Option<uuid::Uuid>,
    pub estimated_delivery: Option<chrono::NaiveDate>,
    pub priority: Priority,
    pub status: RepairStatus,
    pub estimated_cost: Option<Decimal>,
    pub final_cost: Option<Decimal>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromRow)]
pub struct RepairUpdate {
    pub id: uuid::Uuid,
    pub repair_id: uuid::Uuid,
    pub status: Option<String>,
    pub description: Option<String>,
    pub created_by: Option<uuid::Uuid>,
    pub created_at: DateTime<Utc>,
}

// ==================== PROVEEDORES ====================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromRow)]
pub struct Supplier {
    pub id: uuid::Uuid,
    pub name: String,
    pub contact_person: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub tax_id: Option<String>,
    pub payment_terms: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ==================== USUARIOS ====================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: uuid::Uuid,
    pub email: String,
    pub display_name: Option<String>,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub role: UserRole,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

// ==================== AUDITORÍA ====================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromRow)]
pub struct AuditLog {
    pub id: i64,
    pub user_id: Option<uuid::Uuid>,
    pub action: String,
    pub entity_type: Option<String>,
    pub entity_id: Option<uuid::Uuid>,
    pub old_values: Option<serde_json::Value>,
    pub new_values: Option<serde_json::Value>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
}

// ==================== ENUMS ====================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[sqlx(type_name = "payment_method", rename_all = "snake_case")]
pub enum PaymentMethod {
    Cash,
    Card,
    Transfer,
}

impl std::fmt::Display for PaymentMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PaymentMethod::Cash => write!(f, "cash"),
            PaymentMethod::Card => write!(f, "card"),
            PaymentMethod::Transfer => write!(f, "transfer"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[sqlx(type_name = "repair_status", rename_all = "snake_case")]
pub enum RepairStatus {
    Pending,
    InProgress,
    Completed,
    Cancelled,
}

impl std::fmt::Display for RepairStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RepairStatus::Pending => write!(f, "pending"),
            RepairStatus::InProgress => write!(f, "in_progress"),
            RepairStatus::Completed => write!(f, "completed"),
            RepairStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[sqlx(type_name = "priority", rename_all = "snake_case")]
pub enum Priority {
    High,
    Medium,
    Low,
}

impl std::fmt::Display for Priority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Priority::High => write!(f, "high"),
            Priority::Medium => write!(f, "medium"),
            Priority::Low => write!(f, "low"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[sqlx(type_name = "user_role", rename_all = "snake_case")]
pub enum UserRole {
    Admin,
    Mechanic,
    Seller,
}

impl std::fmt::Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserRole::Admin => write!(f, "admin"),
            UserRole::Mechanic => write!(f, "mechanic"),
            UserRole::Seller => write!(f, "seller"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_payment_method_display() {
        assert_eq!(PaymentMethod::Cash.to_string(), "cash");
        assert_eq!(PaymentMethod::Card.to_string(), "card");
        assert_eq!(PaymentMethod::Transfer.to_string(), "transfer");
    }

    #[test]
    fn test_repair_status_display() {
        assert_eq!(RepairStatus::Pending.to_string(), "pending");
        assert_eq!(RepairStatus::InProgress.to_string(), "in_progress");
        assert_eq!(RepairStatus::Completed.to_string(), "completed");
        assert_eq!(RepairStatus::Cancelled.to_string(), "cancelled");
    }

    #[test]
    fn test_priority_display() {
        assert_eq!(Priority::High.to_string(), "high");
        assert_eq!(Priority::Medium.to_string(), "medium");
        assert_eq!(Priority::Low.to_string(), "low");
    }

    #[test]
    fn test_user_role_display() {
        assert_eq!(UserRole::Admin.to_string(), "admin");
        assert_eq!(UserRole::Mechanic.to_string(), "mechanic");
        assert_eq!(UserRole::Seller.to_string(), "seller");
    }
}

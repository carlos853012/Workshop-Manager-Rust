use inventory_common::{PaymentMethod, RepairStatus, UserRole};

pub fn translate_payment(method: &PaymentMethod) -> &'static str {
    match method {
        PaymentMethod::Cash => "Efectivo",
        PaymentMethod::Card => "Tarjeta",
        PaymentMethod::Transfer => "Transferencia",
    }
}

pub fn translate_sale_status(status: &str) -> String {
    match status {
        "pending" => "Pendiente".to_string(),
        "completed" => "Completada".to_string(),
        "cancelled" => "Cancelada".to_string(),
        "refunded" => "Reembolsada".to_string(),
        other => other.to_string(),
    }
}

#[allow(dead_code)]
pub fn translate_repair_status(status: &RepairStatus) -> &'static str {
    match status {
        RepairStatus::Pending => "Pendiente",
        RepairStatus::InProgress => "En Progreso",
        RepairStatus::Completed => "Completado",
        RepairStatus::Cancelled => "Cancelado",
        RepairStatus::Deleted => "Eliminado",
    }
}

#[allow(dead_code)]
pub fn translate_priority(priority: &inventory_common::Priority) -> &'static str {
    match priority {
        inventory_common::Priority::High => "Alta",
        inventory_common::Priority::Medium => "Media",
        inventory_common::Priority::Low => "Baja",
    }
}

pub fn translate_role(role: &UserRole) -> &'static str {
    match role {
        UserRole::Admin => "Admin",
        UserRole::Mechanic => "Mecánico",
        UserRole::Seller => "Vendedor",
    }
}

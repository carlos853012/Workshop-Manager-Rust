use workshop_common::{PaymentMethod, Priority, RepairStatus, UserRole};

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

pub fn translate_role(role: &UserRole) -> &'static str {
    match role {
        UserRole::Admin => "Admin",
        UserRole::Mechanic => "Mecánico",
        UserRole::Seller => "Vendedor",
    }
}

pub fn translate_repair_status(status: &RepairStatus) -> &'static str {
    match status {
        RepairStatus::Pending => "Pendiente",
        RepairStatus::InProgress => "En Progreso",
        RepairStatus::Completed => "Completada",
        RepairStatus::Cancelled => "Cancelada",
        RepairStatus::Deleted => "Eliminada",
    }
}

pub fn translate_repair_status_str(status: &str) -> String {
    match status {
        "pending" => "Pendiente".to_string(),
        "in_progress" => "En Progreso".to_string(),
        "completed" => "Completada".to_string(),
        "cancelled" => "Cancelada".to_string(),
        "deleted" => "Eliminada".to_string(),
        other => other.to_string(),
    }
}

pub fn translate_priority(priority: &Priority) -> &'static str {
    match priority {
        Priority::High => "Alta",
        Priority::Medium => "Media",
        Priority::Low => "Baja",
    }
}

pub fn translate_supplier_status(status: &str) -> String {
    match status {
        "active" => "Activo".to_string(),
        "inactive" => "Inactivo".to_string(),
        other => other.to_string(),
    }
}

pub fn translate_user_status(status: &str) -> String {
    match status {
        "active" => "Activo".to_string(),
        "inactive" => "Inactivo".to_string(),
        other => other.to_string(),
    }
}

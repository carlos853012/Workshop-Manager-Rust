use workshop_common::{PaymentMethod, UserRole};

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

use serde_json::Value;
use sqlx::PgPool;

/// Inserta un registro de auditoría en la base de datos.
/// Los valores sensibles deben ser removidos o enmascarados antes de llamar esta función.
#[allow(clippy::too_many_arguments)]
#[allow(dead_code)]
pub async fn log_change(
    pool: &PgPool,
    user_id: Option<uuid::Uuid>,
    action: &str,
    entity_type: &str,
    entity_id: uuid::Uuid,
    old_values: Option<Value>,
    new_values: Option<Value>,
    ip_address: Option<&str>,
    user_agent: Option<&str>,
) -> anyhow::Result<()> {
    sqlx::query(
        "INSERT INTO audit_log \
         (user_id, action, entity_type, entity_id, old_values, new_values, ip_address, user_agent, created_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW())"
    )
    .bind(user_id)
    .bind(action)
    .bind(entity_type)
    .bind(entity_id)
    .bind(old_values)
    .bind(new_values)
    .bind(ip_address)
    .bind(user_agent)
    .execute(pool)
    .await
    .map_err(|e| anyhow::anyhow!("Failed to write audit log: {}", e))?;

    tracing::info!(
        user_id = ?user_id,
        action = action,
        entity_type = entity_type,
        entity_id = %entity_id,
        "Audit log written"
    );

    Ok(())
}

/// Remueve campos sensibles de un objeto JSON antes de auditarlo.
#[allow(dead_code)]
pub fn redact_sensitive(value: &mut Value) {
    if let Value::Object(map) = value {
        for key in ["password_hash", "password", "crypto_key", "jwt_secret"] {
            if map.contains_key(key) {
                map.insert(key.to_string(), Value::String("[REDACTED]".to_string()));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redact_sensitive() {
        let mut value = serde_json::json!({
            "id": "123",
            "password_hash": "secret_hash",
            "email": "test@example.com"
        });

        redact_sensitive(&mut value);

        assert_eq!(value["password_hash"], "[REDACTED]");
        assert_eq!(value["email"], "test@example.com");
    }
}

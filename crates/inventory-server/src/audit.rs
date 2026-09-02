// Audit module
// TODO: Implementar en Fase 4

use serde_json::Value;

pub fn log_change(
    _user_id: Option<uuid::Uuid>,
    _action: &str,
    _entity_type: &str,
    _entity_id: uuid::Uuid,
    _old_values: Option<Value>,
    _new_values: Option<Value>,
    _ip_address: Option<&str>,
) -> anyhow::Result<()> {
    // TODO: Insertar en audit_log
    tracing::info!(
        action = _action,
        entity_type = _entity_type,
        entity_id = %_entity_id,
        "Audit log"
    );
    Ok(())
}

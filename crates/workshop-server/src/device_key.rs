use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::state::AppState;

pub const HEADER: &str = "X-WorkshopManager-Device-Key";

pub fn generate() -> String {
    format!("wm_{}", Uuid::new_v4().simple())
}

pub fn hash(key: &str) -> String {
    let digest = Sha256::digest(key.as_bytes());
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

pub async fn require_device_key(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    if !state.config.require_device_key {
        return Ok(next.run(request).await);
    }

    let key = request
        .headers()
        .get(HEADER)
        .and_then(|value| value.to_str().ok())
        .filter(|value| !value.is_empty())
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let key_hash = hash(key);

    let active: Option<bool> =
        sqlx::query_scalar("SELECT active FROM device_keys WHERE key_hash = $1")
            .bind(&key_hash)
            .fetch_optional(&state.pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if active != Some(true) {
        return Err(StatusCode::UNAUTHORIZED);
    }

    sqlx::query("UPDATE device_keys SET last_seen_at = NOW() WHERE key_hash = $1")
        .bind(&key_hash)
        .execute(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(next.run(request).await)
}

#[derive(Debug, serde::Serialize)]
pub struct GeneratedDeviceKey {
    pub key: String,
}

#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct DeviceKeySummary {
    pub id: Uuid,
    pub bound_ip: Option<String>,
    pub active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_seen_at: Option<chrono::DateTime<chrono::Utc>>,
}

pub async fn create(pool: &sqlx::PgPool) -> anyhow::Result<String> {
    let key = generate();
    sqlx::query("INSERT INTO device_keys (key_hash) VALUES ($1)")
        .bind(hash(&key))
        .execute(pool)
        .await?;
    Ok(key)
}

/// Cuenta las device keys activas.
pub async fn count_active(pool: &sqlx::PgPool) -> anyhow::Result<i64> {
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM device_keys WHERE active = TRUE")
        .fetch_one(pool)
        .await?;
    Ok(count.0)
}

/// Verifica si se puede crear una nueva device key (respeta max_viewers).
pub async fn can_create(pool: &sqlx::PgPool, max_viewers: u32) -> anyhow::Result<bool> {
    let count = count_active(pool).await?;
    Ok((count as u32) < max_viewers)
}

pub async fn list(pool: &sqlx::PgPool) -> anyhow::Result<Vec<DeviceKeySummary>> {
    Ok(sqlx::query_as(
        "SELECT id, bound_ip, active, created_at, last_seen_at FROM device_keys ORDER BY created_at DESC",
    )
    .fetch_all(pool)
    .await?)
}

pub async fn revoke(pool: &sqlx::PgPool, id: Uuid) -> anyhow::Result<bool> {
    let result = sqlx::query("UPDATE device_keys SET active = FALSE WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() == 1)
}

pub async fn unbind(pool: &sqlx::PgPool, id: Uuid) -> anyhow::Result<bool> {
    let result = sqlx::query("UPDATE device_keys SET bound_ip = NULL WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() == 1)
}

use axum::extract::FromRef;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::rate_limiter::RateLimiter;

#[derive(Clone)]
pub struct AppState {
    pub secrets: Secrets,
    pub config: ServerConfig,
    pub pool: PgPool,
    pub login_rate_limiter: std::sync::Arc<RateLimiter>,
}

#[derive(Clone)]
pub struct Secrets {
    pub jwt_secret: String,
    pub crypto_key: Vec<u8>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub api_key: String,
    pub require_device_key: bool,
    pub iva_rate: f64,
    pub icon_bg: [u8; 3],
    pub icon_fg: [u8; 3],
}

impl FromRef<AppState> for Secrets {
    fn from_ref(state: &AppState) -> Self {
        state.secrets.clone()
    }
}

impl FromRef<AppState> for ServerConfig {
    fn from_ref(state: &AppState) -> Self {
        state.config.clone()
    }
}

impl FromRef<AppState> for PgPool {
    fn from_ref(state: &AppState) -> Self {
        state.pool.clone()
    }
}

impl AppState {
    pub fn new(secrets: Secrets, config: ServerConfig, pool: PgPool) -> Self {
        Self {
            secrets,
            config,
            pool,
            login_rate_limiter: std::sync::Arc::new(RateLimiter::new(5, 300)),
        }
    }
}

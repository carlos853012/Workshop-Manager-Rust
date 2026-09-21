use std::sync::Arc;

use axum::extract::FromRef;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tokio::sync::RwLock;
use workshop_common::features::License;

use crate::rate_limiter::RateLimiter;

#[derive(Clone)]
pub struct AppState {
    pub secrets: Arc<Secrets>,
    pub config: ServerConfig,
    pub pool: PgPool,
    pub login_rate_limiter: Arc<RateLimiter>,
    pub license: Arc<RwLock<Option<License>>>,
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
    pub max_viewers: u32,
    pub iva_rate: f64,
    pub icon_bg: [u8; 3],
    pub icon_fg: [u8; 3],
    pub cors_origins: Vec<String>,
    pub license_api_url: String,
}

impl FromRef<AppState> for Arc<Secrets> {
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
    pub fn new(
        secrets: Secrets,
        config: ServerConfig,
        pool: PgPool,
        license: Option<License>,
    ) -> Self {
        Self {
            secrets: Arc::new(secrets),
            config,
            pool,
            login_rate_limiter: Arc::new(RateLimiter::new(5, 300)),
            license: Arc::new(RwLock::new(license)),
        }
    }
}

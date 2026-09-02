use axum::extract::FromRef;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct AppState {
    pub secrets: Secrets,
    pub config: ServerConfig,
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

impl AppState {
    pub fn new(secrets: Secrets, config: ServerConfig) -> Self {
        Self { secrets, config }
    }
}

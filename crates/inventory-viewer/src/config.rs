use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewerConfig {
    pub server: ServerSection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerSection {
    #[serde(default = "default_base_url")]
    pub base_url: String,
    #[serde(default = "default_api_key")]
    pub api_key: String,
}

fn default_base_url() -> String {
    "https://127.0.0.1:8443".to_string()
}

fn default_api_key() -> String {
    "dev-key-change-in-production".to_string()
}

impl Default for ViewerConfig {
    fn default() -> Self {
        Self {
            server: ServerSection {
                base_url: default_base_url(),
                api_key: default_api_key(),
            },
        }
    }
}

static CONFIG: OnceLock<ViewerConfig> = OnceLock::new();

fn load_config() -> ViewerConfig {
    let config_path = std::env::current_dir()
        .unwrap_or_default()
        .join("config")
        .join("viewer.toml");

    if config_path.exists() {
        let content = std::fs::read_to_string(&config_path).unwrap_or_default();
        toml::from_str(&content).unwrap_or_default()
    } else {
        ViewerConfig::default()
    }
}

/// Acceso global a la configuración del viewer.
pub fn config() -> ViewerConfig {
    CONFIG.get_or_init(load_config).clone()
}

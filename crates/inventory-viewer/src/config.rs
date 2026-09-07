use serde::{Deserialize, Serialize};
use std::sync::{OnceLock, RwLock};

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
    #[serde(default)]
    pub device_key: String,
    #[serde(default = "default_true")]
    pub tls_accept_invalid_certs: bool,
}

fn default_true() -> bool {
    true
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
                device_key: String::new(),
                tls_accept_invalid_certs: true,
            },
        }
    }
}

static CONFIG: OnceLock<RwLock<ViewerConfig>> = OnceLock::new();

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
    CONFIG
        .get_or_init(|| RwLock::new(load_config()))
        .read()
        .map(|config| config.clone())
        .unwrap_or_default()
}

pub fn save_config(new_config: ViewerConfig) -> Result<(), String> {
    let config_path = std::env::current_dir()
        .map_err(|error| format!("No se pudo localizar la configuración: {error}"))?
        .join("config")
        .join("viewer.toml");
    let content = toml::to_string_pretty(&new_config)
        .map_err(|error| format!("No se pudo serializar la configuración: {error}"))?;

    std::fs::create_dir_all(
        config_path
            .parent()
            .ok_or_else(|| "Ruta de configuración inválida".to_string())?,
    )
    .map_err(|error| format!("No se pudo crear la carpeta de configuración: {error}"))?;
    std::fs::write(&config_path, content)
        .map_err(|error| format!("No se pudo guardar la configuración: {error}"))?;

    let config = CONFIG.get_or_init(|| RwLock::new(load_config()));
    let mut current = config
        .write()
        .map_err(|_| "No se pudo actualizar la configuración en memoria".to_string())?;
    *current = new_config;
    Ok(())
}

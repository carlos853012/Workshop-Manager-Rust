use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerSection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerSection {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
}

fn default_host() -> String {
    "0.0.0.0".to_string()
}

fn default_port() -> u16 {
    8443
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerSection {
                host: default_host(),
                port: default_port(),
            },
        }
    }
}

pub fn load_config() -> anyhow::Result<super::state::ServerConfig> {
    let config_path = std::env::current_dir()?.join("config").join("server.toml");

    if config_path.exists() {
        let content = std::fs::read_to_string(&config_path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(super::state::ServerConfig {
            host: config.server.host,
            port: config.server.port,
        })
    } else {
        // Crear config por defecto
        let default_config = Config::default();
        let config_dir = std::env::current_dir()?.join("config");
        std::fs::create_dir_all(&config_dir)?;
        let content = toml::to_string_pretty(&default_config)?;
        std::fs::write(config_path, content)?;
        Ok(super::state::ServerConfig {
            host: default_config.server.host,
            port: default_config.server.port,
        })
    }
}

use serde::{Deserialize, Serialize};

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
    #[serde(default = "default_api_key")]
    pub api_key: String,
    #[serde(default)]
    pub require_device_key: bool,
}

fn default_host() -> String {
    "127.0.0.1".to_string()
}

fn default_port() -> u16 {
    8443
}

fn default_api_key() -> String {
    "dev-key-change-in-production".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerSection {
                host: default_host(),
                port: default_port(),
                api_key: default_api_key(),
                require_device_key: false,
            },
        }
    }
}

pub fn load_config() -> anyhow::Result<super::state::ServerConfig> {
    let config_path = std::env::current_dir()?.join("config").join("server.toml");

    let mut server_config = if config_path.exists() {
        let content = std::fs::read_to_string(&config_path)?;
        let config: Config = toml::from_str(&content)?;
        super::state::ServerConfig {
            host: config.server.host,
            port: config.server.port,
            api_key: config.server.api_key,
            require_device_key: config.server.require_device_key,
        }
    } else {
        let default_config = Config::default();
        let config_dir = std::env::current_dir()?.join("config");
        std::fs::create_dir_all(&config_dir)?;
        let content = toml::to_string_pretty(&default_config)?;
        std::fs::write(config_path, content)?;
        super::state::ServerConfig {
            host: default_config.server.host,
            port: default_config.server.port,
            api_key: default_config.server.api_key,
            require_device_key: default_config.server.require_device_key,
        }
    };

    // Allow env var to override API key (for production deployments)
    if let Ok(env_key) = std::env::var("WORKSHOP_MANAGER_API_KEY") {
        if !env_key.is_empty() {
            server_config.api_key = env_key;
        }
    }

    Ok(server_config)
}

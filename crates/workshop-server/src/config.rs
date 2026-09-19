use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerSection,
    #[serde(default)]
    pub tax: TaxSection,
    #[serde(default)]
    pub icon: IconSection,
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
    #[serde(default = "default_max_viewers")]
    pub max_viewers: u32,
    #[serde(default)]
    pub cors_origins: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TaxSection {
    /// IVA rate as decimal fraction (e.g., 0.19 for 19%).
    #[serde(default = "default_iva_rate")]
    pub iva_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IconSection {
    /// Background color as hex string (e.g., "#F59E0B").
    #[serde(default = "default_icon_bg")]
    pub bg: String,
    /// Foreground (wrench) color as hex string (e.g., "#FFFFFF").
    #[serde(default = "default_icon_fg")]
    pub fg: String,
}

impl Default for IconSection {
    fn default() -> Self {
        Self {
            bg: default_icon_bg(),
            fg: default_icon_fg(),
        }
    }
}

fn default_host() -> String {
    "0.0.0.0".to_string()
}

fn default_port() -> u16 {
    8443
}

fn default_iva_rate() -> f64 {
    0.19
}

fn default_icon_bg() -> String {
    "#F59E0B".to_string()
}

fn default_icon_fg() -> String {
    "#FFFFFF".to_string()
}

fn default_api_key() -> String {
    // Auto-generate a random API key on first run
    use rand::rngs::OsRng;
    use rand::Fill;
    let mut bytes = [0u8; 32];
    bytes.try_fill(&mut OsRng).expect("OsRng should not fail");
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

fn default_max_viewers() -> u32 {
    2
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerSection {
                host: default_host(),
                port: default_port(),
                api_key: default_api_key(),
                require_device_key: false,
                max_viewers: default_max_viewers(),
                cors_origins: Vec::new(),
            },
            tax: TaxSection {
                iva_rate: default_iva_rate(),
            },
            icon: IconSection::default(),
        }
    }
}

pub fn load_config(data_dir: &Path) -> anyhow::Result<super::state::ServerConfig> {
    let config_dir = data_dir.parent().unwrap_or(data_dir).join("config");
    let config_path = config_dir.join("server.toml");

    let mut server_config = if config_path.exists() {
        let content = std::fs::read_to_string(&config_path)?;
        let config: Config = toml::from_str(&content)?;
        super::state::ServerConfig {
            host: config.server.host,
            port: config.server.port,
            api_key: config.server.api_key,
            require_device_key: config.server.require_device_key,
            max_viewers: config.server.max_viewers,
            iva_rate: config.tax.iva_rate,
            icon_bg: parse_hex_color(&config.icon.bg),
            icon_fg: parse_hex_color(&config.icon.fg),
            cors_origins: config.server.cors_origins,
        }
    } else {
        let default_config = Config::default();
        std::fs::create_dir_all(&config_dir)?;
        let content = toml::to_string_pretty(&default_config)?;
        std::fs::write(&config_path, content)?;
        super::state::ServerConfig {
            host: default_config.server.host,
            port: default_config.server.port,
            api_key: default_config.server.api_key,
            require_device_key: default_config.server.require_device_key,
            max_viewers: default_config.server.max_viewers,
            iva_rate: default_config.tax.iva_rate,
            icon_bg: parse_hex_color(&default_config.icon.bg),
            icon_fg: parse_hex_color(&default_config.icon.fg),
            cors_origins: default_config.server.cors_origins,
        }
    };

    // Allow env var to override API key (for production deployments)
    if let Ok(env_key) = std::env::var("WORKSHOP_MANAGER_API_KEY") {
        if !env_key.is_empty() {
            server_config.api_key = env_key;
        }
    }

    // Force regenerate API key if it's the insecure default or empty
    if server_config.api_key.is_empty() || server_config.api_key == "dev-key-change-in-production" {
        tracing::warn!("API key is insecure or empty — generating a new random key");
        server_config.api_key = default_api_key();
    }

    Ok(server_config)
}

/// Parse a hex color string like "#F59E0B" or "#fff" into [u8; 3] RGB.
/// Falls back to defaults on invalid input.
fn parse_hex_color(hex: &str) -> [u8; 3] {
    let hex = hex.trim_start_matches('#');
    match hex.len() {
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0xF5);
            let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0x9E);
            let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0x0B);
            [r, g, b]
        }
        3 => {
            let r = u8::from_str_radix(&hex[0..1], 16).unwrap_or(0xF);
            let g = u8::from_str_radix(&hex[1..2], 16).unwrap_or(0x9);
            let b = u8::from_str_radix(&hex[2..3], 16).unwrap_or(0xB);
            [r * 17, g * 17, b * 17]
        }
        _ => [0xF5, 0x9E, 0x0B],
    }
}

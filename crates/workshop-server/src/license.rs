use std::path::Path;

use workshop_common::features::{License, LicenseTier};
use workshop_common::hardware;
use workshop_common::license as lic;

/// Clave pública del vendor para verificar licencias.
/// Generada con `license-tool generate-keypair`.
/// La clave secreta queda en Cloudflare como Worker secret — nunca en el cliente.
const VENDOR_PUBLIC_KEY: &[u8] = &[
    116, 80, 30, 248, 220, 61, 49, 213, 177, 101, 235, 162, 219, 226, 176, 93, 109, 149, 71, 232,
    177, 19, 129, 24, 44, 238, 188, 229, 87, 2, 141, 71,
];

/// Verifica que la clave pública no sea el placeholder de todos ceros.
pub fn is_placeholder_key() -> bool {
    VENDOR_PUBLIC_KEY.iter().all(|&b| b == 0)
}

/// Resultado de la validación de licencia desde disco.
#[derive(Debug)]
pub enum LicenseStatus {
    /// Licencia válida y activa.
    Valid(License),
    /// Primer uso, sin licencia — requiere activación online.
    FirstRun(String),
    /// Licencia inválida (firma, hardware, o expirada).
    Invalid(String),
}

/// Resultado de la validación online contra el Worker.
#[derive(Debug)]
pub enum OnlineResult {
    /// Worker respondió OK — incluye bytes firmados y licencia parseada.
    Ok { signed: Vec<u8>, license: License },
    /// Worker rechazó la activación (403/404/429 con mensaje).
    Rejected(String),
    /// No se pudo conectar al Worker (sin internet, timeout, etc.).
    Unreachable(String),
}

/// Carga y valida la licencia desde disco.
pub fn load_license(data_dir: &Path) -> LicenseStatus {
    let license_path = data_dir.join("license.dat");

    if !license_path.exists() {
        let hw_hash = hardware::get_hardware_id().unwrap_or_default();
        LicenseStatus::FirstRun(hw_hash)
    } else {
        match std::fs::read(&license_path) {
            Ok(data) => match lic::verify_license(&data, VENDOR_PUBLIC_KEY) {
                Ok(license) => LicenseStatus::Valid(license),
                Err(e) => LicenseStatus::Invalid(format!("Licencia inválida: {e}")),
            },
            Err(e) => LicenseStatus::Invalid(format!("Error leyendo licencia: {e}")),
        }
    }
}

/// Guarda bytes firmados (recibidos del Worker) en license.dat.
/// Escritura atómica: escribe a .tmp y renombra.
pub fn save_signed_license(data: &[u8], data_dir: &Path) -> Result<(), String> {
    let license_path = data_dir.join("license.dat");
    let tmp_path = data_dir.join("license.dat.tmp");
    std::fs::write(&tmp_path, data)
        .map_err(|e| format!("Error escribiendo {}: {e}", tmp_path.display()))?;
    std::fs::rename(&tmp_path, &license_path)
        .map_err(|e| format!("Error renombrando a {}: {e}", license_path.display()))?;
    Ok(())
}

/// Valida la licencia contra el hardware actual y expiración.
pub fn validate_license(license: &License) -> Result<(), String> {
    let hw_hash =
        hardware::get_hardware_id().map_err(|e| format!("No se pudo extraer hardware: {e}"))?;

    if !lic::validate_hardware(license, &hw_hash) {
        return Err("Licencia no corresponde a este equipo".to_string());
    }

    if !license.is_valid() {
        return Err("Licencia inválida".to_string());
    }

    if license.is_expired() {
        let days = license.days_until_expiry().unwrap_or(0);
        return Err(format!("Licencia expirada hace {days} días"));
    }

    Ok(())
}

/// Valida la licencia online contra el Cloudflare Worker.
/// Retorna `OnlineResult` distinguiendo rechazo de desconexión.
pub async fn validate_online(
    api_url: &str,
    license_key: &str,
    hardware_hash: &str,
) -> OnlineResult {
    if api_url.is_empty() {
        return OnlineResult::Unreachable("license_api_url vacío".to_string());
    }

    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
    {
        Ok(c) => c,
        Err(e) => return OnlineResult::Unreachable(format!("Error creando HTTP client: {e}")),
    };

    let url = format!("{}/api/v1/licenses/activate", api_url.trim_end_matches('/'));
    let body = serde_json::json!({
        "license_key": license_key,
        "hardware_hash": hardware_hash,
    });

    tracing::info!("Validando licencia online: {} en {}", license_key, url);
    let resp = match client.post(&url).json(&body).send().await {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!("No se pudo conectar al Worker: {}", e);
            return OnlineResult::Unreachable(format!("No se pudo conectar: {e}"));
        }
    };

    let status = resp.status();
    let resp_body = resp.text().await.unwrap_or_default();

    if !status.is_success() {
        let json: serde_json::Value = serde_json::from_str(&resp_body).unwrap_or_default();
        let error = json
            .get("error")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        tracing::warn!("Worker rechazó ({}): {}", status, error);
        return OnlineResult::Rejected(error.to_string());
    }

    let json: serde_json::Value = match serde_json::from_str(&resp_body) {
        Ok(v) => v,
        Err(e) => return OnlineResult::Unreachable(format!("Error parseando respuesta: {e}")),
    };

    // Preferir signed_license del Worker (bytes firmados Ed25519)
    if let Some(signed_b64) = json.get("signed_license").and_then(|v| v.as_str()) {
        match base64_decode(signed_b64) {
            Ok(signed_bytes) => match lic::verify_license(&signed_bytes, VENDOR_PUBLIC_KEY) {
                Ok(license) => {
                    tracing::info!(
                        "Licencia activada online (firmada): key={}, tier={}, hw={}",
                        license.license_key,
                        license.tier,
                        license.hardware_hash
                    );
                    return OnlineResult::Ok {
                        signed: signed_bytes,
                        license,
                    };
                }
                Err(e) => {
                    tracing::warn!("Firma del Worker inválida: {}", e);
                    return OnlineResult::Rejected(format!("Firma inválida: {e}"));
                }
            },
            Err(e) => {
                tracing::warn!("Error decodificando signed_license: {}", e);
                return OnlineResult::Unreachable(format!("signed_license base64 inválido: {e}"));
            }
        }
    }

    // Fallback: si el Worker no envió signed_license, parsear el JSON directamente
    // (compatibilidad con Workers antiguos)
    if json.get("success").and_then(|v| v.as_bool()) != Some(true) {
        let error = json
            .get("error")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        return OnlineResult::Rejected(error.to_string());
    }

    let lic_data = match json.get("license") {
        Some(d) => d,
        None => return OnlineResult::Unreachable("Respuesta sin campo 'license'".to_string()),
    };

    match parse_license_from_json(lic_data, hardware_hash) {
        Ok(license) => {
            tracing::warn!("Worker no envió signed_license — usando JSON plano (compatibilidad)");
            OnlineResult::Ok {
                signed: Vec::new(),
                license,
            }
        }
        Err(e) => OnlineResult::Rejected(e),
    }
}

/// Parsea un License desde un JSON del Worker (fallback sin firma).
fn parse_license_from_json(
    lic_data: &serde_json::Value,
    default_hw: &str,
) -> Result<License, String> {
    let tier_str = lic_data
        .get("tier")
        .and_then(|v| v.as_str())
        .ok_or("Campo 'tier' faltante")?;

    // Worker now sends PascalCase ("Trial", "Base") — also handle lowercase for compat
    let tier = match tier_str {
        "Trial" | "trial" => LicenseTier::Trial,
        "Base" | "base" => LicenseTier::Base,
        "Reports" | "reports" => LicenseTier::Reports,
        "Advanced" | "advanced" => LicenseTier::Advanced,
        "Api" | "api" => LicenseTier::Api,
        _ => return Err(format!("Tier desconocido: {tier_str}")),
    };

    let license_key = lic_data
        .get("license_key")
        .and_then(|v| v.as_str())
        .ok_or("Campo 'license_key' faltante")?
        .to_string();

    let hw = lic_data
        .get("hardware_hash")
        .and_then(|v| v.as_str())
        .unwrap_or(default_hw)
        .to_string();

    let max_viewers = lic_data
        .get("max_viewers")
        .or_else(|| lic_data.get("max_concurrent_viewers"))
        .and_then(|v| v.as_u64())
        .unwrap_or(tier.max_viewers() as u64) as u32;

    let max_transfers = lic_data
        .get("max_transfers")
        .and_then(|v| v.as_u64())
        .unwrap_or(tier.max_transfers() as u64) as u32;

    let transfer_count = lic_data
        .get("transfer_count")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u32;

    let activated_at = lic_data
        .get("activated_at")
        .and_then(|v| v.as_str())
        .and_then(|s| {
            chrono::DateTime::parse_from_rfc3339(s)
                .or_else(|_| chrono::DateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%.f"))
                .or_else(|_| chrono::DateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%SZ"))
                .ok()
        })
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .unwrap_or_else(chrono::Utc::now);

    let expires_at = lic_data
        .get("expires_at")
        .and_then(|v| v.as_str())
        .and_then(|s| {
            chrono::DateTime::parse_from_rfc3339(s)
                .or_else(|_| chrono::DateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%.f"))
                .or_else(|_| chrono::DateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%SZ"))
                .or_else(|_| chrono::DateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S"))
                .ok()
        })
        .map(|dt| dt.with_timezone(&chrono::Utc));

    Ok(License {
        license_key,
        tier,
        hardware_hash: hw,
        max_viewers,
        max_transfers,
        transfer_count,
        activated_at,
        expires_at,
    })
}

/// Decodifica base64 a bytes.
fn base64_decode(input: &str) -> Result<Vec<u8>, String> {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD
        .decode(input)
        .map_err(|e| format!("base64 decode error: {e}"))
}

/// Obtiene el hardware hash actual.
#[allow(dead_code)]
pub fn get_current_hardware_hash() -> Result<String, String> {
    hardware::get_hardware_id()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_license_first_run() {
        let dir = std::env::temp_dir().join("wm_test_license");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let status = load_license(&dir);
        assert!(matches!(status, LicenseStatus::FirstRun(_)));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_save_signed_license_atomic() {
        let dir = std::env::temp_dir().join("wm_test_signed");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let data = b"test license data";
        assert!(save_signed_license(data, &dir).is_ok());

        let saved = std::fs::read(dir.join("license.dat")).unwrap();
        assert_eq!(saved, data);
        assert!(!dir.join("license.dat.tmp").exists());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_parse_license_json_pascal_case() {
        let json = serde_json::json!({
            "license_key": "TEST-KEY",
            "tier": "Trial",
            "hardware_hash": "abc123",
            "max_viewers": 1,
            "max_transfers": 0,
            "transfer_count": 0,
            "activated_at": "2026-09-20T15:58:03Z",
            "expires_at": "2026-09-27T15:58:03Z"
        });

        let lic = parse_license_from_json(&json, "default_hw").unwrap();
        assert_eq!(lic.tier, LicenseTier::Trial);
        assert_eq!(lic.license_key, "TEST-KEY");
        assert!(lic.expires_at.is_some());
    }

    #[test]
    fn test_parse_license_json_legacy_lowercase() {
        let json = serde_json::json!({
            "license_key": "OLD-KEY",
            "tier": "base",
            "hardware_hash": "abc123",
            "max_concurrent_viewers": 2,
            "max_transfers": 3,
            "transfer_count": 0,
            "activated_at": "2026-09-20T15:58:03Z"
        });

        let lic = parse_license_from_json(&json, "default_hw").unwrap();
        assert_eq!(lic.tier, LicenseTier::Base);
        assert!(lic.expires_at.is_none());
    }

    #[test]
    fn test_is_placeholder_key() {
        assert!(!is_placeholder_key());
    }
}

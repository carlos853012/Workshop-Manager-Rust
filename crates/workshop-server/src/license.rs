use std::path::Path;

use workshop_common::features::{License, LicenseTier};
use workshop_common::hardware;
use workshop_common::license as lic;

/// Clave pública del vendor para verificar licencias.
/// Generada con `license-tool generate-keypair`.
/// La clave secreta se guarda en `vendor_secret_key.bin` (no commitear al repo).
const VENDOR_PUBLIC_KEY: &[u8] = &[
    116, 80, 30, 248, 220, 61, 49, 213, 177, 101, 235, 162, 219, 226, 176, 93,
    109, 149, 71, 232, 177, 19, 129, 24, 44, 238, 188, 229, 87, 2, 141, 71,
];

/// Verifica que la clave pública no sea el placeholder de todos ceros.
pub fn is_placeholder_key() -> bool {
    VENDOR_PUBLIC_KEY.iter().all(|&b| b == 0)
}

/// Resultado de la validación de licencia.
#[derive(Debug)]
pub enum LicenseStatus {
    /// Licencia válida y activa.
    Valid(License),
    /// Primer uso, sin licencia — genera trial.
    FirstRun(String),
    /// Licencia inválida (firma, hardware, o expirada).
    Invalid(String),
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

/// Guarda una licencia validada en disco.
pub fn save_license(license: &License, data_dir: &Path) -> Result<(), String> {
    let secret_key = load_vendor_secret_key(data_dir)?;
    let license_path = data_dir.join("license.dat");
    let data =
        lic::sign_license(license, &secret_key).map_err(|e| format!("Error firmando: {e}"))?;
    std::fs::write(&license_path, data).map_err(|e| format!("Error guardando licencia: {e}"))?;
    Ok(())
}

/// Carga la clave secreta del vendor desde vendor_secret_key.bin.
fn load_vendor_secret_key(data_dir: &Path) -> Result<Vec<u8>, String> {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_default();

    let candidates = [
        exe_dir.join("vendor_secret_key.bin"),
        data_dir.join("vendor_secret_key.bin"),
    ];

    for path in &candidates {
        if path.exists() {
            let key = std::fs::read(path)
                .map_err(|e| format!("Error leyendo {}: {e}", path.display()))?;
            if key.len() == 32 || key.len() == 64 {
                return Ok(key);
            }
        }
    }

    Err("vendor_secret_key.bin no encontrado (se requiere 64 bytes). Ejecuta: license-tool generate-keypair".to_string())
}

/// Guarda una licencia firmada (bytes ya firmados) en disco.
#[allow(dead_code)]
pub fn save_signed_license(data: &[u8], data_dir: &Path) -> Result<(), String> {
    let license_path = data_dir.join("license.dat");
    std::fs::write(&license_path, data).map_err(|e| format!("Error guardando licencia: {e}"))?;
    Ok(())
}

/// Valida la licencia contra el hardware actual.
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

/// Intenta validar la licencia online contra el Cloudflare Worker.
/// Retorna Some(licencia) si la activación fue exitosa, None si no hay internet o falla.
pub async fn validate_online(
    api_url: &str,
    license_key: &str,
    hardware_hash: &str,
    trial_days: u32,
) -> Option<License> {
    if api_url.is_empty() {
        tracing::warn!("license_api_url vacío — saltando validación online");
        return None;
    }

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .danger_accept_invalid_certs(true)
        .build()
        .ok()?;

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
            return None;
        }
    };

    if !resp.status().is_success() {
        let status = resp.status();
        let body_text = resp.text().await.unwrap_or_default();
        tracing::warn!("Worker respondió con status {}: {}", status, body_text);
        return None;
    }

    let json: serde_json::Value = resp.json().await.ok()?;

    if json.get("success").and_then(|v| v.as_bool()) != Some(true) {
        let error = json
            .get("error")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        tracing::warn!("Worker rechazó activación: {}", error);
        return None;
    }

    let lic_data = json.get("license")?;

    let tier_str = lic_data.get("tier").and_then(|v| v.as_str())?;
    let tier = match tier_str {
        "trial" => LicenseTier::Trial,
        "base" => LicenseTier::Base,
        "reports" => LicenseTier::Reports,
        "advanced" => LicenseTier::Advanced,
        "api" => LicenseTier::Api,
        _ => {
            tracing::warn!("Tier desconocido: {}", tier_str);
            return None;
        }
    };

    let hw = lic_data
        .get("hardware_hash")
        .and_then(|v| v.as_str())
        .unwrap_or(hardware_hash);

    let max_viewers = lic_data
        .get("max_concurrent_viewers")
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
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .unwrap_or_else(chrono::Utc::now);

    // Usar la key real del Worker (no "trial")
    let real_key = lic_data
        .get("license_key")
        .and_then(|v| v.as_str())
        .unwrap_or(license_key)
        .to_string();

    let expires_at = if matches!(tier, LicenseTier::Trial) {
        Some(activated_at + chrono::Duration::days(trial_days as i64))
    } else {
        None
    };

    tracing::info!(
        "Licencia activada online: key={}, tier={}, hw={}, expires={:?}",
        real_key, tier_str, hw, expires_at
    );

    Some(License {
        license_key: real_key,
        tier,
        hardware_hash: hw.to_string(),
        max_viewers,
        max_transfers,
        transfer_count,
        activated_at,
        expires_at,
    })
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
}

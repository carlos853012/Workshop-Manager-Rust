use std::path::Path;

use workshop_common::features::License;
use workshop_common::hardware;
use workshop_common::license as lic;

/// Clave pública del vendor para verificar licencias.
/// IMPORTANTE: En producción, generar un par de claves y reemplazar esta constant.
/// Usar `license-tool generate-keypair` para generar las claves.
const VENDOR_PUBLIC_KEY: &[u8] = &[
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

/// URL del servidor de validación online.
const VALIDATION_URL: &str = "https://tudominio.com/api/validate";

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
        // Primer uso: extraer hardware hash y crear trial
        match hardware::get_hardware_id() {
            Ok(hw_hash) => LicenseStatus::FirstRun(hw_hash),
            Err(e) => LicenseStatus::Invalid(format!("No se pudo extraer hardware: {e}")),
        }
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
    let license_path = data_dir.join("license.dat");
    let data = lic::sign_license(license, &[]).map_err(|e| format!("Error firmando: {e}"))?;
    std::fs::write(&license_path, data).map_err(|e| format!("Error guardando licencia: {e}"))?;
    Ok(())
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

    Ok(())
}

/// Intenta validar la licencia online (para primer uso).
/// Retorna Some(licencia) si la validación fue exitosa, None si no hay internet.
pub async fn validate_online(license_key: &str, hardware_hash: &str) -> Option<License> {
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .ok()?;

    let body = serde_json::json!({
        "license_key": license_key,
        "hardware_hash": hardware_hash,
    });

    let resp = client.post(VALIDATION_URL).json(&body).send().await.ok()?;

    if !resp.status().is_success() {
        return None;
    }

    resp.json::<License>().await.ok()
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

use crate::features::License;
use ed25519_dalek::{Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;
use sha2::{Digest, Sha256};

/// Extrae un hash del hardware para vincular la licencia a una máquina específica.
pub fn extract_hardware_hash(cpu_id: &str, motherboard: &str, disk: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(cpu_id.as_bytes());
    hasher.update(motherboard.as_bytes());
    hasher.update(disk.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Genera un par de claves Ed25519 (secret_key, public_key).
/// La secret_key se guarda con el vendor, la public_key se embebe en el binario.
pub fn generate_keypair() -> (Vec<u8>, Vec<u8>) {
    let signing_key = SigningKey::generate(&mut OsRng);
    let verifying_key = signing_key.verifying_key();
    (
        signing_key.to_bytes().to_vec(),
        verifying_key.to_bytes().to_vec(),
    )
}

/// Firma una licencia con la clave privada del vendor.
/// Retorna la licencia serializada + firma.
pub fn sign_license(license: &License, secret_key: &[u8]) -> Result<Vec<u8>, String> {
    let signing_key = SigningKey::from_bytes(
        secret_key
            .try_into()
            .map_err(|_| "Clave secreta inválida (se esperan 32 bytes)".to_string())?,
    );

    let license_json =
        serde_json::to_vec(license).map_err(|e| format!("Error serializando licencia: {e}"))?;

    let signature = signing_key.sign(&license_json);

    // Empaquetar: [4 bytes len][license_json][64 bytes signature]
    let len = (license_json.len() as u32).to_le_bytes();
    let mut payload = Vec::with_capacity(4 + license_json.len() + 64);
    payload.extend_from_slice(&len);
    payload.extend_from_slice(&license_json);
    payload.extend_from_slice(&signature.to_bytes());

    Ok(payload)
}

/// Verifica y desempaqueta una licencia firmada.
pub fn verify_license(data: &[u8], public_key: &[u8]) -> Result<License, String> {
    if data.len() < 4 + 64 {
        return Err("Datos de licencia inválidos".to_string());
    }

    let verifying_key = VerifyingKey::from_bytes(
        public_key
            .try_into()
            .map_err(|_| "Clave pública inválida".to_string())?,
    )
    .map_err(|e| format!("Clave pública inválida: {e}"))?;

    // Leer longitud del JSON
    let len = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;

    if data.len() < 4 + len + 64 {
        return Err("Datos de licencia truncados".to_string());
    }

    let license_json = &data[4..4 + len];
    let signature_bytes = &data[4 + len..4 + len + 64];

    let signature = ed25519_dalek::Signature::from_bytes(
        signature_bytes
            .try_into()
            .map_err(|_| "Firma inválida".to_string())?,
    );

    verifying_key
        .verify(license_json, &signature)
        .map_err(|e| format!("Firma inválida: {e}"))?;

    let license: License = serde_json::from_slice(license_json)
        .map_err(|e| format!("Error deserializando licencia: {e}"))?;

    Ok(license)
}

/// Valida que la licencia corresponda al hardware actual.
pub fn validate_hardware(license: &License, current_hardware_hash: &str) -> bool {
    license.hardware_hash == current_hardware_hash
}

/// Genera una licencia de trial (sin firma, para primer uso).
/// Expira en `trial_days` días desde la creación (default: 7).
pub fn create_trial_license(hardware_hash: &str) -> License {
    create_trial_license_with_days(hardware_hash, 7)
}

/// Genera una licencia de trial con duración personalizada.
pub fn create_trial_license_with_days(hardware_hash: &str, trial_days: u32) -> License {
    let now = chrono::Utc::now();
    License {
        license_key: "TRIAL".to_string(),
        tier: crate::features::LicenseTier::Trial,
        hardware_hash: hardware_hash.to_string(),
        max_viewers: 1,
        max_transfers: 0,
        transfer_count: 0,
        activated_at: now,
        expires_at: Some(now + chrono::Duration::days(trial_days as i64)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::LicenseTier;

    #[test]
    fn test_extract_hardware_hash() {
        let hash1 = extract_hardware_hash("cpu1", "mb1", "disk1");
        let hash2 = extract_hardware_hash("cpu1", "mb1", "disk1");
        let hash3 = extract_hardware_hash("cpu2", "mb1", "disk1");

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
        assert_eq!(hash1.len(), 64);
    }

    #[test]
    fn test_sign_and_verify() {
        let (secret_key, public_key) = generate_keypair();

        let license = License {
            license_key: "TEST-1234".to_string(),
            tier: LicenseTier::Base,
            hardware_hash: "abc123".to_string(),
            max_viewers: 2,
            max_transfers: 3,
            transfer_count: 0,
            activated_at: chrono::Utc::now(),
            expires_at: None,
        };

        let signed = sign_license(&license, &secret_key).expect("sign should succeed");
        let verified = verify_license(&signed, &public_key).expect("verify should succeed");

        assert_eq!(verified.license_key, "TEST-1234");
        assert_eq!(verified.tier, LicenseTier::Base);
        assert_eq!(verified.hardware_hash, "abc123");
    }

    #[test]
    fn test_verify_wrong_key_fails() {
        let (secret_key, _) = generate_keypair();
        let (_, wrong_public_key) = generate_keypair();

        let license = License {
            license_key: "TEST".to_string(),
            tier: LicenseTier::Base,
            hardware_hash: "abc".to_string(),
            max_viewers: 2,
            max_transfers: 3,
            transfer_count: 0,
            activated_at: chrono::Utc::now(),
            expires_at: None,
        };

        let signed = sign_license(&license, &secret_key).expect("sign should succeed");
        assert!(verify_license(&signed, &wrong_public_key).is_err());
    }

    #[test]
    fn test_validate_hardware() {
        let license = License {
            license_key: "TEST".to_string(),
            tier: LicenseTier::Base,
            hardware_hash: "abc123".to_string(),
            max_viewers: 2,
            max_transfers: 3,
            transfer_count: 0,
            activated_at: chrono::Utc::now(),
            expires_at: None,
        };

        assert!(validate_hardware(&license, "abc123"));
        assert!(!validate_hardware(&license, "xyz789"));
    }

    #[test]
    fn test_create_trial_license() {
        let license = create_trial_license("hw_hash");
        assert_eq!(license.license_key, "TRIAL");
        assert_eq!(license.tier, LicenseTier::Trial);
        assert_eq!(license.max_viewers, 1);
        assert!(!license.can_transfer());
        assert!(license.expires_at.is_some());
        assert!(license.is_valid());
        assert!(!license.is_expired());
    }
}

use crate::features::License;
use sha2::{Digest, Sha256};

/// Extrae un hash del hardware para vincular la licencia a una máquina específica
pub fn extract_hardware_hash(cpu_id: &str, motherboard: &str, disk: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(cpu_id.as_bytes());
    hasher.update(motherboard.as_bytes());
    hasher.update(disk.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Valida que la licencia corresponda al hardware actual
pub fn validate_hardware(license: &License) -> bool {
    match &license.hardware_hash {
        Some(_) => true, // En producción, comparar con hardware actual
        None => true,    // Sin binding de hardware = válido
    }
}

/// Verifica la firma Ed25519 de la licencia
pub fn verify_license_signature(_license: &License, _public_key: &[u8]) -> bool {
    // En producción, verificar firma Ed25519
    // Por ahora, retornar true para desarrollo
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::{Feature, LicenseTier};

    #[test]
    fn test_extract_hardware_hash() {
        let hash1 = extract_hardware_hash("cpu1", "mb1", "disk1");
        let hash2 = extract_hardware_hash("cpu1", "mb1", "disk1");
        let hash3 = extract_hardware_hash("cpu2", "mb1", "disk1");

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_validate_hardware_no_binding() {
        let license = License {
            key: "TEST".to_string(),
            tier: LicenseTier::Base,
            features: vec![Feature::Inventory],
            purchased_at: chrono::Utc::now(),
            hardware_hash: None,
        };
        assert!(validate_hardware(&license));
    }
}

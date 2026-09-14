use sha2::{Digest, Sha256};
use std::process::Command;

/// Extrae un identificador de hardware del PC actual usando comandos de Windows.
/// Retorna un hash SHA-256 de CPU + Motherboard + Disk.
pub fn get_hardware_id() -> Result<String, String> {
    let cpu = wmic_value("cpu", "ProcessorId").unwrap_or_default();
    let mb = wmic_value("baseboard", "SerialNumber").unwrap_or_default();
    let disk = wmic_value("diskdrive", "SerialNumber").unwrap_or_default();

    if cpu.is_empty() && mb.is_empty() && disk.is_empty() {
        return Err("No se pudo extraer información de hardware".to_string());
    }

    let mut hasher = Sha256::new();
    hasher.update(cpu.as_bytes());
    hasher.update(mb.as_bytes());
    hasher.update(disk.as_bytes());
    Ok(format!("{:x}", hasher.finalize()))
}

/// Ejecuta `wmic <class> get <field>` y limpia el resultado.
fn wmic_value(class: &str, field: &str) -> Option<String> {
    let output = Command::new("wmic")
        .args([class, "get", field])
        .output()
        .ok()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();

    if lines.len() < 2 {
        return None;
    }

    let value = lines[1].trim();
    if value.is_empty() || value == "To be filled by O.E.M." {
        None
    } else {
        Some(value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_hardware_id_returns_hash() {
        let result = get_hardware_id();
        if let Ok(hash) = result {
            assert_eq!(hash.len(), 64); // SHA-256 hex = 64 chars
        }
    }

    #[test]
    fn test_deterministic() {
        let h1 = get_hardware_id();
        let h2 = get_hardware_id();
        if let (Ok(a), Ok(b)) = (h1, h2) {
            assert_eq!(a, b);
        }
    }
}

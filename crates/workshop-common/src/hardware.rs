use sha2::{Digest, Sha256};
use std::process::Command;

/// Extrae un identificador de hardware del PC actual.
/// En Windows usa `PowerShell Get-CimInstance`, en Linux lee `/sys/class/dmi/id/`.
/// Retorna un hash SHA-256 de CPU + Motherboard + Disk.
pub fn get_hardware_id() -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
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

    #[cfg(target_os = "linux")]
    {
        let cpu = std::fs::read_to_string("/sys/class/dmi/id/board_serial")
            .or_else(|_| std::fs::read_to_string("/sys/class/dmi/id/product_serial"))
            .unwrap_or_default()
            .trim()
            .to_string();
        let mb = std::fs::read_to_string("/sys/class/dmi/id/board_name")
            .unwrap_or_default()
            .trim()
            .to_string();
        let disk = disk_serial_linux();

        if cpu.is_empty() && mb.is_empty() && disk.is_empty() {
            return Err("No se pudo extraer información de hardware".to_string());
        }

        let mut hasher = Sha256::new();
        hasher.update(cpu.as_bytes());
        hasher.update(mb.as_bytes());
        hasher.update(disk.as_bytes());
        Ok(format!("{:x}", hasher.finalize()))
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        Err("Plataforma no soportada para extracción de hardware".to_string())
    }
}

/// Intenta obtener el serial del disco principal en Linux.
#[cfg(target_os = "linux")]
fn disk_serial_linux() -> String {
    if let Ok(output) = Command::new("lsblk")
        .args(["-dno", "SERIAL", "/dev/sda"])
        .output()
    {
        let serial = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !serial.is_empty() {
            return serial;
        }
    }
    std::fs::read_to_string("/sys/block/sda/device/../serial")
        .or_else(|_| std::fs::read_to_string("/sys/class/block/sda/device/serial"))
        .unwrap_or_default()
        .trim()
        .to_string()
}

/// Ejecuta `Get-CimInstance <class> | Select-Object -ExpandProperty <field>` y limpia el resultado.
/// Reemplaza wmic (deprecado en Windows 11+).
#[cfg(target_os = "windows")]
fn wmic_value(class: &str, field: &str) -> Option<String> {
    use std::os::windows::process::CommandExt;

    let cim_class = match class {
        "cpu" => "Win32_Processor",
        "baseboard" => "Win32_BaseBoard",
        "diskdrive" => "Win32_DiskDrive",
        _ => return None,
    };

    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            &format!(
                "Get-CimInstance {} | Select-Object -ExpandProperty {}",
                cim_class, field
            ),
        ])
        .creation_flags(0x0800_0000)
        .output()
        .ok()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let value = stdout.trim();

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

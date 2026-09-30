use sha2::{Digest, Sha256};

/// Extrae un identificador de hardware del PC actual.
/// En Windows usa `MachineGuid` del registro (primario) o PowerShell como fallback.
/// En Linux lee `/sys/class/dmi/id/`.
/// Retorna un hash SHA-256 del identificador de hardware.
pub fn get_hardware_id() -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        if let Some(guid) = read_machine_guid() {
            return Ok(hash_single(&guid));
        }

        if let Some(id) = hardware_id_via_powershell() {
            return Ok(hash_single(&id));
        }

        Err("No se pudo extraer información de hardware".to_string())
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
    use std::process::Command;

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

/// Hashea un valor único con SHA-256.
#[cfg(target_os = "windows")]
fn hash_single(value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Lee `MachineGuid` del registro de Windows.
/// Es estable, único por máquina y no lanza procesos.
#[cfg(target_os = "windows")]
fn read_machine_guid() -> Option<String> {
    use winreg::enums::{HKEY_LOCAL_MACHINE, KEY_READ};
    use winreg::RegKey;

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let key = hklm
        .open_subkey_with_flags("SOFTWARE\\Microsoft\\Cryptography", KEY_READ)
        .ok()?;

    let value: String = key.get_value("MachineGuid").ok()?;
    let trimmed = value.trim().to_string();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

/// Fallback: ejecuta un ÚNICO proceso PowerShell para obtener CPU + BaseBoard + Disk serials.
/// Retorna un identificador compuesto o `None` si falla.
#[cfg(target_os = "windows")]
fn hardware_id_via_powershell() -> Option<String> {
    use std::os::windows::process::CommandExt;

    let script = r#"
        $cpu = (Get-CimInstance Win32_Processor | Select-Object -ExpandProperty ProcessorId).Trim()
        $mb  = (Get-CimInstance Win32_BaseBoard | Select-Object -ExpandProperty SerialNumber).Trim()
        $disk = (Get-CimInstance Win32_DiskDrive | Select-Object -ExpandProperty SerialNumber).Trim()
        Write-Output $cpu
        Write-Output $mb
        Write-Output $disk
    "#;

    let output = std::process::Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", script])
        .creation_flags(0x0800_0000)
        .output()
        .ok()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    if lines.len() != 3 {
        return None;
    }

    let invalid = ["", "To be filled by O.E.M."];
    let fields: Vec<String> = lines
        .iter()
        .map(|p| p.trim().to_string())
        .filter(|p| !invalid.contains(&p.as_str()))
        .collect();

    if fields.is_empty() {
        return None;
    }

    Some(fields.join("|"))
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

    #[cfg(target_os = "windows")]
    #[test]
    fn test_hash_single_deterministic() {
        let a = hash_single("test-value-123");
        let b = hash_single("test-value-123");
        assert_eq!(a, b);
        assert_eq!(a.len(), 64);
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_hash_single_different_inputs() {
        let a = hash_single("abc");
        let b = hash_single("xyz");
        assert_ne!(a, b);
    }
}

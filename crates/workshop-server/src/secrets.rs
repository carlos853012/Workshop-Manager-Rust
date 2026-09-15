use rand::rngs::OsRng;
use rand::Rng;
use std::path::Path;

use super::state::Secrets;

pub fn init_secrets(data_dir: &Path) -> anyhow::Result<Secrets> {
    let jwt_secret = load_or_generate_jwt_secret(data_dir)?;
    let crypto_key = load_or_generate_crypto_key(data_dir)?;

    Ok(Secrets {
        jwt_secret,
        crypto_key,
    })
}

fn load_or_generate_jwt_secret(data_dir: &Path) -> anyhow::Result<String> {
    let secret_path = data_dir.join(".jwt_secret");

    if secret_path.exists() {
        let secret = std::fs::read_to_string(&secret_path)?;
        return Ok(secret.trim().to_string());
    }

    // Generar nuevo secreto con OsRng (CSPRNG) — 32 bytes hex = 64 chars
    use rand::Fill;
    let mut bytes = [0u8; 32];
    bytes.try_fill(&mut OsRng).expect("OsRng should not fail");
    let secret: String = bytes.iter().map(|b| format!("{:02x}", b)).collect();

    std::fs::write(&secret_path, &secret)?;

    // Restringir permisos en Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&secret_path)?.permissions();
        perms.set_mode(0o600);
        std::fs::set_permissions(&secret_path, perms)?;
    }

    // En Windows, marcar como oculto para proteger archivos sensibles
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("attrib")
            .arg("+H")
            .arg(&secret_path)
            .output();
    }

    Ok(secret)
}

fn load_or_generate_crypto_key(data_dir: &Path) -> anyhow::Result<Vec<u8>> {
    let key_path = data_dir.join(".crypto_key");

    if key_path.exists() {
        let key = std::fs::read(&key_path)?;
        if key.len() == 32 {
            return Ok(key);
        }
    }

    // Generar nueva clave con OsRng (CSPRNG)
    let mut key = vec![0u8; 32];
    OsRng.fill(&mut key[..]);
    std::fs::write(&key_path, &key)?;

    // Restringir permisos en Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&key_path)?.permissions();
        perms.set_mode(0o600);
        std::fs::set_permissions(&key_path, perms)?;
    }

    // En Windows, marcar como oculto para proteger archivos sensibles
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("attrib")
            .arg("+H")
            .arg(&key_path)
            .output();
    }

    Ok(key)
}

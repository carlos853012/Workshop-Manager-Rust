use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose, Engine as _};
use rand::RngCore;
use std::path::Path;
use std::sync::OnceLock;

static CIPHER: OnceLock<Aes256Gcm> = OnceLock::new();

pub fn init(data_dir: &Path) -> anyhow::Result<()> {
    let key_path = data_dir.join(".crypto_key");

    if !key_path.exists() {
        return Err(anyhow::anyhow!(
            "Crypto key not found. Run secrets::init_secrets first."
        ));
    }

    let key = std::fs::read(&key_path)?;
    if key.len() != 32 {
        return Err(anyhow::anyhow!("Invalid crypto key length"));
    }

    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| anyhow::anyhow!("Failed to create cipher: {}", e))?;

    let _ = CIPHER.set(cipher);

    Ok(())
}

fn get_cipher() -> anyhow::Result<&'static Aes256Gcm> {
    CIPHER
        .get()
        .ok_or_else(|| anyhow::anyhow!("Cipher not initialized"))
}

pub fn encrypt(plaintext: &str) -> anyhow::Result<String> {
    encrypt_bytes(plaintext.as_bytes())
}

pub fn encrypt_bytes(plaintext: &[u8]) -> anyhow::Result<String> {
    let cipher = get_cipher()?;

    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;

    // Formato: base64(nonce || ciphertext)
    let mut combined = nonce_bytes.to_vec();
    combined.extend(ciphertext);

    Ok(general_purpose::STANDARD.encode(combined))
}

pub fn decrypt(ciphertext: &str) -> anyhow::Result<String> {
    let cipher = get_cipher()?;

    let combined = general_purpose::STANDARD.decode(ciphertext)?;
    if combined.len() < 12 {
        return Err(anyhow::anyhow!("Invalid ciphertext length"));
    }

    let (nonce_bytes, ciphertext_bytes) = combined.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);

    let plaintext = cipher
        .decrypt(nonce, ciphertext_bytes)
        .map_err(|e| anyhow::anyhow!("Decryption failed: {}", e))?;

    String::from_utf8(plaintext).map_err(|e| anyhow::anyhow!("Invalid UTF-8: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::OnceLock;

    static SETUP_RESULT: OnceLock<Option<String>> = OnceLock::new();

    fn setup_test() -> anyhow::Result<()> {
        let err_msg = SETUP_RESULT.get_or_init(|| {
            let test_dir = PathBuf::from("test_data");
            if let Err(e) = std::fs::create_dir_all(&test_dir) {
                return Some(format!("create_dir: {e}"));
            }

            let key_path = test_dir.join(".crypto_key");
            let mut key = vec![0u8; 32];
            OsRng.fill_bytes(&mut key);
            if let Err(e) = std::fs::write(&key_path, &key) {
                return Some(format!("write key: {e}"));
            }

            if let Err(e) = init(&test_dir) {
                return Some(format!("init: {e}"));
            }
            None
        });
        match err_msg {
            Some(msg) => Err(anyhow::anyhow!("{msg}")),
            None => Ok(()),
        }
    }

    #[test]
    fn test_encrypt_decrypt() -> anyhow::Result<()> {
        setup_test()?;

        let original = "Hello, WorkshopManager!";
        let encrypted = encrypt(original)?;
        let decrypted = decrypt(&encrypted)?;

        assert_eq!(original, decrypted);
        assert_ne!(original, encrypted);
        Ok(())
    }

    #[test]
    fn test_different_ciphertexts() -> anyhow::Result<()> {
        setup_test()?;

        let original = "Same text";
        let enc1 = encrypt(original)?;
        let enc2 = encrypt(original)?;

        // Nonces aleatorios hacen que los ciphertexts sean diferentes
        assert_ne!(enc1, enc2);
        Ok(())
    }
}

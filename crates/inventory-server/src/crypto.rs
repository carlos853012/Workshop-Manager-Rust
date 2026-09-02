use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use base64::{Engine as _, engine::general_purpose};
use rand::RngCore;
use std::path::Path;

static mut CIPHER: Option<Aes256Gcm> = None;

pub fn init(data_dir: &Path) -> anyhow::Result<()> {
    let key_path = data_dir.join(".crypto_key");

    if !key_path.exists() {
        return Err(anyhow::anyhow!("Crypto key not found. Run secrets::init_secrets first."));
    }

    let key = std::fs::read(&key_path)?;
    if key.len() != 32 {
        return Err(anyhow::anyhow!("Invalid crypto key length"));
    }

    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| anyhow::anyhow!("Failed to create cipher: {}", e))?;

    unsafe {
        CIPHER = Some(cipher);
    }

    Ok(())
}

pub fn encrypt(plaintext: &str) -> anyhow::Result<String> {
    let cipher = unsafe {
        CIPHER.as_ref().ok_or_else(|| anyhow::anyhow!("Cipher not initialized"))?
    };

    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;

    // Formato: base64(nonce || ciphertext)
    let mut combined = nonce_bytes.to_vec();
    combined.extend(ciphertext);

    Ok(general_purpose::STANDARD.encode(combined))
}

pub fn decrypt(ciphertext: &str) -> anyhow::Result<String> {
    let cipher = unsafe {
        CIPHER.as_ref().ok_or_else(|| anyhow::anyhow!("Cipher not initialized"))?
    };

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

    fn setup_test() {
        let test_dir = PathBuf::from("test_data");
        std::fs::create_dir_all(&test_dir).unwrap();

        let key_path = test_dir.join(".crypto_key");
        if !key_path.exists() {
            let mut key = vec![0u8; 32];
            OsRng.fill_bytes(&mut key);
            std::fs::write(&key_path, &key).unwrap();
        }

        init(&test_dir).unwrap();
    }

    #[test]
    fn test_encrypt_decrypt() {
        setup_test();

        let original = "Hello, WorkshopManager!";
        let encrypted = encrypt(original).unwrap();
        let decrypted = decrypt(&encrypted).unwrap();

        assert_eq!(original, decrypted);
        assert_ne!(original, encrypted);
    }

    #[test]
    fn test_different_ciphertexts() {
        setup_test();

        let original = "Same text";
        let enc1 = encrypt(original).unwrap();
        let enc2 = encrypt(original).unwrap();

        // Nonces aleatorios hacen que los ciphertexts sean diferentes
        assert_ne!(enc1, enc2);
    }
}

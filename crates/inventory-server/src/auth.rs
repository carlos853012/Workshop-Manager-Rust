// Auth module - JWT + Argon2
// TODO: Implementar en Fase 3

pub fn hash_password(password: &str) -> anyhow::Result<String> {
    use argon2::password_hash::SaltString;
    use argon2::password_hash::PasswordHasher;
    use rand::rngs::OsRng;

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = argon2::Argon2::default();
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!("Failed to hash password: {}", e))?
        .to_string();

    Ok(hash)
}

pub fn verify_password(hash: &str, password: &str) -> anyhow::Result<bool> {
    use argon2::password_hash::PasswordHash;
    use argon2::PasswordVerifier;

    let parsed_hash = PasswordHash::new(hash)
        .map_err(|e| anyhow::anyhow!("Failed to parse hash: {}", e))?;

    let result = argon2::Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash);

    Ok(result.is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_and_verify() {
        let password = "mi_password_seguro";
        let hash = hash_password(password).unwrap();
        assert!(verify_password(&hash, password).unwrap());
        assert!(!verify_password(&hash, "wrong_password").unwrap());
    }
}

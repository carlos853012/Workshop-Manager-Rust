use argon2::password_hash::{PasswordHash, PasswordHasher, SaltString};
use argon2::{Argon2, PasswordVerifier};
use chrono::{Duration, Utc};
use inventory_common::UserRole;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const TOKEN_DURATION_HOURS: i64 = 8;

/// Claims incluidos en el JWT.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    /// ID del usuario (subject).
    pub sub: String,
    /// Email del usuario.
    pub email: String,
    /// Rol como string serializado.
    pub role: String,
    /// Workshop ID
    pub workshop_id: String,
    /// Tiempo de expiración (timestamp UTC).
    pub exp: usize,
}

/// Hashea una contraseña usando Argon2id.
pub fn hash_password(password: &str) -> anyhow::Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!("Failed to hash password: {}", e))?
        .to_string();

    Ok(hash)
}

/// Verifica una contraseña contra un hash Argon2.
pub fn verify_password(hash: &str, password: &str) -> anyhow::Result<bool> {
    let parsed_hash =
        PasswordHash::new(hash).map_err(|e| anyhow::anyhow!("Failed to parse hash: {}", e))?;

    let result = Argon2::default().verify_password(password.as_bytes(), &parsed_hash);

    Ok(result.is_ok())
}

/// Crea un JWT para el usuario dado.
pub fn create_token(
    user_id: Uuid,
    email: &str,
    role: UserRole,
    workshop_id: Uuid,
    secret: &str,
) -> anyhow::Result<String> {
    let exp = Utc::now()
        .checked_add_signed(Duration::hours(TOKEN_DURATION_HOURS))
        .ok_or_else(|| anyhow::anyhow!("Failed to compute token expiration"))?
        .timestamp() as usize;

    let claims = Claims {
        sub: user_id.to_string(),
        email: email.to_string(),
        role: role.to_string(),
        workshop_id: workshop_id.to_string(),
        exp,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| anyhow::anyhow!("Failed to create token: {}", e))
}

/// Valida un JWT y retorna sus claims.
pub fn validate_token(token: &str, secret: &str) -> anyhow::Result<Claims> {
    let validation = Validation::default();
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .map_err(|e| anyhow::anyhow!("Invalid token: {}", e))?;

    Ok(token_data.claims)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_and_verify() -> anyhow::Result<()> {
        let password = "mi_password_seguro";
        let hash = hash_password(password)?;
        assert!(verify_password(&hash, password)?);
        assert!(!verify_password(&hash, "wrong_password")?);
        Ok(())
    }

    #[test]
    fn test_create_and_validate_token() -> anyhow::Result<()> {
        let user_id = Uuid::new_v4();
        let workshop_id = Uuid::new_v4();
        let email = "test@example.com";
        let role = UserRole::Admin;
        let secret = "test_secret_32_bytes_long_value";

        let token = create_token(user_id, email, role.clone(), workshop_id, secret)?;
        let claims = validate_token(&token, secret)?;

        assert_eq!(claims.sub, user_id.to_string());
        assert_eq!(claims.email, email);
        assert_eq!(claims.role, role.to_string());
        assert_eq!(claims.workshop_id, workshop_id.to_string());
        Ok(())
    }

    #[test]
    fn test_validate_token_fails_with_wrong_secret() -> anyhow::Result<()> {
        let user_id = Uuid::new_v4();
        let workshop_id = Uuid::new_v4();
        let token = create_token(
            user_id,
            "test@example.com",
            UserRole::Seller,
            workshop_id,
            "secret_a",
        )?;
        assert!(validate_token(&token, "secret_b").is_err());
        Ok(())
    }
}

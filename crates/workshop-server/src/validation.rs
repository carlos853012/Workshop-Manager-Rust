use once_cell::sync::Lazy;
use regex::Regex;
use workshop_common::User;

use crate::error::AppError;

static EMAIL_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[^\s@]+@[^\s@]+\.[^\s@]+$").expect("Invalid email regex"));

/// Validates an email address. Returns Ok(()) if valid, Err(AppError) if invalid.
pub fn validate_email(email: &str) -> Result<(), AppError> {
    if email.is_empty() || email.len() > 200 {
        return Err(AppError::Validation(
            "Email inválido: longitud fuera de rango".to_string(),
        ));
    }
    if !EMAIL_RE.is_match(email) {
        return Err(AppError::Validation(
            "Email inválido: formato incorrecto".to_string(),
        ));
    }
    Ok(())
}

/// Validates that a password meets minimum requirements.
pub fn validate_password(password: &str) -> Result<(), AppError> {
    if password.len() < 8 {
        return Err(AppError::Validation(
            "Password debe tener al menos 8 caracteres".to_string(),
        ));
    }
    Ok(())
}

/// Removes the password hash from a User struct for safe transmission.
pub fn hide_password_hash(mut user: User) -> User {
    user.password_hash = String::new();
    user
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_email() {
        assert!(validate_email("user@example.com").is_ok());
        assert!(validate_email("admin@taller.cl").is_ok());
        assert!(validate_email("test.user@domain.co").is_ok());
    }

    #[test]
    fn test_invalid_email() {
        assert!(validate_email("").is_err());
        assert!(validate_email("invalid").is_err());
        assert!(validate_email("@.").is_err());
        assert!(validate_email("no-at-sign.com").is_err());
        assert!(validate_email("no@domain").is_err());
    }

    #[test]
    fn test_valid_password() {
        assert!(validate_password("12345678").is_ok());
        assert!(validate_password("longpassword").is_ok());
    }

    #[test]
    fn test_invalid_password() {
        assert!(validate_password("short").is_err());
        assert!(validate_password("").is_err());
    }
}

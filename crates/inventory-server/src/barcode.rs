/// Generate an EAN-13 barcode from a 12-digit prefix + item number
pub fn generate_ean13(prefix: &str) -> String {
    // prefix is 6 digits (workshop) + 6 digits (item) = 12 digits
    // We need 12 digits for EAN-13 (12 + check digit = 13)
    // Format: 6 digits prefix + 6 digits item + 1 check digit = 13
    // For simplicity, we use a 6-char prefix + 6-digit counter + check digit

    // Extract numeric prefix (6 digits)
    let prefix = prefix.chars().filter(|c| c.is_ascii_digit()).collect::<String>();
    let prefix = format!("{:0>6}", prefix); // Pad to 6 digits

    // For now, use a simple counter approach - in production use a DB sequence
    // For this implementation, we'll generate a random 6-digit suffix
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let suffix: u32 = rng.gen_range(0..999999);
    let suffix_str = format!("{:06}", suffix);

    let base = format!("{}{}", prefix, suffix_str);
    let check_digit = calculate_ean13_check_digit(&base);
    format!("{}{}", base, check_digit)
}

#[allow(dead_code)]
pub fn validate_ean13(code: &str) -> Result<(), String> {
    if code.len() != 13 {
        return Err("EAN-13 must be 13 digits".to_string());
    }
    if !code.chars().all(|c| c.is_ascii_digit()) {
        return Err("EAN-13 must contain only digits".to_string());
    }

    let base = &code[..12];
    let check_digit = code.chars().nth(12).unwrap() as u32 - '0' as u32;
    let expected = calculate_ean13_check_digit(base);

    if check_digit == expected {
        Ok(())
    } else {
        Err("Invalid EAN-13 check digit".to_string())
    }
}

fn calculate_ean13_check_digit(base: &str) -> u32 {
    let mut sum = 0u32;
    for (i, ch) in base.chars().enumerate() {
        let digit = ch as u32 - '0' as u32;
        if i % 2 == 0 {
            sum += digit;
        } else {
            sum += digit * 3;
        }
    }
    let remainder = sum % 10;
    if remainder == 0 {
        0
    } else {
        10 - remainder
    }
}

/// Get the workshop's barcode prefix
pub async fn get_workshop_prefix(pool: &sqlx::PgPool, workshop_id: uuid::Uuid) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let prefix: Option<String> = sqlx::query_scalar("SELECT barcode_prefix FROM workshops WHERE id = $1")
        .bind(workshop_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

    match prefix {
        Some(p) if !p.is_empty() => Ok(p),
        _ => {
            // Generate a prefix from workshop ID - use first 6 hex chars
            let prefix = format!("{:06}", workshop_id.simple().to_string()[..6].parse::<u32>().unwrap_or(0) % 1000000);
            // Update the workshop with this prefix
            sqlx::query("UPDATE workshops SET barcode_prefix = $1 WHERE id = $2")
                .bind(&prefix)
                .bind(workshop_id)
                .execute(pool)
                .await
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
            Ok(prefix)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ean13_generation() {
        let prefix = "123456";
        let ean = generate_ean13(prefix);
        assert_eq!(ean.len(), 13);
        assert!(validate_ean13(&ean).is_ok());
    }

    #[test]
    fn test_ean13_validation() {
        assert!(validate_ean13("4006381333931").is_ok());
        assert!(validate_ean13("1234567890123").is_err());
        assert!(validate_ean13("abc").is_err());
    }
}
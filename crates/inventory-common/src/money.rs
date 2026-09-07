use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;

/// Tasa de IVA chilena vigente (19%).
pub fn iva_rate() -> Decimal {
    Decimal::new(19, 2)
}

/// Calcula el IVA de un monto (monto * tasa IVA), redondeado a múltiplo de 10.
pub fn calculate_iva(amount: Decimal) -> Decimal {
    round_to_ten(amount * iva_rate())
}

/// Redondea un monto CLP a la múltiplo de 10 más cercana (Ley del Redondeo chilena).
/// Ej: 15678 → 15680, 15673 → 15670, 15675 → 15680, 999 → 1000
/// Negativos: -15678 → -15680 (redondea hacia afuera del cero)
pub fn round_to_ten(amount: Decimal) -> Decimal {
    let whole = amount.to_i64().unwrap_or(0);
    let abs = whole.abs();
    let remainder = abs % 10;
    let rounded_abs = if remainder >= 5 {
        abs + (10 - remainder)
    } else {
        abs - remainder
    };
    Decimal::from(if whole < 0 { -rounded_abs } else { rounded_abs })
}

/// Formatea un Decimal como CLP: $1.500, $15.680, $100.000
/// Sin decimales, separador de miles con punto, prefijo $
/// Negativos: -$1.500
pub fn format_clp(amount: Decimal) -> String {
    let rounded = round_to_ten(amount);
    let whole = rounded.to_i64().unwrap_or(0);
    let is_negative = whole < 0;
    let abs_str = format!("{}", whole.abs());
    let with_thousands: String = abs_str
        .chars()
        .rev()
        .enumerate()
        .flat_map(|(i, c)| {
            if i > 0 && i % 3 == 0 {
                vec!['.', c]
            } else {
                vec![c]
            }
        })
        .collect::<String>()
        .chars()
        .rev()
        .collect();
    if is_negative {
        format!("-${}", with_thousands)
    } else {
        format!("${}", with_thousands)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iva_rate() {
        assert_eq!(iva_rate(), Decimal::new(19, 2));
    }

    #[test]
    fn test_calculate_iva() {
        // 100 * 0.19 = 19
        assert_eq!(calculate_iva(Decimal::from(100)), Decimal::from(20));
        // 1000 * 0.19 = 190
        assert_eq!(calculate_iva(Decimal::from(1000)), Decimal::from(190));
        // 15000 * 0.19 = 2850
        assert_eq!(calculate_iva(Decimal::from(15000)), Decimal::from(2850));
        // 0 * 0.19 = 0
        assert_eq!(calculate_iva(Decimal::ZERO), Decimal::ZERO);
        // 50 * 0.19 = 9.5 → 10 (redondeo a múltiplo de 10)
        assert_eq!(calculate_iva(Decimal::from(50)), Decimal::from(10));
    }

    #[test]
    fn test_round_to_ten() {
        assert_eq!(round_to_ten(Decimal::from(15678)), Decimal::from(15680));
        assert_eq!(round_to_ten(Decimal::from(15673)), Decimal::from(15670));
        assert_eq!(round_to_ten(Decimal::from(15675)), Decimal::from(15680));
        assert_eq!(round_to_ten(Decimal::from(15685)), Decimal::from(15690));
        assert_eq!(round_to_ten(Decimal::from(999)), Decimal::from(1000));
        assert_eq!(round_to_ten(Decimal::from(15)), Decimal::from(20));
        assert_eq!(round_to_ten(Decimal::from(5)), Decimal::from(10));
        assert_eq!(round_to_ten(Decimal::ZERO), Decimal::ZERO);
        assert_eq!(round_to_ten(Decimal::from(10)), Decimal::from(10));
        assert_eq!(round_to_ten(Decimal::from(14)), Decimal::from(10));
        assert_eq!(round_to_ten(Decimal::from(16)), Decimal::from(20));
        // Negativos
        assert_eq!(round_to_ten(Decimal::from(-15678)), Decimal::from(-15680));
        assert_eq!(round_to_ten(Decimal::from(-5)), Decimal::from(-10));
        // Grandes
        assert_eq!(
            round_to_ten(Decimal::from(999_999_999)),
            Decimal::from(1_000_000_000)
        );
    }

    #[test]
    fn test_format_clp() {
        assert_eq!(format_clp(Decimal::from(1500)), "$1.500");
        assert_eq!(format_clp(Decimal::from(15680)), "$15.680");
        assert_eq!(format_clp(Decimal::from(100000)), "$100.000");
        assert_eq!(format_clp(Decimal::from(999)), "$1.000");
        assert_eq!(format_clp(Decimal::ZERO), "$0");
        // Negativos
        assert_eq!(format_clp(Decimal::from(-1500)), "-$1.500");
        // Sin separador de miles
        assert_eq!(format_clp(Decimal::from(990)), "$990");
        // Un solo dígito redondeado
        assert_eq!(format_clp(Decimal::from(5)), "$10");
        // Monto exacto múltiplo de 10
        assert_eq!(format_clp(Decimal::from(5000)), "$5.000");
    }

    #[test]
    fn test_iva_rounding_chain() {
        // Simulaflujo completo: precio con IVA + redondeo
        let base = Decimal::from(10000);
        let iva = calculate_iva(base); // 10000 * 0.19 = 1900
        assert_eq!(iva, Decimal::from(1900));
        let total = base + iva;
        assert_eq!(total, Decimal::from(11900));
        assert_eq!(format_clp(total), "$11.900");
    }
}

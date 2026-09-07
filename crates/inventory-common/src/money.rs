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
pub fn round_to_ten(amount: Decimal) -> Decimal {
    let whole = amount.to_i64().unwrap_or(0);
    let remainder = whole % 10;
    if remainder >= 5 {
        Decimal::from(whole + (10 - remainder))
    } else {
        Decimal::from(whole - remainder)
    }
}

/// Formatea un Decimal como CLP: $1.500, $15.680, $100.000
/// Sin decimales, separador de miles con punto, prefijo $
pub fn format_clp(amount: Decimal) -> String {
    let rounded = round_to_ten(amount);
    let whole = rounded.to_i64().unwrap_or(0);
    let formatted = format!("{}", whole.abs());
    let with_thousands: String = formatted
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
    format!("${}", with_thousands)
}

#[cfg(test)]
mod tests {
    use super::*;

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
    }

    #[test]
    fn test_format_clp() {
        assert_eq!(format_clp(Decimal::from(1500)), "$1.500");
        assert_eq!(format_clp(Decimal::from(15680)), "$15.680");
        assert_eq!(format_clp(Decimal::from(100000)), "$100.000");
        assert_eq!(format_clp(Decimal::from(999)), "$1.000");
        assert_eq!(format_clp(Decimal::ZERO), "$0");
    }
}

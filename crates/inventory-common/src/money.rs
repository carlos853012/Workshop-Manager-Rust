use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use std::sync::OnceLock;

/// Default IVA rate: 19% (Chilean tax rate).
const DEFAULT_IVA_RATE: (i64, u32) = (19, 2);

static IVA_RATE: OnceLock<Decimal> = OnceLock::new();

/// Returns the configured IVA rate. Defaults to 19% if not set.
/// Call `set_iva_rate()` at application startup to override.
pub fn iva_rate() -> Decimal {
    *IVA_RATE.get_or_init(|| Decimal::new(DEFAULT_IVA_RATE.0, DEFAULT_IVA_RATE.1))
}

/// Set the IVA rate at startup. Must be called before any IVA calculations.
/// Rate is expressed as a decimal fraction (e.g., 0.19 for 19%).
pub fn set_iva_rate(rate: Decimal) {
    let _ = IVA_RATE.set(rate);
}

/// Calcula el IVA de un monto (monto * tasa IVA), redondeado a múltiplo de 10.
pub fn calculate_iva(amount: Decimal) -> Decimal {
    round_to_ten(amount * iva_rate())
}

/// Extrae el IVA de un precio que ya incluye IVA (Chile: precio final = base + IVA).
/// Ej: $11.900 → base=$10.000, iva=$1.900
pub fn extract_iva(price_including_iva: Decimal) -> (Decimal, Decimal) {
    let one_plus_iva = Decimal::ONE + iva_rate();
    let base = round_to_ten(price_including_iva / one_plus_iva);
    let iva = price_including_iva - base;
    (base, iva)
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
        // Default rate should be 19%
        let rate = iva_rate();
        assert_eq!(rate, Decimal::new(19, 2));
    }

    #[test]
    fn test_set_iva_rate_before_first_access() {
        // OnceLock can only be set once per process; test that set_iva_rate
        // is a no-op after the first access (iva_rate() already initialized it).
        let before = iva_rate();
        set_iva_rate(Decimal::new(21, 2)); // should be ignored
        assert_eq!(iva_rate(), before); // still 19%
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
    fn test_extract_iva() {
        // $11.900 → base=$10.000, iva=$1.900
        let (base, iva) = extract_iva(Decimal::from(11900));
        assert_eq!(base, Decimal::from(10000));
        assert_eq!(iva, Decimal::from(1900));
        // $10.000 → base=$8.410, iva=$1.590 (redondeado)
        let (base, iva) = extract_iva(Decimal::from(10000));
        assert_eq!(base + iva, Decimal::from(10000));
        // $0 → base=$0, iva=$0
        let (base, iva) = extract_iva(Decimal::ZERO);
        assert_eq!(base, Decimal::ZERO);
        assert_eq!(iva, Decimal::ZERO);
        // $1.190 → base=$1.000, iva=$190
        let (base, iva) = extract_iva(Decimal::from(1190));
        assert_eq!(base, Decimal::from(1000));
        assert_eq!(iva, Decimal::from(190));
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
        // Simula flujo completo: precio con IVA + redondeo
        let base = Decimal::from(10000);
        let iva = calculate_iva(base); // 10000 * 0.19 = 1900
        assert_eq!(iva, Decimal::from(1900));
        let total = base + iva;
        assert_eq!(total, Decimal::from(11900));
        assert_eq!(format_clp(total), "$11.900");
    }

    #[test]
    fn test_extract_iva_roundtrip() {
        // Extraer IVA de un precio y verificar que base + iva = precio
        let price = Decimal::from(15680);
        let (base, iva) = extract_iva(price);
        assert_eq!(base + iva, price);
        // Verificar que el IVA es razonable (~19%)
        assert!(iva > Decimal::from(2000));
        assert!(iva < Decimal::from(3500));
    }
}

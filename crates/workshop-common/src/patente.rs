//! Validación de patentes vehiculares chilenas (PPU).
//!
//! Formatos soportados:
//! - **Antiguo** (1985–2007): `LLnnnn` — 2 letras + 4 dígitos (ej: `AR1240`)
//! - **Nuevo** (2007–presente): `LLLLnn` — 4 letras + 2 dígitos (ej: `BCDF12`)
//! - **Motos** (2014–presente): `LLLnn` — 3 letras + 2 dígitos (ej: `BJH61`)
//! - **Policía**: `Lnnnn` — 1 letra + 4 dígitos (ej: `Z1234`, `CD1202`)
//! - **Ambulancia**: `Annnn` — 1 letra + 4 dígitos (ej: `A6709`)

use std::fmt;

/// Tipo de patente detectada.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatenteType {
    /// Formato antiguo: 2 letras + 4 dígitos (AR1240)
    Antigua,
    /// Formato nuevo: 4 letras + 2 dígitos (BCDF12)
    Nueva,
    /// Motos: 3 letras + 2 dígitos (BJH61)
    Moto,
    /// Policía / especial: 1 letra + 4 dígitos (Z1234)
    Policia,
}

impl fmt::Display for PatenteType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PatenteType::Antigua => write!(f, "Antigua (LLnnnn)"),
            PatenteType::Nueva => write!(f, "Nueva (LLLLnn)"),
            PatenteType::Moto => write!(f, "Moto (LLLnn)"),
            PatenteType::Policia => write!(f, "Policía/Especial (Lnnnn)"),
        }
    }
}

/// Letras válidas para patentes antiguas (formato LLnnnn).
/// Se excluyen I y O. O está reservada para diplomáticas.
const LETRAS_ANTIGUAS: &[u8] = b"ABCDEFGHKLPRSTUVWXYZMN";

/// Letras válidas para patentes nuevas (formato LLLLnn).
/// Se excluyen vocales (A, E, I, O, U), M, N, Q por similitud visual.
const LETRAS_NUEVAS: &[u8] = b"BCDFGHJKLPRSTVWXYZ";

/// Letras válidas para motos (formato LLLnn).
/// Se excluyen solo vocales (A, E, I, O, U). M, N, Q sí están permitidos.
const LETRAS_MOTO: &[u8] = b"BCDFGHJKLMNPQRSTUVWXYZ";

fn is_digito(c: u8) -> bool {
    c.is_ascii_digit()
}

fn is_letra_antigua(c: u8) -> bool {
    LETRAS_ANTIGUAS.contains(&c)
}

fn is_letra_nueva(c: u8) -> bool {
    LETRAS_NUEVAS.contains(&c)
}

fn is_letra_moto(c: u8) -> bool {
    LETRAS_MOTO.contains(&c)
}

/// Normaliza una patente: elimina espacios, guiones, puntos y convierte a mayúsculas.
pub fn normalize_patente(input: &str) -> String {
    input
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '-' && *c != '.' && *c != '·')
        .map(|c| c.to_ascii_uppercase())
        .collect()
}

/// Detecta el tipo de patente chilena a partir de una cadena normalizada (sin separadores).
pub fn detectar_tipo(patente_normalizada: &str) -> Option<PatenteType> {
    let p = patente_normalizada.as_bytes();
    let len = p.len();

    match len {
        // Nuevo formato: 4 letras + 2 dígitos
        6 if p[0..4].iter().all(|&c| is_letra_nueva(c))
            && p[4..6].iter().all(|&c| is_digito(c)) =>
        {
            Some(PatenteType::Nueva)
        }
        // Antiguo formato: 2 letras + 4 dígitos (desde 1000)
        6 if p[0..2].iter().all(|&c| is_letra_antigua(c))
            && p[2..6].iter().all(|&c| is_digito(c))
            && p[2] >= b'1' =>
        {
            Some(PatenteType::Antigua)
        }
        // Moto: 3 letras + 2 dígitos
        5 if p[0..3].iter().all(|&c| is_letra_moto(c)) && p[3..5].iter().all(|&c| is_digito(c)) => {
            Some(PatenteType::Moto)
        }
        // Policía / especial: 1 letra + 4 dígitos
        5 if p[0] == b'A' || p[0] == b'Z' || p[0] == b'D' || p[0] == b'C' => {
            // CD = diplomático/consular, Z = carabineros, A = ambulancia
            if p[0] == b'D' && len >= 2 && p[1] == b'I' {
                // DI = diplomático, pero esto es solo 1 letra + 4 dígitos
                // para DI necesitaríamos más lógica, ignoramos por ahora
            }
            if p[1..5].iter().all(|&c| is_digito(c)) {
                Some(PatenteType::Policia)
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Valida si una patente chilena tiene un formato válido.
/// Acepta entrada con o sin separadores (guiones, puntos, espacios).
pub fn validar_patente(input: &str) -> bool {
    let normalizada = normalize_patente(input);
    detectar_tipo(&normalizada).is_some()
}

/// Valida una patente y retorna un error descriptivo si no es válida.
pub fn validar_patente_con_error(input: &str) -> Result<PatenteType, PatenteError> {
    let cleaned: String = input
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '-' && *c != '.' && *c != '·')
        .collect();

    if cleaned.is_empty() {
        return Err(PatenteError::Vacia);
    }

    let normalizada = cleaned.to_uppercase();

    // Verificar que solo contiene letras y dígitos
    if !normalizada.chars().all(|c| c.is_ascii_alphanumeric()) {
        return Err(PatenteError::CaracteresInvalidos);
    }

    // Verificar que no tiene longitud excesiva
    if normalizada.len() > 6 {
        return Err(PatenteError::MuyLarga);
    }

    detectar_tipo(&normalizada).ok_or(PatenteError::FormatoInvalido(normalizada))
}

/// Errores de validación de patente.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PatenteError {
    Vacia,
    CaracteresInvalidos,
    MuyLarga,
    FormatoInvalido(String),
}

impl fmt::Display for PatenteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PatenteError::Vacia => write!(f, "La patente no puede estar vacía"),
            PatenteError::CaracteresInvalidos => {
                write!(f, "La patente solo puede contener letras y números")
            }
            PatenteError::MuyLarga => write!(f, "La patente es demasiado larga"),
            PatenteError::FormatoInvalido(p) => write!(
                f,
                "Formato de patente inválido: '{}'. Formatos válidos: \
                 LLnnnn (ej: AR1240), LLLLnn (ej: BCDF12), LLLnn (ej: BJH61)",
                p
            ),
        }
    }
}

impl std::error::Error for PatenteError {}

/// Formatea una patente normalizada con separadores visuales.
/// Ej: "BCDF12" → "BC·DF·12", "AR1240" → "AR·12·40", "BJH61" → "BJH·61"
pub fn formatear_patente(input: &str) -> String {
    let normalizada = normalize_patente(input);
    let p = normalizada.as_str();
    let len = p.len();

    match detectar_tipo(&normalizada) {
        Some(PatenteType::Nueva) if len == 6 => {
            format!("{}·{}·{}", &p[0..2], &p[2..4], &p[4..6])
        }
        Some(PatenteType::Antigua) if len == 6 => {
            format!("{}·{}·{}", &p[0..2], &p[2..4], &p[4..6])
        }
        Some(PatenteType::Moto) if len == 5 => {
            format!("{}·{}", &p[0..3], &p[3..5])
        }
        Some(PatenteType::Policia) if len == 5 => {
            format!("{}·{}", &p[0..1], &p[1..5])
        }
        _ => normalizada,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // === Normalización ===

    #[test]
    fn test_normalize_patente() {
        assert_eq!(normalize_patente("bc-df12"), "BCDF12");
        assert_eq!(normalize_patente(" ar.12.40 "), "AR1240");
        assert_eq!(normalize_patente("bJh·61"), "BJH61");
        assert_eq!(normalize_patente("Z1234"), "Z1234");
    }

    // === Patentes nuevas (LLLLnn) ===

    #[test]
    fn test_patente_nueva_valida() {
        assert!(validar_patente("BCDF12"));
        assert!(validar_patente("BC-DF12"));
        assert!(validar_patente("bb·cd·12"));
        assert!(validar_patente("WKTR99"));
        assert!(validar_patente("XYZW10"));
    }

    #[test]
    fn test_patente_nueva_invalida() {
        assert!(!validar_patente("ABCD12")); // A no permitida en nuevas
        assert!(!validar_patente("BCDE12")); // E no permitida
        assert!(!validar_patente("MMMM12")); // M no permitida
        assert!(!validar_patente("NNNN12")); // N no permitida
        assert!(!validar_patente("QQQQ12")); // Q no permitida
    }

    // === Patentes antiguas (LLnnnn) ===

    #[test]
    fn test_patente_antigua_valida() {
        assert!(validar_patente("AR1240"));
        assert!(validar_patente("AB-1234"));
        assert!(validar_patente("xz·5678"));
        assert!(validar_patente("WX1000"));
    }

    #[test]
    fn test_patente_antigua_invalida() {
        assert!(!validar_patente("AI1234")); // I no permitida
        assert!(!validar_patente("AO1234")); // O reservada para diplomáticos
        assert!(!validar_patente("AA0999")); // menores a 1000
    }

    // === Motos (LLLnn) ===

    #[test]
    fn test_moto_valida() {
        assert!(validar_patente("BJH61"));
        assert!(validar_patente("BJH-61"));
        assert!(validar_patente("MMN01"));
        assert!(validar_patente("ZBQ31"));
    }

    #[test]
    fn test_moto_invalida() {
        assert!(!validar_patente("BAH61")); // A no permitida en motos
        assert!(!validar_patente("BEI61")); // E, I no permitidas
    }

    // === Policía / Especial (Lnnnn) ===

    #[test]
    fn test_policia_valida() {
        assert!(validar_patente("Z1234")); // Carabineros
        assert!(validar_patente("A6709")); // Ambulancia
        assert!(validar_patente("CD1202")); // Consular — esto es 6 chars, detectado como nueva
    }

    // === Detección de tipo ===

    #[test]
    fn test_detectar_tipo() {
        assert_eq!(detectar_tipo("BCDF12"), Some(PatenteType::Nueva));
        assert_eq!(detectar_tipo("AR1240"), Some(PatenteType::Antigua));
        assert_eq!(detectar_tipo("BJH61"), Some(PatenteType::Moto));
        assert_eq!(detectar_tipo("Z1234"), Some(PatenteType::Policia));
        assert_eq!(detectar_tipo("A6709"), Some(PatenteType::Policia));
        assert_eq!(detectar_tipo("INVALID"), None);
        assert_eq!(detectar_tipo(""), None);
    }

    // === Formateo ===

    #[test]
    fn test_formatear_patente() {
        assert_eq!(formatear_patente("BCDF12"), "BC·DF·12");
        assert_eq!(formatear_patente("AR1240"), "AR·12·40");
        assert_eq!(formatear_patente("BJH61"), "BJH·61");
        assert_eq!(formatear_patente("bc-df12"), "BC·DF·12");
        assert_eq!(formatear_patente("Z1234"), "Z·1234");
    }

    // === Con error ===

    #[test]
    fn test_validar_con_error_ok() {
        assert_eq!(validar_patente_con_error("BCDF12"), Ok(PatenteType::Nueva));
        assert_eq!(
            validar_patente_con_error("AR1240"),
            Ok(PatenteType::Antigua)
        );
    }

    #[test]
    fn test_validar_con_error_vacia() {
        assert_eq!(validar_patente_con_error(""), Err(PatenteError::Vacia));
    }

    #[test]
    fn test_validar_con_error_formato() {
        assert!(matches!(
            validar_patente_con_error("QQQQ00"),
            Err(PatenteError::FormatoInvalido(_))
        ));
        assert!(matches!(
            validar_patente_con_error("A1B2C3"),
            Err(PatenteError::FormatoInvalido(_))
        ));
    }
}

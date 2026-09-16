use dioxus::prelude::*;
use plotters::prelude::RGBColor;

use crate::charts::render_line_chart;
use crate::theme::use_theme;

/// Parsea un hex "#RRGGBB" a RGBColor de plotters.
#[allow(dead_code)]
fn parse_hex(hex: &str) -> RGBColor {
    let hex = hex.trim_start_matches('#');
    match hex.len() {
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
            let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
            let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
            RGBColor(r, g, b)
        }
        _ => RGBColor(245, 158, 11),
    }
}

/// Componente reutilizable de gráfico de líneas que renderiza SVG vía plotters.
///
/// Recalcula automáticamente cuando cambian los datos o el tema (claro/oscuro).
#[component]
pub fn LineChart(
    data: Vec<(String, f64)>,
    #[props(default = 600)] width: u32,
    #[props(default = 400)] height: u32,
    #[props(default)] class: Option<String>,
) -> Element {
    let theme = use_theme();

    let svg_html = use_memo(move || {
        let tokens = theme.tokens();
        let color = parse_hex(&tokens.colors.primary);
        match render_line_chart(&data, color, width, height) {
            Ok(svg) => svg,
            Err(_) => "<svg></svg>".to_string(),
        }
    });

    let class_str = class.unwrap_or_default();

    rsx! {
        div {
            class: "{class_str}",
            dangerous_inner_html: "{svg_html}"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hex_valid() {
        let c = parse_hex("#F59E0B");
        assert_eq!(c, RGBColor(245, 158, 11));
    }

    #[test]
    fn test_parse_hex_no_hash() {
        let c = parse_hex("FF0000");
        assert_eq!(c, RGBColor(255, 0, 0));
    }

    #[test]
    fn test_parse_hex_invalid_defaults() {
        let c = parse_hex("xyz");
        assert_eq!(c, RGBColor(245, 158, 11));
    }
}

use plotters::prelude::*;

/// Genera un gráfico de líneas con área sombreada como SVG en memoria.
///
/// * `data` - Pares de (etiqueta, monto) para cada punto.
/// * `line_color` - Color RGB de la línea y área.
/// * `width` - Ancho del gráfico en píxeles.
/// * `height` - Alto del gráfico en píxeles.
///
/// Devuelve un `String` con el SVG completo. Cada punto incluye un tooltip nativo.
#[allow(dead_code)]
pub fn render_line_chart(
    data: &[(String, f64)],
    line_color: RGBColor,
    width: u32,
    height: u32,
) -> Result<String, Box<dyn std::error::Error>> {
    let max_value = data.iter().map(|(_, v)| *v).fold(0.0_f64, f64::max);
    let y_max = if max_value > 0.0 {
        (max_value * 1.15).ceil()
    } else {
        100.0
    };
    let x_max = data.len() as f64;

    let mut buf = String::new();
    {
        let backend = SVGBackend::with_string(&mut buf, (width, height));
        let root: DrawingArea<_, plotters::coord::Shift> = backend.into();

        let mut chart = ChartBuilder::on(&root)
            .margin(10)
            .x_label_area_size(60)
            .y_label_area_size(70)
            .build_cartesian_2d(0.0..x_max, 0.0..y_max)?;

        chart
            .configure_mesh()
            .disable_x_mesh()
            .disable_y_mesh()
            .y_labels(5)
            .x_labels(data.len())
            .label_style(("sans-serif", 12).into_font())
            .y_label_formatter(&|v| format!("${:.0}", v))
            .x_label_formatter(&|v| {
                let idx = *v as usize;
                data.get(idx)
                    .map(|(label, _)| label.clone())
                    .unwrap_or_default()
            })
            .draw()?;

        let draw_area = line_color.mix(0.15);

        chart.draw_series(AreaSeries::new(
            data.iter()
                .enumerate()
                .map(|(i, (_, v))| (i as f64 + 0.5, *v))
                .collect::<Vec<_>>(),
            0.0,
            draw_area,
        ))?;

        chart.draw_series(LineSeries::new(
            data.iter()
                .enumerate()
                .map(|(i, (_, v))| (i as f64 + 0.5, *v))
                .collect::<Vec<_>>(),
            line_color,
        ))?;

        chart.draw_series(
            data.iter()
                .enumerate()
                .map(|(i, (_, v))| Circle::new((i as f64 + 0.5, *v), 4, line_color.filled())),
        )?;

        root.present()?;
    }

    inject_tooltips(&buf, data)
}

/// Inyecta `<title>` dentro de cada `<circle>` del SVG para tooltips nativos.
fn inject_tooltips(
    svg: &str,
    data: &[(String, f64)],
) -> Result<String, Box<dyn std::error::Error>> {
    let mut result = String::with_capacity(svg.len() + data.len() * 64);
    let mut circle_idx = 0usize;
    let mut remaining = svg;

    while let Some(circle_pos) = remaining.find("<circle") {
        result.push_str(&remaining[..circle_pos]);
        let after_circle = &remaining[circle_pos..];

        if let Some(tag_end) = after_circle.find("/>") {
            let full_tag = &after_circle[..tag_end + 2];
            result.push_str(full_tag);

            if circle_idx < data.len() {
                let value = data[circle_idx].1;
                let label = format!("${:.0}", value);
                result.push_str(&format!("<title>{label}</title>"));
            }
            circle_idx += 1;
            remaining = &after_circle[tag_end + 2..];
        } else {
            result.push_str(after_circle);
            remaining = "";
        }
    }
    result.push_str(remaining);
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_line_chart_returns_svg() {
        let data = vec![
            ("Ene".to_string(), 15000.0),
            ("Feb".to_string(), 22000.0),
            ("Mar".to_string(), 18000.0),
        ];
        let svg = render_line_chart(&data, RGBColor(245, 158, 11), 700, 350);
        assert!(svg.is_ok());
        let svg = svg.unwrap();
        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));
        assert!(svg.contains("<circle"));
        assert!(svg.contains("<title>$15000</title>"));
        assert!(svg.contains("<title>$22000</title>"));
        assert!(svg.contains("<title>$18000</title>"));
    }

    #[test]
    fn test_render_line_chart_empty_data() {
        let data: Vec<(String, f64)> = vec![];
        let svg = render_line_chart(&data, RGBColor(245, 158, 11), 700, 350);
        assert!(svg.is_ok());
        assert!(!svg.unwrap().contains("<circle"));
    }
}

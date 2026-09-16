use plotters::prelude::*;

/// Genera un gráfico de líneas con área sombreada como SVG en memoria.
///
/// * `data` - Pares de (etiqueta, monto) para cada punto.
/// * `line_color` - Color RGB de la línea y área.
/// * `width` - Ancho del gráfico en píxeles.
/// * `height` - Alto del gráfico en píxeles.
///
/// Devuelve un `String` con el SVG completo, incluyendo etiquetas de valor.
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

    for (i, (_, v)) in data.iter().enumerate() {
        let label = format!("${:.0}", v);
        let x_pct = ((i as f64 + 0.5) / x_max * 100.0) as u32;
        let y_pct = if y_max > 0.0 {
            (100.0 - (*v / y_max * 100.0)) as u32
        } else {
            50
        };
        let svg_text = format!(
            r#"<text x="{}%" y="{}%" text-anchor="middle" font-family="sans-serif" font-size="11" dy="-8">{}</text>"#,
            x_pct, y_pct, label
        );
        buf.insert_str(buf.rfind("</svg>").unwrap_or(buf.len()), &svg_text);
    }

    Ok(buf)
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
        assert!(svg.contains("$15000"));
        assert!(svg.contains("$22000"));
        assert!(svg.contains("$18000"));
    }

    #[test]
    fn test_render_line_chart_empty_data() {
        let data: Vec<(String, f64)> = vec![];
        let svg = render_line_chart(&data, RGBColor(245, 158, 11), 700, 350);
        assert!(svg.is_ok());
    }
}

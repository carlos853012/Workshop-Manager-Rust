use plotters::prelude::*;

/// Genera un gráfico de barras como SVG en memoria.
///
/// * `data` - Pares de (etiqueta, monto) para cada barra.
/// * `bar_color` - Color RGB de las barras.
/// * `width` - Ancho del gráfico en píxeles.
/// * `height` - Alto del gráfico en píxeles.
///
/// Devuelve un `String` con el SVG completo.
#[allow(dead_code)]
pub fn render_bar_chart(
    data: &[(String, f64)],
    bar_color: RGBColor,
    width: u32,
    height: u32,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut buf = String::new();
    {
        let backend = SVGBackend::with_string(&mut buf, (width, height));
        let root: DrawingArea<_, plotters::coord::Shift> = backend.into();

        let max_value = data.iter().map(|(_, v)| *v).fold(0.0_f64, f64::max);

        let y_max = if max_value > 0.0 {
            (max_value * 1.15).ceil()
        } else {
            100.0
        };

        let x_max = data.len() as f64;

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

        chart.draw_series(data.iter().enumerate().map(|(i, (_, v))| {
            Rectangle::new([(i as f64, 0.0), (i as f64 + 1.0, *v)], bar_color.filled())
        }))?;

        for (i, (_, v)) in data.iter().enumerate() {
            let label = format!("${:.0}", v);
            chart.draw_series([Text::new(
                label,
                (i as f64 + 0.5, *v + y_max * 0.02),
                ("sans-serif", 11).into_font(),
            )])?;
        }

        root.present()?;
    }
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_bar_chart_returns_svg() {
        let data = vec![
            ("Ene".to_string(), 15000.0),
            ("Feb".to_string(), 22000.0),
            ("Mar".to_string(), 18000.0),
        ];
        let svg = render_bar_chart(&data, RGBColor(0, 120, 215), 600, 400);
        assert!(svg.is_ok());
        let svg = svg.unwrap();
        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));
    }

    #[test]
    fn test_render_bar_chart_empty_data() {
        let data: Vec<(String, f64)> = vec![];
        let svg = render_bar_chart(&data, RGBColor(0, 120, 215), 600, 400);
        assert!(svg.is_ok());
    }
}

//! Generador de icono RGBA para la aplicación WorkshopManager.
//! Rasteriza el SVG del wrench (Heroicons) usando resvg sobre fondo redondeado.

use resvg::tiny_skia::Pixmap;
use resvg::usvg;

/// Genera un SVG string del wrench con el color de foreground dado.
fn wrench_svg_with_color(fg: [u8; 3]) -> String {
    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="#{:02X}{:02X}{:02X}">
  <path d="M14.7 6.3a4.5 4.5 0 0 0-5.9 5.9l-5.2 5.2a2.1 2.1 0 1 0 3 3l5.2-5.2a4.5 4.5 0 0 0 5.9-5.9l-2.8 2.8-2.1-.7-.7-2.1 2.6-3z"/>
</svg>"##,
        fg[0], fg[1], fg[2]
    )
}

/// Genera un icono RGBA de tamaño `size`×`size` píxeles.
/// Fondo con bordes redondeados y wrench rasterizado desde el SVG.
pub fn generate_wrench_icon(size: u32, bg_color: [u8; 3], fg_color: [u8; 3]) -> Vec<u8> {
    let s = size as f64;

    let svg = wrench_svg_with_color(fg_color);

    // 1. Renderizar el SVG del wrench a un pixmap del tamaño deseado
    let wrench_pixels = render_svg_to_rgba(&svg, size, size);

    let bg_r = bg_color[0] as f64;
    let bg_g = bg_color[1] as f64;
    let bg_b = bg_color[2] as f64;

    // 2. Construir el icono final: fondo + wrench encima
    let mut pixels = Vec::with_capacity((size * size * 4) as usize);

    for y in 0..size {
        for x in 0..size {
            let px = x as f64 + 0.5;
            let py = y as f64 + 0.5;

            let bg = is_background(px, py, s);

            // Obtener el píxel del wrench en esta posición
            let wrench_idx = ((y * size + x) * 4) as usize;
            let w_r = wrench_pixels[wrench_idx];
            let w_g = wrench_pixels[wrench_idx + 1];
            let w_b = wrench_pixels[wrench_idx + 2];
            let w_a = wrench_pixels[wrench_idx + 3];

            if bg {
                if w_a > 0 {
                    // Wrench encima del fondo: mezclar
                    let alpha = w_a as f64 / 255.0;
                    let r = (bg_r * (1.0 - alpha) + w_r as f64 * alpha) as u8;
                    let g = (bg_g * (1.0 - alpha) + w_g as f64 * alpha) as u8;
                    let b = (bg_b * (1.0 - alpha) + w_b as f64 * alpha) as u8;
                    pixels.extend_from_slice(&[r, g, b, 255]);
                } else {
                    // Solo fondo
                    pixels.extend_from_slice(&[bg_color[0], bg_color[1], bg_color[2], 255]);
                }
            } else if w_a > 0 {
                // Wrench fuera del fondo (no debería pasar, pero por si acaso)
                pixels.extend_from_slice(&[w_r, w_g, w_b, w_a]);
            } else {
                // Transparente
                pixels.extend_from_slice(&[0, 0, 0, 0]);
            }
        }
    }
    pixels
}

/// Genera el icono con los colores por defecto (fondo azul #2563EB, wrench blanco).
pub fn generate_wrench_icon_default(size: u32) -> Vec<u8> {
    generate_wrench_icon(size, [37, 99, 235], [255, 255, 255])
}

/// Rasteriza un SVG string a RGBA pixels en el tamaño dado.
fn render_svg_to_rgba(svg_str: &str, width: u32, height: u32) -> Vec<u8> {
    let opt = usvg::Options::default();
    let tree = usvg::Tree::from_str(svg_str, &opt).expect("Failed to parse SVG");

    let svg_size = tree.size();
    let scale_x = width as f32 / svg_size.width();
    let scale_y = height as f32 / svg_size.height();

    let mut pixmap =
        Pixmap::new(width, height).expect("Failed to create pixmap for icon rendering");
    let transform = resvg::tiny_skia::Transform::from_scale(scale_x, scale_y);
    resvg::render(&tree, transform, &mut pixmap.as_mut());

    pixmap.take_demultiplied()
}

/// Fondo azul con bordes redondeados.
fn is_background(px: f64, py: f64, s: f64) -> bool {
    if px < 0.0 || px >= s || py < 0.0 || py >= s {
        return false;
    }
    let r = s * 0.18;
    let ix = px.clamp(r, s - r);
    let iy = py.clamp(r, s - r);
    let dx = px - ix;
    let dy = py - iy;
    dx * dx + dy * dy <= r * r
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_icon_32() {
        let pixels = generate_wrench_icon_default(32);
        assert_eq!(pixels.len(), 32 * 32 * 4);
    }

    #[test]
    fn test_generate_icon_16() {
        let pixels = generate_wrench_icon_default(16);
        assert_eq!(pixels.len(), 16 * 16 * 4);
    }

    #[test]
    fn test_generate_icon_256() {
        let pixels = generate_wrench_icon_default(256);
        assert_eq!(pixels.len(), 256 * 256 * 4);
    }

    #[test]
    fn test_background_corners_are_transparent() {
        let pixels = generate_wrench_icon_default(32);
        let idx = ((1 * 32 + 1) * 4) as usize;
        assert_eq!(
            pixels[idx + 3],
            0,
            "top-left near-corner should be transparent"
        );
    }

    #[test]
    fn test_background_is_blue() {
        let pixels = generate_wrench_icon_default(32);
        // Pixel (2,28) should be blue background (far from wrench, inside rounded rect)
        let idx = ((28 * 32 + 2) * 4) as usize;
        assert_eq!(pixels[idx], 37, "background R should be 37");
        assert_eq!(pixels[idx + 1], 99, "background G should be 99");
        assert_eq!(pixels[idx + 2], 235, "background B should be 235");
        assert_eq!(pixels[idx + 3], 255, "background A should be 255");
    }

    #[test]
    fn test_wrench_pixels_are_not_all_transparent() {
        let pixels = generate_wrench_icon_default(64);
        // At least some pixels should have alpha > 0 (wrench or background)
        let has_content = pixels.chunks(4).any(|p| p[3] > 0);
        assert!(has_content, "Icon should have some visible pixels");
    }

    #[test]
    fn test_wrench_has_white_pixels() {
        let pixels = generate_wrench_icon_default(64);
        // Some pixels should be white (the wrench itself)
        let has_white = pixels
            .chunks(4)
            .any(|p| p[0] == 255 && p[1] == 255 && p[2] == 255 && p[3] == 255);
        assert!(has_white, "Icon should contain white wrench pixels");
    }

    #[test]
    fn test_render_svg_to_rgba_dimensions() {
        let svg = wrench_svg_with_color([255, 255, 255]);
        let pixels = render_svg_to_rgba(&svg, 48, 48);
        assert_eq!(pixels.len(), 48 * 48 * 4);
    }

    #[test]
    fn test_custom_colors() {
        let pixels = generate_wrench_icon(32, [245, 158, 11], [30, 33, 38]);
        // Pixel (2,28) should be the custom amber background
        let idx = ((28 * 32 + 2) * 4) as usize;
        assert_eq!(pixels[idx], 245, "background R should be 245 (amber)");
        assert_eq!(pixels[idx + 1], 158, "background G should be 158");
        assert_eq!(pixels[idx + 2], 11, "background B should be 11");
        assert_eq!(pixels[idx + 3], 255, "background A should be 255");
    }
}

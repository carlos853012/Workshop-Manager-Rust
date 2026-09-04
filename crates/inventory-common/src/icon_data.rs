//! Generador de icono RGBA para la aplicación WorkshopManager.
//! Dibuja una llave inglesa (wrench) sobre fondo azul, renderizada a píxeles.

/// Genera un icono RGBA de tamaño `size`×`size` píxeles.
/// Fondo azul (#2563EB) con bordes redondeados y wrench blanco.
pub fn generate_wrench_icon(size: u32) -> Vec<u8> {
    let mut pixels = Vec::with_capacity((size * size * 4) as usize);
    let s = size as f64;
    let center = s / 2.0;

    for y in 0..size {
        for x in 0..size {
            let px = x as f64 + 0.5;
            let py = y as f64 + 0.5;

            let color = if is_wrench(px, py, s, center) {
                [255u8, 255, 255, 255] // wrench blanco
            } else if is_background(px, py, s) {
                [37u8, 99, 235, 255] // fondo azul
            } else {
                [0u8, 0, 0, 0] // transparente
            };
            pixels.extend_from_slice(&color);
        }
    }
    pixels
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

/// Determina si un píxel pertenece al wrench.
/// El wrench se dibuja en el espacio [4, 28]×[4, 28] de un icono 32×32.
fn is_wrench(px: f64, py: f64, s: f64, _center: f64) -> bool {
    let scale = s / 32.0;

    // Coordenadas normalizadas al espacio 32×32
    let x = px / scale;
    let y = py / scale;

    // === MANGO (handle) ===
    // Línea diagonal desde (16, 22) hasta (26, 12)
    if is_thick_line(x, y, 16.0, 22.0, 26.0, 12.0, 2.2) {
        return true;
    }

    // === CABEZA (jaw circular) ===
    // Círculo en la parte superior-izquierda, radio ~5.5, centro (9.5, 9.5)
    let jaw_cx = 9.5;
    let jaw_cy = 9.5;
    let jaw_r = 5.5;
    let dx_j = x - jaw_cx;
    let dy_j = y - jaw_cy;
    let dist_j = (dx_j * dx_j + dy_j * dy_j).sqrt();

    // Anillo exterior del jaw
    if dist_j <= jaw_r && dist_j >= jaw_r - 2.0 {
        // Abrir la mordaza: recortar un sector
        let angle = dy_j.atan2(dx_j);
        let angle_deg = angle.to_degrees();
        // Abrir entre 200° y 340° (la parte inferior del círculo)
        if !(-20.0..=200.0).contains(&angle_deg) {
            // Dentro del rango de apertura — no dibujar
        } else {
            return true;
        }
    }

    // === CONEXIÓN mango-cabeza ===
    // Pequeño rectángulo que conecta el jaw con el mango
    if is_thick_line(x, y, 13.0, 14.0, 17.0, 19.0, 1.8) {
        return true;
    }

    // === PUNTA DEL MANGO ===
    // Pequeño círculo al final del mango (como empuñadura)
    let grip_cx = 26.5;
    let grip_cy = 11.5;
    let grip_r = 2.0;
    let dx_g = x - grip_cx;
    let dy_g = y - grip_cy;
    if dx_g * dx_g + dy_g * dy_g <= grip_r * grip_r {
        return true;
    }

    false
}

/// Dibuja una línea gruesa entre dos puntos.
fn is_thick_line(x: f64, y: f64, x1: f64, y1: f64, x2: f64, y2: f64, thickness: f64) -> bool {
    let dx = x2 - x1;
    let dy = y2 - y1;
    let len_sq = dx * dx + dy * dy;
    if len_sq == 0.0 {
        return false;
    }
    let t = ((x - x1) * dx + (y - y1) * dy) / len_sq;
    let t = t.clamp(0.0, 1.0);
    let proj_x = x1 + t * dx;
    let proj_y = y1 + t * dy;
    let dist_sq = (x - proj_x) * (x - proj_x) + (y - proj_y) * (y - proj_y);
    dist_sq <= thickness * thickness
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_icon尺寸正确() {
        let pixels = generate_wrench_icon(32);
        assert_eq!(pixels.len(), 32 * 32 * 4);
    }

    #[test]
    fn test_generate_icon_16() {
        let pixels = generate_wrench_icon(16);
        assert_eq!(pixels.len(), 16 * 16 * 4);
    }

    #[test]
    fn test_generate_icon_256() {
        let pixels = generate_wrench_icon(256);
        assert_eq!(pixels.len(), 256 * 256 * 4);
    }

    #[test]
    fn test_background_corners_are_transparent() {
        let pixels = generate_wrench_icon(32);
        // Pixel (1,1) should be transparent (outside rounded corner)
        let idx = ((1 * 32 + 1) * 4) as usize;
        assert_eq!(
            pixels[idx + 3],
            0,
            "top-left near-corner should be transparent"
        );
    }

    #[test]
    fn test_center_is_blue() {
        let pixels = generate_wrench_icon(32);
        // Pixel (2,28) should be blue background (far from wrench)
        let idx = ((28 * 32 + 2) * 4) as usize;
        assert_eq!(pixels[idx], 37, "background R should be 37");
        assert_eq!(pixels[idx + 1], 99, "background G should be 99");
        assert_eq!(pixels[idx + 2], 235, "background B should be 235");
        assert_eq!(pixels[idx + 3], 255, "background A should be 255");
    }

    #[test]
    fn test_is_thick_line_basic() {
        // Point on the line
        assert!(is_thick_line(5.0, 5.0, 0.0, 0.0, 10.0, 10.0, 1.0));
        // Point far from the line
        assert!(!is_thick_line(0.0, 10.0, 0.0, 0.0, 10.0, 0.0, 1.0));
    }
}

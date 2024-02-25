use image::Rgba;

pub fn str_to_color(s: &str) -> Option<Rgba<u8>> {
    if s == "auto" || s == "none" {
        return None;
    }
    if s.is_empty() {
        return None;
    }
    if s.as_bytes()[0] != b'#' {
        return None;
    }
    let mut color = u32::from_str_radix(&s[1..], 16).unwrap();

    match s.len() {
        // RGB or RGBA
        4 | 5 => {
            let a = if s.len() == 5 {
                let alpha = (color & 0xf) as u8;
                color >>= 4;
                alpha
            } else {
                0xff
            };

            let r = ((color >> 8) & 0xf) as u8;
            let g = ((color >> 4) & 0xf) as u8;
            let b = (color & 0xf) as u8;

            Some(Rgba([r << 4 | r, g << 4 | g, b << 4 | b, a << 4 | a]))
        }
        // RRGGBB or RRGGBBAA
        7 | 9 => {
            let alpha = if s.len() == 9 {
                let alpha = (color & 0xff) as u8;
                color >>= 8;
                alpha
            } else {
                0xff
            };

            Some(Rgba([
                (color >> 16) as u8,
                (color >> 8) as u8,
                color as u8,
                alpha,
            ]))
        }
        _ => None,
    }
}

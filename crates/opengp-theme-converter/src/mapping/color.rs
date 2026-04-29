use crate::error::ThemeConverterError;

pub fn map_color_field(color: &str) -> String {
    match parse_hex(color) {
        Ok((r, g, b)) => to_opengp_color(r, g, b),
        Err(_) => color.to_string(),
    }
}

pub fn parse_hex(hex: &str) -> Result<(u8, u8, u8), ThemeConverterError> {
    let hex = hex.strip_prefix("#").unwrap_or(hex);
    if hex.len() < 6 {
        return Err(ThemeConverterError::InvalidHexColor(hex.to_string()));
    }

    let r: u8 = u8::from_str_radix(&hex[0..2], 16)
        .map_err(|_| ThemeConverterError::InvalidHexColor(hex.to_string()))?;
    let g: u8 = u8::from_str_radix(&hex[2..4], 16)
        .map_err(|_| ThemeConverterError::InvalidHexColor(hex.to_string()))?;
    let b: u8 = u8::from_str_radix(&hex[4..6], 16)
        .map_err(|_| ThemeConverterError::InvalidHexColor(hex.to_string()))?;

    Ok((r, g, b))
}

pub fn to_opengp_color(r: u8, g: u8, b: u8) -> String {
    format!("Rgb({r}, {g}, {b})")
}

pub fn relative_luminance(r: u8, g: u8, b: u8) -> f32 {
    let r = linearize(r);
    let g = linearize(g);
    let b = linearize(b);
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

fn linearize(c: u8) -> f32 {
    let c = c as f32 / 255.0;
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

pub fn contrast_ratio(fg: (u8, u8, u8), bg: (u8, u8, u8)) -> f32 {
    let l1 = relative_luminance(fg.0, fg.1, fg.2);
    let l2 = relative_luminance(bg.0, bg.1, bg.2);
    let lighter = l1.max(l2);
    let darker = l1.min(l2);
    (lighter + 0.05) / (darker + 0.05)
}

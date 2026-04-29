use crate::mapping::color::{contrast_ratio, parse_hex};

pub const WCAG_AA_NORMAL: f32 = 4.5;
pub const WCAG_AA_LARGE: f32 = 3.0;
pub const WCAG_AAA_NORMAL: f32 = 7.0;

#[derive(Debug)]
pub struct ContrastWarning {
    pub field: String,
    pub ratio: f32,
    pub level: &'static str,
}

pub fn check_contrast(
    fg_field: &str,
    _bg_field: &str,
    fg_color: &str,
    bg_color: &str,
) -> Option<ContrastWarning> {
    let fg = parse_hex(fg_color).ok()?;
    let bg = parse_hex(bg_color).ok()?;
    let ratio = contrast_ratio(fg, bg);

    let level = if ratio < WCAG_AA_LARGE {
        "critical"
    } else if ratio < WCAG_AA_NORMAL {
        "warning"
    } else {
        return None;
    };

    Some(ContrastWarning {
        field: fg_field.to_string(),
        ratio,
        level,
    })
}

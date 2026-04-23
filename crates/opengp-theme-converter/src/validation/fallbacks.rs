use crate::AlacrittyTheme;
use crate::mapping::color::map_color_field;
pub fn fallback_for_field(field: &str, theme: &AlacrittyTheme) -> String {
    match field {
        "primary" => map_color_field(&theme.colors.bright.blue),
        "secondary" => map_color_field(&theme.colors.bright.magenta),
        "background" => map_color_field(&theme.colors.primary.background),
        "foreground" => map_color_field(&theme.colors.primary.foreground),
        "error" => map_color_field(&theme.colors.normal.red),
        "success" => map_color_field(&theme.colors.normal.green),
        "warning" => map_color_field(&theme.colors.normal.yellow),
        "info" => map_color_field(&theme.colors.normal.blue),
        "border" => map_color_field(&theme.colors.normal.white),
        "selected" => map_color_field(&theme.colors.bright.blue),
        "highlight" => map_color_field(&theme.colors.bright.cyan),
        "disabled" => theme
            .colors
            .dim
            .as_ref()
            .map(|dim| map_color_field(&dim.black))
            .unwrap_or_else(|| "Rgb(128, 128, 128)".to_string()),
        "scrollbar_bg" => theme
            .colors
            .dim
            .as_ref()
            .map(|dim| map_color_field(&dim.black))
            .unwrap_or_else(|| "Rgb(64, 64, 64)".to_string()),
        "scrollbar_thumb" => map_color_field(&theme.colors.normal.white),
        "background_dark" => theme
            .colors
            .dim
            .as_ref()
            .map(|dim| map_color_field(&dim.black))
            .unwrap_or_else(|| "Rgb(0, 0, 0)".to_string()),
        "text_dim" => theme
            .colors
            .dim
            .as_ref()
            .map(|dim| map_color_field(&dim.black))
            .unwrap_or_else(|| "Rgb(128, 128, 128)".to_string()),
        "text_secondary" => map_color_field(&theme.colors.bright.white),
        "appointment_scheduled" => map_color_field(&theme.colors.normal.yellow),
        "appointment_confirmed" => map_color_field(&theme.colors.bright.blue),
        "appointment_arrived" => map_color_field(&theme.colors.normal.green),
        "appointment_in_progress" => map_color_field(&theme.colors.bright.cyan),
        "appointment_completed" => map_color_field(&theme.colors.normal.green),
        "appointment_cancelled" => map_color_field(&theme.colors.bright.red),
        "appointment_dna" => map_color_field(&theme.colors.normal.red),
        "appointment_rescheduled" => map_color_field(&theme.colors.normal.yellow),
        _ => derive_safe_default(field),
    }
}
pub fn derive_safe_default(field: &str) -> String {
    match field {
        "primary" => "Rgb(0, 200, 255)".to_string(),
        "secondary" => "Rgb(200, 100, 200)".to_string(),
        "background" => "Rgb(0, 0, 0)".to_string(),
        "foreground" => "Rgb(255, 255, 255)".to_string(),
        "error" => "Rgb(220, 50, 50)".to_string(),
        "success" => "Rgb(50, 200, 50)".to_string(),
        "warning" => "Rgb(255, 200, 50)".to_string(),
        "info" => "Rgb(50, 100, 255)".to_string(),
        "border" => "Rgb(128, 128, 128)".to_string(),
        "selected" => "Rgb(50, 100, 200)".to_string(),
        "highlight" => "Rgb(100, 200, 255)".to_string(),
        "disabled" => "Rgb(128, 128, 128)".to_string(),
        "scrollbar_bg" => "Rgb(64, 64, 64)".to_string(),
        "scrollbar_thumb" => "Rgb(192, 192, 192)".to_string(),
        "background_dark" => "Rgb(0, 0, 0)".to_string(),
        "text_dim" => "Rgb(128, 128, 128)".to_string(),
        "text_secondary" => "Rgb(192, 192, 192)".to_string(),
        "appointment_scheduled" => "Rgb(255, 200, 50)".to_string(),
        "appointment_confirmed" => "Rgb(0, 200, 255)".to_string(),
        "appointment_arrived" => "Rgb(50, 200, 50)".to_string(),
        "appointment_in_progress" => "Rgb(100, 200, 255)".to_string(),
        "appointment_completed" => "Rgb(50, 200, 50)".to_string(),
        "appointment_cancelled" => "Rgb(220, 50, 50)".to_string(),
        "appointment_dna" => "Rgb(220, 50, 50)".to_string(),
        "appointment_rescheduled" => "Rgb(255, 200, 50)".to_string(),
        _ => "Rgb(128, 128, 128)".to_string(),
    }
}

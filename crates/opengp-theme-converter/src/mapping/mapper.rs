use crate::mapping::color::map_color_field;
use crate::{AlacrittyTheme, OpenGPPalette, OpenGPTheme};

pub fn map_alacritty_to_opengp(theme: &AlacrittyTheme) -> OpenGPTheme {
    let palette = map_colors(theme);
    OpenGPTheme {
        schema_version: 1.0.to_string(),
        dark: palette.clone(),
        light: palette.clone(),
        high_contrast: palette.clone(),
    }
}

fn map_colors(inp: &AlacrittyTheme) -> OpenGPPalette {
    OpenGPPalette {
        primary: map_color_field(&inp.colors.bright.blue),
        secondary: map_color_field(&inp.colors.bright.magenta),
        background: map_color_field(&inp.colors.primary.background),
        foreground: map_color_field(&inp.colors.primary.foreground),
        error: map_color_field(&inp.colors.normal.red),
        success: map_color_field(&inp.colors.normal.green),
        warning: map_color_field(&inp.colors.normal.yellow),
        info: map_color_field(&inp.colors.normal.blue),
        border: map_color_field(&inp.colors.normal.white),
        selected: map_color_field(&inp.colors.bright.blue),
        highlight: map_color_field(&inp.colors.bright.cyan),
        disabled: map_color_field(&inp.colors.normal.blue),
        scrollbar_bg: map_color_field(&inp.colors.normal.black),
        scrollbar_thumb: map_color_field(&inp.colors.normal.white),
        appointment_scheduled: map_color_field(&inp.colors.bright.yellow),
        appointment_confirmed: map_color_field(&inp.colors.bright.blue),
        appointment_arrived: map_color_field(&inp.colors.bright.green),
        appointment_in_progress: map_color_field(&inp.colors.bright.cyan),
        appointment_completed: map_color_field(&inp.colors.normal.green),
        appointment_cancelled: map_color_field(&inp.colors.bright.red),
        appointment_dna: map_color_field(&inp.colors.normal.red),
        appointment_rescheduled: map_color_field(&inp.colors.normal.yellow),
        background_dark: map_color_field(&inp.colors.normal.black),
        text_dim: map_color_field(&inp.colors.normal.white),
        text_secondary: map_color_field(&inp.colors.bright.white),
    }
}

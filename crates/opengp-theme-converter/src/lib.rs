pub mod error;
pub mod mapping;
pub mod output;
pub mod parse;
pub mod validation;

use serde::{Deserialize, Serialize};

pub use crate::error::ThemeConverterError;
pub use crate::mapping::color::{contrast_ratio, parse_hex, to_opengp_color};
pub use crate::mapping::mapper::map_alacritty_to_opengp;
pub use crate::output::render_opengp_toml;
pub use crate::parse::parse_by_extension;
pub use crate::validation::contrast::{check_contrast, ContrastWarning};
pub use crate::validation::fallbacks::fallback_for_field;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AlacrittyTheme {
    pub colors: AlacrittyColors,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AlacrittyColors {
    pub primary: AlacrittyPrimary,
    pub normal: Ansi8,
    pub bright: Ansi8,
    #[serde(default)]
    pub dim: Option<Ansi8>,
    #[serde(default)]
    pub cursor: Option<AlacrittyColorPair>,
    #[serde(default)]
    pub selection: Option<AlacrittyColorPair>,
    #[serde(default)]
    pub indexed_colors: Vec<AlacrittyIndexedColor>,
    #[serde(default)]
    pub vi_mode_cursor: Option<AlacrittyColorPair>,
    #[serde(default)]
    pub search: Option<AlacrittySearch>,
    #[serde(default)]
    pub hints: Option<AlacrittyHints>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Ansi8 {
    pub black: String,
    pub red: String,
    pub green: String,
    pub yellow: String,
    pub blue: String,
    pub magenta: String,
    pub cyan: String,
    pub white: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AlacrittyPrimary {
    pub background: String,
    pub foreground: String,
    #[serde(default)]
    pub dim_foreground: Option<String>,
    #[serde(default)]
    pub bright_foreground: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AlacrittyColorPair {
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub cursor: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AlacrittyIndexedColor {
    pub index: u8,
    pub color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AlacrittyFgBg {
    pub foreground: String,
    pub background: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AlacrittySearch {
    #[serde(default)]
    pub matches: Option<AlacrittyFgBg>,
    #[serde(default)]
    pub focused_match: Option<AlacrittyFgBg>,
    #[serde(default)]
    pub line_indicator: Option<AlacrittyFgBg>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AlacrittyHints {
    #[serde(default)]
    pub start: Option<AlacrittyFgBg>,
    #[serde(default)]
    pub end: Option<AlacrittyFgBg>,
    #[serde(default)]
    pub line_indicator: Option<AlacrittyFgBg>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OpenGPTheme {
    #[serde(default = "default_schema_version")]
    pub schema_version: String,
    #[serde(default)]
    pub dark: OpenGPPalette,
    #[serde(default)]
    pub light: OpenGPPalette,
    #[serde(default)]
    pub high_contrast: OpenGPPalette,
}

fn default_schema_version() -> String {
    "1.0".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OpenGPPalette {
    pub primary: String,
    pub secondary: String,
    pub background: String,
    pub foreground: String,
    pub error: String,
    pub success: String,
    pub warning: String,
    pub info: String,
    pub border: String,
    pub selected: String,
    pub highlight: String,
    pub disabled: String,
    pub scrollbar_bg: String,
    pub scrollbar_thumb: String,
    pub appointment_scheduled: String,
    pub appointment_confirmed: String,
    pub appointment_arrived: String,
    pub appointment_in_progress: String,
    pub appointment_completed: String,
    pub appointment_cancelled: String,
    pub appointment_dna: String,
    pub appointment_rescheduled: String,
    pub background_dark: String,
    pub text_dim: String,
    pub text_secondary: String,
}

pub mod error;
pub mod mapping;
pub mod output;
pub mod parse;
pub mod validation;

use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, Serialize, Deserialize)]
struct AlacrittyTheme { 
    colors: AlacrittyColors,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AlacrittyColors { 
    primary: AlacrittyPrimary,
    normal: Ansi8,
    bright: Ansi8,
    #[serde(default)]
    dim: Option<Ansi8>,
    #[serde(default)]
    cursor: Option<AlacrittyColorPair>,
    #[serde(default)]
    selection: Option<AlacrittyColorPair>,
    #[serde(default)]
    indexed_colors: Vec<AlacrittyIndexedColor>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Ansi8 {
    black: String,
    red: String,
    green: String,
    yellow: String,
    blue: String,
    magenta: String,
    cyan: String,
    white: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AlacrittyPrimary {
    background: String,
    foreground: String,
    #[serde(default)]
    dim_foreground: Option<String>,
    #[serde(default)]
    bright_foreground: Option<String>,
}

struct AlacrittyColorPair {
    text: String,
    cursor: String,
}

struct AlacrittyIndexedColor {
    index: u8,
    color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenGPTheme {
    schema_version: String,
    #[serde(default)]
    dark: OpenGPPalette,
    #[serde(default)]
    light: OpenGPPalette,
    #[serde(default)]
    high_contrast: OpenGPPalette,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenGPPalette {
    primary: String,
    secondary: String,
    background: String,
    foreground: String,
    error: String,
    success: String,
    warning: String,
    info: String,
    border: String,
    selected: String,
    highlight: String,
    disabled: String,
    scrollbar_bg: String,
    scrollbar_thumb: String,
    appointment_scheduled: String,
    appointment_confirmed: String,
    appointment_arrived: String,
    appointment_in_progress: String,
    appointment_completed: String,
    appointment_cancelled: String,
    appointment_dna: String,
    appointment_rescheduled: String,
    background_dark: String,
    text_dim: String,
    text_secondary: String,
}

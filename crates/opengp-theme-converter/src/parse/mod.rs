pub mod toml_parser;
pub mod yaml_parser;

use crate::error::ThemeConverterError;
use crate::AlacrittyTheme;
use std::path::Path;

pub fn parse_by_extension(
    path: &Path,
    content: &str,
) -> Result<AlacrittyTheme, ThemeConverterError> {
    match path.extension().and_then(|e| e.to_str()) {
        Some("toml") => toml_parser::parse_alacritty_toml(content),
        Some("yml") | Some("yaml") => yaml_parser::parse_alacritty_yaml(content),
        ext => Err(ThemeConverterError::UnsupportedExtension(
            ext.unwrap_or("<no extension>").to_string(),
        )),
    }
}

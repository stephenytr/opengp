use crate::error::ThemeConverterError;
use crate::AlacrittyTheme;

pub fn parse_alacritty_yaml(content: &str) -> Result<AlacrittyTheme, ThemeConverterError> {
    let theme: AlacrittyTheme = serde_yaml::from_str(content)?;
    Ok(theme)
}

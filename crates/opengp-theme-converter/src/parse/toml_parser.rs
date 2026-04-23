 use crate::error::ThemeConverterError;
 use crate::AlacrittyTheme;

 pub fn parse_alacritty_toml(content: &str) -> Result<AlacrittyTheme, ThemeConverterError> {
     let theme: AlacrittyTheme = toml::from_str(content)?;
     Ok(theme)
 }

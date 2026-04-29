use crate::error::ThemeConverterError;
use crate::OpenGPTheme;

pub fn render_opengp_toml(theme: &OpenGPTheme) -> Result<String, ThemeConverterError> {
    let toml_value =
        toml::to_string_pretty(theme).map_err(|e| ThemeConverterError::TomlSerializeError(e))?;
    Ok(toml_value)
}

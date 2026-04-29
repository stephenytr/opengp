use serde_yaml::Error as YamlDeError;
use thiserror::Error;
use toml::de::Error as TomlDeError;

#[derive(Debug, Error)]
pub enum ThemeConverterError {
    #[error("not implemented")]
    NotImplemented,

    #[error("failed to parse YAML: {0}")]
    YomlParseError(#[from] YamlDeError),

    #[error("failed to parse TOML: {0}")]
    TomlParseError(#[from] TomlDeError),

    #[error("missing required field '{0}' in Alacritty theme")]
    MissingRequiredField(&'static str),

    #[error("unsupported file extension: '{0}' (expected .toml or .yaml)")]
    UnsupportedExtension(String),

    #[error("invalid hex color: '{0}'")]
    InvalidHexColor(String),

    #[error("failed to serialize TOML: {0}")]
    TomlSerializeError(#[from] toml::ser::Error),
}

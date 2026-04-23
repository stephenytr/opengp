use thiserror::Error;

#[derive(Debug, Error)]
pub enum ThemeConverterError {
    #[error("not implemented")]
    NotImplemented,
}

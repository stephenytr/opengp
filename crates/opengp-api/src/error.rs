use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("invalid API_PORT value: {0}")]
    InvalidPort(String),

    #[error("invalid database URL: {0}")]
    InvalidDatabaseUrl(String),

    #[error("failed to initialize encryption service: {0}")]
    EncryptionInit(String),
}

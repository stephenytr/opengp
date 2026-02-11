//! Top-level error types
//!
//! Defines the main error type used throughout the application.

use crate::config::ConfigError;
use crate::infrastructure::crypto::CryptoError;

/// Result type alias using our Error type
pub type Result<T> = std::result::Result<T, Error>;

/// Top-level application error
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Encryption error: {0}")]
    Crypto(#[from] CryptoError),

    #[error("Application error: {0}")]
    App(String),
}

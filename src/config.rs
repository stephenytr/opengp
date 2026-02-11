//! Configuration management
//!
//! Handles loading and managing application configuration from environment
//! variables and configuration files.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Database connection URL
    pub database_url: String,

    /// Encryption key (hex-encoded)
    pub encryption_key: String,

    /// Session timeout in seconds
    pub session_timeout_secs: u64,

    /// Log level
    pub log_level: String,

    /// Data directory path
    pub data_dir: PathBuf,
}

impl Config {
    /// Load configuration from environment
    pub fn from_env() -> Result<Self, ConfigError> {
        Ok(Self {
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite:opengp.db".to_string()),
            encryption_key: std::env::var("ENCRYPTION_KEY")
                .map_err(|_| ConfigError::MissingEncryptionKey)?,
            session_timeout_secs: std::env::var("SESSION_TIMEOUT_SECS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(900), // 15 minutes default
            log_level: std::env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string()),
            data_dir: std::env::var("DATA_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(|_| PathBuf::from("./data")),
        })
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            database_url: "sqlite:opengp.db".to_string(),
            encryption_key: String::new(),
            session_timeout_secs: 900,
            log_level: "info".to_string(),
            data_dir: PathBuf::from("./data"),
        }
    }
}

/// Configuration errors
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Missing encryption key - set ENCRYPTION_KEY environment variable")]
    MissingEncryptionKey,

    #[error("Invalid configuration: {0}")]
    Invalid(String),
}

//! Configuration management
//!
//! Handles loading and managing application configuration from environment
//! variables and configuration files.

use crate::infrastructure::database::DatabaseConfig;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Database configuration
    pub database: DatabaseConfig,

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
        let database_url =
            std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:opengp.db".to_string());

        let max_connections = std::env::var("DATABASE_MAX_CONNECTIONS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(10);

        let min_connections = std::env::var("DATABASE_MIN_CONNECTIONS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(2);

        Ok(Self {
            database: DatabaseConfig {
                url: database_url,
                max_connections,
                min_connections,
                connect_timeout_secs: 30,
                idle_timeout_secs: 600,
            },
            encryption_key: std::env::var("ENCRYPTION_KEY")
                .map_err(|_| ConfigError::MissingEncryptionKey)?,
            session_timeout_secs: std::env::var("SESSION_TIMEOUT_SECS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(900),
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
            database: DatabaseConfig::default(),
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

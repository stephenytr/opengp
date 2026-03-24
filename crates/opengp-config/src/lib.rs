//! Configuration management
//!
//! Handles loading and managing application configuration from environment
//! variables and configuration files.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Database URL (e.g., "sqlite:opengp.db")
    pub url: String,
    /// Maximum number of connections
    pub max_connections: u32,
    /// Minimum number of connections
    pub min_connections: u32,
    /// Connection timeout in seconds
    pub connect_timeout_secs: u64,
    /// Idle connection timeout in seconds
    pub idle_timeout_secs: u64,
}

/// Redis configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    /// Redis URL (e.g., "redis://localhost:6379")
    /// If None, Redis caching is disabled
    pub url: Option<String>,
    /// Maximum number of connections
    pub max_connections: u32,
    /// Minimum number of connections
    pub min_connections: u32,
    /// Default TTL in seconds
    pub ttl_default_secs: u64,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "sqlite:opengp.db".to_string(),
            max_connections: 10,
            min_connections: 2,
            connect_timeout_secs: 30,
            idle_timeout_secs: 600,
        }
    }
}

impl Default for RedisConfig {
    fn default() -> Self {
        Self {
            url: None,
            max_connections: 10,
            min_connections: 2,
            ttl_default_secs: 3600,
        }
    }
}

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Database configuration
    pub database: DatabaseConfig,

    /// Redis configuration
    pub redis: RedisConfig,

    /// Calendar configuration
    pub calendar: CalendarConfig,

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
        let _ = dotenvy::dotenv();

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

        let redis_url = std::env::var("REDIS_URL").ok();
        let redis_max_connections = std::env::var("REDIS_MAX_CONNECTIONS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(10);
        let redis_min_connections = std::env::var("REDIS_MIN_CONNECTIONS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(2);
        let redis_ttl_default_secs = std::env::var("REDIS_TTL_DEFAULT_SECS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(3600);

        Ok(Self {
            database: DatabaseConfig {
                url: database_url,
                max_connections,
                min_connections,
                connect_timeout_secs: 30,
                idle_timeout_secs: 600,
            },
            redis: RedisConfig {
                url: redis_url,
                max_connections: redis_max_connections,
                min_connections: redis_min_connections,
                ttl_default_secs: redis_ttl_default_secs,
            },
            calendar: CalendarConfig::from_env()?,
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
            redis: RedisConfig::default(),
            calendar: CalendarConfig::default(),
            encryption_key: String::new(),
            session_timeout_secs: 900,
            log_level: "info".to_string(),
            data_dir: PathBuf::from("./data"),
        }
    }
}

/// Calendar configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarConfig {
    /// Minimum hour user can scroll to (default: 6)
    pub min_hour: u8,
    /// Maximum hour user can scroll to (default: 22)
    pub max_hour: u8,
    /// Initial viewport start hour (default: 8)
    pub viewport_start_hour: u8,
    /// Initial viewport end hour (default: 18)
    pub viewport_end_hour: u8,
}

impl Default for CalendarConfig {
    fn default() -> Self {
        Self {
            min_hour: 6,
            max_hour: 22,
            viewport_start_hour: 8,
            viewport_end_hour: 18,
        }
    }
}

impl CalendarConfig {
    /// Load calendar configuration from environment variables
    ///
    /// Reads the following environment variables:
    /// - CALENDAR_MIN_HOUR (default: 6)
    /// - CALENDAR_MAX_HOUR (default: 22)
    /// - CALENDAR_START_HOUR (default: 8)
    /// - CALENDAR_END_HOUR (default: 18)
    ///
    /// # Validation
    /// - viewport_start_hour >= min_hour
    /// - viewport_end_hour <= max_hour
    /// - viewport_start_hour < viewport_end_hour
    ///
    /// # Returns
    /// * `Ok(CalendarConfig)` - Configuration loaded and validated
    /// * `Err(ConfigError::Invalid)` - Validation failed
    pub fn from_env() -> Result<Self, ConfigError> {
        let min_hour = std::env::var("CALENDAR_MIN_HOUR")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(6);

        let max_hour = std::env::var("CALENDAR_MAX_HOUR")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(22);

        let viewport_start_hour = std::env::var("CALENDAR_START_HOUR")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(8);

        let viewport_end_hour = std::env::var("CALENDAR_END_HOUR")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(18);

        let config = Self {
            min_hour,
            max_hour,
            viewport_start_hour,
            viewport_end_hour,
        };

        config.validate()?;
        Ok(config)
    }

    /// Validate calendar configuration
    ///
    /// # Errors
    /// Returns ConfigError::Invalid if:
    /// - viewport_start_hour < min_hour
    /// - viewport_end_hour > max_hour
    /// - viewport_start_hour >= viewport_end_hour
    fn validate(&self) -> Result<(), ConfigError> {
        if self.viewport_start_hour < self.min_hour {
            return Err(ConfigError::Invalid(format!(
                "viewport_start_hour ({}) must be >= min_hour ({})",
                self.viewport_start_hour, self.min_hour
            )));
        }

        if self.viewport_end_hour > self.max_hour {
            return Err(ConfigError::Invalid(format!(
                "viewport_end_hour ({}) must be <= max_hour ({})",
                self.viewport_end_hour, self.max_hour
            )));
        }

        if self.viewport_start_hour >= self.viewport_end_hour {
            return Err(ConfigError::Invalid(format!(
                "viewport_start_hour ({}) must be < viewport_end_hour ({})",
                self.viewport_start_hour, self.viewport_end_hour
            )));
        }

        Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;
    use temp_env;

    #[test]
    fn test_calendar_config_default() {
        let config = CalendarConfig::default();
        assert_eq!(config.min_hour, 6);
        assert_eq!(config.max_hour, 22);
        assert_eq!(config.viewport_start_hour, 8);
        assert_eq!(config.viewport_end_hour, 18);
    }

    #[test]
    fn test_calendar_config_from_env() {
        temp_env::with_vars(
            [
                ("CALENDAR_MIN_HOUR", None::<&str>),
                ("CALENDAR_MAX_HOUR", None::<&str>),
                ("CALENDAR_START_HOUR", None::<&str>),
                ("CALENDAR_END_HOUR", None::<&str>),
            ],
            || {
                let config = CalendarConfig::from_env().expect("should load with defaults");
                assert_eq!(config.min_hour, 6);
                assert_eq!(config.max_hour, 22);
                assert_eq!(config.viewport_start_hour, 8);
                assert_eq!(config.viewport_end_hour, 18);
            },
        );
    }

    #[test]
    fn test_calendar_config_from_env_custom_values() {
        temp_env::with_vars(
            [
                ("CALENDAR_MIN_HOUR", Some("5")),
                ("CALENDAR_MAX_HOUR", Some("23")),
                ("CALENDAR_START_HOUR", Some("7")),
                ("CALENDAR_END_HOUR", Some("19")),
            ],
            || {
                let config = CalendarConfig::from_env().expect("should load custom values");
                assert_eq!(config.min_hour, 5);
                assert_eq!(config.max_hour, 23);
                assert_eq!(config.viewport_start_hour, 7);
                assert_eq!(config.viewport_end_hour, 19);
            },
        );
    }

    #[test]
    fn test_calendar_config_validation_start_less_than_min() {
        let config = CalendarConfig {
            min_hour: 6,
            max_hour: 22,
            viewport_start_hour: 5,
            viewport_end_hour: 18,
        };

        let result = config.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("viewport_start_hour"));
    }

    #[test]
    fn test_calendar_config_validation_end_greater_than_max() {
        let config = CalendarConfig {
            min_hour: 6,
            max_hour: 22,
            viewport_start_hour: 8,
            viewport_end_hour: 23,
        };

        let result = config.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("viewport_end_hour"));
    }

    #[test]
    fn test_calendar_config_validation_start_not_less_than_end() {
        let config = CalendarConfig {
            min_hour: 6,
            max_hour: 22,
            viewport_start_hour: 18,
            viewport_end_hour: 18,
        };

        let result = config.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("viewport_start_hour"));
    }

    #[test]
    fn test_calendar_config_validation_valid() {
        let config = CalendarConfig {
            min_hour: 6,
            max_hour: 22,
            viewport_start_hour: 8,
            viewport_end_hour: 18,
        };

        let result = config.validate();
        assert!(result.is_ok());
    }
}

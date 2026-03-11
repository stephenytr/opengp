use crate::ApiError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub patient_database_url: Option<String>,
    pub database_max_connections: u32,
    pub database_min_connections: u32,
    pub connect_timeout_secs: u64,
    pub idle_timeout_secs: u64,
    pub encryption_key: String,
    pub log_level: String,
}

impl ApiConfig {
    pub fn from_env() -> Result<Self, ApiError> {
        let _ = dotenvy::dotenv();

        let port = std::env::var("API_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse::<u16>()
            .map_err(|_| ApiError::InvalidPort("API_PORT must be a valid u16".to_string()))?;

        Ok(Self {
            host: std::env::var("API_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port,
            database_url: std::env::var("API_DATABASE_URL").unwrap_or_else(|_| {
                "postgres://postgres:postgres@127.0.0.1:5432/opengp".to_string()
            }),
            patient_database_url: std::env::var("API_PATIENT_DATABASE_URL").ok(),
            database_max_connections: std::env::var("API_DATABASE_MAX_CONNECTIONS")
                .ok()
                .and_then(|v| v.parse::<u32>().ok())
                .unwrap_or(10),
            database_min_connections: std::env::var("API_DATABASE_MIN_CONNECTIONS")
                .ok()
                .and_then(|v| v.parse::<u32>().ok())
                .unwrap_or(2),
            connect_timeout_secs: std::env::var("API_DATABASE_CONNECT_TIMEOUT_SECS")
                .ok()
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or(30),
            idle_timeout_secs: std::env::var("API_DATABASE_IDLE_TIMEOUT_SECS")
                .ok()
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or(600),
            encryption_key: std::env::var("ENCRYPTION_KEY").unwrap_or_else(|_| {
                "0000000000000000000000000000000000000000000000000000000000000000".to_string()
            }),
            log_level: std::env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string()),
        })
    }

    pub fn bind_address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_default_port_when_env_missing() {
        unsafe {
            std::env::remove_var("API_PORT");
        }
        let config = ApiConfig::from_env().expect("config should load defaults");
        assert_eq!(config.port, 8080);
    }
}

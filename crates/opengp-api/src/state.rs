use std::{
    str::FromStr,
    sync::atomic::AtomicU64,
    sync::Arc,
    time::{Duration, Instant},
};

use opengp_domain::domain::audit::AuditEmitter;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::PgPool;
use std::path::Path;
use tracing::info;

use crate::services::ApiServices;
use crate::{ApiConfig, ApiError};

#[derive(Clone)]
pub struct ApiState {
    pub pool: PgPool,
    pub services: Arc<ApiServices>,
    pub config: ApiConfig,
    pub metrics: Arc<ApiMetrics>,
    pub audit_emitter: Arc<dyn AuditEmitter>,
}

pub struct ApiMetrics {
    pub request_count: AtomicU64,
    pub error_count: AtomicU64,
    pub active_sessions: AtomicU64,
    pub start_time: Instant,
}

impl ApiMetrics {
    pub fn new() -> Self {
        Self {
            request_count: AtomicU64::new(0),
            error_count: AtomicU64::new(0),
            active_sessions: AtomicU64::new(0),
            start_time: Instant::now(),
        }
    }

    pub fn uptime_seconds(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }
}

impl ApiState {
    pub async fn new(config: ApiConfig) -> Result<Self, ApiError> {
        let connect_options = PgConnectOptions::from_str(&config.database_url)
            .map_err(|e| ApiError::InvalidDatabaseUrl(e.to_string()))?;

        let pool = PgPoolOptions::new()
            .max_connections(config.database_max_connections)
            .min_connections(config.database_min_connections)
            .acquire_timeout(Duration::from_secs(config.connect_timeout_secs))
            .idle_timeout(Duration::from_secs(config.idle_timeout_secs))
            .test_before_acquire(false)
            .connect_lazy_with(connect_options);

        crate::migrations::run_migrations(&pool).await?;

        let services = Arc::new(ApiServices::new(&config, &pool).await?);
        let metrics = Arc::new(ApiMetrics::new());
        let audit_emitter = services.audit_service.clone() as Arc<dyn AuditEmitter>;

        Ok(Self {
            pool,
            services,
            config,
            metrics,
            audit_emitter,
        })
    }

    #[cfg(test)]
    fn from_parts(pool: PgPool, services: Arc<ApiServices>, config: ApiConfig) -> Self {
        let audit_emitter = services.audit_service.clone() as Arc<dyn AuditEmitter>;
        Self {
            pool,
            services,
            config,
            metrics: Arc::new(ApiMetrics::new()),
            audit_emitter,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn initializes_state_with_configured_port() {
        let config = ApiConfig {
            host: "127.0.0.1".to_string(),
            port: 9090,
            database_url: "postgres://postgres:postgres@127.0.0.1:5432/opengp".to_string(),
            patient_database_url: None,
            database_max_connections: 4,
            database_min_connections: 1,
            connect_timeout_secs: 5,
            idle_timeout_secs: 30,
            encryption_key: "0000000000000000000000000000000000000000000000000000000000000000"
                .to_string(),
            session_timeout_minutes: 480,
            log_level: "info".to_string(),
        };

        let state = ApiState::new(config.clone())
            .await
            .expect("state should initialize");
        assert_eq!(state.config.port, 9090);
        assert_eq!(state.config.bind_address(), "127.0.0.1:9090");
    }

    #[tokio::test]
    async fn supports_state_construction_from_parts() {
        let config = ApiConfig {
            host: "0.0.0.0".to_string(),
            port: 8080,
            database_url: "postgres://postgres:postgres@127.0.0.1:5432/opengp".to_string(),
            patient_database_url: None,
            database_max_connections: 10,
            database_min_connections: 2,
            connect_timeout_secs: 30,
            idle_timeout_secs: 600,
            encryption_key: "0000000000000000000000000000000000000000000000000000000000000000"
                .to_string(),
            session_timeout_minutes: 480,
            log_level: "info".to_string(),
        };

        let pool = PgPoolOptions::new()
            .connect_lazy(&config.database_url)
            .expect("pool should initialize lazily");
        let services = Arc::new(
            ApiServices::new(&config, &pool)
                .await
                .expect("services should initialize"),
        );

        let state = ApiState::from_parts(pool, services, config);
        assert_eq!(state.config.port, 8080);
    }
}

use std::{
    str::FromStr,
    sync::atomic::AtomicU64,
    sync::Arc,
    time::{Duration, Instant},
};

use opengp_domain::domain::audit::AuditEmitter;
use opengp_cache::{CacheServiceImpl, CircuitBreaker, CacheConfig, RedisPool};
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::PgPool;

use crate::services::ApiServices;
use crate::{ApiConfig, ApiError};

#[derive(Clone)]
pub struct ApiState {
    pub pool: PgPool,
    pub services: Arc<ApiServices>,
    pub config: ApiConfig,
    pub metrics: Arc<ApiMetrics>,
    pub audit_emitter: Arc<dyn AuditEmitter>,
    pub cache_service: Option<Arc<CacheServiceImpl>>,
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
            .test_before_acquire(true)
            .connect_with(connect_options)
            .await
            .map_err(|e| {
                ApiError::Configuration(format!("Failed to initialize database pool: {e}"))
            })?;

        crate::migrations::run_migrations(&pool).await?;

        let services = Arc::new(ApiServices::new(&config, &pool).await?);
        let metrics = Arc::new(ApiMetrics::new());
        let audit_emitter = services.audit_service.clone() as Arc<dyn AuditEmitter>;

        // Initialize cache service - gracefully handle missing Redis URL
        let cache_service = Self::init_cache_service().await;

        Ok(Self {
            pool,
            services,
            config,
            metrics,
            audit_emitter,
            cache_service,
        })
    }

    /// Initialize cache service from Redis configuration
    /// Returns None if Redis is not configured or connection fails
    async fn init_cache_service() -> Option<Arc<CacheServiceImpl>> {
        // Load Redis config from environment
        let redis_url = std::env::var("REDIS_URL").ok();

        // If no Redis URL, cache is disabled
        if redis_url.is_none() {
            return None;
        }

        // Create RedisConfig with the URL
        let redis_config = opengp_config::RedisConfig {
            url: redis_url,
            max_connections: std::env::var("REDIS_MAX_CONNECTIONS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(10),
            min_connections: std::env::var("REDIS_MIN_CONNECTIONS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(2),
            ttl_default_secs: std::env::var("REDIS_TTL_DEFAULT_SECS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(3600),
        };

        // Attempt to create Redis pool
        let pool = match RedisPool::new(&redis_config).await {
            Ok(pool) => pool,
            Err(_) => {
                // Log and return None on connection failure
                tracing::warn!("Failed to initialize Redis cache service - cache disabled");
                return None;
            }
        };

        // Create cache config and service
        let cache_config = CacheConfig::from_redis_config(&redis_config);
        let circuit_breaker = CircuitBreaker::new();
        let cache_service = CacheServiceImpl::new(pool, cache_config, circuit_breaker);

        Some(Arc::new(cache_service))
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
            cache_service: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires PostgreSQL database connection
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
    #[ignore] // Requires PostgreSQL database connection
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

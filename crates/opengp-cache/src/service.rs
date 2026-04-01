use crate::circuit::CircuitBreaker;
use crate::error::CacheError;
use crate::pool::RedisPool;
use crate::stampede::StampedeGuard;
use deadpool_redis::redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct CacheCircuitBreakerConfig {
    pub failure_threshold: u32,
    pub open_duration_secs: u64,
}

impl Default for CacheCircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            open_duration_secs: 30,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CacheStampedeConfig {
    pub default_ttl_secs: u64,
    pub retry_attempts: u32,
    pub retry_delay_ms: u64,
}

impl Default for CacheStampedeConfig {
    fn default() -> Self {
        Self {
            default_ttl_secs: 5,
            retry_attempts: 3,
            retry_delay_ms: 100,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CacheEntityTtlConfig {
    pub patient_secs: u64,
    pub search_secs: u64,
    pub appointment_secs: u64,
}

impl Default for CacheEntityTtlConfig {
    fn default() -> Self {
        Self {
            patient_secs: 900,
            search_secs: 300,
            appointment_secs: 120,
        }
    }
}

/// Cache configuration
#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub enabled: bool,
    pub default_ttl_secs: u64,
    pub key_prefix: String,
    pub circuit_breaker: CacheCircuitBreakerConfig,
    pub stampede: CacheStampedeConfig,
    pub entity_ttl: CacheEntityTtlConfig,
}

impl CacheConfig {
    pub fn from_redis_config(redis_config: &opengp_config::RedisConfig) -> Self {
        Self {
            enabled: redis_config.url.is_some(),
            default_ttl_secs: redis_config.ttl_default_secs,
            key_prefix: "opengp".to_string(),
            circuit_breaker: CacheCircuitBreakerConfig::default(),
            stampede: CacheStampedeConfig::default(),
            entity_ttl: CacheEntityTtlConfig::default(),
        }
    }

    pub fn from_app_config(app: &opengp_config::AppConfig) -> Self {
        Self {
            enabled: true,
            default_ttl_secs: app.cache.default_ttl_secs,
            key_prefix: app.cache.key_prefix.clone(),
            circuit_breaker: CacheCircuitBreakerConfig {
                failure_threshold: app.cache.circuit_breaker.failure_threshold,
                open_duration_secs: app.cache.circuit_breaker.open_duration_secs,
            },
            stampede: CacheStampedeConfig {
                default_ttl_secs: app.cache.stampede.default_ttl_secs,
                retry_attempts: app.cache.stampede.retry_attempts,
                retry_delay_ms: app.cache.stampede.retry_delay_ms,
            },
            entity_ttl: CacheEntityTtlConfig {
                patient_secs: app.cache.entity_ttl.patient_secs,
                search_secs: app.cache.entity_ttl.search_secs,
                appointment_secs: app.cache.entity_ttl.appointment_secs,
            },
        }
    }
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            default_ttl_secs: 3600,
            key_prefix: "opengp".to_string(),
            circuit_breaker: CacheCircuitBreakerConfig::default(),
            stampede: CacheStampedeConfig::default(),
            entity_ttl: CacheEntityTtlConfig::default(),
        }
    }
}

/// Redis-backed cache service with circuit breaker integration
pub struct CacheServiceImpl {
    pool: RedisPool,
    config: CacheConfig,
    circuit_breaker: CircuitBreaker,
}

impl CacheServiceImpl {
    /// Create a new cache service
    pub fn new(pool: RedisPool, config: CacheConfig, circuit_breaker: CircuitBreaker) -> Self {
        Self {
            pool,
            config,
            circuit_breaker,
        }
    }

    pub fn stampede_lock_ttl(&self) -> Duration {
        Duration::from_secs(self.config.stampede.default_ttl_secs)
    }

    pub fn stampede_retry_attempts(&self) -> u32 {
        self.config.stampede.retry_attempts
    }

    pub fn stampede_retry_delay(&self) -> Duration {
        Duration::from_millis(self.config.stampede.retry_delay_ms)
    }

    pub fn patient_ttl_secs(&self) -> u64 {
        self.config.entity_ttl.patient_secs
    }

    pub fn search_ttl_secs(&self) -> u64 {
        self.config.entity_ttl.search_secs
    }

    pub fn appointment_ttl_secs(&self) -> u64 {
        self.config.entity_ttl.appointment_secs
    }

    /// Get a value from cache, deserializing from JSON
    ///
    /// # Arguments
    /// * `key` - Cache key (will be prefixed)
    ///
    /// # Returns
    /// * `Ok(Some(T))` if key exists and deserializes successfully
    /// * `Ok(None)` if key doesn't exist
    /// * `Err(CacheError)` if circuit is open or Redis operation fails
    pub async fn get<T: for<'de> Deserialize<'de>>(
        &self,
        key: &str,
    ) -> Result<Option<T>, CacheError> {
        // Check circuit breaker state before attempting operation
        if !self.circuit_breaker.allow_request() {
            return Err(CacheError::ConnectionFailed(
                "Circuit breaker is open".to_string(),
            ));
        }

        let full_key = self.build_key(key);

        let mut conn = self.pool.inner().get().await.map_err(|e| {
            self.circuit_breaker.record_failure();
            CacheError::ConnectionFailed(e.to_string())
        })?;

        let value: Option<String> = conn.get(&full_key).await.map_err(|e| {
            self.circuit_breaker.record_failure();
            CacheError::Internal(e.to_string())
        })?;

        match value {
            Some(json_str) => {
                let deserialized = serde_json::from_str::<T>(&json_str).map_err(|e| {
                    CacheError::Serialization(format!("Failed to deserialize: {}", e))
                })?;
                self.circuit_breaker.record_success();
                Ok(Some(deserialized))
            }
            None => {
                self.circuit_breaker.record_success();
                Ok(None)
            }
        }
    }

    /// Set a value in cache, serializing to JSON
    ///
    /// # Arguments
    /// * `key` - Cache key (will be prefixed)
    /// * `value` - Value to cache
    /// * `ttl_secs` - Time to live in seconds (uses config default if None)
    ///
    /// # Returns
    /// * `Ok(())` if successful
    /// * `Err(CacheError)` if circuit is open or Redis operation fails
    pub async fn set<T: Serialize>(
        &self,
        key: &str,
        value: &T,
        ttl_secs: Option<u64>,
    ) -> Result<(), CacheError> {
        // Check circuit breaker state before attempting operation
        if !self.circuit_breaker.allow_request() {
            return Err(CacheError::ConnectionFailed(
                "Circuit breaker is open".to_string(),
            ));
        }

        let full_key = self.build_key(key);
        let ttl = ttl_secs.unwrap_or(self.config.default_ttl_secs);

        let json_str = serde_json::to_string(value)
            .map_err(|e| CacheError::Serialization(format!("Failed to serialize: {}", e)))?;

        let mut conn = self.pool.inner().get().await.map_err(|e| {
            self.circuit_breaker.record_failure();
            CacheError::ConnectionFailed(e.to_string())
        })?;

        let _: String = deadpool_redis::redis::cmd("SET")
            .arg(&full_key)
            .arg(&json_str)
            .arg("EX")
            .arg(ttl)
            .query_async(&mut conn)
            .await
            .map_err(|e| {
                self.circuit_breaker.record_failure();
                CacheError::Internal(e.to_string())
            })?;

        self.circuit_breaker.record_success();
        Ok(())
    }

    /// Delete a single key from cache
    ///
    /// # Arguments
    /// * `key` - Cache key (will be prefixed)
    ///
    /// # Returns
    /// * `Ok(())` if successful (even if key didn't exist)
    /// * `Err(CacheError)` if circuit is open or Redis operation fails
    pub async fn invalidate(&self, key: &str) -> Result<(), CacheError> {
        // Check circuit breaker state before attempting operation
        if !self.circuit_breaker.allow_request() {
            return Err(CacheError::ConnectionFailed(
                "Circuit breaker is open".to_string(),
            ));
        }

        let full_key = self.build_key(key);

        let mut conn = self.pool.inner().get().await.map_err(|e| {
            self.circuit_breaker.record_failure();
            CacheError::ConnectionFailed(e.to_string())
        })?;

        let _: () = conn.del(&full_key).await.map_err(|e| {
            self.circuit_breaker.record_failure();
            CacheError::Internal(e.to_string())
        })?;

        self.circuit_breaker.record_success();
        Ok(())
    }

    /// Delete all keys matching a pattern
    ///
    /// Uses SCAN to iteratively find and delete matching keys.
    ///
    /// # Arguments
    /// * `pattern` - Glob pattern to match (e.g., "patient:*")
    ///
    /// # Returns
    /// * `Ok(count)` - number of keys deleted
    /// * `Err(CacheError)` if circuit is open or Redis operation fails
    pub async fn invalidate_pattern(&self, pattern: &str) -> Result<u64, CacheError> {
        // Check circuit breaker state before attempting operation
        if !self.circuit_breaker.allow_request() {
            return Err(CacheError::ConnectionFailed(
                "Circuit breaker is open".to_string(),
            ));
        }

        let full_pattern = self.build_key(pattern);

        let mut conn = self.pool.inner().get().await.map_err(|e| {
            self.circuit_breaker.record_failure();
            CacheError::ConnectionFailed(e.to_string())
        })?;

        let mut cursor = 0;
        let mut deleted = 0u64;

        loop {
            let (new_cursor, keys): (u64, Vec<String>) = deadpool_redis::redis::cmd("SCAN")
                .arg(cursor)
                .arg("MATCH")
                .arg(&full_pattern)
                .query_async(&mut conn)
                .await
                .map_err(|e| {
                    self.circuit_breaker.record_failure();
                    CacheError::Internal(e.to_string())
                })?;

            for key in keys {
                let _: () = conn.del(&key).await.map_err(|e| {
                    self.circuit_breaker.record_failure();
                    CacheError::Internal(e.to_string())
                })?;
                deleted += 1;
            }

            cursor = new_cursor;
            if cursor == 0 {
                break;
            }
        }

        self.circuit_breaker.record_success();
        Ok(deleted)
    }

    pub async fn get_or_fetch<T, F>(&self, key: &str, fetcher: F, ttl: u64) -> Result<T, CacheError>
    where
        T: Serialize + for<'de> Deserialize<'de> + Clone,
        F: Future<Output = Result<T, CacheError>>,
    {
        if let Some(cached) = self.get::<T>(key).await? {
            return Ok(cached);
        }

        let full_key = self.build_key(key);
        let guard = StampedeGuard::new(&full_key, self.stampede_lock_ttl());

        let lock_acquired = self.try_acquire_stampede_lock(&guard).await?;

        if lock_acquired {
            let result = async {
                let fetched = fetcher.await?;
                self.set(key, &fetched, Some(ttl)).await?;
                Ok::<T, CacheError>(fetched)
            }
            .await;

            let _ = self.release_stampede_lock(&guard).await;
            return result;
        }

        for _ in 0..self.stampede_retry_attempts() {
            tokio::time::sleep(self.stampede_retry_delay()).await;
            if let Some(cached) = self.get::<T>(key).await? {
                return Ok(cached);
            }
        }

        let fetched = fetcher.await?;
        self.set(key, &fetched, Some(ttl)).await?;
        Ok(fetched)
    }

    pub async fn try_acquire_stampede_lock(&self, guard: &StampedeGuard) -> Result<bool, CacheError> {
        if !self.circuit_breaker.allow_request() {
            return Err(CacheError::ConnectionFailed(
                "Circuit breaker is open".to_string(),
            ));
        }

        let mut conn = self.pool.inner().get().await.map_err(|e| {
            self.circuit_breaker.record_failure();
            CacheError::ConnectionFailed(e.to_string())
        })?;

        let result: Option<String> = deadpool_redis::redis::cmd("SET")
            .arg(&guard.lock_key)
            .arg("1")
            .arg("NX")
            .arg("EX")
            .arg(guard.ttl.as_secs())
            .query_async(&mut conn)
            .await
            .map_err(|e| {
                self.circuit_breaker.record_failure();
                CacheError::Internal(e.to_string())
            })?;

        self.circuit_breaker.record_success();
        Ok(result.is_some())
    }

    pub async fn release_stampede_lock(&self, guard: &StampedeGuard) -> Result<(), CacheError> {
        if !self.circuit_breaker.allow_request() {
            return Ok(());
        }

        let mut conn = self.pool.inner().get().await.map_err(|e| {
            self.circuit_breaker.record_failure();
            CacheError::ConnectionFailed(e.to_string())
        })?;

        let _: () = conn.del(&guard.lock_key).await.map_err(|e| {
            self.circuit_breaker.record_failure();
            CacheError::Internal(e.to_string())
        })?;

        self.circuit_breaker.record_success();
        Ok(())
    }

    /// Build a fully-qualified cache key with prefix
    fn build_key(&self, key: &str) -> String {
        format!("{}:{}", self.config.key_prefix, key)
    }
}

#[async_trait::async_trait]
pub trait CacheService: Send + Sync {
    async fn get(&self, key: &str) -> Result<Option<String>, CacheError>;

    async fn set(&self, key: &str, value: String) -> Result<(), CacheError>;

    async fn delete(&self, key: &str) -> Result<(), CacheError>;

    async fn exists(&self, key: &str) -> Result<bool, CacheError>;
}

use crate::error::CacheError;
use deadpool_redis::Pool;
use opengp_config::RedisConfig;

pub struct RedisPool(Pool);

impl RedisPool {
    pub async fn new(config: &RedisConfig) -> Result<Self, CacheError> {
        match &config.url {
            None => Err(CacheError::ConnectionFailed("Redis not configured".into())),
            Some(url) => Self::from_url(url, config.max_connections).await,
        }
    }

    pub async fn from_url(url: &str, _max_connections: u32) -> Result<Self, CacheError> {
        let cfg = deadpool_redis::Config::from_url(url);
        let pool = cfg
            .create_pool(Some(deadpool_redis::Runtime::Tokio1))
            .map_err(|e| CacheError::ConnectionFailed(e.to_string()))?;

        Ok(Self(pool))
    }

    pub async fn get(&self) -> Result<deadpool_redis::Connection, CacheError> {
        self.0
            .get()
            .await
            .map_err(|e| CacheError::ConnectionFailed(e.to_string()))
    }

    pub fn inner(&self) -> &Pool {
        &self.0
    }
}

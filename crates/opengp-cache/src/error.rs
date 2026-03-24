use thiserror::Error;

#[derive(Debug, Error)]
pub enum CacheError {
    #[error("Redis connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Key not found in cache")]
    KeyNotFound,

    #[error("Cache operation timeout")]
    Timeout,

    #[error("Internal cache error: {0}")]
    Internal(String),
}

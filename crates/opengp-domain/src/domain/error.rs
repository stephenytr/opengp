//! Shared domain error types

use thiserror::Error;

/// Common repository errors for all domain modules
#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Not found")]
    NotFound,

    #[error("Constraint violation: {0}")]
    ConstraintViolation(String),
}

impl RepositoryError {
    /// Create a database error from an infrastructure-level error
    pub fn from_infrastructure(error: impl std::fmt::Display) -> Self {
        RepositoryError::Database(error.to_string())
    }
}

/// Trait for mapping infrastructure errors to domain errors
pub trait InfrastructureError: Send + Sync + 'static {
    /// Map a sqlx error to this domain error type
    fn map_sqlx_error<E: std::error::Error + Send + Sync + 'static>(error: E) -> Self;
}

impl InfrastructureError for RepositoryError {
    fn map_sqlx_error<E: std::error::Error + Send + Sync + 'static>(error: E) -> Self {
        RepositoryError::Database(error.to_string())
    }
}

//! Shared domain error types

use thiserror::Error;

/// Common repository errors for all domain modules
#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Not found")]
    NotFound,

    #[error("Constraint violation: {0}")]
    ConstraintViolation(String),
}

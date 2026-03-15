use thiserror::Error;
use uuid::Uuid;

use crate::domain::error::RepositoryError as BaseRepositoryError;

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("Validation error: {0}")]
    Validation(String),

    #[error("User not found: {0}")]
    NotFound(Uuid),

    #[error("User not found with username: {0}")]
    NotFoundByUsername(String),

    #[error("Duplicate user: {0}")]
    Duplicate(String),

    #[error("Authentication failed")]
    AuthenticationFailed,

    #[error("Account locked")]
    AccountLocked,

    #[error("Repository error: {0}")]
    Repository(#[from] RepositoryError),
}

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Account locked")]
    AccountLocked,

    #[error("Session expired")]
    SessionExpired,

    #[error("Repository error: {0}")]
    Repository(#[from] RepositoryError),
}

#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error(transparent)]
    Base(#[from] BaseRepositoryError),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Not found")]
    NotFound,

    #[error("Constraint violation: {0}")]
    ConstraintViolation(String),

    #[error("Conflict: {0}")]
    Conflict(String),
}

impl crate::domain::error::InfrastructureError for RepositoryError {
    fn map_sqlx_error<E: std::error::Error + Send + Sync + 'static>(error: E) -> Self {
        BaseRepositoryError::from_infrastructure(error).into()
    }
}

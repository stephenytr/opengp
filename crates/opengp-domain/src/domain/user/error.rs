use thiserror::Error;
use uuid::Uuid;

pub use crate::domain::error::RepositoryError;

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

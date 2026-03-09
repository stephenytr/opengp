use thiserror::Error;
use uuid::Uuid;

pub use crate::domain::error::RepositoryError;

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Order must include at least one requested test")]
    EmptyTestList,

    #[error("Order number cannot be empty")]
    EmptyOrderNumber,
}

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("Pathology order not found: {0}")]
    OrderNotFound(Uuid),

    #[error("Pathology result not found: {0}")]
    ResultNotFound(Uuid),

    #[error("Validation error: {0}")]
    Validation(#[from] ValidationError),

    #[error("Repository error: {0}")]
    Repository(#[from] RepositoryError),
}

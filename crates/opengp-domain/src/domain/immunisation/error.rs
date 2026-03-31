use thiserror::Error;
use uuid::Uuid;

pub use crate::domain::error::RepositoryError;

/// Validation errors for immunisation data.
#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Dose number must be greater than zero")]
    InvalidDoseNumber,

    #[error("Batch number cannot be empty")]
    EmptyBatchNumber,
}

/// Errors returned from the immunisation service layer.
#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("Immunisation not found: {0}")]
    NotFound(Uuid),

    #[error("Validation error: {0}")]
    Validation(#[from] ValidationError),

    #[error("Repository error: {0}")]
    Repository(#[from] RepositoryError),
}

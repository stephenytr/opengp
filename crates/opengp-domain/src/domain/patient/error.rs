use thiserror::Error;
use uuid::Uuid;

use crate::domain::error::RepositoryError as BaseRepositoryError;

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Empty name: {0}")]
    EmptyName(String),

    #[error("Invalid date of birth")]
    InvalidDateOfBirth,

    #[error("Invalid Medicare number")]
    InvalidMedicareNumber,
}

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("Duplicate patient found")]
    DuplicatePatient,

    #[error("Patient not found: {0}")]
    NotFound(Uuid),

    #[error("Validation error: {0}")]
    Validation(#[from] ValidationError),

    #[error("Repository error: {0}")]
    Repository(#[from] RepositoryError),
}

#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error(transparent)]
    Base(#[from] BaseRepositoryError),

    #[error("Encryption error: {0}")]
    Encryption(String),
}

impl crate::domain::error::InfrastructureError for RepositoryError {
    fn map_sqlx_error<E: std::error::Error + Send + Sync + 'static>(error: E) -> Self {
        BaseRepositoryError::from_infrastructure(error).into()
    }
}

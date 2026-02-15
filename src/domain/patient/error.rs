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
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Not found")]
    NotFound,

    #[error("Constraint violation: {0}")]
    ConstraintViolation(String),

    #[error("Encryption error: {0}")]
    Encryption(String),
}

impl From<BaseRepositoryError> for RepositoryError {
    fn from(err: BaseRepositoryError) -> Self {
        match err {
            BaseRepositoryError::Database(e) => RepositoryError::Database(e),
            BaseRepositoryError::NotFound => RepositoryError::NotFound,
            BaseRepositoryError::ConstraintViolation(s) => RepositoryError::ConstraintViolation(s),
        }
    }
}

use thiserror::Error;
use uuid::Uuid;

use crate::domain::error::RepositoryError as BaseRepositoryError;

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Invalid appointment time: {0}")]
    InvalidTime(String),

    #[error("End time must be after start time")]
    EndTimeBeforeStartTime,

    #[error("Appointment duration must be positive")]
    InvalidDuration,

    #[error("Invalid patient ID")]
    InvalidPatientId,

    #[error("Invalid practitioner ID")]
    InvalidPractitionerId,
}

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("Appointment not found: {0}")]
    NotFound(Uuid),

    #[error("Overlapping appointment detected: {0}")]
    Conflict(String),

    #[error("Validation error: {0}")]
    ValidationError(#[from] ValidationError),

    #[error("Invalid status transition: {0}")]
    InvalidTransition(String),

    #[error("Repository error: {0}")]
    Repository(#[from] RepositoryError),

    #[error("Audit error: {0}")]
    Audit(#[from] crate::domain::audit::ServiceError),
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

impl From<crate::domain::user::RepositoryError> for RepositoryError {
    fn from(error: crate::domain::user::RepositoryError) -> Self {
        match error {
            crate::domain::user::RepositoryError::Base(base) => Self::Base(base),
            crate::domain::user::RepositoryError::Database(message) => Self::Database(message),
            crate::domain::user::RepositoryError::NotFound => Self::NotFound,
            crate::domain::user::RepositoryError::ConstraintViolation(message) => {
                Self::ConstraintViolation(message)
            }
            crate::domain::user::RepositoryError::Conflict(message) => Self::Conflict(message),
        }
    }
}

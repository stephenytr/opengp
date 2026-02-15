use thiserror::Error;
use uuid::Uuid;

pub use crate::domain::error::RepositoryError;

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

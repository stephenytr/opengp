use thiserror::Error;
use uuid::Uuid;

pub use crate::domain::error::RepositoryError;

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Specialty cannot be empty")]
    EmptySpecialty,

    #[error("Reason cannot be empty")]
    EmptyReason,
}

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("Referral not found: {0}")]
    NotFound(Uuid),

    #[error("Referral is already sent")]
    AlreadySent,

    #[error("Validation error: {0}")]
    Validation(#[from] ValidationError),

    #[error("Repository error: {0}")]
    Repository(#[from] RepositoryError),
}

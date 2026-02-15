use thiserror::Error;
use uuid::Uuid;

pub use crate::domain::error::RepositoryError;

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Empty field: {0}")]
    EmptyField(String),

    #[error("Invalid quantity: {0}")]
    InvalidQuantity(String),

    #[error("Invalid repeats: {0}")]
    InvalidRepeats(String),

    #[error("Invalid PBS status: {0}")]
    InvalidPBSStatus(String),

    #[error("Authority approval required but not provided")]
    MissingAuthorityApproval,
}

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("Prescription not found: {0}")]
    NotFound(Uuid),

    #[error("Prescription already cancelled")]
    AlreadyCancelled,

    #[error("Prescription expired")]
    PrescriptionExpired,

    #[error("PBS authority required: {0}")]
    PBSAuthorityRequired(String),

    #[error("Drug interaction detected: {0}")]
    DrugInteraction(String),

    #[error("Validation error: {0}")]
    Validation(#[from] ValidationError),

    #[error("Repository error: {0}")]
    Repository(#[from] RepositoryError),

    #[error("Audit error: {0}")]
    Audit(String),
}

use thiserror::Error;
use uuid::Uuid;

use crate::domain::error::RepositoryError as BaseRepositoryError;

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Invalid entity type: {0}")]
    InvalidEntityType(String),

    #[error("Invalid entity ID")]
    InvalidEntityId,

    #[error("Changed by user ID is required")]
    MissingChangedBy,

    #[error("Timestamp cannot be in the future")]
    FutureTimestamp,

    #[error("Invalid time range: end time must be after start time")]
    InvalidTimeRange,
}

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("Audit entry not found: {0}")]
    NotFound(Uuid),

    #[error("Validation error: {0}")]
    ValidationError(#[from] ValidationError),

    #[error("Repository error: {0}")]
    Repository(#[from] AuditRepositoryError),
}

/// Errors that can occur in the audit repository layer
#[derive(Debug, Error)]
pub enum AuditRepositoryError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Not found")]
    NotFound,

    #[error("Constraint violation: {0}")]
    ConstraintViolation(String),

    #[error("Audit entry cannot be modified or deleted (append-only)")]
    ImmutableViolation,
}

impl From<BaseRepositoryError> for AuditRepositoryError {
    fn from(err: BaseRepositoryError) -> Self {
        match err {
            BaseRepositoryError::Database(e) => AuditRepositoryError::Database(e),
            BaseRepositoryError::NotFound => AuditRepositoryError::NotFound,
            BaseRepositoryError::ConstraintViolation(s) => {
                AuditRepositoryError::ConstraintViolation(s)
            }
        }
    }
}

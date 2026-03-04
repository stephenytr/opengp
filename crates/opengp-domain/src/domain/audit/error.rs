use thiserror::Error;
use uuid::Uuid;

use crate::domain::error::RepositoryError as BaseRepositoryError;

#[derive(Debug, Error)]
pub enum AuditEmitterError {
    #[error("Failed to emit audit entry: {0}")]
    Emit(String),
}

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
    #[error(transparent)]
    Base(#[from] BaseRepositoryError),

    #[error("Audit entry cannot be modified or deleted (append-only)")]
    ImmutableViolation,
}

impl crate::domain::error::InfrastructureError for AuditRepositoryError {
    fn map_sqlx_error<E: std::error::Error + Send + Sync + 'static>(error: E) -> Self {
        BaseRepositoryError::from_infrastructure(error).into()
    }
}

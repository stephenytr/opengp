use thiserror::Error;
use uuid::Uuid;

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

    #[error("Audit entry cannot be modified or deleted (append-only)")]
    ImmutableViolation,
}

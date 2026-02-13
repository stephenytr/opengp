use thiserror::Error;
use uuid::Uuid;

/// Errors that can occur during audit entry validation
#[derive(Debug, Error)]
pub enum ValidationError {
    /// Returned when the entity type string is empty or contains invalid characters
    ///
    /// This error occurs when creating an audit entry with an invalid entity type.
    /// Valid entity types include: "appointment", "patient", "prescription", "user", etc.
    #[error("Invalid entity type: {0}")]
    InvalidEntityType(String),

    /// Returned when the entity ID is a nil UUID or otherwise invalid
    ///
    /// This error occurs when the provided entity UUID is nil or fails validation.
    #[error("Invalid entity ID")]
    InvalidEntityId,

    /// Returned when attempting to create an audit entry without a user ID
    ///
    /// Every audit entry must have a valid `changed_by` user ID to maintain
    /// accountability as required by healthcare compliance regulations.
    #[error("Changed by user ID is required")]
    MissingChangedBy,

    /// Returned when an audit entry timestamp is set in the future
    ///
    /// This error ensures audit timestamps are always accurate and prevents
    /// manipulation of audit records.
    #[error("Timestamp cannot be in the future")]
    FutureTimestamp,

    /// Returned when the end time is not after the start time in a time range query
    ///
    /// This error occurs when querying audit entries with an invalid time range,
    /// such as when start_time >= end_time.
    #[error("Invalid time range: end time must be after start time")]
    InvalidTimeRange,
}

/// Errors that can occur in the audit service layer
#[derive(Debug, Error)]
pub enum ServiceError {
    /// Returned when a specific audit entry cannot be found
    ///
    /// This error occurs when attempting to retrieve a specific audit entry by ID
    /// that does not exist in the database.
    #[error("Audit entry not found: {0}")]
    NotFound(Uuid),

    /// Returned when audit entry validation fails
    ///
    /// This error wraps [`ValidationError`] and occurs when creating or updating
    /// an audit entry with invalid data.
    #[error("Validation error: {0}")]
    ValidationError(#[from] ValidationError),

    /// Returned when a database operation fails
    ///
    /// This error wraps [`RepositoryError`] and occurs when the underlying
    /// database operation fails (connection error, query error, etc.).
    #[error("Repository error: {0}")]
    Repository(#[from] RepositoryError),
}

/// Errors that can occur in the audit repository layer
#[derive(Debug, Error)]
pub enum RepositoryError {
    /// Returned when a database operation fails
    ///
    /// This wraps SQLx database errors. Common causes include:
    /// - Connection failures
    /// - Query syntax errors
    /// - Constraint violations
    /// - Transaction rollbacks
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    /// Returned when an audit entry is not found
    ///
    /// This error occurs when a query for a specific audit entry returns no results.
    #[error("Not found")]
    NotFound,

    /// Returned when a database constraint is violated
    ///
    /// This error occurs when an operation violates a database constraint such as:
    /// - Foreign key constraints
    /// - Unique constraints
    /// - Check constraints
    #[error("Constraint violation: {0}")]
    ConstraintViolation(String),

    /// Returned when attempting to modify or delete an audit entry
    ///
    /// **This error indicates a compliance violation.**
    ///
    /// Audit entries are immutable (append-only by design). This error occurs
    /// when code attempts to update or delete an existing audit entry, which
    /// would compromise the audit trail's integrity.
    ///
    /// # Compliance Note
    /// Never attempt to modify or delete audit entries. If you need to correct
    /// an audit entry, create a new entry documenting the correction.
    #[error("Audit entry cannot be modified or deleted (append-only)")]
    ImmutableViolation,
}

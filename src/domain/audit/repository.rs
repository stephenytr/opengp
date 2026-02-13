use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::error::RepositoryError;
use super::model::AuditEntry;

/// Repository trait for audit entry persistence
///
/// Defines the interface for storing and retrieving audit entries from the database.
/// Implementations must handle all database operations and return appropriate errors.
///
/// # Compliance Note
/// All audit entries are immutable once created (append-only log).
/// Never implement update or delete operations for audit entries.
///
/// # Example
/// ```
/// use async_trait::async_trait;
/// use chrono::Utc;
/// use uuid::Uuid;
/// use opengp::domain::audit::{AuditEntry, AuditRepository};
/// use opengp::domain::audit::RepositoryError;
///
/// struct MyAuditRepository { /* ... */ }
///
/// #[async_trait]
/// impl AuditRepository for MyAuditRepository {
///     async fn create(&self, entry: AuditEntry) -> Result<AuditEntry, RepositoryError> {
///         Ok(entry)
///     }
///     async fn find_by_entity(&self, _entity_type: &str, _entity_id: Uuid) -> Result<Vec<AuditEntry>, RepositoryError> {
///         Ok(vec![])
///     }
///     async fn find_by_user(&self, _user_id: Uuid) -> Result<Vec<AuditEntry>, RepositoryError> {
///         Ok(vec![])
///     }
///     async fn find_by_time_range(&self, _start_time: chrono::DateTime<Utc>, _end_time: chrono::DateTime<Utc>) -> Result<Vec<AuditEntry>, RepositoryError> {
///         Ok(vec![])
///     }
///     async fn find_by_entity_and_time_range(&self, _entity_type: &str, _entity_id: Uuid, _start_time: chrono::DateTime<Utc>, _end_time: chrono::DateTime<Utc>) -> Result<Vec<AuditEntry>, RepositoryError> {
///         Ok(vec![])
///     }
/// }
/// ```
#[async_trait]
pub trait AuditRepository: Send + Sync {
    /// Create a new audit entry
    ///
    /// # Arguments
    /// * `entry` - The audit entry to create
    ///
    /// # Returns
    /// * `Ok(entry)` - Successfully created audit entry
    /// * `Err(RepositoryError)` - Database error or constraint violation
    async fn create(&self, entry: AuditEntry) -> Result<AuditEntry, RepositoryError>;

    /// Find all audit entries for a specific entity
    ///
    /// Returns entries in chronological order (oldest first).
    ///
    /// # Arguments
    /// * `entity_type` - Type of entity (e.g., "appointment", "patient")
    /// * `entity_id` - ID of the entity
    ///
    /// # Returns
    /// * `Ok(entries)` - List of audit entries for the entity
    /// * `Err(RepositoryError)` - Database error
    async fn find_by_entity(
        &self,
        entity_type: &str,
        entity_id: Uuid,
    ) -> Result<Vec<AuditEntry>, RepositoryError>;

    /// Find all audit entries created by a specific user
    ///
    /// Returns entries in reverse chronological order (newest first).
    ///
    /// # Arguments
    /// * `user_id` - ID of the user who performed the actions
    ///
    /// # Returns
    /// * `Ok(entries)` - List of audit entries by the user
    /// * `Err(RepositoryError)` - Database error
    async fn find_by_user(&self, user_id: Uuid) -> Result<Vec<AuditEntry>, RepositoryError>;

    /// Find audit entries within a time range
    ///
    /// Returns entries in chronological order (oldest first).
    ///
    /// # Arguments
    /// * `start_time` - Start of time range (inclusive)
    /// * `end_time` - End of time range (inclusive)
    ///
    /// # Returns
    /// * `Ok(entries)` - List of audit entries in the time range
    /// * `Err(RepositoryError)` - Database error
    async fn find_by_time_range(
        &self,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<Vec<AuditEntry>, RepositoryError>;

    /// Find audit entries for a specific entity within a time range
    ///
    /// Combines entity filtering and time range filtering.
    /// Returns entries in chronological order (oldest first).
    ///
    /// # Arguments
    /// * `entity_type` - Type of entity (e.g., "appointment", "patient")
    /// * `entity_id` - ID of the entity
    /// * `start_time` - Start of time range (inclusive)
    /// * `end_time` - End of time range (inclusive)
    ///
    /// # Returns
    /// * `Ok(entries)` - List of audit entries matching criteria
    /// * `Err(RepositoryError)` - Database error
    async fn find_by_entity_and_time_range(
        &self,
        entity_type: &str,
        entity_id: Uuid,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<Vec<AuditEntry>, RepositoryError>;
}

use async_trait::async_trait;
use uuid::Uuid;

use super::error::RepositoryError;
use chrono::{DateTime, Utc};

use super::model::{Practitioner, Role, Session, User, WorkingHours};

/// Repository trait for practitioner persistence
#[async_trait]
pub trait PractitionerRepository: Send + Sync {
    /// List all active practitioners
    ///
    /// # Returns
    /// * `Ok(Vec<Practitioner>)` - List of active practitioners
    /// * `Err(RepositoryError)` - Database error
    async fn list_active(&self) -> Result<Vec<Practitioner>, RepositoryError>;

    /// Find practitioner by ID
    ///
    /// # Returns
    /// * `Ok(Some(Practitioner))` - Practitioner found
    /// * `Ok(None)` - Practitioner not found
    /// * `Err(RepositoryError)` - Database error
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Practitioner>, RepositoryError>;
}

/// Repository trait for user persistence
///
/// Provides CRUD operations for user management, including username lookup,
/// role-based filtering, and soft delete functionality.
#[async_trait]
pub trait UserRepository: Send + Sync {
    /// Find user by ID
    ///
    /// # Arguments
    /// * `id` - User UUID
    ///
    /// # Returns
    /// * `Ok(Some(User))` - User found
    /// * `Ok(None)` - User not found
    /// * `Err(RepositoryError)` - Database error
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, RepositoryError>;

    /// Find user by username
    ///
    /// # Arguments
    /// * `username` - Username to search for
    ///
    /// # Returns
    /// * `Ok(Some(User))` - User found
    /// * `Ok(None)` - User not found
    /// * `Err(RepositoryError)` - Database error
    async fn find_by_username(&self, username: &str) -> Result<Option<User>, RepositoryError>;

    /// List all users
    ///
    /// Returns all users in the system, including inactive users.
    ///
    /// # Returns
    /// * `Ok(Vec<User>)` - List of all users
    /// * `Err(RepositoryError)` - Database error
    async fn find_all(&self) -> Result<Vec<User>, RepositoryError>;

    /// Find users by role
    ///
    /// # Arguments
    /// * `role` - Role to filter by
    ///
    /// # Returns
    /// * `Ok(Vec<User>)` - List of users with the specified role
    /// * `Err(RepositoryError)` - Database error
    async fn find_by_role(&self, role: Role) -> Result<Vec<User>, RepositoryError>;

    /// Create a new user
    ///
    /// # Arguments
    /// * `user` - User to create
    ///
    /// # Returns
    /// * `Ok(User)` - Created user with generated ID and timestamps
    /// * `Err(RepositoryError)` - Database error or constraint violation
    async fn create(&self, user: User) -> Result<User, RepositoryError>;

    /// Update an existing user
    ///
    /// Updates all fields of the user and sets updated_at to current timestamp.
    ///
    /// # Arguments
    /// * `user` - User with updated fields
    ///
    /// # Returns
    /// * `Ok(User)` - Updated user
    /// * `Err(RepositoryError::NotFound)` - User not found
    /// * `Err(RepositoryError)` - Database error
    async fn update(&self, user: User) -> Result<User, RepositoryError>;

    /// Delete a user (soft delete)
    ///
    /// Sets is_active = false instead of removing the record.
    /// This preserves audit trails and referential integrity.
    ///
    /// # Arguments
    /// * `id` - User UUID to delete
    ///
    /// # Returns
    /// * `Ok(())` - User successfully deactivated
    /// * `Err(RepositoryError::NotFound)` - User not found
    /// * `Err(RepositoryError)` - Database error
    async fn delete(&self, id: Uuid) -> Result<(), RepositoryError>;
}

#[async_trait]
pub trait SessionRepository: Send + Sync {
    async fn create(&self, session: Session) -> Result<Session, RepositoryError>;

    async fn find_by_token(&self, token: &str) -> Result<Option<Session>, RepositoryError>;

    async fn delete_by_token(&self, token: &str) -> Result<(), RepositoryError>;

    async fn cleanup_expired(&self, now: DateTime<Utc>) -> Result<u64, RepositoryError>;
}

/// Repository trait for working hours persistence
///
/// Provides CRUD operations for managing practitioner working hours schedules.
/// Working hours define when practitioners are available on specific days of the week.
#[async_trait]
pub trait WorkingHoursRepository: Send + Sync {
    /// Find all working hours entries for a practitioner
    ///
    /// # Arguments
    /// * `practitioner_id` - The practitioner UUID
    ///
    /// # Returns
    /// * `Ok(Vec<WorkingHours>)` - List of working hours entries (may be empty)
    /// * `Err(RepositoryError)` - Database error
    async fn find_by_practitioner(
        &self,
        practitioner_id: Uuid,
    ) -> Result<Vec<WorkingHours>, RepositoryError>;

    /// Find working hours for a practitioner on a specific day of the week
    ///
    /// # Arguments
    /// * `practitioner_id` - The practitioner UUID
    /// * `day_of_week` - Day of week (0 = Monday, 6 = Sunday)
    ///
    /// # Returns
    /// * `Ok(Some(WorkingHours))` - Working hours found for that day
    /// * `Ok(None)` - No working hours defined for that day
    /// * `Err(RepositoryError)` - Database error
    async fn find_for_day(
        &self,
        practitioner_id: Uuid,
        day_of_week: u8,
    ) -> Result<Option<WorkingHours>, RepositoryError>;

    /// Create a new working hours entry
    ///
    /// # Arguments
    /// * `working_hours` - The working hours to create
    ///
    /// # Returns
    /// * `Ok(WorkingHours)` - Successfully created working hours
    /// * `Err(RepositoryError)` - Database error or constraint violation
    async fn save(&self, working_hours: WorkingHours) -> Result<WorkingHours, RepositoryError>;

    /// Delete a working hours entry
    ///
    /// # Arguments
    /// * `id` - The working hours ID to delete
    ///
    /// # Returns
    /// * `Ok(())` - Successfully deleted
    /// * `Err(RepositoryError::NotFound)` - Working hours not found
    /// * `Err(RepositoryError)` - Database error
    async fn delete(&self, id: Uuid) -> Result<(), RepositoryError>;
}

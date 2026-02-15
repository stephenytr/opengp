use async_trait::async_trait;
use uuid::Uuid;

pub use crate::domain::error::RepositoryError;
use super::error::UserRepositoryError;
use super::model::{Practitioner, Role, User};

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
    /// * `Err(UserRepositoryError)` - Database error
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, UserRepositoryError>;

    /// Find user by username
    ///
    /// # Arguments
    /// * `username` - Username to search for
    ///
    /// # Returns
    /// * `Ok(Some(User))` - User found
    /// * `Ok(None)` - User not found
    /// * `Err(UserRepositoryError)` - Database error
    async fn find_by_username(&self, username: &str) -> Result<Option<User>, UserRepositoryError>;

    /// List all users
    ///
    /// Returns all users in the system, including inactive users.
    ///
    /// # Returns
    /// * `Ok(Vec<User>)` - List of all users
    /// * `Err(UserRepositoryError)` - Database error
    async fn find_all(&self) -> Result<Vec<User>, UserRepositoryError>;

    /// Find users by role
    ///
    /// # Arguments
    /// * `role` - Role to filter by
    ///
    /// # Returns
    /// * `Ok(Vec<User>)` - List of users with the specified role
    /// * `Err(UserRepositoryError)` - Database error
    async fn find_by_role(&self, role: Role) -> Result<Vec<User>, UserRepositoryError>;

    /// Create a new user
    ///
    /// # Arguments
    /// * `user` - User to create
    ///
    /// # Returns
    /// * `Ok(User)` - Created user with generated ID and timestamps
    /// * `Err(UserRepositoryError)` - Database error or constraint violation
    async fn create(&self, user: User) -> Result<User, UserRepositoryError>;

    /// Update an existing user
    ///
    /// Updates all fields of the user and sets updated_at to current timestamp.
    ///
    /// # Arguments
    /// * `user` - User with updated fields
    ///
    /// # Returns
    /// * `Ok(User)` - Updated user
    /// * `Err(UserRepositoryError::NotFound)` - User not found
    /// * `Err(UserRepositoryError)` - Database error
    async fn update(&self, user: User) -> Result<User, UserRepositoryError>;

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
    /// * `Err(UserRepositoryError::NotFound)` - User not found
    /// * `Err(UserRepositoryError)` - Database error
    async fn delete(&self, id: Uuid) -> Result<(), UserRepositoryError>;
}

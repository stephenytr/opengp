use std::sync::Arc;
use tracing::{error, info, instrument, warn};
use uuid::Uuid;

use crate::service;

use super::dto::NewUserData;
use super::error::UserError;
use super::model::{Practitioner, Role, User};
use super::repository::{PractitionerRepository, UserRepository};

#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("Repository error: {0}")]
    Repository(String),

    #[error("Practitioner not found: {0}")]
    NotFound(Uuid),
}

service! {
    PractitionerService {
        repository: Arc<dyn PractitionerRepository>,
    }
}

/// Service layer for practitioner business logic
impl PractitionerService {
    /// Get all active practitioners
    ///
    /// # Returns
    /// * `Ok(Vec<Practitioner>)` - List of active practitioners
    /// * `Err(ServiceError)` - Database error
    pub async fn get_active_practitioners(&self) -> Result<Vec<Practitioner>, ServiceError> {
        info!("Fetching active practitioners");

        match self.repository.list_active().await {
            Ok(practitioners) => {
                info!("Found {} active practitioners", practitioners.len());
                Ok(practitioners)
            }
            Err(e) => {
                error!("Failed to fetch practitioners: {}", e);
                Err(ServiceError::Repository(e.to_string()))
            }
        }
    }
}

service! {
    UserService {
        repository: Arc<dyn UserRepository>,
    }
}

impl UserService {

    /// Create a new user
    ///
    /// Validates user data and checks for duplicate usernames before creation.
    ///
    /// # Arguments
    /// * `data` - User creation data
    ///
    /// # Returns
    /// * `Ok(User)` - Successfully created user
    /// * `Err(UserError::Duplicate)` - Username already exists
    /// * `Err(UserError::Validation)` - Invalid user data
    /// * `Err(UserError::Repository)` - Database error
    #[instrument(skip(self))]
    pub async fn create_user(&self, data: NewUserData) -> Result<User, UserError> {
        info!("Creating new user: {}", data.username);

        // Check for duplicate username
        if self
            .repository
            .find_by_username(&data.username)
            .await?
            .is_some()
        {
            warn!("Duplicate username attempted: {}", data.username);
            return Err(UserError::Duplicate(format!(
                "Username '{}' already exists",
                data.username
            )));
        }

        // Create user with domain validation
        info!("Creating user domain model with role: {:?}", data.role);
        let user = User::new(data)?;

        // Save to database
        info!("Saving user to database with ID: {}", user.id);
        match self.repository.create(user).await {
            Ok(saved) => {
                info!(
                    "User created successfully: {} ({})",
                    saved.username, saved.id
                );
                Ok(saved)
            }
            Err(e) => {
                error!("Failed to create user: {}", e);
                Err(e.into())
            }
        }
    }

    /// Update an existing user
    ///
    /// # Arguments
    /// * `user` - User with updated fields
    ///
    /// # Returns
    /// * `Ok(User)` - Successfully updated user
    /// * `Err(UserError::NotFound)` - User not found
    /// * `Err(UserError::Repository)` - Database error
    #[instrument(skip(self), fields(user_id = %user.id))]
    pub async fn update_user(&self, user: User) -> Result<User, UserError> {
        info!("Updating user: {} ({})", user.username, user.id);

        match self.repository.update(user).await {
            Ok(updated) => {
                info!("User updated successfully: {}", updated.id);
                Ok(updated)
            }
            Err(e) => {
                error!("Failed to update user: {}", e);
                Err(e.into())
            }
        }
    }

    /// Get user by ID
    ///
    /// # Arguments
    /// * `id` - User UUID
    ///
    /// # Returns
    /// * `Ok(User)` - User found
    /// * `Err(UserError::NotFound)` - User not found
    /// * `Err(UserError::Repository)` - Database error
    #[instrument(skip(self))]
    pub async fn get_user_by_id(&self, id: Uuid) -> Result<User, UserError> {
        info!("Fetching user by ID: {}", id);

        match self.repository.find_by_id(id).await? {
            Some(user) => {
                info!("User found: {} ({})", user.username, user.id);
                Ok(user)
            }
            None => {
                warn!("User not found: {}", id);
                Err(UserError::NotFound(format!("User not found: {}", id)))
            }
        }
    }

    /// Get user by username
    ///
    /// # Arguments
    /// * `username` - Username to search for
    ///
    /// # Returns
    /// * `Ok(User)` - User found
    /// * `Err(UserError::NotFound)` - User not found
    /// * `Err(UserError::Repository)` - Database error
    #[instrument(skip(self))]
    pub async fn get_user_by_username(&self, username: &str) -> Result<User, UserError> {
        info!("Fetching user by username: {}", username);

        match self.repository.find_by_username(username).await? {
            Some(user) => {
                info!("User found: {} ({})", user.username, user.id);
                Ok(user)
            }
            None => {
                warn!("User not found with username: {}", username);
                Err(UserError::NotFound(format!(
                    "User not found with username: {}",
                    username
                )))
            }
        }
    }

    /// Get all users
    ///
    /// Returns all users in the system, including inactive users.
    ///
    /// # Returns
    /// * `Ok(Vec<User>)` - List of all users
    /// * `Err(UserError::Repository)` - Database error
    #[instrument(skip(self))]
    pub async fn get_all_users(&self) -> Result<Vec<User>, UserError> {
        info!("Fetching all users");

        match self.repository.find_all().await {
            Ok(users) => {
                info!("Found {} users", users.len());
                Ok(users)
            }
            Err(e) => {
                error!("Failed to fetch users: {}", e);
                Err(e.into())
            }
        }
    }

    /// Get users by role
    ///
    /// # Arguments
    /// * `role` - Role to filter by
    ///
    /// # Returns
    /// * `Ok(Vec<User>)` - List of users with the specified role
    /// * `Err(UserError::Repository)` - Database error
    #[instrument(skip(self))]
    pub async fn get_users_by_role(&self, role: Role) -> Result<Vec<User>, UserError> {
        info!("Fetching users with role: {:?}", role);

        match self.repository.find_by_role(role).await {
            Ok(users) => {
                info!("Found {} users with role: {:?}", users.len(), role);
                Ok(users)
            }
            Err(e) => {
                error!("Failed to fetch users by role: {}", e);
                Err(e.into())
            }
        }
    }

    /// Deactivate a user (soft delete)
    ///
    /// Sets is_active = false instead of removing the record.
    ///
    /// # Arguments
    /// * `id` - User UUID to deactivate
    ///
    /// # Returns
    /// * `Ok(())` - User successfully deactivated
    /// * `Err(UserError::NotFound)` - User not found
    /// * `Err(UserError::Repository)` - Database error
    #[instrument(skip(self))]
    pub async fn deactivate_user(&self, id: Uuid) -> Result<(), UserError> {
        info!("Deactivating user: {}", id);

        match self.repository.delete(id).await {
            Ok(()) => {
                info!("User deactivated successfully: {}", id);
                Ok(())
            }
            Err(e) => {
                error!("Failed to deactivate user: {}", e);
                Err(e.into())
            }
        }
    }
}

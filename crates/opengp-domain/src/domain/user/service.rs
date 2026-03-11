use std::sync::Arc;

use chrono::{Duration, Utc};
use rand::{rngs::OsRng, RngCore};
use tracing::{error, info, instrument, warn};
use uuid::Uuid;

use crate::service;

use super::dto::{LoginRequest, LoginResponse, NewUserData};
use super::error::{AuthError, ServiceError};
use super::model::{Practitioner, Role, Session, User};
use super::password::PasswordHasher;
use super::repository::{PractitionerRepository, SessionRepository, UserRepository};

const MAX_FAILED_LOGIN_ATTEMPTS: u8 = 5;
const SESSION_DURATION_HOURS: i64 = 8;

#[derive(Debug, thiserror::Error)]
pub enum PractitionerServiceError {
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
    /// * `Err(PractitionerServiceError)` - Database error
    pub async fn get_active_practitioners(
        &self,
    ) -> Result<Vec<Practitioner>, PractitionerServiceError> {
        info!("Fetching active practitioners");

        match self.repository.list_active().await {
            Ok(practitioners) => {
                info!("Found {} active practitioners", practitioners.len());
                Ok(practitioners)
            }
            Err(e) => {
                error!("Failed to fetch practitioners: {}", e);
                Err(PractitionerServiceError::Repository(e.to_string()))
            }
        }
    }
}

service! {
    UserService {
        repository: Arc<dyn UserRepository>,
    }
}

pub struct AuthService {
    pub user_repository: Arc<dyn UserRepository>,
    pub password_hasher: Arc<dyn PasswordHasher>,
    pub session_repository: Arc<dyn SessionRepository>,
    session_duration: Duration,
}

impl AuthService {
    pub fn new(
        user_repository: Arc<dyn UserRepository>,
        password_hasher: Arc<dyn PasswordHasher>,
        session_repository: Arc<dyn SessionRepository>,
    ) -> Self {
        Self {
            user_repository,
            password_hasher,
            session_repository,
            session_duration: Duration::hours(SESSION_DURATION_HOURS),
        }
    }

    pub async fn login(&self, request: LoginRequest) -> Result<LoginResponse, AuthError> {
        let mut user = self
            .user_repository
            .find_by_username(&request.username)
            .await?
            .ok_or(AuthError::InvalidCredentials)?;

        if user.is_locked || user.failed_login_attempts >= MAX_FAILED_LOGIN_ATTEMPTS {
            if !user.is_locked {
                user.is_locked = true;
                user.updated_at = Utc::now();
                self.user_repository.update(user).await?;
            }

            return Err(AuthError::AccountLocked);
        }

        let Some(password_hash) = user.password_hash.as_deref() else {
            return Err(AuthError::InvalidCredentials);
        };

        if self
            .password_hasher
            .verify_password(password_hash, &request.password)
            .is_err()
        {
            user.failed_login_attempts = user.failed_login_attempts.saturating_add(1);

            if user.failed_login_attempts >= MAX_FAILED_LOGIN_ATTEMPTS {
                user.is_locked = true;
            }

            user.updated_at = Utc::now();
            self.user_repository.update(user).await?;

            return Err(AuthError::InvalidCredentials);
        }

        user.failed_login_attempts = 0;
        user.is_locked = false;
        let now = Utc::now();
        user.last_login = Some(now);
        user.updated_at = now;
        let user = self.user_repository.update(user).await?;

        let session_token = Self::generate_session_token();
        let session = Session::new(user.id, session_token.clone(), self.session_duration);
        self.session_repository.create(session).await?;

        Ok(LoginResponse {
            user_id: user.id,
            session_token,
        })
    }

    pub async fn validate_session(&self, session_token: &str) -> Result<Uuid, AuthError> {
        let now = Utc::now();
        self.session_repository.cleanup_expired(now).await?;

        let session = self
            .session_repository
            .find_by_token(session_token)
            .await?
            .ok_or(AuthError::SessionExpired)?;

        if session.is_expired_at(now) {
            self.session_repository
                .delete_by_token(session_token)
                .await?;
            return Err(AuthError::SessionExpired);
        }

        Ok(session.user_id)
    }

    pub async fn logout(&self, session_token: &str) -> Result<(), AuthError> {
        match self.session_repository.delete_by_token(session_token).await {
            Ok(()) => Ok(()),
            Err(crate::domain::error::RepositoryError::NotFound) => Err(AuthError::SessionExpired),
            Err(err) => Err(AuthError::Repository(err)),
        }
    }

    pub async fn refresh_session(&self, session_token: &str) -> Result<Session, AuthError> {
        let now = Utc::now();
        self.session_repository.cleanup_expired(now).await?;

        let existing = self
            .session_repository
            .find_by_token(session_token)
            .await?
            .ok_or(AuthError::SessionExpired)?;

        if existing.is_expired_at(now) {
            self.session_repository
                .delete_by_token(session_token)
                .await?;
            return Err(AuthError::SessionExpired);
        }

        self.session_repository
            .delete_by_token(session_token)
            .await?;

        let refreshed = Session::new(existing.user_id, existing.token, self.session_duration);
        self.session_repository.create(refreshed.clone()).await?;
        Ok(refreshed)
    }

    pub fn session_ttl_seconds(&self) -> i64 {
        self.session_duration.num_seconds()
    }

    pub async fn cleanup_expired_sessions(&self) -> Result<u64, AuthError> {
        match self.session_repository.cleanup_expired(Utc::now()).await {
            Ok(removed) => Ok(removed),
            Err(err) => Err(AuthError::Repository(err)),
        }
    }

    fn generate_session_token() -> String {
        let mut bytes = [0u8; 32];
        OsRng.fill_bytes(&mut bytes);
        hex::encode(bytes)
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
    /// * `Err(ServiceError::Duplicate)` - Username already exists
    /// * `Err(ServiceError::Validation)` - Invalid user data
    /// * `Err(ServiceError::Repository)` - Database error
    #[instrument(skip(self))]
    pub async fn create_user(&self, data: NewUserData) -> Result<User, ServiceError> {
        info!("Creating new user: {}", data.username);

        // Check for duplicate username
        if self
            .repository
            .find_by_username(&data.username)
            .await?
            .is_some()
        {
            warn!("Duplicate username attempted: {}", data.username);
            return Err(ServiceError::Duplicate(format!(
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
    /// * `Err(ServiceError::NotFoundByUsername)` - User not found
    /// * `Err(ServiceError::Repository)` - Database error
    #[instrument(skip(self), fields(user_id = %user.id))]
    pub async fn update_user(&self, user: User) -> Result<User, ServiceError> {
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
    /// * `Err(ServiceError::NotFound)` - User not found
    /// * `Err(ServiceError::Repository)` - Database error
    #[instrument(skip(self))]
    pub async fn get_user_by_id(&self, id: Uuid) -> Result<User, ServiceError> {
        info!("Fetching user by ID: {}", id);

        match self.repository.find_by_id(id).await? {
            Some(user) => {
                info!("User found: {} ({})", user.username, user.id);
                Ok(user)
            }
            None => {
                warn!("User not found: {}", id);
                Err(ServiceError::NotFound(id))
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
    /// * `Err(ServiceError::NotFound)` - User not found
    /// * `Err(ServiceError::Repository)` - Database error
    #[instrument(skip(self))]
    pub async fn get_user_by_username(&self, username: &str) -> Result<User, ServiceError> {
        info!("Fetching user by username: {}", username);

        match self.repository.find_by_username(username).await? {
            Some(user) => {
                info!("User found: {} ({})", user.username, user.id);
                Ok(user)
            }
            None => {
                warn!("User not found with username: {}", username);
                Err(ServiceError::NotFoundByUsername(username.to_string()))
            }
        }
    }

    /// Get all users
    ///
    /// Returns all users in the system, including inactive users.
    ///
    /// # Returns
    /// * `Ok(Vec<User>)` - List of all users
    /// * `Err(ServiceError::Repository)` - Database error
    #[instrument(skip(self))]
    pub async fn get_all_users(&self) -> Result<Vec<User>, ServiceError> {
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
    /// * `Err(ServiceError::Repository)` - Database error
    #[instrument(skip(self))]
    pub async fn get_users_by_role(&self, role: Role) -> Result<Vec<User>, ServiceError> {
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
    /// * `Err(ServiceError::NotFound)` - User not found
    /// * `Err(ServiceError::Repository)` - Database error
    #[instrument(skip(self))]
    pub async fn deactivate_user(&self, id: Uuid) -> Result<(), ServiceError> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::sync::Mutex;

    use crate::domain::error::RepositoryError;

    struct MockPasswordHasher {
        valid_hash: String,
    }

    impl PasswordHasher for MockPasswordHasher {
        fn hash_password(&self, _password: &str) -> Result<String, super::super::PasswordError> {
            unimplemented!("not needed in auth service tests")
        }

        fn verify_password(
            &self,
            password_hash: &str,
            password: &str,
        ) -> Result<(), super::super::PasswordError> {
            if password_hash == self.valid_hash && password == "correct-password" {
                Ok(())
            } else {
                Err(super::super::PasswordError::VerificationFailed)
            }
        }
    }

    struct MockUserRepository {
        user: Mutex<Option<User>>,
    }

    struct MockSessionRepository {
        sessions: Mutex<Vec<Session>>,
    }

    #[async_trait]
    impl SessionRepository for MockSessionRepository {
        async fn create(&self, session: Session) -> Result<Session, RepositoryError> {
            self.sessions
                .lock()
                .expect("session lock poisoned")
                .push(session.clone());
            Ok(session)
        }

        async fn find_by_token(&self, token: &str) -> Result<Option<Session>, RepositoryError> {
            Ok(self
                .sessions
                .lock()
                .expect("session lock poisoned")
                .iter()
                .find(|s| s.token == token)
                .cloned())
        }

        async fn delete_by_token(&self, token: &str) -> Result<(), RepositoryError> {
            let mut sessions = self.sessions.lock().expect("session lock poisoned");
            let before = sessions.len();
            sessions.retain(|s| s.token != token);

            if sessions.len() == before {
                Err(RepositoryError::NotFound)
            } else {
                Ok(())
            }
        }

        async fn cleanup_expired(
            &self,
            now: chrono::DateTime<Utc>,
        ) -> Result<u64, RepositoryError> {
            let mut sessions = self.sessions.lock().expect("session lock poisoned");
            let before = sessions.len();
            sessions.retain(|s| !s.is_expired_at(now));
            Ok((before - sessions.len()) as u64)
        }
    }

    #[async_trait]
    impl UserRepository for MockUserRepository {
        async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, RepositoryError> {
            Ok(self
                .user
                .lock()
                .expect("user lock poisoned")
                .as_ref()
                .filter(|u| u.id == id)
                .cloned())
        }

        async fn find_by_username(&self, username: &str) -> Result<Option<User>, RepositoryError> {
            Ok(self
                .user
                .lock()
                .expect("user lock poisoned")
                .as_ref()
                .filter(|u| u.username == username)
                .cloned())
        }

        async fn find_all(&self) -> Result<Vec<User>, RepositoryError> {
            Ok(self
                .user
                .lock()
                .expect("user lock poisoned")
                .as_ref()
                .cloned()
                .into_iter()
                .collect())
        }

        async fn find_by_role(&self, role: Role) -> Result<Vec<User>, RepositoryError> {
            Ok(self
                .user
                .lock()
                .expect("user lock poisoned")
                .as_ref()
                .filter(|u| u.role == role)
                .cloned()
                .into_iter()
                .collect())
        }

        async fn create(&self, user: User) -> Result<User, RepositoryError> {
            let mut lock = self.user.lock().expect("user lock poisoned");
            *lock = Some(user.clone());
            Ok(user)
        }

        async fn update(&self, user: User) -> Result<User, RepositoryError> {
            let mut lock = self.user.lock().expect("user lock poisoned");
            *lock = Some(user.clone());
            Ok(user)
        }

        async fn delete(&self, _id: Uuid) -> Result<(), RepositoryError> {
            Ok(())
        }
    }

    fn test_user() -> User {
        let now = Utc::now();

        User {
            id: Uuid::new_v4(),
            username: "doctor1".to_string(),
            password_hash: Some("valid-hash".to_string()),
            email: Some("doctor@example.com".to_string()),
            first_name: "Test".to_string(),
            last_name: "Doctor".to_string(),
            role: Role::Doctor,
            additional_permissions: vec![],
            is_active: true,
            is_locked: false,
            failed_login_attempts: 0,
            last_login: None,
            password_changed_at: now,
            created_at: now,
            updated_at: now,
        }
    }

    fn new_auth_service(user: User) -> AuthService {
        AuthService::new(
            Arc::new(MockUserRepository {
                user: Mutex::new(Some(user)),
            }),
            Arc::new(MockPasswordHasher {
                valid_hash: "valid-hash".to_string(),
            }),
            Arc::new(MockSessionRepository {
                sessions: Mutex::new(Vec::new()),
            }),
        )
    }

    #[tokio::test]
    async fn login_with_valid_credentials_returns_user_id_and_session_token() {
        let user = test_user();
        let expected_user_id = user.id;
        let service = new_auth_service(user);

        let response = service
            .login(LoginRequest {
                username: "doctor1".to_string(),
                password: "correct-password".to_string(),
            })
            .await
            .expect("login should succeed");

        assert_eq!(response.user_id, expected_user_id);
        assert_eq!(response.session_token.len(), 64);
        let validated_user_id = service
            .validate_session(&response.session_token)
            .await
            .expect("session should be valid");
        assert_eq!(validated_user_id, expected_user_id);
    }

    #[tokio::test]
    async fn login_generates_unique_session_tokens() {
        let service = new_auth_service(test_user());

        let first = service
            .login(LoginRequest {
                username: "doctor1".to_string(),
                password: "correct-password".to_string(),
            })
            .await
            .expect("first login should succeed");

        let second = service
            .login(LoginRequest {
                username: "doctor1".to_string(),
                password: "correct-password".to_string(),
            })
            .await
            .expect("second login should succeed");

        assert_ne!(first.session_token, second.session_token);
    }

    #[tokio::test]
    async fn login_with_invalid_credentials_returns_invalid_credentials() {
        let service = new_auth_service(test_user());

        let result = service
            .login(LoginRequest {
                username: "doctor1".to_string(),
                password: "wrong-password".to_string(),
            })
            .await;

        assert!(matches!(result, Err(AuthError::InvalidCredentials)));
    }

    #[tokio::test]
    async fn logout_invalidates_session() {
        let service = new_auth_service(test_user());

        let login = service
            .login(LoginRequest {
                username: "doctor1".to_string(),
                password: "correct-password".to_string(),
            })
            .await
            .expect("login should succeed");

        assert!(service.logout(&login.session_token).await.is_ok());
        assert!(matches!(
            service.logout(&login.session_token).await,
            Err(AuthError::SessionExpired)
        ));
    }

    #[tokio::test]
    async fn validate_session_rejects_expired_session() {
        let service = new_auth_service(test_user());
        let expired = Session {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            created_at: Utc::now() - Duration::hours(2),
            expires_at: Utc::now() - Duration::hours(1),
            token: "expired-token".to_string(),
        };

        service
            .session_repository
            .create(expired)
            .await
            .expect("session insert should succeed");

        let result = service.validate_session("expired-token").await;
        assert!(matches!(result, Err(AuthError::SessionExpired)));
    }

    #[tokio::test]
    async fn cleanup_expired_sessions_removes_only_expired() {
        let service = new_auth_service(test_user());
        let now = Utc::now();

        service
            .session_repository
            .create(Session {
                id: Uuid::new_v4(),
                user_id: Uuid::new_v4(),
                created_at: now - Duration::hours(2),
                expires_at: now - Duration::minutes(1),
                token: "expired-token".to_string(),
            })
            .await
            .expect("expired session insert should succeed");

        service
            .session_repository
            .create(Session {
                id: Uuid::new_v4(),
                user_id: Uuid::new_v4(),
                created_at: now,
                expires_at: now + Duration::hours(1),
                token: "active-token".to_string(),
            })
            .await
            .expect("active session insert should succeed");

        let removed = service
            .cleanup_expired_sessions()
            .await
            .expect("cleanup should succeed");
        assert_eq!(removed, 1);

        let remaining = service
            .validate_session("active-token")
            .await
            .expect("active session should remain valid");
        assert!(!remaining.is_nil());
    }

    #[tokio::test]
    async fn locked_account_cannot_login() {
        let mut user = test_user();
        user.is_locked = true;
        user.failed_login_attempts = MAX_FAILED_LOGIN_ATTEMPTS;
        let service = new_auth_service(user);

        let result = service
            .login(LoginRequest {
                username: "doctor1".to_string(),
                password: "correct-password".to_string(),
            })
            .await;

        assert!(matches!(result, Err(AuthError::AccountLocked)));
    }
}

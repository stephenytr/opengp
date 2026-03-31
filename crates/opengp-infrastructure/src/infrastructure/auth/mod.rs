use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// In-memory representation of an authenticated user session
///
/// Tracks expiry and activity metadata for a session token issued
/// by the authentication layer, including timestamps and optional
/// network context used in Australian clinic deployments.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: Uuid,
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

impl Session {
    /// Create a new session for a user
    ///
    /// The session starts with `created_at`, `last_activity`, and
    /// `expires_at` based on the supplied timeout.
    ///
    /// # Arguments
    ///
    /// * `user_id` - Authenticated user's identifier
    /// * `timeout_minutes` - Session timeout in minutes
    pub fn new(user_id: Uuid, timeout_minutes: i64) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            user_id,
            created_at: now,
            expires_at: now + Duration::minutes(timeout_minutes),
            last_activity: now,
            ip_address: None,
            user_agent: None,
        }
    }

    /// Return true if the session has passed its expiry time
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Refresh last activity and extend expiry using the timeout
    pub fn update_activity(&mut self, timeout_minutes: i64) {
        let now = Utc::now();
        self.last_activity = now;
        self.expires_at = now + Duration::minutes(timeout_minutes);
    }
}

/// Authentication and session errors for user login flows
///
/// These errors describe invalid credentials, locked or disabled
/// accounts, expired or invalid sessions, and MFA problems.
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Account disabled")]
    AccountDisabled,

    #[error("Account locked")]
    AccountLocked,

    #[error("Session expired")]
    SessionExpired,

    #[error("Invalid session")]
    InvalidSession,

    #[error("Password hashing failed")]
    HashingFailed,

    #[error("Invalid hash format")]
    InvalidHash,

    #[error("MFA required")]
    MFARequired,

    #[error("Invalid MFA token")]
    InvalidMFAToken,
}

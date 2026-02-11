use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    pub fn update_activity(&mut self, timeout_minutes: i64) {
        let now = Utc::now();
        self.last_activity = now;
        self.expires_at = now + Duration::minutes(timeout_minutes);
    }
}

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

use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum PasswordError {
    #[error("Password must not be empty")]
    EmptyPassword,

    #[error("Password hash generation failed")]
    HashingFailed,

    #[error("Password hash format is invalid")]
    InvalidHash,

    #[error("Password verification failed")]
    VerificationFailed,
}

pub trait PasswordHasher: Send + Sync {
    fn hash_password(&self, password: &str) -> Result<String, PasswordError>;

    fn verify_password(&self, password_hash: &str, password: &str) -> Result<(), PasswordError>;
}

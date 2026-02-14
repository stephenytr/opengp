use thiserror::Error;

#[derive(Debug, Error)]
pub enum UserError {
    #[error("Validation error: {0}")]
    Validation(String),

    #[error("User not found: {0}")]
    NotFound(String),

    #[error("Duplicate user: {0}")]
    Duplicate(String),

    #[error("Authentication failed")]
    AuthenticationFailed,

    #[error("Account locked")]
    AccountLocked,

    #[error("Repository error: {0}")]
    Repository(#[from] UserRepositoryError),
}

#[derive(Debug, Error)]
pub enum UserRepositoryError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Not found")]
    NotFound,

    #[error("Constraint violation: {0}")]
    ConstraintViolation(String),
}

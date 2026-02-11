use thiserror::Error;

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Empty name: {0}")]
    EmptyName(String),

    #[error("Invalid date of birth")]
    InvalidDateOfBirth,

    #[error("Invalid Medicare number")]
    InvalidMedicareNumber,
}

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("Duplicate patient found")]
    DuplicatePatient,

    #[error("Validation error: {0}")]
    Validation(#[from] ValidationError),

    #[error("Repository error: {0}")]
    Repository(#[from] RepositoryError),
}

#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Not found")]
    NotFound,

    #[error("Constraint violation: {0}")]
    ConstraintViolation(String),
}

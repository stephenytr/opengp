use thiserror::Error;
use uuid::Uuid;

use crate::domain::error::RepositoryError as BaseRepositoryError;

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("Consultation not found: {0}")]
    ConsultationNotFound(Uuid),

    #[error("Patient not found: {0}")]
    PatientNotFound(Uuid),

    #[error("Allergy not found: {0}")]
    AllergyNotFound(Uuid),

    #[error("Medical history not found: {0}")]
    MedicalHistoryNotFound(Uuid),

    #[error("Vital signs not found: {0}")]
    VitalSignsNotFound(Uuid),

    #[error("Family history not found: {0}")]
    FamilyHistoryNotFound(Uuid),

    #[error("Social history not found for patient: {0}")]
    SocialHistoryNotFound(Uuid),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Repository error: {0}")]
    Repository(#[from] RepositoryError),

    #[error("Unauthorized access")]
    Unauthorized,

    #[error("Consultation already signed")]
    AlreadySigned,
}

#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Not found: {0}")]
    NotFound(Uuid),

    #[error("Constraint violation: {0}")]
    ConstraintViolation(String),

    #[error("Encryption error: {0}")]
    Encryption(String),

    #[error("Decryption error: {0}")]
    Decryption(String),
}

impl From<BaseRepositoryError> for RepositoryError {
    fn from(err: BaseRepositoryError) -> Self {
        match err {
            BaseRepositoryError::Database(e) => RepositoryError::Database(e),
            BaseRepositoryError::NotFound => RepositoryError::NotFound(Uuid::nil()),
            BaseRepositoryError::ConstraintViolation(s) => RepositoryError::ConstraintViolation(s),
        }
    }
}

impl crate::domain::error::InfrastructureError for RepositoryError {
    fn map_sqlx_error<E: std::error::Error + Send + Sync + 'static>(error: E) -> Self {
        RepositoryError::Database(error.to_string())
    }
}

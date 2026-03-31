use thiserror::Error;
use uuid::Uuid;

use crate::domain::error::RepositoryError as BaseRepositoryError;

/// High‑level errors produced by clinical services.
///
/// Wraps missing records (consultations, allergies and history
/// entries), validation problems and repository failures so that UI
/// layers can present user‑friendly messages.
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

    #[error("Concurrent modification conflict: {0}")]
    Conflict(String),

    #[error("Repository error: {0}")]
    Repository(#[from] RepositoryError),

    #[error("Unauthorized access")]
    Unauthorized,

    #[error("Consultation already signed")]
    AlreadySigned,
}

/// Errors originating from clinical repository implementations.
///
/// This type wraps the shared base repository error plus
/// clinical‑specific infrastructure issues.
#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error(transparent)]
    Base(#[from] BaseRepositoryError),

    #[error("Encryption error: {0}")]
    Encryption(String),

    #[error("Decryption error: {0}")]
    Decryption(String),
}

impl crate::domain::error::InfrastructureError for RepositoryError {
    fn map_sqlx_error<E: std::error::Error + Send + Sync + 'static>(error: E) -> Self {
        BaseRepositoryError::from_infrastructure(error).into()
    }
}

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use thiserror::Error;
use uuid::Uuid;

use super::model::{Consultation, SocialHistory};

/// Repository errors for clinical domain
#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Consultation not found: {0}")]
    NotFound(Uuid),

    #[error("Encryption error: {0}")]
    Encryption(String),

    #[error("Decryption error: {0}")]
    Decryption(String),
}

/// Repository trait for Consultation entities
#[async_trait]
pub trait ConsultationRepository: Send + Sync {
    /// Find a consultation by ID
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Consultation>, RepositoryError>;

    /// Find all consultations for a patient
    async fn find_by_patient(&self, patient_id: Uuid)
        -> Result<Vec<Consultation>, RepositoryError>;

    /// Find consultations within a date range
    async fn find_by_date_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<Consultation>, RepositoryError>;

    /// Create a new consultation
    async fn create(&self, consultation: Consultation) -> Result<Consultation, RepositoryError>;

    /// Update an existing consultation
    async fn update(&self, consultation: Consultation) -> Result<Consultation, RepositoryError>;

    /// Sign a consultation (mark as finalized)
    async fn sign(&self, id: Uuid, user_id: Uuid) -> Result<(), RepositoryError>;
}

/// Repository trait for SocialHistory entities
#[async_trait]
pub trait SocialHistoryRepository: Send + Sync {
    /// Find social history for a patient
    async fn find_by_patient(
        &self,
        patient_id: Uuid,
    ) -> Result<Option<SocialHistory>, RepositoryError>;

    /// Create social history for a patient
    async fn create(&self, history: SocialHistory) -> Result<SocialHistory, RepositoryError>;

    /// Update social history
    async fn update(&self, history: SocialHistory) -> Result<SocialHistory, RepositoryError>;
}

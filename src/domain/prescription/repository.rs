use async_trait::async_trait;
use uuid::Uuid;

use super::error::RepositoryError;
use super::model::Prescription;

#[async_trait]
pub trait PrescriptionRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Prescription>, RepositoryError>;
    async fn find_by_patient(&self, patient_id: Uuid)
        -> Result<Vec<Prescription>, RepositoryError>;
    async fn find_active_by_patient(
        &self,
        patient_id: Uuid,
    ) -> Result<Vec<Prescription>, RepositoryError>;
    async fn create(&self, prescription: Prescription) -> Result<Prescription, RepositoryError>;
    async fn update(&self, prescription: Prescription) -> Result<Prescription, RepositoryError>;
    async fn cancel(&self, id: Uuid) -> Result<(), RepositoryError>;
}

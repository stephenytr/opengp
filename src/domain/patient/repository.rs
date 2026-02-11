use async_trait::async_trait;
use uuid::Uuid;

use super::error::RepositoryError;
use super::model::Patient;

#[async_trait]
pub trait PatientRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Patient>, RepositoryError>;
    async fn find_by_medicare(&self, medicare: &str) -> Result<Option<Patient>, RepositoryError>;
    async fn list_active(&self) -> Result<Vec<Patient>, RepositoryError>;
    async fn create(&self, patient: Patient) -> Result<Patient, RepositoryError>;
    async fn update(&self, patient: Patient) -> Result<Patient, RepositoryError>;
    async fn deactivate(&self, id: Uuid) -> Result<(), RepositoryError>;
}

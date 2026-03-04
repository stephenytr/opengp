use async_trait::async_trait;
use uuid::Uuid;

use super::error::RepositoryError;
use super::model::{Immunisation, VaccinationSchedule};

#[async_trait]
pub trait ImmunisationRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Immunisation>, RepositoryError>;
    async fn find_by_patient(&self, patient_id: Uuid) -> Result<Vec<Immunisation>, RepositoryError>;
    async fn create(&self, immunisation: Immunisation) -> Result<Immunisation, RepositoryError>;
    async fn update(&self, immunisation: Immunisation) -> Result<Immunisation, RepositoryError>;
    async fn find_due_schedules(
        &self,
        patient_id: Uuid,
    ) -> Result<Vec<VaccinationSchedule>, RepositoryError>;
}

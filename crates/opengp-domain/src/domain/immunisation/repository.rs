use async_trait::async_trait;
use uuid::Uuid;

use super::error::RepositoryError;
use super::model::{Immunisation, VaccinationSchedule};

/// Repository boundary for immunisation records and schedules.
#[async_trait]
pub trait ImmunisationRepository: Send + Sync {
    /// Find an immunisation record by identifier.
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Immunisation>, RepositoryError>;

    /// List immunisations given to a patient.
    async fn find_by_patient(&self, patient_id: Uuid)
        -> Result<Vec<Immunisation>, RepositoryError>;

    /// Persist a newly recorded immunisation.
    async fn create(&self, immunisation: Immunisation) -> Result<Immunisation, RepositoryError>;

    /// Persist changes to an existing immunisation.
    async fn update(&self, immunisation: Immunisation) -> Result<Immunisation, RepositoryError>;

    /// Find due vaccination schedule entries for a patient.
    async fn find_due_schedules(
        &self,
        patient_id: Uuid,
    ) -> Result<Vec<VaccinationSchedule>, RepositoryError>;
}

use async_trait::async_trait;
use uuid::Uuid;

use super::error::RepositoryError;
use super::model::Prescription;

/// Repository boundary for prescribing data.
///
/// Implementations handle persistence of prescriptions and queries
/// used by the prescribing workflow.
#[async_trait]
pub trait PrescriptionRepository: Send + Sync {
    /// Find a prescription by identifier.
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Prescription>, RepositoryError>;

    /// Find all prescriptions for a given patient.
    async fn find_by_patient(&self, patient_id: Uuid)
        -> Result<Vec<Prescription>, RepositoryError>;

    /// Find active prescriptions for a patient.
    async fn find_active_by_patient(
        &self,
        patient_id: Uuid,
    ) -> Result<Vec<Prescription>, RepositoryError>;

    /// Persist a new prescription.
    async fn create(&self, prescription: Prescription) -> Result<Prescription, RepositoryError>;

    /// Persist changes to an existing prescription.
    async fn update(&self, prescription: Prescription) -> Result<Prescription, RepositoryError>;

    /// Mark a prescription as cancelled in persistent storage.
    async fn cancel(&self, id: Uuid) -> Result<(), RepositoryError>;
}

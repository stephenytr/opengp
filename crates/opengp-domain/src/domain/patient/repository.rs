use async_trait::async_trait;
use uuid::Uuid;

use super::error::RepositoryError;
use super::model::Patient;

/// Repository abstraction for storing and retrieving patient records.
#[async_trait]
pub trait PatientRepository: Send + Sync {
    /// Find a patient by their unique identifier.
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Patient>, RepositoryError>;

    /// Find a patient by Medicare number string as used on Australian Medicare cards.
    async fn find_by_medicare(&self, medicare: &str) -> Result<Option<Patient>, RepositoryError>;

    /// List active patients, optionally limited to a maximum number of rows.
    async fn list_active(&self, limit: Option<i64>) -> Result<Vec<Patient>, RepositoryError>;

    /// Search patients by name or other identifying data suitable for front desk lookup.
    async fn search(&self, query: &str) -> Result<Vec<Patient>, RepositoryError>;

    /// Persist a newly created patient record.
    async fn create(&self, patient: Patient) -> Result<Patient, RepositoryError>;

    /// Persist changes to an existing patient record.
    async fn update(&self, patient: Patient) -> Result<Patient, RepositoryError>;

    /// Soft deactivate a patient so they no longer appear in active lists.
    async fn deactivate(&self, id: Uuid) -> Result<(), RepositoryError>;
}

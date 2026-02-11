use std::sync::Arc;
use uuid::Uuid;

use super::dto::NewPatientData;
use super::error::ServiceError;
use super::model::Patient;
use super::repository::PatientRepository;

pub struct PatientService {
    repository: Arc<dyn PatientRepository>,
}

impl PatientService {
    pub fn new(repository: Arc<dyn PatientRepository>) -> Self {
        Self { repository }
    }

    pub async fn register_patient(&self, data: NewPatientData) -> Result<Patient, ServiceError> {
        if let Some(ref medicare) = data.medicare_number {
            if self.repository.find_by_medicare(medicare).await?.is_some() {
                return Err(ServiceError::DuplicatePatient);
            }
        }

        let patient = Patient::new(data)?;
        let saved = self.repository.create(patient).await?;

        Ok(saved)
    }

    pub async fn find_patient(&self, id: Uuid) -> Result<Option<Patient>, ServiceError> {
        let patient = self.repository.find_by_id(id).await?;
        Ok(patient)
    }
}

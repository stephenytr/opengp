use std::sync::Arc;
use uuid::Uuid;
use tracing::{info, error};

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
        info!("Registering new patient: {} {}", data.first_name, data.last_name);
        
        if let Some(ref medicare) = data.medicare_number {
            info!("Checking for duplicate Medicare number: {}", medicare);
            if self.repository.find_by_medicare(medicare).await?.is_some() {
                error!("Duplicate Medicare number found: {}", medicare);
                return Err(ServiceError::DuplicatePatient);
            }
        }

        info!("Creating patient domain model");
        let patient = Patient::new(data)?;
        
        info!("Saving patient to database with ID: {}", patient.id);
        match self.repository.create(patient.clone()).await {
            Ok(saved) => {
                info!("Patient saved successfully: {}", saved.id);
                Ok(saved)
            }
            Err(e) => {
                error!("Failed to save patient to database: {}", e);
                Err(e.into())
            }
        }
    }

    pub async fn find_patient(&self, id: Uuid) -> Result<Option<Patient>, ServiceError> {
        let patient = self.repository.find_by_id(id).await?;
        Ok(patient)
    }

    pub async fn list_active_patients(&self) -> Result<Vec<Patient>, ServiceError> {
        let patients = self.repository.list_active().await?;
        Ok(patients)
    }
}

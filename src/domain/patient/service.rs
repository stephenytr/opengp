use std::sync::Arc;
use tracing::{error, info};
use uuid::Uuid;

use crate::service;

use super::dto::{NewPatientData, UpdatePatientData};
use super::error::ServiceError;
use super::model::Patient;
use super::repository::PatientRepository;

service! {
    PatientService {
        repository: Arc<dyn PatientRepository>,
    }
}

impl PatientService {
    pub async fn register_patient(&self, data: NewPatientData) -> Result<Patient, ServiceError> {
        info!(
            "Registering new patient: {} {}",
            data.first_name, data.last_name
        );

        if let Some(ref medicare) = data.medicare_number {
            info!("Checking for duplicate Medicare number: {}", medicare);
            if self.repository.find_by_medicare(medicare).await?.is_some() {
                error!("Duplicate Medicare number found: {}", medicare);
                return Err(ServiceError::DuplicatePatient);
            }
        }

        info!("Creating patient domain model");
        let patient = Patient::new(
            data.first_name,
            data.last_name,
            data.date_of_birth,
            data.gender,
            data.ihi,
            data.medicare_number,
            data.medicare_irn,
            data.medicare_expiry,
            data.title,
            data.middle_name,
            data.preferred_name,
            data.address,
            data.phone_home,
            data.phone_mobile,
            data.email,
            data.emergency_contact,
            data.concession_type,
            data.concession_number,
            data.preferred_language,
            data.interpreter_required,
            data.aboriginal_torres_strait_islander,
        )?;

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

    pub async fn update_patient(
        &self,
        id: Uuid,
        data: UpdatePatientData,
    ) -> Result<Patient, ServiceError> {
        info!("Updating patient with ID: {}", id);

        let mut patient = self
            .repository
            .find_by_id(id)
            .await?
            .ok_or(ServiceError::NotFound(id))?;

        patient.update(data)?;

        let updated = self.repository.update(patient).await?;
        info!("Patient updated successfully: {}", updated.id);
        Ok(updated)
    }

    pub async fn list_active_patients(&self) -> Result<Vec<Patient>, ServiceError> {
        let patients = self.repository.list_active().await?;
        Ok(patients)
    }
}

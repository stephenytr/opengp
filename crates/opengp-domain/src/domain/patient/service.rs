use std::sync::Arc;
use tracing::{error, info};
use uuid::Uuid;

use crate::service;

use super::dto::{NewPatientData, UpdatePatientData};
use super::error::{ServiceError, RepositoryError as PatientRepositoryError};
use super::model::Patient;
use super::repository::PatientRepository;
use crate::domain::error::RepositoryError as BaseRepositoryError;

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

        let updated = self.repository.update(patient).await.map_err(|err| match err {
            PatientRepositoryError::Base(BaseRepositoryError::Conflict(message)) => {
                ServiceError::Conflict(message)
            }
            other => ServiceError::Repository(other),
        })?;
        info!("Patient updated successfully: {}", updated.id);
        Ok(updated)
    }

    pub async fn list_active_patients(&self) -> Result<Vec<Patient>, ServiceError> {
        let patients = self.repository.list_active().await?;
        Ok(patients)
    }

    pub async fn search_patients(&self, query: &str) -> Result<Vec<Patient>, ServiceError> {
        let patients = self.repository.search(query).await?;
        Ok(patients)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::patient::{Address, Gender, PatientRepository, RepositoryError};
    use async_trait::async_trait;
    use chrono::NaiveDate;
    use std::sync::Mutex;

    struct MockPatientRepository {
        existing_patients: Vec<Patient>,
        created_patients: Mutex<Vec<Patient>>,
        update_error: Mutex<Option<RepositoryError>>,
    }

    #[async_trait]
    impl PatientRepository for MockPatientRepository {
        async fn find_by_id(&self, id: Uuid) -> Result<Option<Patient>, RepositoryError> {
            Ok(self.existing_patients.iter().find(|p| p.id == id).cloned())
        }

        async fn find_by_medicare(&self, medicare: &str) -> Result<Option<Patient>, RepositoryError> {
            Ok(self
                .existing_patients
                .iter()
                .find(|p| p.medicare_number.as_deref() == Some(medicare))
                .cloned())
        }

        async fn list_active(&self) -> Result<Vec<Patient>, RepositoryError> {
            Ok(self.existing_patients.clone())
        }

        async fn search(&self, query: &str) -> Result<Vec<Patient>, RepositoryError> {
            Ok(self
                .existing_patients
                .iter()
                .filter(|p| p.first_name.contains(query) || p.last_name.contains(query))
                .cloned()
                .collect())
        }

        async fn create(&self, patient: Patient) -> Result<Patient, RepositoryError> {
            self.created_patients
                .lock()
                .expect("created_patients lock poisoned")
                .push(patient.clone());
            Ok(patient)
        }

        async fn update(&self, patient: Patient) -> Result<Patient, RepositoryError> {
            if let Some(err) = self
                .update_error
                .lock()
                .expect("update_error lock poisoned")
                .take()
            {
                return Err(err);
            }
            Ok(patient)
        }

        async fn deactivate(&self, _id: Uuid) -> Result<(), RepositoryError> {
            Ok(())
        }
    }

    fn create_new_patient_data(medicare_number: Option<&str>) -> NewPatientData {
        NewPatientData {
            ihi: Some("8003600000000000".to_string()),
            medicare_number: medicare_number.map(ToString::to_string),
            medicare_irn: Some(1),
            medicare_expiry: None,
            title: None,
            first_name: "Sam".to_string(),
            middle_name: None,
            last_name: "Test".to_string(),
            preferred_name: None,
            date_of_birth: NaiveDate::from_ymd_opt(1988, 5, 20).expect("valid date"),
            gender: Gender::Other,
            address: Address::default(),
            phone_home: None,
            phone_mobile: Some("0400000000".to_string()),
            email: None,
            emergency_contact: None,
            concession_type: None,
            concession_number: None,
            preferred_language: Some("English".to_string()),
            interpreter_required: Some(false),
            aboriginal_torres_strait_islander: None,
        }
    }

    fn create_existing_patient_with_medicare(medicare_number: &str) -> Patient {
        Patient::new(
            "Alex".to_string(),
            "Existing".to_string(),
            NaiveDate::from_ymd_opt(1970, 1, 1).expect("valid date"),
            Gender::Male,
            Some("8003601111111111".to_string()),
            Some(medicare_number.to_string()),
            Some(1),
            None,
            None,
            None,
            None,
            Address::default(),
            None,
            None,
            None,
            None,
            None,
            None,
            Some("English".to_string()),
            Some(false),
            None,
        )
        .expect("valid existing patient")
    }

    #[tokio::test]
    async fn test_register_patient_rejects_duplicate_medicare_number() {
        let duplicate_medicare = "1234567890";
        let repo = Arc::new(MockPatientRepository {
            existing_patients: vec![create_existing_patient_with_medicare(duplicate_medicare)],
            created_patients: Mutex::new(vec![]),
            update_error: Mutex::new(None),
        });
        let service = PatientService::new(repo);

        let result = service
            .register_patient(create_new_patient_data(Some(duplicate_medicare)))
            .await;

        assert!(matches!(result, Err(ServiceError::DuplicatePatient)));
    }

    #[tokio::test]
    async fn test_register_patient_allows_unique_medicare_number() {
        let repo = Arc::new(MockPatientRepository {
            existing_patients: vec![create_existing_patient_with_medicare("9999999999")],
            created_patients: Mutex::new(vec![]),
            update_error: Mutex::new(None),
        });
        let service = PatientService::new(repo);

        let result = service
            .register_patient(create_new_patient_data(Some("1234567890")))
            .await;

        assert!(result.is_ok());
        let patient = result.expect("patient should be created");
        assert_eq!(patient.first_name, "Sam");
        assert_eq!(patient.medicare_number.as_deref(), Some("1234567890"));
    }

    fn update_data_with_first_name(first_name: &str) -> UpdatePatientData {
        UpdatePatientData {
            ihi: None,
            medicare_number: None,
            medicare_irn: None,
            medicare_expiry: None,
            title: None,
            first_name: Some(first_name.to_string()),
            middle_name: None,
            last_name: None,
            preferred_name: None,
            date_of_birth: None,
            gender: None,
            address: None,
            phone_home: None,
            phone_mobile: None,
            email: None,
            emergency_contact: None,
            concession_type: None,
            concession_number: None,
            preferred_language: None,
            interpreter_required: None,
            aboriginal_torres_strait_islander: None,
        }
    }

    #[tokio::test]
    async fn test_update_patient_returns_conflict_for_concurrent_modification() {
        let existing = create_existing_patient_with_medicare("1234567890");
        let patient_id = existing.id;

        let repo = Arc::new(MockPatientRepository {
            existing_patients: vec![existing],
            created_patients: Mutex::new(vec![]),
            update_error: Mutex::new(Some(RepositoryError::Base(
                crate::domain::error::RepositoryError::Conflict(
                    "Patient was modified by another user".to_string(),
                ),
            ))),
        });

        let service = PatientService::new(repo);
        let result = service
            .update_patient(patient_id, update_data_with_first_name("Updated"))
            .await;

        assert!(matches!(result, Err(ServiceError::Conflict(_))));
    }
}

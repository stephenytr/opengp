//! Patient UI Service
//!
//! Bridge between UI components and domain layer for patient operations.

use std::sync::Arc;

use uuid::Uuid;

use super::shared::{ToUiError, UiResult, UiServiceError};
use crate::ui::view_models::PatientListItem;
use opengp_domain::domain::patient::{
    NewPatientData, Patient, PatientService, UpdatePatientData,
};

#[cfg(test)]
use opengp_domain::domain::patient::Address;

/// Patient UI Service - bridges UI to domain layer
pub struct PatientUiService {
    /// Domain patient service
    service: Arc<PatientService>,
}

impl PatientUiService {
    /// Create a new patient UI service
    pub fn new(service: Arc<PatientService>) -> Self {
        Self { service }
    }

    /// List all active patients
    pub async fn list_patients(&self) -> UiResult<Vec<Patient>> {
        self.service
            .list_active_patients()
            .await
            .map_err(|e| e.to_ui_repository_error())
    }

    /// List all active patients as view items
    pub async fn list_patients_as_view_items(&self) -> UiResult<Vec<PatientListItem>> {
        let patients = self.list_patients().await?;
        Ok(patients.into_iter().map(PatientListItem::from).collect())
    }

    /// Search patients by query
    pub async fn search_patients(&self, query: &str) -> UiResult<Vec<Patient>> {
        self.service
            .search_patients(query)
            .await
            .map_err(|e| e.to_ui_repository_error())
    }

    /// Get a patient by ID
    pub async fn get_patient(&self, id: Uuid) -> UiResult<Patient> {
        self.service
            .find_patient(id)
            .await
            .map_err(|e| e.to_ui_repository_error())?
            .ok_or(UiServiceError::NotFound(format!("Patient not found: {}", id)))
    }

    /// Create a new patient
    pub async fn create_patient(&self, data: NewPatientData) -> UiResult<Patient> {
        self.service
            .register_patient(data)
            .await
            .map_err(|e| e.to_ui_repository_error())
    }

    /// Update an existing patient
    pub async fn update_patient(&self, id: Uuid, data: UpdatePatientData) -> UiResult<Patient> {
        let expected_version = self
            .service
            .find_patient(id)
            .await
            .map_err(|e| e.to_ui_repository_error())?
            .ok_or(UiServiceError::NotFound(format!("Patient not found: {}", id)))?
            .version;

        self.service
            .update_patient(id, data, expected_version)
            .await
            .map_err(|e| e.to_ui_repository_error())
    }

    /// Deactivate (soft delete) a patient
    pub async fn deactivate_patient(&self, id: Uuid) -> UiResult<()> {
        // First check if patient exists
        self.service
            .find_patient(id)
            .await
            .map_err(|e| e.to_ui_repository_error())?
            .ok_or(UiServiceError::NotFound(format!("Patient not found: {}", id)))?;

        // TODO: Implement deactivate in repository
        // For now, we'll just return Ok
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use opengp_domain::domain::patient::{Gender, Patient};
    use opengp_infrastructure::infrastructure::database::mocks::MockPatientRepository;

    fn create_test_patient(first_name: &str, last_name: &str) -> Patient {
        Patient::new(
            first_name.to_string(),
            last_name.to_string(),
            chrono::NaiveDate::from_ymd_opt(1990, 1, 15).unwrap(),
            Gender::Male,
            None,
            None,
            None,
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
        .expect("valid test patient")
    }

    #[tokio::test]
    async fn test_list_patients_returns_patients_from_service() {
        let patient1 = create_test_patient("John", "Smith");
        let patient2 = create_test_patient("Jane", "Doe");
        let repo = Arc::new(MockPatientRepository::with_patients(vec![
            patient1.clone(),
            patient2.clone(),
        ]));

        let domain_service = Arc::new(PatientService::new(repo));
        let ui_service = PatientUiService::new(domain_service);

        let patients = ui_service.list_patients().await.expect("should succeed");

        assert_eq!(patients.len(), 2);
        assert_eq!(patients[0].first_name, "John");
        assert_eq!(patients[1].first_name, "Jane");
    }

    #[tokio::test]
    async fn test_list_patients_as_view_items_converts_to_view_model() {
        let patient = create_test_patient("John", "Smith");
        let repo = Arc::new(MockPatientRepository::with_patients(vec![patient]));

        let domain_service = Arc::new(PatientService::new(repo));
        let ui_service = PatientUiService::new(domain_service);

        let view_items = ui_service
            .list_patients_as_view_items()
            .await
            .expect("should succeed");

        assert_eq!(view_items.len(), 1);
        assert!(view_items[0].full_name.contains("John"));
        assert!(view_items[0].full_name.contains("Smith"));
    }

    #[tokio::test]
    async fn test_get_patient_returns_patient_for_valid_uuid() {
        let patient = create_test_patient("John", "Smith");
        let patient_id = patient.id;
        let repo = Arc::new(MockPatientRepository::with_patients(vec![patient.clone()]));

        let domain_service = Arc::new(PatientService::new(repo));
        let ui_service = PatientUiService::new(domain_service);

        let result = ui_service
            .get_patient(patient_id)
            .await
            .expect("should succeed");

        assert_eq!(result.id, patient_id);
        assert_eq!(result.first_name, "John");
    }

    #[tokio::test]
    async fn test_get_patient_returns_not_found_for_non_existent_uuid() {
        let repo = Arc::new(MockPatientRepository::new());
        let domain_service = Arc::new(PatientService::new(repo));
        let ui_service = PatientUiService::new(domain_service);

        let non_existent_id = Uuid::new_v4();
        let result = ui_service.get_patient(non_existent_id).await;

        assert!(result.is_err());
        match result {
            Err(UiServiceError::NotFound(msg)) => {
                assert!(msg.contains("Patient not found"));
                assert!(msg.contains(&non_existent_id.to_string()));
            }
            _ => panic!("Expected NotFound error"),
        }
    }

    #[tokio::test]
    async fn test_error_translation_from_repository_error() {
        let repo = Arc::new(MockPatientRepository::new());
        let domain_service = Arc::new(PatientService::new(repo));
        let ui_service = PatientUiService::new(domain_service);

        let non_existent_id = Uuid::new_v4();
        let result = ui_service.get_patient(non_existent_id).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Patient not found"));
    }

    #[tokio::test]
    async fn test_ui_service_error_display_formatting() {
        let id = Uuid::new_v4();

        let not_found_err = UiServiceError::NotFound(format!("Patient not found: {}", id));
        assert!(not_found_err.to_string().contains("Patient not found"));
        assert!(not_found_err.to_string().contains(&id.to_string()));

        let validation_err = UiServiceError::Validation("invalid email".to_string());
        assert_eq!(
            validation_err.to_string(),
            "Validation error: invalid email"
        );

        let repo_err = UiServiceError::Repository("connection failed".to_string());
        assert_eq!(repo_err.to_string(), "Repository error: connection failed");

        let unknown_err = UiServiceError::Unknown("something went wrong".to_string());
        assert_eq!(unknown_err.to_string(), "Error: something went wrong");
    }
}

//! Patient UI Service
//!
//! Bridge between UI components and domain layer for patient operations.

use std::sync::Arc;

use uuid::Uuid;

use crate::domain::patient::dto::{NewPatientData, UpdatePatientData};
use crate::domain::patient::model::Patient;
use crate::domain::patient::service::PatientService;
use crate::domain::patient::RepositoryError;

/// Result type for UI operations
pub type UiResult<T> = Result<T, UiServiceError>;

/// UI Service errors
#[derive(Debug)]
pub enum UiServiceError {
    /// Not found
    NotFound(Uuid),
    /// Validation error
    Validation(String),
    /// Repository error
    Repository(String),
    /// Unknown error
    Unknown(String),
}

impl std::fmt::Display for UiServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UiServiceError::NotFound(id) => write!(f, "Patient not found: {}", id),
            UiServiceError::Validation(msg) => write!(f, "Validation error: {}", msg),
            UiServiceError::Repository(msg) => write!(f, "Repository error: {}", msg),
            UiServiceError::Unknown(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for UiServiceError {}

impl From<RepositoryError> for UiServiceError {
    fn from(err: RepositoryError) -> Self {
        UiServiceError::Repository(err.to_string())
    }
}

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
            .map_err(|e| UiServiceError::Repository(e.to_string()))
    }

    /// Search patients by query
    pub async fn search_patients(&self, query: &str) -> UiResult<Vec<Patient>> {
        self.service
            .search_patients(query)
            .await
            .map_err(|e| UiServiceError::Repository(e.to_string()))
    }

    /// Get a patient by ID
    pub async fn get_patient(&self, id: Uuid) -> UiResult<Patient> {
        self.service
            .find_patient(id)
            .await
            .map_err(|e| UiServiceError::Repository(e.to_string()))?
            .ok_or_else(|| UiServiceError::NotFound(id))
    }

    /// Create a new patient
    pub async fn create_patient(&self, data: NewPatientData) -> UiResult<Patient> {
        self.service
            .register_patient(data)
            .await
            .map_err(|e| UiServiceError::Repository(e.to_string()))
    }

    /// Update an existing patient
    pub async fn update_patient(&self, id: Uuid, data: UpdatePatientData) -> UiResult<Patient> {
        self.service
            .update_patient(id, data)
            .await
            .map_err(|e| UiServiceError::Repository(e.to_string()))
    }

    /// Deactivate (soft delete) a patient
    pub async fn deactivate_patient(&self, id: Uuid) -> UiResult<()> {
        // First check if patient exists
        self.service
            .find_patient(id)
            .await
            .map_err(|e| UiServiceError::Repository(e.to_string()))?
            .ok_or_else(|| UiServiceError::NotFound(id))?;

        // TODO: Implement deactivate in repository
        // For now, we'll just return Ok
        Ok(())
    }
}

//! Mock repository implementations for testing
//!
//! This module provides in-memory mock implementations of all repository traits.
//! These mocks use `Arc<Mutex<Vec<T>>>` for thread-safe storage and are suitable
//! for unit and integration testing without requiring a real database.
//!
//! # Example
//! ```
//! use opengp_infrastructure::infrastructure::database::mocks::MockPatientRepository;
//! use opengp_domain::domain::patient::PatientRepository;
//!
//! #[tokio::test]
//! async fn test_with_mock() {
//!     let repo = MockPatientRepository::new();
//!     // Use repo in tests
//! }
//! ```

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

use opengp_domain::domain::appointment::{
    Appointment, AppointmentRepository, AppointmentSearchCriteria,
    RepositoryError as AppointmentRepositoryError,
};

use opengp_domain::domain::audit::{AuditEntry, AuditRepository, AuditRepositoryError};

use opengp_domain::domain::clinical::{
    Consultation, ConsultationRepository, RepositoryError as ClinicalRepositoryError,
};

use opengp_domain::domain::patient::{
    Patient, PatientRepository, RepositoryError as PatientRepositoryError,
};

/// Mock implementation of PatientRepository for testing
///
/// Stores patients in an in-memory vector protected by Arc<Mutex<>>.
/// All operations are thread-safe and async-compatible.
///
/// # Behavior
/// - `find_by_id`: Iterates through storage and returns cloned patient if found
/// - `find_by_medicare`: Searches by Medicare number
/// - `list_active`: Returns all patients with `is_active = true`
/// - `create`: Appends patient to storage and returns cloned copy
/// - `update`: Finds and replaces patient in storage
/// - `deactivate`: Sets `is_active = false` for patient
#[derive(Clone)]
pub struct MockPatientRepository {
    storage: Arc<Mutex<Vec<Patient>>>,
}

impl MockPatientRepository {
    /// Create a new empty mock patient repository
    pub fn new() -> Self {
        Self {
            storage: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Create a mock repository with initial patients (for testing)
    pub fn with_patients(patients: Vec<Patient>) -> Self {
        Self {
            storage: Arc::new(Mutex::new(patients)),
        }
    }
}

impl Default for MockPatientRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PatientRepository for MockPatientRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Patient>, PatientRepositoryError> {
        let storage = self.storage.lock().await;
        Ok(storage.iter().find(|p| p.id == id).cloned())
    }

    async fn find_by_medicare(
        &self,
        medicare: &str,
    ) -> Result<Option<Patient>, PatientRepositoryError> {
        let storage = self.storage.lock().await;
        Ok(storage
            .iter()
            .find(|p| p.medicare_number.as_deref() == Some(medicare))
            .cloned())
    }

    async fn list_active(&self, limit: Option<i64>) -> Result<Vec<Patient>, PatientRepositoryError> {
        let storage = self.storage.lock().await;
        let mut active: Vec<Patient> = storage.iter().filter(|p| p.is_active).cloned().collect();
        if let Some(limit_value) = limit {
            active.truncate(limit_value.max(0) as usize);
        }
        Ok(active)
    }

    async fn search(&self, query: &str) -> Result<Vec<Patient>, PatientRepositoryError> {
        let storage = self.storage.lock().await;
        if query.is_empty() {
            return Ok(storage.iter().filter(|p| p.is_active).cloned().collect());
        }
        let query_lower = query.to_lowercase();
        Ok(storage
            .iter()
            .filter(|p| {
                if !p.is_active {
                    return false;
                }
                let full_name = format!("{} {}", p.first_name, p.last_name).to_lowercase();
                let preferred = p
                    .preferred_name
                    .as_ref()
                    .map(|n| n.to_lowercase())
                    .unwrap_or_default();
                full_name.contains(&query_lower) || preferred.contains(&query_lower)
            })
            .cloned()
            .collect())
    }

    async fn create(&self, patient: Patient) -> Result<Patient, PatientRepositoryError> {
        let mut storage = self.storage.lock().await;
        storage.push(patient.clone());
        Ok(patient)
    }

    async fn update(&self, patient: Patient) -> Result<Patient, PatientRepositoryError> {
        let mut storage = self.storage.lock().await;
        if let Some(pos) = storage.iter().position(|p| p.id == patient.id) {
            storage[pos] = patient.clone();
            Ok(patient)
        } else {
            Err(PatientRepositoryError::Base(
                opengp_domain::domain::error::RepositoryError::NotFound,
            ))
        }
    }

    async fn deactivate(&self, id: Uuid) -> Result<(), PatientRepositoryError> {
        let mut storage = self.storage.lock().await;
        if let Some(patient) = storage.iter_mut().find(|p| p.id == id) {
            patient.is_active = false;
            Ok(())
        } else {
            Err(PatientRepositoryError::Base(
                opengp_domain::domain::error::RepositoryError::NotFound,
            ))
        }
    }
}

/// Mock implementation of AppointmentRepository for testing
///
/// Stores appointments in an in-memory vector protected by Arc<Mutex<>>.
/// All operations are thread-safe and async-compatible.
///
/// # Behavior
/// - `find_by_id`: Iterates through storage and returns cloned appointment if found
/// - `create`: Appends appointment to storage and returns cloned copy
/// - `update`: Finds and replaces appointment in storage
/// - `delete`: Soft delete - sets appointment status to Cancelled
/// - `find_by_criteria`: Filters appointments by all provided criteria fields
/// - `find_overlapping`: Returns appointments that overlap with given time range
#[derive(Clone)]
pub struct MockAppointmentRepository {
    storage: Arc<Mutex<Vec<Appointment>>>,
}

#[derive(Clone)]
pub struct MockConsultationRepository {
    storage: Arc<Mutex<Vec<Consultation>>>,
}

impl MockConsultationRepository {
    pub fn new() -> Self {
        Self {
            storage: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl Default for MockConsultationRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ConsultationRepository for MockConsultationRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Consultation>, ClinicalRepositoryError> {
        let storage = self.storage.lock().await;
        Ok(storage.iter().find(|c| c.id == id).cloned())
    }

    async fn find_by_patient(
        &self,
        patient_id: Uuid,
        limit: Option<i64>,
    ) -> Result<Vec<Consultation>, ClinicalRepositoryError> {
        let storage = self.storage.lock().await;
        let mut consultations: Vec<Consultation> = storage
            .iter()
            .filter(|c| c.patient_id == patient_id)
            .cloned()
            .collect();
        consultations.sort_by(|a, b| b.consultation_date.cmp(&a.consultation_date));
        if let Some(l) = limit {
            consultations.truncate(l as usize);
        }
        Ok(consultations)
    }

    async fn find_by_date_range(
        &self,
        patient_id: Uuid,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<Consultation>, ClinicalRepositoryError> {
        let storage = self.storage.lock().await;
        let mut consultations: Vec<Consultation> = storage
            .iter()
            .filter(|c| {
                c.patient_id == patient_id
                    && c.consultation_date >= start
                    && c.consultation_date <= end
            })
            .cloned()
            .collect();
        consultations.sort_by(|a, b| b.consultation_date.cmp(&a.consultation_date));
        Ok(consultations)
    }

    async fn create(
        &self,
        consultation: Consultation,
    ) -> Result<Consultation, ClinicalRepositoryError> {
        let mut storage = self.storage.lock().await;
        storage.push(consultation.clone());
        Ok(consultation)
    }

    async fn update(
        &self,
        consultation: Consultation,
    ) -> Result<Consultation, ClinicalRepositoryError> {
        let mut storage = self.storage.lock().await;
        if let Some(pos) = storage.iter().position(|c| c.id == consultation.id) {
            storage[pos] = consultation.clone();
            Ok(consultation)
        } else {
            Err(ClinicalRepositoryError::Base(
                opengp_domain::domain::error::RepositoryError::NotFound,
            ))
        }
    }

    async fn sign(&self, id: Uuid, user_id: Uuid) -> Result<(), ClinicalRepositoryError> {
        let mut storage = self.storage.lock().await;
        if let Some(consultation) = storage.iter_mut().find(|c| c.id == id) {
            consultation.sign(user_id);
            Ok(())
        } else {
            Err(ClinicalRepositoryError::Base(
                opengp_domain::domain::error::RepositoryError::NotFound,
            ))
        }
    }
}

impl MockAppointmentRepository {
    /// Create a new empty mock appointment repository
    pub fn new() -> Self {
        Self {
            storage: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Create a mock repository with initial appointments (for testing)
    pub fn with_appointments(appointments: Vec<Appointment>) -> Self {
        Self {
            storage: Arc::new(Mutex::new(appointments)),
        }
    }
}

impl Default for MockAppointmentRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AppointmentRepository for MockAppointmentRepository {
    async fn find_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<Appointment>, AppointmentRepositoryError> {
        let storage = self.storage.lock().await;
        Ok(storage.iter().find(|a| a.id == id).cloned())
    }

    async fn create(
        &self,
        appointment: Appointment,
    ) -> Result<Appointment, AppointmentRepositoryError> {
        let mut storage = self.storage.lock().await;
        storage.push(appointment.clone());
        Ok(appointment)
    }

    async fn update(
        &self,
        appointment: Appointment,
    ) -> Result<Appointment, AppointmentRepositoryError> {
        let mut storage = self.storage.lock().await;
        if let Some(pos) = storage.iter().position(|a| a.id == appointment.id) {
            storage[pos] = appointment.clone();
            Ok(appointment)
        } else {
            Err(AppointmentRepositoryError::NotFound)
        }
    }

    async fn delete(&self, id: Uuid) -> Result<(), AppointmentRepositoryError> {
        let mut storage = self.storage.lock().await;
        if let Some(appointment) = storage.iter_mut().find(|a| a.id == id) {
            // Soft delete - mark as cancelled
            use opengp_domain::domain::appointment::AppointmentStatus;
            appointment.status = AppointmentStatus::Cancelled;
            Ok(())
        } else {
            Err(AppointmentRepositoryError::NotFound)
        }
    }

    async fn find_by_criteria(
        &self,
        criteria: &AppointmentSearchCriteria,
    ) -> Result<Vec<Appointment>, AppointmentRepositoryError> {
        let storage = self.storage.lock().await;
        let results = storage
            .iter()
            .filter(|a| {
                // Filter by patient_id if provided
                if let Some(patient_id) = criteria.patient_id {
                    if a.patient_id != patient_id {
                        return false;
                    }
                }

                // Filter by practitioner_id if provided
                if let Some(practitioner_id) = criteria.practitioner_id {
                    if a.practitioner_id != practitioner_id {
                        return false;
                    }
                }

                // Filter by date_from if provided
                if let Some(date_from) = criteria.date_from {
                    if a.start_time < date_from {
                        return false;
                    }
                }

                // Filter by date_to if provided
                if let Some(date_to) = criteria.date_to {
                    if a.end_time > date_to {
                        return false;
                    }
                }

                // Filter by status if provided
                if let Some(ref status) = criteria.status {
                    if a.status != *status {
                        return false;
                    }
                }

                // Filter by appointment_type if provided
                if let Some(ref appointment_type) = criteria.appointment_type {
                    if a.appointment_type != *appointment_type {
                        return false;
                    }
                }

                // Filter by is_urgent if provided
                if let Some(is_urgent) = criteria.is_urgent {
                    if a.is_urgent != is_urgent {
                        return false;
                    }
                }

                // Filter by confirmed if provided
                if let Some(confirmed) = criteria.confirmed {
                    if a.confirmed != confirmed {
                        return false;
                    }
                }

                true
            })
            .cloned()
            .collect();

        Ok(results)
    }

    async fn find_overlapping(
        &self,
        practitioner_id: Uuid,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<Vec<Appointment>, AppointmentRepositoryError> {
        let storage = self.storage.lock().await;
        let results = storage
            .iter()
            .filter(|a| {
                // Must be for the same practitioner
                if a.practitioner_id != practitioner_id {
                    return false;
                }

                // Check for time overlap: appointment overlaps if:
                // - appointment starts before the range ends AND
                // - appointment ends after the range starts
                a.start_time < end_time && a.end_time > start_time
            })
            .cloned()
            .collect();

        Ok(results)
    }
}

/// Mock implementation of AuditRepository for testing
///
/// Stores audit entries in an in-memory vector protected by Arc<Mutex<>>.
/// All operations are thread-safe and async-compatible.
/// Audit entries are immutable once created (append-only log).
///
/// # Behavior
/// - `create`: Appends audit entry to storage and returns cloned copy
/// - `find_by_entity`: Returns all entries for a specific entity (chronological order)
/// - `find_by_user`: Returns all entries created by a user (reverse chronological order)
/// - `find_by_time_range`: Returns entries within time range (chronological order)
/// - `find_by_entity_and_time_range`: Combines entity and time range filters
#[derive(Clone)]
pub struct MockAuditRepository {
    storage: Arc<Mutex<Vec<AuditEntry>>>,
}

impl MockAuditRepository {
    /// Create a new empty mock audit repository
    pub fn new() -> Self {
        Self {
            storage: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Create a mock repository with initial audit entries (for testing)
    pub fn with_entries(entries: Vec<AuditEntry>) -> Self {
        Self {
            storage: Arc::new(Mutex::new(entries)),
        }
    }
}

impl Default for MockAuditRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AuditRepository for MockAuditRepository {
    async fn create(&self, entry: AuditEntry) -> Result<AuditEntry, AuditRepositoryError> {
        let mut storage = self.storage.lock().await;
        storage.push(entry.clone());
        Ok(entry)
    }

    async fn find_by_entity(
        &self,
        entity_type: &str,
        entity_id: Uuid,
    ) -> Result<Vec<AuditEntry>, AuditRepositoryError> {
        let storage = self.storage.lock().await;
        let mut results: Vec<AuditEntry> = storage
            .iter()
            .filter(|e| e.entity_type == entity_type && e.entity_id == entity_id)
            .cloned()
            .collect();

        // Sort chronologically (oldest first)
        results.sort_by(|a, b| a.changed_at.cmp(&b.changed_at));
        Ok(results)
    }

    async fn find_by_user(&self, user_id: Uuid) -> Result<Vec<AuditEntry>, AuditRepositoryError> {
        let storage = self.storage.lock().await;
        let mut results: Vec<AuditEntry> = storage
            .iter()
            .filter(|e| e.changed_by == user_id)
            .cloned()
            .collect();

        // Sort reverse chronologically (newest first)
        results.sort_by(|a, b| b.changed_at.cmp(&a.changed_at));
        Ok(results)
    }

    async fn find_by_time_range(
        &self,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<Vec<AuditEntry>, AuditRepositoryError> {
        let storage = self.storage.lock().await;
        let mut results: Vec<AuditEntry> = storage
            .iter()
            .filter(|e| e.changed_at >= start_time && e.changed_at <= end_time)
            .cloned()
            .collect();

        // Sort chronologically (oldest first)
        results.sort_by(|a, b| a.changed_at.cmp(&b.changed_at));
        Ok(results)
    }

    async fn find_by_entity_and_time_range(
        &self,
        entity_type: &str,
        entity_id: Uuid,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<Vec<AuditEntry>, AuditRepositoryError> {
        let storage = self.storage.lock().await;
        let mut results: Vec<AuditEntry> = storage
            .iter()
            .filter(|e| {
                e.entity_type == entity_type
                    && e.entity_id == entity_id
                    && e.changed_at >= start_time
                    && e.changed_at <= end_time
            })
            .cloned()
            .collect();

        // Sort chronologically (oldest first)
        results.sort_by(|a, b| a.changed_at.cmp(&b.changed_at));
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use opengp_domain::domain::audit::AuditAction;
    use opengp_domain::domain::patient::Gender;
    use opengp_domain::domain::patient::NewPatientData;

    #[tokio::test]
    async fn test_mock_patient_repository_create_and_find() {
        let repo = MockPatientRepository::new();

        // Create a test patient
        let patient_data = NewPatientData {
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            date_of_birth: NaiveDate::from_ymd_opt(1980, 1, 1).unwrap(),
            gender: Gender::Male,
            medicare_number: Some("1234567890".to_string()),
            ihi: None,
            medicare_irn: None,
            medicare_expiry: None,
            title: None,
            middle_name: None,
            preferred_name: None,
            address: opengp_domain::domain::patient::Address {
                line1: Some("123 Main St".to_string()),
                line2: None,
                suburb: Some("Sydney".to_string()),
                state: Some("NSW".to_string()),
                postcode: Some("2000".to_string()),
                country: "Australia".to_string(),
            },
            phone_home: None,
            phone_mobile: None,
            email: None,
            emergency_contact: None,
            concession_type: None,
            concession_number: None,
            preferred_language: Some("English".to_string()),
            interpreter_required: Some(false),
            aboriginal_torres_strait_islander: None,
        };

        let patient = Patient::new(
            patient_data.first_name,
            patient_data.last_name,
            patient_data.date_of_birth,
            patient_data.gender,
            patient_data.ihi,
            patient_data.medicare_number,
            patient_data.medicare_irn,
            patient_data.medicare_expiry,
            patient_data.title,
            patient_data.middle_name,
            patient_data.preferred_name,
            patient_data.address,
            patient_data.phone_home,
            patient_data.phone_mobile,
            patient_data.email,
            patient_data.emergency_contact,
            patient_data.concession_type,
            patient_data.concession_number,
            patient_data.preferred_language,
            patient_data.interpreter_required,
            patient_data.aboriginal_torres_strait_islander,
        )
        .unwrap();
        let patient_id = patient.id;

        // Create patient
        let created = repo.create(patient).await.unwrap();
        assert_eq!(created.id, patient_id);

        // Find by ID
        let found = repo.find_by_id(patient_id).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, patient_id);
    }

    #[tokio::test]
    async fn test_mock_patient_repository_find_by_medicare() {
        let repo = MockPatientRepository::new();

        let patient_data = NewPatientData {
            first_name: "Jane".to_string(),
            last_name: "Smith".to_string(),
            date_of_birth: NaiveDate::from_ymd_opt(1985, 5, 15).unwrap(),
            gender: Gender::Female,
            medicare_number: Some("9876543210".to_string()),
            ihi: None,
            medicare_irn: None,
            medicare_expiry: None,
            title: None,
            middle_name: None,
            preferred_name: None,
            address: opengp_domain::domain::patient::Address {
                line1: Some("456 Oak Ave".to_string()),
                line2: None,
                suburb: Some("Melbourne".to_string()),
                state: Some("VIC".to_string()),
                postcode: Some("3000".to_string()),
                country: "Australia".to_string(),
            },
            phone_home: None,
            phone_mobile: None,
            email: None,
            emergency_contact: None,
            concession_type: None,
            concession_number: None,
            preferred_language: Some("English".to_string()),
            interpreter_required: Some(false),
            aboriginal_torres_strait_islander: None,
        };

        let patient = Patient::new(
            patient_data.first_name,
            patient_data.last_name,
            patient_data.date_of_birth,
            patient_data.gender,
            patient_data.ihi,
            patient_data.medicare_number,
            patient_data.medicare_irn,
            patient_data.medicare_expiry,
            patient_data.title,
            patient_data.middle_name,
            patient_data.preferred_name,
            patient_data.address,
            patient_data.phone_home,
            patient_data.phone_mobile,
            patient_data.email,
            patient_data.emergency_contact,
            patient_data.concession_type,
            patient_data.concession_number,
            patient_data.preferred_language,
            patient_data.interpreter_required,
            patient_data.aboriginal_torres_strait_islander,
        )
        .unwrap();
        repo.create(patient).await.unwrap();

        // Find by Medicare number
        let found = repo.find_by_medicare("9876543210").await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().first_name, "Jane");
    }

    #[tokio::test]
    async fn test_mock_audit_repository_create_and_find() {
        let repo = MockAuditRepository::new();

        let entry = AuditEntry {
            id: Uuid::new_v4(),
            entity_type: "patient".to_string(),
            entity_id: Uuid::new_v4(),
            action: AuditAction::Created,
            old_value: None,
            new_value: Some("{}".to_string()),
            changed_by: Uuid::new_v4(),
            changed_at: Utc::now(),
            source: "database".to_string(),
        };

        let entry_id = entry.id;
        let entity_id = entry.entity_id;

        repo.create(entry).await.unwrap();

        // Find by entity
        let found = repo.find_by_entity("patient", entity_id).await.unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].id, entry_id);
    }
}

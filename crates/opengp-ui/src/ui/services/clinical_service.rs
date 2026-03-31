use std::sync::Arc;

use uuid::Uuid;

use opengp_domain::domain::clinical::{
    Allergy, ClinicalService, ConditionStatus, Consultation, FamilyHistory, MedicalHistory,
    NewAllergyData, NewConsultationData, NewFamilyHistoryData, NewMedicalHistoryData,
    NewVitalSignsData, ServiceError as DomainServiceError, Severity, SocialHistory,
    UpdateSocialHistoryData, VitalSigns,
};

/// Result type used by clinical UI services.
pub type UiResult<T> = Result<T, UiServiceError>;

/// Errors that can occur when calling clinical UI services.
#[derive(Debug)]
pub enum UiServiceError {
    /// Requested entity was not found.
    NotFound(String),
    /// Validation failure for user supplied data.
    Validation(String),
    /// Underlying repository or domain level failure.
    Repository(String),
    /// Any other unexpected error.
    Unknown(String),
}

impl std::fmt::Display for UiServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UiServiceError::NotFound(msg) => write!(f, "Not found: {}", msg),
            UiServiceError::Validation(msg) => write!(f, "Validation error: {}", msg),
            UiServiceError::Repository(msg) => write!(f, "Repository error: {}", msg),
            UiServiceError::Unknown(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for UiServiceError {}

impl From<DomainServiceError> for UiServiceError {
    fn from(err: DomainServiceError) -> Self {
        UiServiceError::Repository(err.to_string())
    }
}

/// UI facing wrapper around the clinical domain service.
pub struct ClinicalUiService {
    service: Arc<ClinicalService>,
}

impl ClinicalUiService {
    /// Creates a new clinical UI service from the domain service.
    pub fn new(service: Arc<ClinicalService>) -> Self {
        Self { service }
    }

    /// Lists all consultations for a given patient.
    pub async fn list_consultations(&self, patient_id: Uuid) -> UiResult<Vec<Consultation>> {
        self.service
            .list_patient_consultations(patient_id)
            .await
            .map_err(|e| UiServiceError::Repository(e.to_string()))
    }

    /// Retrieves a single consultation by id.
    pub async fn get_consultation(&self, id: Uuid) -> UiResult<Option<Consultation>> {
        self.service
            .find_consultation(id)
            .await
            .map_err(|e| UiServiceError::Repository(e.to_string()))
    }

    /// Creates a new consultation for the given patient and practitioner.
    pub async fn create_consultation(
        &self,
        patient_id: Uuid,
        practitioner_id: Uuid,
        user_id: Uuid,
        reason: Option<String>,
        clinical_notes: Option<String>,
    ) -> UiResult<Consultation> {
        let data = NewConsultationData {
            patient_id,
            practitioner_id,
            appointment_id: None,
            reason,
            clinical_notes,
        };
        self.service
            .create_consultation(data, user_id)
            .await
            .map_err(|e| UiServiceError::Repository(e.to_string()))
    }

    /// Updates the clinical notes and reason fields for a consultation.
    pub async fn update_clinical_notes(
        &self,
        consultation_id: Uuid,
        reason: Option<String>,
        clinical_notes: Option<String>,
        expected_version: i32,
        user_id: Uuid,
    ) -> UiResult<Consultation> {
        self.service
            .update_clinical_notes(
                consultation_id,
                reason,
                clinical_notes,
                expected_version,
                user_id,
            )
            .await
            .map_err(|e| UiServiceError::Repository(e.to_string()))
    }

    /// Signs a consultation to indicate completion.
    pub async fn sign_consultation(&self, consultation_id: Uuid, user_id: Uuid) -> UiResult<()> {
        self.service
            .sign_consultation(consultation_id, user_id)
            .await
            .map_err(|e| UiServiceError::Repository(e.to_string()))
    }

    /// Lists allergies for a patient, optionally filtering to active only.
    pub async fn list_allergies(
        &self,
        patient_id: Uuid,
        active_only: bool,
    ) -> UiResult<Vec<Allergy>> {
        self.service
            .list_patient_allergies(patient_id, active_only)
            .await
            .map_err(|e| UiServiceError::Repository(e.to_string()))
    }

    #[allow(clippy::too_many_arguments)]
    /// Records a new allergy for a patient.
    pub async fn add_allergy(
        &self,
        patient_id: Uuid,
        allergen: String,
        allergy_type: opengp_domain::domain::clinical::AllergyType,
        severity: Severity,
        reaction: Option<String>,
        notes: Option<String>,
        user_id: Uuid,
    ) -> UiResult<Allergy> {
        let data = NewAllergyData {
            patient_id,
            allergen,
            allergy_type,
            severity,
            reaction,
            onset_date: None,
            notes,
        };
        self.service
            .add_allergy(data, user_id)
            .await
            .map_err(|e| UiServiceError::Repository(e.to_string()))
    }

    /// Marks an allergy as inactive.
    pub async fn deactivate_allergy(&self, allergy_id: Uuid, user_id: Uuid) -> UiResult<()> {
        self.service
            .deactivate_allergy(allergy_id, user_id)
            .await
            .map_err(|e| UiServiceError::Repository(e.to_string()))
    }

    /// Lists medical history entries for a patient.
    pub async fn list_medical_history(
        &self,
        patient_id: Uuid,
        active_only: bool,
    ) -> UiResult<Vec<MedicalHistory>> {
        self.service
            .list_medical_history(patient_id, active_only)
            .await
            .map_err(|e| UiServiceError::Repository(e.to_string()))
    }

    /// Adds a new medical history entry for a patient.
    pub async fn add_medical_history(
        &self,
        patient_id: Uuid,
        condition: String,
        status: ConditionStatus,
        severity: Option<Severity>,
        notes: Option<String>,
        user_id: Uuid,
    ) -> UiResult<MedicalHistory> {
        let data = NewMedicalHistoryData {
            patient_id,
            condition,
            diagnosis_date: None,
            status,
            severity,
            notes,
        };
        self.service
            .add_medical_history(data, user_id)
            .await
            .map_err(|e| UiServiceError::Repository(e.to_string()))
    }

    /// Updates the status field of a medical history entry.
    pub async fn update_condition_status(
        &self,
        history_id: Uuid,
        status: ConditionStatus,
        user_id: Uuid,
    ) -> UiResult<MedicalHistory> {
        self.service
            .update_condition_status(history_id, status, user_id)
            .await
            .map_err(|e| UiServiceError::Repository(e.to_string()))
    }

    #[allow(clippy::too_many_arguments)]
    /// Records a new set of vital signs for a patient.
    pub async fn record_vitals(
        &self,
        patient_id: Uuid,
        systolic_bp: Option<u16>,
        diastolic_bp: Option<u16>,
        heart_rate: Option<u16>,
        respiratory_rate: Option<u16>,
        temperature: Option<f32>,
        oxygen_saturation: Option<u8>,
        height_cm: Option<u16>,
        weight_kg: Option<f32>,
        notes: Option<String>,
        user_id: Uuid,
    ) -> UiResult<VitalSigns> {
        let data = NewVitalSignsData {
            patient_id,
            consultation_id: None,
            systolic_bp,
            diastolic_bp,
            heart_rate,
            respiratory_rate,
            temperature,
            oxygen_saturation,
            height_cm,
            weight_kg,
            notes,
        };
        self.service
            .record_vital_signs(data, user_id)
            .await
            .map_err(|e| UiServiceError::Repository(e.to_string()))
    }

    /// Gets the most recent vital signs for a patient, if any.
    pub async fn get_latest_vitals(&self, patient_id: Uuid) -> UiResult<Option<VitalSigns>> {
        self.service
            .get_latest_vital_signs(patient_id)
            .await
            .map_err(|e| UiServiceError::Repository(e.to_string()))
    }

    /// Lists historical vital sign entries for a patient.
    pub async fn list_vitals_history(
        &self,
        patient_id: Uuid,
        limit: usize,
    ) -> UiResult<Vec<VitalSigns>> {
        self.service
            .list_vital_signs_history(patient_id, limit)
            .await
            .map_err(|e| UiServiceError::Repository(e.to_string()))
    }

    /// Retrieves the social history record for a patient, if present.
    pub async fn get_social_history(&self, patient_id: Uuid) -> UiResult<Option<SocialHistory>> {
        self.service
            .get_social_history(patient_id)
            .await
            .map_err(|e| UiServiceError::Repository(e.to_string()))
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn update_social_history(
        &self,
        patient_id: Uuid,
        smoking_status: opengp_domain::domain::clinical::SmokingStatus,
        cigarettes_per_day: Option<u8>,
        smoking_quit_date: Option<chrono::NaiveDate>,
        alcohol_status: opengp_domain::domain::clinical::AlcoholStatus,
        standard_drinks_per_week: Option<u8>,
        exercise_frequency: Option<opengp_domain::domain::clinical::ExerciseFrequency>,
        occupation: Option<String>,
        living_situation: Option<String>,
        support_network: Option<String>,
        notes: Option<String>,
        user_id: Uuid,
    ) -> UiResult<SocialHistory> {
        let data = UpdateSocialHistoryData {
            smoking_status,
            cigarettes_per_day,
            smoking_quit_date,
            alcohol_status,
            standard_drinks_per_week,
            exercise_frequency,
            occupation,
            living_situation,
            support_network,
            notes,
        };
        self.service
            .update_social_history(patient_id, data, user_id)
            .await
            .map_err(|e| UiServiceError::Repository(e.to_string()))
    }

    pub async fn list_family_history(&self, patient_id: Uuid) -> UiResult<Vec<FamilyHistory>> {
        self.service
            .list_family_history(patient_id)
            .await
            .map_err(|e| UiServiceError::Repository(e.to_string()))
    }

    pub async fn add_family_history(
        &self,
        patient_id: Uuid,
        relative_relationship: String,
        condition: String,
        age_at_diagnosis: Option<u8>,
        notes: Option<String>,
        user_id: Uuid,
    ) -> UiResult<FamilyHistory> {
        let data = NewFamilyHistoryData {
            patient_id,
            relative_relationship,
            condition,
            age_at_diagnosis,
            notes,
        };
        self.service
            .add_family_history(data, user_id)
            .await
            .map_err(|e| UiServiceError::Repository(e.to_string()))
    }

    pub async fn delete_family_history(&self, history_id: Uuid, user_id: Uuid) -> UiResult<()> {
        self.service
            .delete_family_history(history_id, user_id)
            .await
            .map_err(|e| UiServiceError::Repository(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ui_service_error_display_not_found() {
        let err = UiServiceError::NotFound("test message".to_string());
        assert_eq!(err.to_string(), "Not found: test message");
    }

    #[test]
    fn test_ui_service_error_display_validation() {
        let err = UiServiceError::Validation("invalid input".to_string());
        assert_eq!(err.to_string(), "Validation error: invalid input");
    }

    #[test]
    fn test_ui_service_error_display_repository() {
        let err = UiServiceError::Repository("db error".to_string());
        assert_eq!(err.to_string(), "Repository error: db error");
    }

    #[test]
    fn test_ui_service_error_display_unknown() {
        let err = UiServiceError::Unknown("something went wrong".to_string());
        assert_eq!(err.to_string(), "Error: something went wrong");
    }

    #[test]
    fn test_ui_service_error_is_error() {
        let err: Box<dyn std::error::Error> =
            Box::new(UiServiceError::Repository("test".to_string()));
        assert!(err.to_string().contains("Repository error"));
    }

    #[test]
    fn test_from_domain_service_error() {
        let patient_id = uuid::Uuid::new_v4();
        let domain_err = DomainServiceError::PatientNotFound(patient_id);
        let ui_err: UiServiceError = domain_err.into();
        match ui_err {
            UiServiceError::Repository(msg) => {
                assert!(msg.contains("Patient not found"));
            }
            _ => panic!("Expected Repository error"),
        }
    }

    #[test]
    fn test_ui_service_error_debug() {
        let err = UiServiceError::Repository("test error".to_string());
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("Repository"));
    }

    #[test]
    fn test_ui_service_error_variants() {
        let not_found = UiServiceError::NotFound("id".to_string());
        let validation = UiServiceError::Validation("msg".to_string());
        let repository = UiServiceError::Repository("msg".to_string());
        let unknown = UiServiceError::Unknown("msg".to_string());

        assert!(matches!(not_found, UiServiceError::NotFound(_)));
        assert!(matches!(validation, UiServiceError::Validation(_)));
        assert!(matches!(repository, UiServiceError::Repository(_)));
        assert!(matches!(unknown, UiServiceError::Unknown(_)));
    }
}

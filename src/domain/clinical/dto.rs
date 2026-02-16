use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::model::{
    AlcoholStatus, AllergyType, ConditionStatus, ExerciseFrequency, Severity, SmokingStatus,
};

/// Data for creating a new consultation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewConsultationData {
    pub patient_id: Uuid,
    pub practitioner_id: Uuid,
    pub appointment_id: Option<Uuid>,
}

/// Data for updating SOAP notes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSOAPNotesData {
    pub subjective: Option<String>,
    pub objective: Option<String>,
    pub assessment: Option<String>,
    pub plan: Option<String>,
}

/// Data for adding a new allergy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewAllergyData {
    pub patient_id: Uuid,
    pub allergen: String,
    pub allergy_type: AllergyType,
    pub severity: Severity,
    pub reaction: Option<String>,
    pub onset_date: Option<NaiveDate>,
    pub notes: Option<String>,
}

/// Data for adding a new medical history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewMedicalHistoryData {
    pub patient_id: Uuid,
    pub condition: String,
    pub diagnosis_date: Option<NaiveDate>,
    pub status: ConditionStatus,
    pub severity: Option<Severity>,
    pub notes: Option<String>,
}

/// Data for recording vital signs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewVitalSignsData {
    pub patient_id: Uuid,
    pub consultation_id: Option<Uuid>,
    pub systolic_bp: Option<u16>,
    pub diastolic_bp: Option<u16>,
    pub heart_rate: Option<u16>,
    pub respiratory_rate: Option<u16>,
    pub temperature: Option<f32>,
    pub oxygen_saturation: Option<u8>,
    pub height_cm: Option<u16>,
    pub weight_kg: Option<f32>,
    pub notes: Option<String>,
}

/// Data for updating social history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSocialHistoryData {
    pub smoking_status: SmokingStatus,
    pub cigarettes_per_day: Option<u8>,
    pub smoking_quit_date: Option<NaiveDate>,
    pub alcohol_status: AlcoholStatus,
    pub standard_drinks_per_week: Option<u8>,
    pub exercise_frequency: Option<ExerciseFrequency>,
    pub occupation: Option<String>,
    pub living_situation: Option<String>,
    pub support_network: Option<String>,
    pub notes: Option<String>,
}

/// Data for adding family history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewFamilyHistoryData {
    pub patient_id: Uuid,
    pub relative_relationship: String,
    pub condition: String,
    pub age_at_diagnosis: Option<u8>,
    pub notes: Option<String>,
}

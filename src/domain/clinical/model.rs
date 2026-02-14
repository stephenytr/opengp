use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Consultation {
    pub id: Uuid,
    pub patient_id: Uuid,
    pub practitioner_id: Uuid,
    pub appointment_id: Option<Uuid>,

    pub consultation_date: DateTime<Utc>,

    pub soap_notes: SOAPNotes,

    pub is_signed: bool,
    pub signed_at: Option<DateTime<Utc>>,
    pub signed_by: Option<Uuid>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Uuid,
    pub updated_by: Option<Uuid>,
}

impl Consultation {
    pub fn new(
        patient_id: Uuid,
        practitioner_id: Uuid,
        appointment_id: Option<Uuid>,
        created_by: Uuid,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            patient_id,
            practitioner_id,
            appointment_id,
            consultation_date: Utc::now(),
            soap_notes: SOAPNotes::default(),
            is_signed: false,
            signed_at: None,
            signed_by: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_by,
            updated_by: None,
        }
    }

    pub fn sign(&mut self, user_id: Uuid) {
        self.is_signed = true;
        self.signed_at = Some(Utc::now());
        self.signed_by = Some(user_id);
        self.updated_at = Utc::now();
        self.updated_by = Some(user_id);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SOAPNotes {
    pub subjective: Option<String>,
    pub objective: Option<String>,
    pub assessment: Option<String>,
    pub plan: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MedicalHistory {
    pub id: Uuid,
    pub patient_id: Uuid,

    pub condition: String,
    pub diagnosis_date: Option<NaiveDate>,
    pub status: ConditionStatus,
    pub severity: Option<Severity>,
    pub notes: Option<String>,

    pub is_active: bool,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Uuid,
    pub updated_by: Option<Uuid>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Display, EnumString)]
pub enum ConditionStatus {
    Active,
    Resolved,
    Chronic,
    Recurring,
    InRemission,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Display, EnumString)]
pub enum Severity {
    Mild,
    Moderate,
    Severe,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Allergy {
    pub id: Uuid,
    pub patient_id: Uuid,

    pub allergen: String,
    pub allergy_type: AllergyType,
    pub severity: Severity,
    pub reaction: Option<String>,
    pub onset_date: Option<NaiveDate>,
    pub notes: Option<String>,

    pub is_active: bool,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Uuid,
    pub updated_by: Option<Uuid>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Display, EnumString)]
pub enum AllergyType {
    Drug,
    Food,
    Environmental,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VitalSigns {
    pub id: Uuid,
    pub patient_id: Uuid,
    pub consultation_id: Option<Uuid>,

    pub measured_at: DateTime<Utc>,

    pub systolic_bp: Option<u16>,
    pub diastolic_bp: Option<u16>,
    pub heart_rate: Option<u16>,
    pub respiratory_rate: Option<u16>,
    pub temperature: Option<f32>,
    pub oxygen_saturation: Option<u8>,
    pub height_cm: Option<u16>,
    pub weight_kg: Option<f32>,
    pub bmi: Option<f32>,

    pub notes: Option<String>,

    pub created_at: DateTime<Utc>,
    pub created_by: Uuid,
}

impl VitalSigns {
    pub fn calculate_bmi(&mut self) {
        if let (Some(height), Some(weight)) = (self.height_cm, self.weight_kg) {
            let height_m = height as f32 / 100.0;
            self.bmi = Some(weight / (height_m * height_m));
        }
    }

    pub fn blood_pressure_string(&self) -> Option<String> {
        match (self.systolic_bp, self.diastolic_bp) {
            (Some(sys), Some(dia)) => Some(format!("{}/{}", sys, dia)),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialHistory {
    pub id: Uuid,
    pub patient_id: Uuid,

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

    pub updated_at: DateTime<Utc>,
    pub updated_by: Uuid,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Display, EnumString)]
pub enum SmokingStatus {
    NeverSmoked,
    CurrentSmoker,
    ExSmoker,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Display, EnumString)]
pub enum AlcoholStatus {
    None,
    Occasional,
    Moderate,
    Heavy,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Display, EnumString)]
pub enum ExerciseFrequency {
    None,
    Rarely,
    OnceOrTwicePerWeek,
    ThreeToFiveTimes,
    Daily,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FamilyHistory {
    pub id: Uuid,
    pub patient_id: Uuid,

    pub relative_relationship: String,
    pub condition: String,
    pub age_at_diagnosis: Option<u8>,
    pub notes: Option<String>,

    pub created_at: DateTime<Utc>,
    pub created_by: Uuid,
}

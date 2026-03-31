use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};
use uuid::Uuid;

/// Represents a single clinical consultation between a patient and
/// practitioner.
///
/// Consultations capture the reason for encounter, free‑text clinical
/// notes, signing state and audit information. They are linked to
/// appointments and are the anchor for many other clinical records.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Consultation {
    pub id: Uuid,
    pub patient_id: Uuid,
    pub practitioner_id: Uuid,
    pub appointment_id: Option<Uuid>,

    pub consultation_date: DateTime<Utc>,
    pub reason: Option<String>,

    pub clinical_notes: Option<String>,

    pub is_signed: bool,
    pub signed_at: Option<DateTime<Utc>>,
    pub signed_by: Option<Uuid>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: i32,
    pub created_by: Uuid,
    pub updated_by: Option<Uuid>,
}

impl Consultation {
    /// Create a new consultation for a patient and practitioner.
    ///
    /// The consultation date is initialised to the current UTC time
    /// and the record starts in an unsigned state.
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
            reason: None,
            clinical_notes: None,
            is_signed: false,
            signed_at: None,
            signed_by: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            version: 1,
            created_by,
            updated_by: None,
        }
    }

    /// Mark the consultation as signed by the given user.
    ///
    /// Updates the signing metadata and audit fields. Once signed the
    /// consultation is treated as final and further edits should be
    /// restricted at the service layer.
    pub fn sign(&mut self, user_id: Uuid) {
        self.is_signed = true;
        self.signed_at = Some(Utc::now());
        self.signed_by = Some(user_id);
        self.updated_at = Utc::now();
        self.updated_by = Some(user_id);
    }
}

/// Long‑term or past medical conditions recorded for a patient.
///
/// Entries track diagnosis dates, status (eg chronic, resolved) and
/// severity, and are generally maintained over the life of the
/// patient within the practice.
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

/// Current status of a medical condition.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Display, EnumString)]
pub enum ConditionStatus {
    /// Ongoing active condition.
    Active,
    /// Condition has resolved.
    Resolved,
    /// Chronic condition requiring long‑term management.
    Chronic,
    /// Condition that recurs intermittently.
    Recurring,
    /// Condition currently in remission.
    InRemission,
}

/// Clinical severity grading for conditions and allergies.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Display, EnumString)]
pub enum Severity {
    /// Mild severity.
    Mild,
    /// Moderate severity.
    Moderate,
    /// Severe or life‑threatening.
    Severe,
}

/// Allergy or adverse reaction recorded for a patient.
///
/// Captures allergen, reaction description, onset and severity. Used
/// prominently in prescribing and immunisation safety checks.
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

/// Category of allergen that caused the reaction.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Display, EnumString)]
pub enum AllergyType {
    /// Medicine or vaccine.
    Drug,
    /// Food‑related allergy.
    Food,
    /// Environmental allergy (eg pollen, dust mite).
    Environmental,
    /// Any other allergen type.
    Other,
}

/// Vital sign measurements recorded during or around a consultation.
///
/// Includes blood pressure, pulse, temperature, respiratory rate and
/// anthropometric measurements such as height, weight and BMI.
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
    /// Calculate body mass index from height and weight, if present.
    pub fn calculate_bmi(&mut self) {
        if let (Some(height), Some(weight)) = (self.height_cm, self.weight_kg) {
            let height_m = height as f32 / 100.0;
            self.bmi = Some(weight / (height_m * height_m));
        }
    }

    /// Render blood pressure as a formatted string like `120/80`.
    pub fn blood_pressure_string(&self) -> Option<String> {
        match (self.systolic_bp, self.diastolic_bp) {
            (Some(sys), Some(dia)) => Some(format!("{}/{}", sys, dia)),
            _ => None,
        }
    }
}

/// Social history snapshot for a patient.
///
/// Records smoking and alcohol status, exercise patterns and broader
/// social context relevant to clinical care.
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

/// Smoking status classification used in risk recording.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Display, EnumString)]
pub enum SmokingStatus {
    /// Has never smoked.
    NeverSmoked,
    /// Currently smokes.
    CurrentSmoker,
    /// Previously smoked but has quit.
    ExSmoker,
}

/// Alcohol consumption pattern.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Display, EnumString)]
pub enum AlcoholStatus {
    /// No alcohol use.
    None,
    /// Occasional alcohol use.
    Occasional,
    /// Moderate regular alcohol use.
    Moderate,
    /// Heavy or high‑risk alcohol use.
    Heavy,
}

/// Self‑reported exercise frequency.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Display, EnumString)]
pub enum ExerciseFrequency {
    /// No regular exercise.
    None,
    /// Rare or infrequent exercise.
    Rarely,
    /// Exercise once or twice per week.
    OnceOrTwicePerWeek,
    /// Exercise three to five times per week.
    ThreeToFiveTimes,
    /// Daily exercise.
    Daily,
}

/// Family history of significant conditions for a patient.
///
/// Used for risk assessment and preventive care (eg cardiovascular
/// disease, cancer screening).
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

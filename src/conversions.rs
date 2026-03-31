use chrono::Utc;
use opengp_domain::domain::api::{
    AllergyResponse, AppointmentRequest, FamilyHistoryResponse, MedicalHistoryResponse,
    PatientRequest, PatientResponse, SocialHistoryResponse, VitalSignsResponse,
};
use opengp_domain::domain::appointment::AppointmentType;
use opengp_domain::domain::patient::{Gender, Patient, PhoneNumber};

pub fn patient_request_from_new(
    data: opengp_domain::domain::patient::NewPatientData,
) -> PatientRequest {
    PatientRequest {
        first_name: data.first_name,
        last_name: data.last_name,
        date_of_birth: data.date_of_birth,
        gender: gender_to_api_string(data.gender),
        phone_mobile: data.phone_mobile.map(|p| p.to_string()),
        email: data.email,
        medicare_number: data.medicare_number.map(|m| m.to_string()),
        version: 1,
    }
}

pub fn appointment_request_from_new(
    data: opengp_domain::domain::appointment::NewAppointmentData,
) -> AppointmentRequest {
    AppointmentRequest {
        patient_id: data.patient_id,
        practitioner_id: data.practitioner_id,
        start_time: data.start_time,
        duration_minutes: data.duration.num_minutes(),
        appointment_type: appointment_type_to_api_string(data.appointment_type).to_string(),
        reason: data.reason,
        is_urgent: data.is_urgent,
        version: 1,
    }
}

pub fn patient_request_from_update(
    data: opengp_domain::domain::patient::UpdatePatientData,
    current: &PatientResponse,
) -> PatientRequest {
    PatientRequest {
        first_name: data
            .first_name
            .unwrap_or_else(|| current.first_name.clone()),
        last_name: data.last_name.unwrap_or_else(|| current.last_name.clone()),
        date_of_birth: data.date_of_birth.unwrap_or(current.date_of_birth),
        gender: data
            .gender
            .map(gender_to_api_string)
            .unwrap_or_else(|| current.gender.clone()),
        phone_mobile: data
            .phone_mobile
            .map(|p| p.to_string())
            .or_else(|| current.phone_mobile.clone()),
        email: data.email.or_else(|| current.email.clone()),
        medicare_number: data.medicare_number.map(|m| m.to_string()),
        version: current.version,
    }
}

pub fn domain_patient_from_api_response(response: PatientResponse) -> Patient {
    Patient {
        id: response.id,
        ihi: None,
        medicare_number: None,
        medicare_irn: None,
        medicare_expiry: None,
        title: None,
        first_name: response.first_name,
        middle_name: None,
        last_name: response.last_name,
        preferred_name: None,
        date_of_birth: response.date_of_birth,
        gender: parse_api_gender(&response.gender).unwrap_or_else(|e| {
            tracing::warn!("Failed to parse gender: {}", e);
            Gender::PreferNotToSay
        }),
        address: opengp_domain::domain::patient::Address::default(),
        phone_home: None,
        phone_mobile: response.phone_mobile.map(PhoneNumber::new_lenient),
        email: response.email,
        emergency_contact: None,
        concession_type: None,
        concession_number: None,
        preferred_language: "English".to_string(),
        interpreter_required: false,
        aboriginal_torres_strait_islander: None,
        is_active: response.is_active,
        is_deceased: false,
        deceased_date: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        version: response.version,
    }
}

pub fn domain_allergy_from_api_response(
    response: AllergyResponse,
) -> opengp_domain::domain::clinical::Allergy {
    opengp_domain::domain::clinical::Allergy {
        id: response.id,
        patient_id: response.patient_id,
        allergen: response.allergen,
        allergy_type: parse_api_allergy_type(&response.allergy_type).unwrap_or_else(|e| {
            tracing::warn!("Failed to parse allergy type: {}", e);
            opengp_domain::domain::clinical::AllergyType::Other
        }),
        severity: parse_api_severity(&response.severity).unwrap_or_else(|e| {
            tracing::warn!("Failed to parse severity: {}", e);
            opengp_domain::domain::clinical::Severity::Severe
        }),
        reaction: response.reaction,
        onset_date: response.onset_date,
        notes: response.notes,
        is_active: response.is_active,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        created_by: uuid::Uuid::nil(),
        updated_by: None,
    }
}

pub fn domain_medical_history_from_api_response(
    response: MedicalHistoryResponse,
) -> opengp_domain::domain::clinical::MedicalHistory {
    opengp_domain::domain::clinical::MedicalHistory {
        id: response.id,
        patient_id: response.patient_id,
        condition: response.condition,
        diagnosis_date: response.diagnosis_date,
        status: parse_api_condition_status(&response.status).unwrap_or_else(|e| {
            tracing::warn!("Failed to parse condition status: {}", e);
            opengp_domain::domain::clinical::ConditionStatus::Active
        }),
        severity: response.severity.map(|severity| {
            parse_api_severity(&severity).unwrap_or_else(|e| {
                tracing::warn!("Failed to parse severity: {}", e);
                opengp_domain::domain::clinical::Severity::Severe
            })
        }),
        notes: response.notes,
        is_active: response.is_active,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        created_by: uuid::Uuid::nil(),
        updated_by: None,
    }
}

pub fn domain_vital_signs_from_api_response(
    response: VitalSignsResponse,
) -> opengp_domain::domain::clinical::VitalSigns {
    opengp_domain::domain::clinical::VitalSigns {
        id: response.id,
        patient_id: response.patient_id,
        consultation_id: response.consultation_id,
        measured_at: response.measured_at,
        systolic_bp: response.systolic_bp,
        diastolic_bp: response.diastolic_bp,
        heart_rate: response.heart_rate,
        respiratory_rate: response.respiratory_rate,
        temperature: response.temperature,
        oxygen_saturation: response.oxygen_saturation,
        height_cm: response.height_cm,
        weight_kg: response.weight_kg,
        bmi: response.bmi,
        notes: response.notes,
        created_at: response.measured_at,
        created_by: uuid::Uuid::nil(),
    }
}

pub fn domain_social_history_from_api_response(
    response: SocialHistoryResponse,
) -> opengp_domain::domain::clinical::SocialHistory {
    opengp_domain::domain::clinical::SocialHistory {
        id: response.id,
        patient_id: response.patient_id,
        smoking_status: parse_api_smoking_status(&response.smoking_status).unwrap_or_else(|e| {
            tracing::warn!("Failed to parse smoking status: {}", e);
            opengp_domain::domain::clinical::SmokingStatus::NeverSmoked
        }),
        cigarettes_per_day: response.cigarettes_per_day,
        smoking_quit_date: response.smoking_quit_date,
        alcohol_status: parse_api_alcohol_status(&response.alcohol_status).unwrap_or_else(|e| {
            tracing::warn!("Failed to parse alcohol status: {}", e);
            opengp_domain::domain::clinical::AlcoholStatus::None
        }),
        standard_drinks_per_week: response.standard_drinks_per_week,
        exercise_frequency: response.exercise_frequency.map(|frequency| {
            parse_api_exercise_frequency(&frequency).unwrap_or_else(|e| {
                tracing::warn!("Failed to parse exercise frequency: {}", e);
                opengp_domain::domain::clinical::ExerciseFrequency::None
            })
        }),
        occupation: response.occupation,
        living_situation: response.living_situation,
        support_network: response.support_network,
        notes: response.notes,
        updated_at: response.updated_at,
        updated_by: response.updated_by,
    }
}

pub fn domain_family_history_from_api_response(
    response: FamilyHistoryResponse,
) -> opengp_domain::domain::clinical::FamilyHistory {
    opengp_domain::domain::clinical::FamilyHistory {
        id: response.id,
        patient_id: response.patient_id,
        relative_relationship: response.relative_relationship,
        condition: response.condition,
        age_at_diagnosis: response.age_at_diagnosis,
        notes: response.notes,
        created_at: response.created_at,
        created_by: response.created_by,
    }
}

pub fn gender_to_api_string(gender: Gender) -> String {
    match gender {
        Gender::Male => "male".to_string(),
        Gender::Female => "female".to_string(),
        Gender::Other => "other".to_string(),
        Gender::PreferNotToSay => "prefer_not_to_say".to_string(),
    }
}

pub fn appointment_type_to_api_string(appointment_type: AppointmentType) -> &'static str {
    match appointment_type {
        AppointmentType::Standard => "standard",
        AppointmentType::Long => "long",
        AppointmentType::Brief => "brief",
        AppointmentType::NewPatient => "new_patient",
        AppointmentType::HealthAssessment => "health_assessment",
        AppointmentType::ChronicDiseaseReview => "chronic_disease_review",
        AppointmentType::MentalHealthPlan => "mental_health_plan",
        AppointmentType::Immunisation => "immunisation",
        AppointmentType::Procedure => "procedure",
        AppointmentType::Telephone => "telephone",
        AppointmentType::Telehealth => "telehealth",
        AppointmentType::HomeVisit => "home_visit",
        AppointmentType::Emergency => "emergency",
    }
}

pub fn allergy_type_to_api_string(
    allergy_type: opengp_domain::domain::clinical::AllergyType,
) -> &'static str {
    match allergy_type {
        opengp_domain::domain::clinical::AllergyType::Drug => "drug",
        opengp_domain::domain::clinical::AllergyType::Food => "food",
        opengp_domain::domain::clinical::AllergyType::Environmental => "environmental",
        opengp_domain::domain::clinical::AllergyType::Other => "other",
    }
}

pub fn severity_to_api_string(severity: opengp_domain::domain::clinical::Severity) -> &'static str {
    match severity {
        opengp_domain::domain::clinical::Severity::Mild => "mild",
        opengp_domain::domain::clinical::Severity::Moderate => "moderate",
        opengp_domain::domain::clinical::Severity::Severe => "severe",
    }
}

pub fn condition_status_to_api_string(
    condition_status: opengp_domain::domain::clinical::ConditionStatus,
) -> &'static str {
    match condition_status {
        opengp_domain::domain::clinical::ConditionStatus::Active => "active",
        opengp_domain::domain::clinical::ConditionStatus::Resolved => "resolved",
        opengp_domain::domain::clinical::ConditionStatus::Chronic => "chronic",
        opengp_domain::domain::clinical::ConditionStatus::Recurring => "recurring",
        opengp_domain::domain::clinical::ConditionStatus::InRemission => "in_remission",
    }
}

pub fn smoking_status_to_api_string(
    smoking_status: opengp_domain::domain::clinical::SmokingStatus,
) -> &'static str {
    match smoking_status {
        opengp_domain::domain::clinical::SmokingStatus::NeverSmoked => "never_smoked",
        opengp_domain::domain::clinical::SmokingStatus::CurrentSmoker => "current_smoker",
        opengp_domain::domain::clinical::SmokingStatus::ExSmoker => "ex_smoker",
    }
}

pub fn alcohol_status_to_api_string(
    alcohol_status: opengp_domain::domain::clinical::AlcoholStatus,
) -> &'static str {
    match alcohol_status {
        opengp_domain::domain::clinical::AlcoholStatus::None => "none",
        opengp_domain::domain::clinical::AlcoholStatus::Occasional => "occasional",
        opengp_domain::domain::clinical::AlcoholStatus::Moderate => "moderate",
        opengp_domain::domain::clinical::AlcoholStatus::Heavy => "heavy",
    }
}

pub fn exercise_frequency_to_api_string(
    exercise_frequency: opengp_domain::domain::clinical::ExerciseFrequency,
) -> &'static str {
    match exercise_frequency {
        opengp_domain::domain::clinical::ExerciseFrequency::None => "none",
        opengp_domain::domain::clinical::ExerciseFrequency::Rarely => "rarely",
        opengp_domain::domain::clinical::ExerciseFrequency::OnceOrTwicePerWeek => {
            "once_or_twice_per_week"
        }
        opengp_domain::domain::clinical::ExerciseFrequency::ThreeToFiveTimes => {
            "three_to_five_times"
        }
        opengp_domain::domain::clinical::ExerciseFrequency::Daily => "daily",
    }
}

pub fn parse_api_gender(gender: &str) -> Result<Gender, String> {
    match gender.trim().to_ascii_lowercase().as_str() {
        "male" => Ok(Gender::Male),
        "female" => Ok(Gender::Female),
        "other" => Ok(Gender::Other),
        "prefer_not_to_say" | "prefer-not-to-say" => Ok(Gender::PreferNotToSay),
        _ => Err(format!("Unknown gender: {}", gender)),
    }
}

pub fn parse_api_allergy_type(
    allergy_type: &str,
) -> Result<opengp_domain::domain::clinical::AllergyType, String> {
    match allergy_type.trim().to_ascii_lowercase().as_str() {
        "drug" => Ok(opengp_domain::domain::clinical::AllergyType::Drug),
        "food" => Ok(opengp_domain::domain::clinical::AllergyType::Food),
        "environmental" => Ok(opengp_domain::domain::clinical::AllergyType::Environmental),
        _ => Err(format!("Unknown allergy type: {}", allergy_type)),
    }
}

pub fn parse_api_severity(
    severity: &str,
) -> Result<opengp_domain::domain::clinical::Severity, String> {
    match severity.trim().to_ascii_lowercase().as_str() {
        "mild" => Ok(opengp_domain::domain::clinical::Severity::Mild),
        "moderate" => Ok(opengp_domain::domain::clinical::Severity::Moderate),
        _ => Err(format!("Unknown severity: {}", severity)),
    }
}

pub fn parse_api_condition_status(
    condition_status: &str,
) -> Result<opengp_domain::domain::clinical::ConditionStatus, String> {
    match condition_status.trim().to_ascii_lowercase().as_str() {
        "active" => Ok(opengp_domain::domain::clinical::ConditionStatus::Active),
        "resolved" => Ok(opengp_domain::domain::clinical::ConditionStatus::Resolved),
        "chronic" => Ok(opengp_domain::domain::clinical::ConditionStatus::Chronic),
        "recurring" => Ok(opengp_domain::domain::clinical::ConditionStatus::Recurring),
        "in_remission" | "in-remission" => {
            Ok(opengp_domain::domain::clinical::ConditionStatus::InRemission)
        }
        _ => Err(format!("Unknown condition status: {}", condition_status)),
    }
}

pub fn parse_api_smoking_status(
    smoking_status: &str,
) -> Result<opengp_domain::domain::clinical::SmokingStatus, String> {
    match smoking_status.trim().to_ascii_lowercase().as_str() {
        "never_smoked" | "never-smoked" => {
            Ok(opengp_domain::domain::clinical::SmokingStatus::NeverSmoked)
        }
        "current_smoker" | "current-smoker" => {
            Ok(opengp_domain::domain::clinical::SmokingStatus::CurrentSmoker)
        }
        "ex_smoker" | "ex-smoker" => Ok(opengp_domain::domain::clinical::SmokingStatus::ExSmoker),
        _ => Err(format!("Unknown smoking status: {}", smoking_status)),
    }
}

pub fn parse_api_alcohol_status(
    alcohol_status: &str,
) -> Result<opengp_domain::domain::clinical::AlcoholStatus, String> {
    match alcohol_status.trim().to_ascii_lowercase().as_str() {
        "none" => Ok(opengp_domain::domain::clinical::AlcoholStatus::None),
        "occasional" => Ok(opengp_domain::domain::clinical::AlcoholStatus::Occasional),
        "moderate" => Ok(opengp_domain::domain::clinical::AlcoholStatus::Moderate),
        "heavy" => Ok(opengp_domain::domain::clinical::AlcoholStatus::Heavy),
        _ => Err(format!("Unknown alcohol status: {}", alcohol_status)),
    }
}

pub fn parse_api_exercise_frequency(
    exercise_frequency: &str,
) -> Result<opengp_domain::domain::clinical::ExerciseFrequency, String> {
    match exercise_frequency.trim().to_ascii_lowercase().as_str() {
        "none" => Ok(opengp_domain::domain::clinical::ExerciseFrequency::None),
        "rarely" => Ok(opengp_domain::domain::clinical::ExerciseFrequency::Rarely),
        "once_or_twice_per_week" | "once-or-twice-per-week" => {
            Ok(opengp_domain::domain::clinical::ExerciseFrequency::OnceOrTwicePerWeek)
        }
        "three_to_five_times" | "three-to-five-times" => {
            Ok(opengp_domain::domain::clinical::ExerciseFrequency::ThreeToFiveTimes)
        }
        "daily" => Ok(opengp_domain::domain::clinical::ExerciseFrequency::Daily),
        _ => Err(format!(
            "Unknown exercise frequency: {}",
            exercise_frequency
        )),
    }
}

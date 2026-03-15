use axum::{
    http::{HeaderValue, Method, StatusCode, Uri},
    Json,
};
use chrono::{DateTime, Duration, NaiveDate, Utc};
use http::header::ORIGIN;
use opengp_domain::domain::api::{
    AllergyRequest, AllergyResponse, ApiErrorResponse, AppointmentRequest, AppointmentResponse,
    ConsultationRequest, ConsultationResponse, FamilyHistoryRequest, FamilyHistoryResponse,
    MedicalHistoryRequest, MedicalHistoryResponse, PatientRequest, PatientResponse,
    SocialHistoryRequest, SocialHistoryResponse, VitalSignsRequest, VitalSignsResponse,
};
use opengp_domain::domain::appointment::{
    AppointmentStatus, AppointmentType, NewAppointmentData, ServiceError as AppointmentServiceError,
    UpdateAppointmentData,
};
use opengp_domain::domain::audit::AuditEntry;
use opengp_domain::domain::clinical::{
    AlcoholStatus, AllergyType, ConditionStatus, ExerciseFrequency, NewAllergyData, NewConsultationData,
    NewFamilyHistoryData, NewMedicalHistoryData, NewVitalSignsData, ServiceError as ClinicalServiceError,
    Severity, SmokingStatus, UpdateSocialHistoryData,
};
use opengp_domain::domain::patient::{Address, Gender, NewPatientData, ServiceError as PatientServiceError, UpdatePatientData};
use opengp_domain::domain::user::Role;
use serde::Deserialize;
use tower_http::cors::{AllowOrigin, CorsLayer};
use uuid::Uuid;

pub(super) fn emit_audit_event_non_blocking(
    audit_emitter: std::sync::Arc<dyn opengp_domain::domain::audit::AuditEmitter>,
    audit_entry: AuditEntry,
) {
    tokio::spawn(async move {
        if let Err(err) = audit_emitter.emit(audit_entry).await {
            tracing::warn!(error = %err, "audit emission failed");
        }
    });
}


pub(super) fn unauthorized_response(code: &str, message: &str) -> (StatusCode, Json<ApiErrorResponse>) {
    (
        StatusCode::UNAUTHORIZED,
        Json(ApiErrorResponse {
            status: StatusCode::UNAUTHORIZED.as_u16(),
            message: message.to_string(),
            code: code.to_string(),
        }),
    )
}


pub(super) fn forbidden_response(code: &str, message: &str) -> (StatusCode, Json<ApiErrorResponse>) {
    (
        StatusCode::FORBIDDEN,
        Json(ApiErrorResponse {
            status: StatusCode::FORBIDDEN.as_u16(),
            message: message.to_string(),
            code: code.to_string(),
        }),
    )
}


pub(super) fn bad_request_response(code: &str, message: &str) -> (StatusCode, Json<ApiErrorResponse>) {
    (
        StatusCode::BAD_REQUEST,
        Json(ApiErrorResponse {
            status: StatusCode::BAD_REQUEST.as_u16(),
            message: message.to_string(),
            code: code.to_string(),
        }),
    )
}


pub(super) fn not_found_response(code: &str, message: &str) -> (StatusCode, Json<ApiErrorResponse>) {
    (
        StatusCode::NOT_FOUND,
        Json(ApiErrorResponse {
            status: StatusCode::NOT_FOUND.as_u16(),
            message: message.to_string(),
            code: code.to_string(),
        }),
    )
}


pub(super) fn internal_server_error_response(
    code: &str,
    message: &str,
) -> (StatusCode, Json<ApiErrorResponse>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ApiErrorResponse {
            status: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
            message: message.to_string(),
            code: code.to_string(),
        }),
    )
}


pub(super) fn authorize_read(context: &AuthContext) -> Result<(), (StatusCode, Json<ApiErrorResponse>)> {
    if is_reader(context.role) {
        Ok(())
    } else {
        Err(forbidden_response(
            "insufficient_permissions",
            "Role cannot read patients",
        ))
    }
}


pub(super) fn authorize_write(context: &AuthContext) -> Result<(), (StatusCode, Json<ApiErrorResponse>)> {
    if is_practitioner(context.role) {
        Ok(())
    } else {
        Err(forbidden_response(
            "insufficient_permissions",
            "Only practitioners can modify patients",
        ))
    }
}


pub(super) fn authorize_practitioner_write(
    context: &AuthContext,
) -> Result<(), (StatusCode, Json<ApiErrorResponse>)> {
    if is_practitioner(context.role) {
        Ok(())
    } else {
        Err(forbidden_response(
            "insufficient_permissions",
            "Only practitioners can modify appointments",
        ))
    }
}


pub(super) fn authorize_practitioner_access(
    context: &AuthContext,
) -> Result<(), (StatusCode, Json<ApiErrorResponse>)> {
    if is_practitioner(context.role) {
        Ok(())
    } else {
        Err(forbidden_response(
            "insufficient_permissions",
            "Only practitioners can access clinical notes",
        ))
    }
}


pub(super) fn is_reader(role: Role) -> bool {
    matches!(
        role,
        Role::Receptionist | Role::Doctor | Role::Nurse | Role::Admin
    )
}


pub(super) fn is_practitioner(role: Role) -> bool {
    matches!(role, Role::Doctor | Role::Nurse)
}


pub(super) fn practitioner_specialty(role: Role) -> &'static str {
    match role {
        Role::Doctor => "General Practice",
        Role::Nurse => "Nursing",
        _ => "General Practice",
    }
}


pub(super) fn patient_service_error_to_response(
    error: PatientServiceError,
) -> (StatusCode, Json<ApiErrorResponse>) {
    match error {
        PatientServiceError::DuplicatePatient => conflict_response(
            "duplicate_patient",
            "Patient with provided details already exists",
        ),
        PatientServiceError::NotFound(_) => {
            not_found_response("patient_not_found", "Patient not found")
        }
        PatientServiceError::Validation(_) => {
            bad_request_response("validation_error", "Invalid patient payload")
        }
        PatientServiceError::Conflict(_) => optimistic_lock_conflict_response("patient_conflict"),
        PatientServiceError::Repository(_) => {
            internal_server_error_response("internal_error", "Unable to process patient request")
        }
    }
}


pub(super) fn appointment_service_error_to_response(
    error: AppointmentServiceError,
) -> (StatusCode, Json<ApiErrorResponse>) {
    match error {
        AppointmentServiceError::NotFound(_) => {
            not_found_response("appointment_not_found", "Appointment not found")
        }
        AppointmentServiceError::Conflict(message)
            if message.to_ascii_lowercase().contains("overlapping") =>
        {
            conflict_response("appointment_conflict", "Overlapping appointment")
        }
        AppointmentServiceError::Conflict(_) => optimistic_lock_conflict_response("appointment_conflict"),
        AppointmentServiceError::ValidationError(_)
        | AppointmentServiceError::InvalidTransition(_) => {
            bad_request_response("validation_error", "Invalid appointment payload")
        }
        AppointmentServiceError::Repository(_) | AppointmentServiceError::Audit(_) => {
            internal_server_error_response(
                "internal_error",
                "Unable to process appointment request",
            )
        }
    }
}


pub(super) fn clinical_service_error_to_response(
    error: ClinicalServiceError,
) -> (StatusCode, Json<ApiErrorResponse>) {
    match error {
        ClinicalServiceError::ConsultationNotFound(_) => {
            not_found_response("consultation_not_found", "Consultation not found")
        }
        ClinicalServiceError::PatientNotFound(_) => {
            not_found_response("patient_not_found", "Patient not found")
        }
        ClinicalServiceError::AllergyNotFound(_) => {
            not_found_response("allergy_not_found", "Allergy not found")
        }
        ClinicalServiceError::MedicalHistoryNotFound(_) => {
            not_found_response("medical_history_not_found", "Medical history not found")
        }
        ClinicalServiceError::VitalSignsNotFound(_) => {
            not_found_response("vital_signs_not_found", "Vital signs not found")
        }
        ClinicalServiceError::FamilyHistoryNotFound(_) => {
            not_found_response("family_history_not_found", "Family history not found")
        }
        ClinicalServiceError::SocialHistoryNotFound(_) => {
            not_found_response("social_history_not_found", "Social history not found")
        }
        ClinicalServiceError::Validation(_) => {
            bad_request_response("validation_error", "Invalid consultation payload")
        }
        ClinicalServiceError::Conflict(_) => {
            optimistic_lock_conflict_response("consultation_conflict")
        }
        ClinicalServiceError::AlreadySigned => {
            bad_request_response("consultation_signed", "Consultation already signed")
        }
        ClinicalServiceError::Unauthorized => forbidden_response(
            "insufficient_permissions",
            "Role cannot access consultations",
        ),
        ClinicalServiceError::Repository(_) => {
            internal_server_error_response("internal_error", "Unable to process consultation request")
        }
    }
}


pub(super) fn conflict_response(code: &str, message: &str) -> (StatusCode, Json<ApiErrorResponse>) {
    (
        StatusCode::CONFLICT,
        Json(ApiErrorResponse {
            status: StatusCode::CONFLICT.as_u16(),
            message: message.to_string(),
            code: code.to_string(),
        }),
    )
}


pub(super) fn optimistic_lock_conflict_response(code: &str) -> (StatusCode, Json<ApiErrorResponse>) {
    conflict_response(
        code,
        "Resource was modified. Please refresh and try again.",
    )
}


pub(super) fn patient_request_to_new_data(
    payload: PatientRequest,
) -> Result<NewPatientData, (StatusCode, Json<ApiErrorResponse>)> {
    Ok(NewPatientData {
        ihi: None,
        medicare_number: payload.medicare_number,
        medicare_irn: None,
        medicare_expiry: None,
        title: None,
        first_name: payload.first_name,
        middle_name: None,
        last_name: payload.last_name,
        preferred_name: None,
        date_of_birth: payload.date_of_birth,
        gender: parse_gender(&payload.gender)?,
        address: Address::default(),
        phone_home: None,
        phone_mobile: payload.phone_mobile,
        email: payload.email,
        emergency_contact: None,
        concession_type: None,
        concession_number: None,
        preferred_language: Some("English".to_string()),
        interpreter_required: Some(false),
        aboriginal_torres_strait_islander: None,
    })
}


pub(super) fn patient_request_to_update_data(
    payload: PatientRequest,
) -> Result<UpdatePatientData, (StatusCode, Json<ApiErrorResponse>)> {
    Ok(UpdatePatientData {
        ihi: None,
        medicare_number: payload.medicare_number,
        medicare_irn: None,
        medicare_expiry: None,
        title: None,
        first_name: Some(payload.first_name),
        middle_name: None,
        last_name: Some(payload.last_name),
        preferred_name: None,
        date_of_birth: Some(payload.date_of_birth),
        gender: Some(parse_gender(&payload.gender)?),
        address: None,
        phone_home: None,
        phone_mobile: payload.phone_mobile,
        email: payload.email,
        emergency_contact: None,
        concession_type: None,
        concession_number: None,
        preferred_language: None,
        interpreter_required: None,
        aboriginal_torres_strait_islander: None,
    })
}


pub(super) fn appointment_request_to_new_data(
    payload: AppointmentRequest,
) -> Result<NewAppointmentData, (StatusCode, Json<ApiErrorResponse>)> {
    if payload.duration_minutes <= 0 {
        return Err(bad_request_response(
            "validation_error",
            "Duration must be greater than zero",
        ));
    }

    Ok(NewAppointmentData {
        patient_id: payload.patient_id,
        practitioner_id: payload.practitioner_id,
        start_time: payload.start_time,
        duration: Duration::minutes(payload.duration_minutes),
        appointment_type: parse_appointment_type(&payload.appointment_type)?,
        reason: payload.reason,
        is_urgent: payload.is_urgent,
    })
}


pub(super) fn appointment_request_to_update_data(
    payload: AppointmentRequest,
) -> Result<UpdateAppointmentData, (StatusCode, Json<ApiErrorResponse>)> {
    if payload.duration_minutes <= 0 {
        return Err(bad_request_response(
            "validation_error",
            "Duration must be greater than zero",
        ));
    }

    Ok(UpdateAppointmentData {
        patient_id: Some(payload.patient_id),
        practitioner_id: Some(payload.practitioner_id),
        start_time: Some(payload.start_time),
        duration: Some(Duration::minutes(payload.duration_minutes)),
        status: None,
        appointment_type: Some(parse_appointment_type(&payload.appointment_type)?),
        reason: payload.reason,
        notes: None,
        is_urgent: Some(payload.is_urgent),
        confirmed: None,
        reminder_sent: None,
        cancellation_reason: None,
    })
}


pub(super) fn consultation_request_to_new_data(payload: ConsultationRequest) -> NewConsultationData {
    NewConsultationData {
        patient_id: payload.patient_id,
        practitioner_id: payload.practitioner_id,
        appointment_id: payload.appointment_id,
        reason: payload.reason,
        clinical_notes: payload.clinical_notes,
    }
}


pub(super) fn allergy_request_to_new_data(
    patient_id: Uuid,
    payload: AllergyRequest,
) -> Result<NewAllergyData, (StatusCode, Json<ApiErrorResponse>)> {
    Ok(NewAllergyData {
        patient_id,
        allergen: payload.allergen,
        allergy_type: parse_allergy_type(&payload.allergy_type)?,
        severity: parse_severity(&payload.severity)?,
        reaction: payload.reaction,
        onset_date: payload.onset_date,
        notes: payload.notes,
    })
}


pub(super) fn medical_history_request_to_new_data(
    patient_id: Uuid,
    payload: MedicalHistoryRequest,
) -> Result<NewMedicalHistoryData, (StatusCode, Json<ApiErrorResponse>)> {
    Ok(NewMedicalHistoryData {
        patient_id,
        condition: payload.condition,
        diagnosis_date: payload.diagnosis_date,
        status: parse_condition_status(&payload.status)?,
        severity: payload
            .severity
            .as_deref()
            .map(parse_severity)
            .transpose()?,
        notes: payload.notes,
    })
}


pub(super) fn family_history_request_to_new_data(
    patient_id: Uuid,
    payload: FamilyHistoryRequest,
) -> NewFamilyHistoryData {
    NewFamilyHistoryData {
        patient_id,
        relative_relationship: payload.relative_relationship,
        condition: payload.condition,
        age_at_diagnosis: payload.age_at_diagnosis,
        notes: payload.notes,
    }
}


pub(super) fn vital_signs_request_to_new_data(
    patient_id: Uuid,
    payload: VitalSignsRequest,
) -> NewVitalSignsData {
    NewVitalSignsData {
        patient_id,
        consultation_id: payload.consultation_id,
        systolic_bp: payload.systolic_bp,
        diastolic_bp: payload.diastolic_bp,
        heart_rate: payload.heart_rate,
        respiratory_rate: payload.respiratory_rate,
        temperature: payload.temperature,
        oxygen_saturation: payload.oxygen_saturation,
        height_cm: payload.height_cm,
        weight_kg: payload.weight_kg,
        notes: payload.notes,
    }
}


pub(super) fn social_history_request_to_update_data(
    payload: SocialHistoryRequest,
) -> Result<UpdateSocialHistoryData, (StatusCode, Json<ApiErrorResponse>)> {
    Ok(UpdateSocialHistoryData {
        smoking_status: parse_smoking_status(&payload.smoking_status)?,
        cigarettes_per_day: payload.cigarettes_per_day,
        smoking_quit_date: payload.smoking_quit_date,
        alcohol_status: parse_alcohol_status(&payload.alcohol_status)?,
        standard_drinks_per_week: payload.standard_drinks_per_week,
        exercise_frequency: payload
            .exercise_frequency
            .as_deref()
            .map(parse_exercise_frequency)
            .transpose()?,
        occupation: payload.occupation,
        living_situation: payload.living_situation,
        support_network: payload.support_network,
        notes: payload.notes,
    })
}


pub(super) fn validate_appointment_booking_time(
    start_time: DateTime<Utc>,
) -> Result<(), (StatusCode, Json<ApiErrorResponse>)> {
    if start_time <= Utc::now() {
        return Err(bad_request_response(
            "validation_error",
            "Appointment start time must be in the future",
        ));
    }

    Ok(())
}


pub(super) fn parse_gender(gender: &str) -> Result<Gender, (StatusCode, Json<ApiErrorResponse>)> {
    match gender.trim().to_ascii_lowercase().as_str() {
        "male" => Ok(Gender::Male),
        "female" => Ok(Gender::Female),
        "other" => Ok(Gender::Other),
        "prefer_not_to_say" | "prefer-not-to-say" => Ok(Gender::PreferNotToSay),
        _ => Err(bad_request_response(
            "validation_error",
            "Invalid gender value",
        )),
    }
}


pub(super) fn parse_appointment_type(
    appointment_type: &str,
) -> Result<AppointmentType, (StatusCode, Json<ApiErrorResponse>)> {
    match appointment_type.trim().to_ascii_lowercase().as_str() {
        "standard" => Ok(AppointmentType::Standard),
        "long" => Ok(AppointmentType::Long),
        "brief" => Ok(AppointmentType::Brief),
        "new_patient" | "new-patient" => Ok(AppointmentType::NewPatient),
        "health_assessment" | "health-assessment" => Ok(AppointmentType::HealthAssessment),
        "chronic_disease_review" | "chronic-disease-review" => {
            Ok(AppointmentType::ChronicDiseaseReview)
        }
        "mental_health_plan" | "mental-health-plan" => Ok(AppointmentType::MentalHealthPlan),
        "immunisation" => Ok(AppointmentType::Immunisation),
        "procedure" => Ok(AppointmentType::Procedure),
        "telephone" => Ok(AppointmentType::Telephone),
        "telehealth" => Ok(AppointmentType::Telehealth),
        "home_visit" | "home-visit" => Ok(AppointmentType::HomeVisit),
        "emergency" => Ok(AppointmentType::Emergency),
        _ => Err(bad_request_response(
            "validation_error",
            "Invalid appointment type",
        )),
    }
}


pub(super) fn parse_allergy_type(
    allergy_type: &str,
) -> Result<AllergyType, (StatusCode, Json<ApiErrorResponse>)> {
    match allergy_type.trim().to_ascii_lowercase().as_str() {
        "drug" => Ok(AllergyType::Drug),
        "food" => Ok(AllergyType::Food),
        "environmental" => Ok(AllergyType::Environmental),
        "other" => Ok(AllergyType::Other),
        _ => Err(bad_request_response(
            "validation_error",
            "Invalid allergy type",
        )),
    }
}


pub(super) fn parse_severity(severity: &str) -> Result<Severity, (StatusCode, Json<ApiErrorResponse>)> {
    match severity.trim().to_ascii_lowercase().as_str() {
        "mild" => Ok(Severity::Mild),
        "moderate" => Ok(Severity::Moderate),
        "severe" => Ok(Severity::Severe),
        _ => Err(bad_request_response(
            "validation_error",
            "Invalid allergy severity",
        )),
    }
}


pub(super) fn parse_condition_status(
    condition_status: &str,
) -> Result<ConditionStatus, (StatusCode, Json<ApiErrorResponse>)> {
    match condition_status.trim().to_ascii_lowercase().as_str() {
        "active" => Ok(ConditionStatus::Active),
        "resolved" => Ok(ConditionStatus::Resolved),
        "chronic" => Ok(ConditionStatus::Chronic),
        "recurring" => Ok(ConditionStatus::Recurring),
        "in_remission" | "in-remission" => Ok(ConditionStatus::InRemission),
        _ => Err(bad_request_response(
            "validation_error",
            "Invalid condition status",
        )),
    }
}


pub(super) fn parse_smoking_status(
    smoking_status: &str,
) -> Result<SmokingStatus, (StatusCode, Json<ApiErrorResponse>)> {
    match smoking_status.trim().to_ascii_lowercase().as_str() {
        "never_smoked" | "never-smoked" => Ok(SmokingStatus::NeverSmoked),
        "current_smoker" | "current-smoker" => Ok(SmokingStatus::CurrentSmoker),
        "ex_smoker" | "ex-smoker" => Ok(SmokingStatus::ExSmoker),
        _ => Err(bad_request_response(
            "validation_error",
            "Invalid smoking status",
        )),
    }
}


pub(super) fn parse_alcohol_status(
    alcohol_status: &str,
) -> Result<AlcoholStatus, (StatusCode, Json<ApiErrorResponse>)> {
    match alcohol_status.trim().to_ascii_lowercase().as_str() {
        "none" => Ok(AlcoholStatus::None),
        "occasional" => Ok(AlcoholStatus::Occasional),
        "moderate" => Ok(AlcoholStatus::Moderate),
        "heavy" => Ok(AlcoholStatus::Heavy),
        _ => Err(bad_request_response(
            "validation_error",
            "Invalid alcohol status",
        )),
    }
}


pub(super) fn parse_exercise_frequency(
    exercise_frequency: &str,
) -> Result<ExerciseFrequency, (StatusCode, Json<ApiErrorResponse>)> {
    match exercise_frequency.trim().to_ascii_lowercase().as_str() {
        "none" => Ok(ExerciseFrequency::None),
        "rarely" => Ok(ExerciseFrequency::Rarely),
        "once_or_twice_per_week" | "once-or-twice-per-week" => {
            Ok(ExerciseFrequency::OnceOrTwicePerWeek)
        }
        "three_to_five_times" | "three-to-five-times" => Ok(ExerciseFrequency::ThreeToFiveTimes),
        "daily" => Ok(ExerciseFrequency::Daily),
        _ => Err(bad_request_response(
            "validation_error",
            "Invalid exercise frequency",
        )),
    }
}


pub(super) fn appointment_type_to_string(appointment_type: AppointmentType) -> &'static str {
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


pub(super) fn allergy_type_to_string(allergy_type: AllergyType) -> &'static str {
    match allergy_type {
        AllergyType::Drug => "drug",
        AllergyType::Food => "food",
        AllergyType::Environmental => "environmental",
        AllergyType::Other => "other",
    }
}


pub(super) fn severity_to_string(severity: Severity) -> &'static str {
    match severity {
        Severity::Mild => "mild",
        Severity::Moderate => "moderate",
        Severity::Severe => "severe",
    }
}


pub(super) fn condition_status_to_string(condition_status: ConditionStatus) -> &'static str {
    match condition_status {
        ConditionStatus::Active => "active",
        ConditionStatus::Resolved => "resolved",
        ConditionStatus::Chronic => "chronic",
        ConditionStatus::Recurring => "recurring",
        ConditionStatus::InRemission => "in_remission",
    }
}


pub(super) fn smoking_status_to_string(smoking_status: SmokingStatus) -> &'static str {
    match smoking_status {
        SmokingStatus::NeverSmoked => "never_smoked",
        SmokingStatus::CurrentSmoker => "current_smoker",
        SmokingStatus::ExSmoker => "ex_smoker",
    }
}


pub(super) fn alcohol_status_to_string(alcohol_status: AlcoholStatus) -> &'static str {
    match alcohol_status {
        AlcoholStatus::None => "none",
        AlcoholStatus::Occasional => "occasional",
        AlcoholStatus::Moderate => "moderate",
        AlcoholStatus::Heavy => "heavy",
    }
}


pub(super) fn exercise_frequency_to_string(exercise_frequency: ExerciseFrequency) -> &'static str {
    match exercise_frequency {
        ExerciseFrequency::None => "none",
        ExerciseFrequency::Rarely => "rarely",
        ExerciseFrequency::OnceOrTwicePerWeek => "once_or_twice_per_week",
        ExerciseFrequency::ThreeToFiveTimes => "three_to_five_times",
        ExerciseFrequency::Daily => "daily",
    }
}


pub(super) fn appointment_status_to_string(status: AppointmentStatus) -> &'static str {
    match status {
        AppointmentStatus::Scheduled => "scheduled",
        AppointmentStatus::Confirmed => "confirmed",
        AppointmentStatus::Arrived => "arrived",
        AppointmentStatus::InProgress => "in_progress",
        AppointmentStatus::Completed => "completed",
        AppointmentStatus::NoShow => "no_show",
        AppointmentStatus::Cancelled => "cancelled",
        AppointmentStatus::Rescheduled => "rescheduled",
    }
}


pub(super) fn patient_to_response(patient: opengp_domain::domain::patient::Patient) -> PatientResponse {
    PatientResponse {
        id: patient.id,
        first_name: patient.first_name,
        last_name: patient.last_name,
        date_of_birth: patient.date_of_birth,
        gender: patient.gender.to_string().to_lowercase(),
        phone_mobile: patient.phone_mobile,
        email: patient.email,
        is_active: patient.is_active,
        version: patient.version,
    }
}


pub(super) fn appointment_to_response(
    appointment: opengp_domain::domain::appointment::Appointment,
) -> AppointmentResponse {
    AppointmentResponse {
        id: appointment.id,
        patient_id: appointment.patient_id,
        practitioner_id: appointment.practitioner_id,
        start_time: appointment.start_time,
        end_time: appointment.end_time,
        status: appointment_status_to_string(appointment.status).to_string(),
        appointment_type: appointment_type_to_string(appointment.appointment_type).to_string(),
        is_urgent: appointment.is_urgent,
        reason: appointment.reason,
        version: appointment.version,
    }
}


pub(super) fn consultation_to_response(
    consultation: opengp_domain::domain::clinical::Consultation,
) -> ConsultationResponse {
    ConsultationResponse {
        id: consultation.id,
        patient_id: consultation.patient_id,
        practitioner_id: consultation.practitioner_id,
        appointment_id: consultation.appointment_id,
        consultation_date: consultation.consultation_date,
        reason: consultation.reason,
        clinical_notes: consultation.clinical_notes,
        is_signed: consultation.is_signed,
        version: consultation.version,
    }
}


pub(super) fn allergy_to_response(allergy: opengp_domain::domain::clinical::Allergy) -> AllergyResponse {
    AllergyResponse {
        id: allergy.id,
        patient_id: allergy.patient_id,
        allergen: allergy.allergen,
        allergy_type: allergy_type_to_string(allergy.allergy_type).to_string(),
        severity: severity_to_string(allergy.severity).to_string(),
        reaction: allergy.reaction,
        onset_date: allergy.onset_date,
        notes: allergy.notes,
        is_active: allergy.is_active,
    }
}


pub(super) fn medical_history_to_response(
    history: opengp_domain::domain::clinical::MedicalHistory,
) -> MedicalHistoryResponse {
    MedicalHistoryResponse {
        id: history.id,
        patient_id: history.patient_id,
        condition: history.condition,
        diagnosis_date: history.diagnosis_date,
        status: condition_status_to_string(history.status).to_string(),
        severity: history
            .severity
            .map(|severity| severity_to_string(severity).to_string()),
        notes: history.notes,
        is_active: history.is_active,
    }
}


pub(super) fn family_history_to_response(
    history: opengp_domain::domain::clinical::FamilyHistory,
) -> FamilyHistoryResponse {
    FamilyHistoryResponse {
        id: history.id,
        patient_id: history.patient_id,
        relative_relationship: history.relative_relationship,
        condition: history.condition,
        age_at_diagnosis: history.age_at_diagnosis,
        notes: history.notes,
        created_at: history.created_at,
        created_by: history.created_by,
    }
}


pub(super) fn social_history_to_response(
    history: opengp_domain::domain::clinical::SocialHistory,
) -> SocialHistoryResponse {
    SocialHistoryResponse {
        id: history.id,
        patient_id: history.patient_id,
        smoking_status: smoking_status_to_string(history.smoking_status).to_string(),
        cigarettes_per_day: history.cigarettes_per_day,
        smoking_quit_date: history.smoking_quit_date,
        alcohol_status: alcohol_status_to_string(history.alcohol_status).to_string(),
        standard_drinks_per_week: history.standard_drinks_per_week,
        exercise_frequency: history
            .exercise_frequency
            .map(|frequency| exercise_frequency_to_string(frequency).to_string()),
        occupation: history.occupation,
        living_situation: history.living_situation,
        support_network: history.support_network,
        notes: history.notes,
        updated_at: history.updated_at,
        updated_by: history.updated_by,
    }
}


pub(super) fn vital_signs_to_response(vitals: opengp_domain::domain::clinical::VitalSigns) -> VitalSignsResponse {
    VitalSignsResponse {
        id: vitals.id,
        patient_id: vitals.patient_id,
        consultation_id: vitals.consultation_id,
        measured_at: vitals.measured_at,
        systolic_bp: vitals.systolic_bp,
        diastolic_bp: vitals.diastolic_bp,
        heart_rate: vitals.heart_rate,
        respiratory_rate: vitals.respiratory_rate,
        temperature: vitals.temperature,
        oxygen_saturation: vitals.oxygen_saturation,
        height_cm: vitals.height_cm,
        weight_kg: vitals.weight_kg,
        bmi: vitals.bmi,
        notes: vitals.notes,
    }
}


pub(super) fn cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            ORIGIN,
            http::header::CONTENT_TYPE,
            http::header::ACCEPT,
            http::header::AUTHORIZATION,
        ])
        .allow_credentials(true)
        .allow_origin(AllowOrigin::predicate(|origin: &HeaderValue, _request| {
            is_allowed_origin(origin)
        }))
}


pub(super) fn is_allowed_origin(origin: &HeaderValue) -> bool {
    let origin = match origin.to_str() {
        Ok(v) => v,
        Err(_) => return false,
    };

    let uri = match origin.parse::<Uri>() {
        Ok(v) => v,
        Err(_) => return false,
    };

    let host = match uri.host() {
        Some(v) => v,
        None => return false,
    };

    host == "localhost" || host == "127.0.0.1" || host.starts_with("192.168.")
}


#[derive(Debug, Clone)]
pub(super) struct AuthContext {
    #[allow(dead_code)]
    pub(super) user_id: Uuid,
    pub(super) token: String,
    pub(super) role: Role,
}


#[derive(Debug, Deserialize)]
pub(super) struct PaginationQuery {
    pub(super) page: Option<u32>,
    pub(super) limit: Option<u32>,
}


#[derive(Debug, Deserialize)]
pub(super) struct AppointmentListQuery {
    pub(super) page: Option<u32>,
    pub(super) limit: Option<u32>,
    pub(super) date: Option<NaiveDate>,
    pub(super) date_from: Option<DateTime<Utc>>,
    pub(super) date_to: Option<DateTime<Utc>>,
    #[serde(alias = "practitioner")]
    pub(super) practitioner_id: Option<Uuid>,
}


#[derive(Debug, Deserialize)]
pub(super) struct AppointmentAvailabilityQuery {
    pub(super) practitioner_id: Uuid,
    pub(super) date: NaiveDate,
    pub(super) duration: i64,
}


#[derive(Debug, Deserialize)]
pub(super) struct AppointmentStatusActionRequest {
    pub(super) action: String,
}


#[derive(Debug, Deserialize)]
pub(super) struct ConsultationListQuery {
    pub(super) patient_id: Option<Uuid>,
    pub(super) page: Option<u32>,
    pub(super) limit: Option<u32>,
}

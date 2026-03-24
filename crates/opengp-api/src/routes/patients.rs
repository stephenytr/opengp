use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Extension, Json,
};
use opengp_domain::domain::api::{
    AllergyRequest, AllergyResponse, ApiErrorResponse, FamilyHistoryRequest, FamilyHistoryResponse,
    MedicalHistoryRequest, MedicalHistoryResponse, PaginatedResponse, PatientRequest,
    PatientResponse, SocialHistoryRequest, SocialHistoryResponse, VitalSignsRequest,
    VitalSignsResponse,
};
use opengp_domain::domain::audit::AuditEntry;
use uuid::Uuid;

use crate::ApiState;

use super::middleware::{
    allergy_request_to_new_data, allergy_to_response, authorize_practitioner_access,
    authorize_read, authorize_write, clinical_service_error_to_response,
    emit_audit_event_non_blocking, family_history_request_to_new_data, family_history_to_response,
    medical_history_request_to_new_data, medical_history_to_response, not_found_response,
    patient_request_to_new_data, patient_request_to_update_data, patient_service_error_to_response,
    patient_to_response, social_history_request_to_update_data, social_history_to_response,
    vital_signs_request_to_new_data, vital_signs_to_response, AuthContext, PaginationQuery,
};

pub(super) async fn list_patients(
    State(state): State<ApiState>,
    Extension(context): Extension<AuthContext>,
    Query(query): Query<PaginationQuery>,
) -> Result<
    (StatusCode, Json<PaginatedResponse<PatientResponse>>),
    (StatusCode, Json<ApiErrorResponse>),
> {
    authorize_read(&context)?;

    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(25).clamp(1, 100);

    tracing::debug!(user_id = %context.user_id, page, limit, "Handling list_patients request");

    let patients = state
        .services
        .patient_service
        .list_active_patients()
        .await
        .map_err(|err| {
            tracing::error!(
                user_id = %context.user_id,
                page,
                limit,
                error = %err,
                "list_patients failed while retrieving active patients"
            );
            patient_service_error_to_response(err)
        })?;

    let total = patients.len() as u64;
    tracing::debug!(user_id = %context.user_id, page, limit, total, "list_patients retrieved active patient set");
    let offset = ((page - 1) * limit) as usize;
    let data: Vec<PatientResponse> = patients
        .into_iter()
        .skip(offset)
        .take(limit as usize)
        .map(patient_to_response)
        .collect();

    tracing::debug!(
        user_id = %context.user_id,
        page,
        limit,
        total,
        returned_count = data.len(),
        "list_patients returning paginated response"
    );

    Ok((
        StatusCode::OK,
        Json(PaginatedResponse {
            data,
            total,
            page,
            limit,
        }),
    ))
}

pub(super) async fn get_patient(
    State(state): State<ApiState>,
    Extension(context): Extension<AuthContext>,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<PatientResponse>), (StatusCode, Json<ApiErrorResponse>)> {
    authorize_read(&context)?;

    let mut source = "database";
    let patient = if let Some(cache_service) = &state.cache_service {
        match opengp_cache::patient_cache::get_patient_by_id(cache_service, id).await {
            Ok(Some(cached_patient)) => {
                source = "cache";
                Some(cached_patient)
            }
            _ => {
                state
                    .services
                    .patient_service
                    .find_patient(id)
                    .await
                    .map_err(patient_service_error_to_response)?
            }
        }
    } else {
        state
            .services
            .patient_service
            .find_patient(id)
            .await
            .map_err(patient_service_error_to_response)?
    };

    let patient = patient
        .ok_or_else(|| not_found_response("patient_not_found", "Patient not found"))?;

    if !patient.is_active {
        return Err(not_found_response("patient_not_found", "Patient not found"));
    }

    let mut audit_entry = AuditEntry::new_read("patient", id, context.user_id);
    audit_entry.source = source.to_string();
    emit_audit_event_non_blocking(state.audit_emitter.clone(), audit_entry);

    Ok((StatusCode::OK, Json(patient_to_response(patient))))
}

pub(super) async fn create_patient(
    State(state): State<ApiState>,
    Extension(context): Extension<AuthContext>,
    Json(payload): Json<PatientRequest>,
) -> Result<(StatusCode, Json<PatientResponse>), (StatusCode, Json<ApiErrorResponse>)> {
    authorize_write(&context)?;

    let patient = state
        .services
        .patient_service
        .register_patient(patient_request_to_new_data(payload)?)
        .await
        .map_err(patient_service_error_to_response)?;

    let patient_id = patient.id;
    let audit_entry = AuditEntry::new_created(
        "patient",
        patient_id,
        serde_json::to_string(&patient).unwrap_or_default(),
        context.user_id,
    );
    emit_audit_event_non_blocking(state.audit_emitter.clone(), audit_entry);

    Ok((StatusCode::CREATED, Json(patient_to_response(patient))))
}

pub(super) async fn update_patient(
    State(state): State<ApiState>,
    Extension(context): Extension<AuthContext>,
    Path(id): Path<Uuid>,
    Json(payload): Json<PatientRequest>,
) -> Result<(StatusCode, Json<PatientResponse>), (StatusCode, Json<ApiErrorResponse>)> {
    authorize_write(&context)?;
    let expected_version = payload.version;

    let patient = state
        .services
        .patient_service
        .update_patient(
            id,
            patient_request_to_update_data(payload)?,
            expected_version,
        )
        .await
        .map_err(patient_service_error_to_response)?;

    let audit_entry = AuditEntry::new_updated(
        "patient",
        id,
        "{}",
        serde_json::to_string(&patient).unwrap_or_default(),
        context.user_id,
    );
    emit_audit_event_non_blocking(state.audit_emitter.clone(), audit_entry);

    Ok((StatusCode::OK, Json(patient_to_response(patient))))
}

pub(super) async fn delete_patient(
    State(state): State<ApiState>,
    Extension(context): Extension<AuthContext>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<ApiErrorResponse>)> {
    authorize_write(&context)?;

    state
        .services
        .patient_service
        .deactivate_patient(id)
        .await
        .map_err(patient_service_error_to_response)?;

    let audit_entry =
        AuditEntry::new_cancelled("patient", id, "Patient deactivated", context.user_id);
    emit_audit_event_non_blocking(state.audit_emitter.clone(), audit_entry);

    Ok(StatusCode::NO_CONTENT)
}

pub(super) async fn list_allergies(
    State(state): State<ApiState>,
    Extension(context): Extension<AuthContext>,
    Path(patient_id): Path<Uuid>,
) -> Result<(StatusCode, Json<Vec<AllergyResponse>>), (StatusCode, Json<ApiErrorResponse>)> {
    authorize_read(&context)?;

    let allergies = state
        .services
        .clinical_service
        .list_patient_allergies(patient_id, false)
        .await
        .map_err(clinical_service_error_to_response)?;

    Ok((
        StatusCode::OK,
        Json(allergies.into_iter().map(allergy_to_response).collect()),
    ))
}

pub(super) async fn create_allergy(
    State(state): State<ApiState>,
    Extension(context): Extension<AuthContext>,
    Path(patient_id): Path<Uuid>,
    Json(payload): Json<AllergyRequest>,
) -> Result<(StatusCode, Json<AllergyResponse>), (StatusCode, Json<ApiErrorResponse>)> {
    authorize_practitioner_access(&context)?;

    let allergy = state
        .services
        .clinical_service
        .add_allergy(
            allergy_request_to_new_data(patient_id, payload)?,
            context.user_id,
        )
        .await
        .map_err(clinical_service_error_to_response)?;

    let audit_entry = AuditEntry::new_created(
        "allergy",
        allergy.id,
        serde_json::to_string(&allergy).unwrap_or_default(),
        context.user_id,
    );
    emit_audit_event_non_blocking(state.audit_emitter.clone(), audit_entry);

    Ok((StatusCode::CREATED, Json(allergy_to_response(allergy))))
}

pub(super) async fn list_medical_history(
    State(state): State<ApiState>,
    Extension(context): Extension<AuthContext>,
    Path(patient_id): Path<Uuid>,
) -> Result<(StatusCode, Json<Vec<MedicalHistoryResponse>>), (StatusCode, Json<ApiErrorResponse>)> {
    authorize_read(&context)?;

    let history = state
        .services
        .clinical_service
        .list_medical_history(patient_id, false)
        .await
        .map_err(clinical_service_error_to_response)?;

    Ok((
        StatusCode::OK,
        Json(
            history
                .into_iter()
                .map(medical_history_to_response)
                .collect(),
        ),
    ))
}

pub(super) async fn create_medical_history(
    State(state): State<ApiState>,
    Extension(context): Extension<AuthContext>,
    Path(patient_id): Path<Uuid>,
    Json(payload): Json<MedicalHistoryRequest>,
) -> Result<(StatusCode, Json<MedicalHistoryResponse>), (StatusCode, Json<ApiErrorResponse>)> {
    authorize_practitioner_access(&context)?;

    let history = state
        .services
        .clinical_service
        .add_medical_history(
            medical_history_request_to_new_data(patient_id, payload)?,
            context.user_id,
        )
        .await
        .map_err(clinical_service_error_to_response)?;

    let audit_entry = AuditEntry::new_created(
        "medical_history",
        history.id,
        serde_json::to_string(&history).unwrap_or_default(),
        context.user_id,
    );
    emit_audit_event_non_blocking(state.audit_emitter.clone(), audit_entry);

    Ok((
        StatusCode::CREATED,
        Json(medical_history_to_response(history)),
    ))
}

pub(super) async fn list_family_history(
    State(state): State<ApiState>,
    Extension(context): Extension<AuthContext>,
    Path(patient_id): Path<Uuid>,
) -> Result<(StatusCode, Json<Vec<FamilyHistoryResponse>>), (StatusCode, Json<ApiErrorResponse>)> {
    authorize_read(&context)?;

    let history = state
        .services
        .clinical_service
        .list_family_history(patient_id)
        .await
        .map_err(clinical_service_error_to_response)?;

    Ok((
        StatusCode::OK,
        Json(
            history
                .into_iter()
                .map(family_history_to_response)
                .collect(),
        ),
    ))
}

pub(super) async fn create_family_history(
    State(state): State<ApiState>,
    Extension(context): Extension<AuthContext>,
    Path(patient_id): Path<Uuid>,
    Json(payload): Json<FamilyHistoryRequest>,
) -> Result<(StatusCode, Json<FamilyHistoryResponse>), (StatusCode, Json<ApiErrorResponse>)> {
    authorize_practitioner_access(&context)?;

    let history = state
        .services
        .clinical_service
        .add_family_history(
            family_history_request_to_new_data(patient_id, payload),
            context.user_id,
        )
        .await
        .map_err(clinical_service_error_to_response)?;

    let audit_entry = AuditEntry::new_created(
        "family_history",
        history.id,
        serde_json::to_string(&history).unwrap_or_default(),
        context.user_id,
    );
    emit_audit_event_non_blocking(state.audit_emitter.clone(), audit_entry);

    Ok((
        StatusCode::CREATED,
        Json(family_history_to_response(history)),
    ))
}

pub(super) async fn get_social_history(
    State(state): State<ApiState>,
    Extension(context): Extension<AuthContext>,
    Path(patient_id): Path<Uuid>,
) -> Result<(StatusCode, Json<SocialHistoryResponse>), (StatusCode, Json<ApiErrorResponse>)> {
    authorize_read(&context)?;

    let history = state
        .services
        .clinical_service
        .get_social_history(patient_id)
        .await
        .map_err(clinical_service_error_to_response)?
        .ok_or_else(|| {
            not_found_response("social_history_not_found", "Social history not found")
        })?;

    Ok((StatusCode::OK, Json(social_history_to_response(history))))
}

pub(super) async fn update_social_history(
    State(state): State<ApiState>,
    Extension(context): Extension<AuthContext>,
    Path(patient_id): Path<Uuid>,
    Json(payload): Json<SocialHistoryRequest>,
) -> Result<(StatusCode, Json<SocialHistoryResponse>), (StatusCode, Json<ApiErrorResponse>)> {
    authorize_practitioner_access(&context)?;

    let history = state
        .services
        .clinical_service
        .update_social_history(
            patient_id,
            social_history_request_to_update_data(payload)?,
            context.user_id,
        )
        .await
        .map_err(clinical_service_error_to_response)?;

    let audit_entry = AuditEntry::new_updated(
        "social_history",
        history.id,
        "{}",
        serde_json::to_string(&history).unwrap_or_default(),
        context.user_id,
    );
    emit_audit_event_non_blocking(state.audit_emitter.clone(), audit_entry);

    Ok((StatusCode::OK, Json(social_history_to_response(history))))
}

pub(super) async fn list_vitals(
    State(state): State<ApiState>,
    Extension(context): Extension<AuthContext>,
    Path(patient_id): Path<Uuid>,
) -> Result<(StatusCode, Json<Vec<VitalSignsResponse>>), (StatusCode, Json<ApiErrorResponse>)> {
    authorize_read(&context)?;

    let vitals = state
        .services
        .clinical_service
        .list_vital_signs_history(patient_id, 100)
        .await
        .map_err(clinical_service_error_to_response)?;

    Ok((
        StatusCode::OK,
        Json(vitals.into_iter().map(vital_signs_to_response).collect()),
    ))
}

pub(super) async fn create_vitals(
    State(state): State<ApiState>,
    Extension(context): Extension<AuthContext>,
    Path(patient_id): Path<Uuid>,
    Json(payload): Json<VitalSignsRequest>,
) -> Result<(StatusCode, Json<VitalSignsResponse>), (StatusCode, Json<ApiErrorResponse>)> {
    authorize_practitioner_access(&context)?;

    let vitals = state
        .services
        .clinical_service
        .record_vital_signs(
            vital_signs_request_to_new_data(patient_id, payload),
            context.user_id,
        )
        .await
        .map_err(clinical_service_error_to_response)?;

    let audit_entry = AuditEntry::new_created(
        "vital_signs",
        vitals.id,
        serde_json::to_string(&vitals).unwrap_or_default(),
        context.user_id,
    );
    emit_audit_event_non_blocking(state.audit_emitter.clone(), audit_entry);

    Ok((StatusCode::CREATED, Json(vital_signs_to_response(vitals))))
}

pub(super) async fn delete_allergy(
    State(state): State<ApiState>,
    Extension(context): Extension<AuthContext>,
    Path((patient_id, allergy_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, (StatusCode, Json<ApiErrorResponse>)> {
    authorize_practitioner_access(&context)?;

    let allergies = state
        .services
        .clinical_service
        .list_patient_allergies(patient_id, false)
        .await
        .map_err(clinical_service_error_to_response)?;

    if !allergies.iter().any(|allergy| allergy.id == allergy_id) {
        return Err(not_found_response("allergy_not_found", "Allergy not found"));
    }

    state
        .services
        .clinical_service
        .deactivate_allergy(allergy_id, context.user_id)
        .await
        .map_err(clinical_service_error_to_response)?;

    let audit_entry = AuditEntry::new_cancelled(
        "allergy",
        allergy_id,
        "Allergy deactivated via API",
        context.user_id,
    );
    emit_audit_event_non_blocking(state.audit_emitter.clone(), audit_entry);

    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pagination_query_defaults_to_page_1_limit_25() {
        let query = PaginationQuery {
            page: None,
            limit: None,
        };

        let page = query.page.unwrap_or(1).max(1);
        let limit = query.limit.unwrap_or(25).clamp(1, 100);

        assert_eq!(page, 1);
        assert_eq!(limit, 25);
    }

    #[test]
    fn pagination_query_clamps_limit_to_max_100() {
        let query = PaginationQuery {
            page: Some(1),
            limit: Some(500),
        };

        let limit = query.limit.unwrap_or(25).clamp(1, 100);

        assert_eq!(limit, 100);
    }

    #[test]
    fn pagination_query_enforces_minimum_page_1() {
        let query = PaginationQuery {
            page: Some(0),
            limit: Some(10),
        };

        let page = query.page.unwrap_or(1).max(1);

        assert_eq!(page, 1);
    }
}

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Extension, Json,
};
use opengp_domain::domain::api::{
    ApiErrorResponse, ConsultationRequest, ConsultationResponse, PaginatedResponse,
};
use opengp_domain::domain::audit::AuditEntry;
use uuid::Uuid;

use crate::ApiState;

use super::middleware::{
    authorize_practitioner_access, bad_request_response, clinical_service_error_to_response,
    consultation_request_to_new_data, consultation_to_response, emit_audit_event_non_blocking,
    not_found_response, AuthContext, ConsultationListQuery,
};

pub(super) async fn list_consultations(
    State(state): State<ApiState>,
    Extension(context): Extension<AuthContext>,
    Query(query): Query<ConsultationListQuery>,
) -> Result<
    (StatusCode, Json<PaginatedResponse<ConsultationResponse>>),
    (StatusCode, Json<ApiErrorResponse>),
> {
    authorize_practitioner_access(&context)?;

    let patient_id = query
        .patient_id
        .ok_or_else(|| bad_request_response("validation_error", "patient_id is required"))?;
    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(25).clamp(1, 100);

    let consultations = state
        .services
        .clinical_service
        .list_patient_consultations(patient_id)
        .await
        .map_err(clinical_service_error_to_response)?;

    let total = consultations.len() as u64;
    let offset = ((page - 1) * limit) as usize;
    let data = consultations
        .into_iter()
        .skip(offset)
        .take(limit as usize)
        .map(consultation_to_response)
        .collect();

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

pub(super) async fn get_consultation(
    State(state): State<ApiState>,
    Extension(context): Extension<AuthContext>,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<ConsultationResponse>), (StatusCode, Json<ApiErrorResponse>)> {
    authorize_practitioner_access(&context)?;

    let consultation = state
        .services
        .clinical_service
        .find_consultation(id)
        .await
        .map_err(clinical_service_error_to_response)?
        .ok_or_else(|| not_found_response("consultation_not_found", "Consultation not found"))?;

    Ok((StatusCode::OK, Json(consultation_to_response(consultation))))
}

pub(super) async fn create_consultation(
    State(state): State<ApiState>,
    Extension(context): Extension<AuthContext>,
    Json(payload): Json<ConsultationRequest>,
) -> Result<(StatusCode, Json<ConsultationResponse>), (StatusCode, Json<ApiErrorResponse>)> {
    authorize_practitioner_access(&context)?;

    let consultation = state
        .services
        .clinical_service
        .create_consultation(consultation_request_to_new_data(payload), context.user_id)
        .await
        .map_err(clinical_service_error_to_response)?;

    let consultation_id = consultation.id;
    let audit_entry = AuditEntry::new_created(
        "consultation",
        consultation_id,
        serde_json::to_string(&consultation).unwrap_or_default(),
        context.user_id,
    );
    emit_audit_event_non_blocking(state.audit_emitter.clone(), audit_entry);

    Ok((
        StatusCode::CREATED,
        Json(consultation_to_response(consultation)),
    ))
}

pub(super) async fn update_consultation(
    State(state): State<ApiState>,
    Extension(context): Extension<AuthContext>,
    Path(id): Path<Uuid>,
    Json(payload): Json<ConsultationRequest>,
) -> Result<(StatusCode, Json<ConsultationResponse>), (StatusCode, Json<ApiErrorResponse>)> {
    authorize_practitioner_access(&context)?;
    let expected_version = payload.version;

    let consultation = state
        .services
        .clinical_service
        .update_clinical_notes(
            id,
            payload.reason,
            payload.clinical_notes,
            expected_version,
            context.user_id,
        )
        .await
        .map_err(clinical_service_error_to_response)?;

    let audit_entry = AuditEntry::new_updated(
        "consultation",
        id,
        "{}",
        serde_json::to_string(&consultation).unwrap_or_default(),
        context.user_id,
    );
    emit_audit_event_non_blocking(state.audit_emitter.clone(), audit_entry);

    Ok((StatusCode::OK, Json(consultation_to_response(consultation))))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn consultation_list_query_defaults_to_page_1_limit_25() {
        let query = ConsultationListQuery {
            patient_id: Some(Uuid::new_v4()),
            page: None,
            limit: None,
        };

        let page = query.page.unwrap_or(1).max(1);
        let limit = query.limit.unwrap_or(25).clamp(1, 100);

        assert_eq!(page, 1);
        assert_eq!(limit, 25);
    }

    #[test]
    fn consultation_list_query_clamps_limit_to_100_maximum() {
        let query = ConsultationListQuery {
            patient_id: Some(Uuid::new_v4()),
            page: Some(2),
            limit: Some(200),
        };

        let limit = query.limit.unwrap_or(25).clamp(1, 100);

        assert_eq!(limit, 100);
    }
}

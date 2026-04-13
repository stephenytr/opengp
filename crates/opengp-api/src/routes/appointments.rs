use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Extension, Json,
};
use chrono::{TimeZone, Utc};
use opengp_domain::domain::api::{
    ApiErrorResponse, AppointmentRequest, AppointmentResponse, PaginatedResponse,
};
use opengp_domain::domain::appointment::AppointmentSearchCriteria;
use opengp_domain::domain::audit::AuditEntry;
use uuid::Uuid;

use crate::ApiState;

use super::middleware::{
    appointment_request_to_new_data, appointment_request_to_update_data,
    appointment_service_error_to_response, appointment_to_response, authorize_practitioner_access,
    authorize_practitioner_write, authorize_read, bad_request_response,
    emit_audit_event_non_blocking, not_found_response, validate_appointment_booking_time,
    AppointmentAvailabilityQuery, AppointmentListQuery, AppointmentStatusActionRequest,
    AuthContext,
};

pub(super) async fn list_appointments(
    State(state): State<ApiState>,
    Extension(context): Extension<AuthContext>,
    Query(query): Query<AppointmentListQuery>,
) -> Result<
    (StatusCode, Json<PaginatedResponse<AppointmentResponse>>),
    (StatusCode, Json<ApiErrorResponse>),
> {
    authorize_read(&context)?;

    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(25).clamp(1, 100);
    let repository_limit = i64::from(page.saturating_mul(limit)).max(100);
    let (date_from, date_to) =
        if let Some(date) = query.date {
            (
                date.and_hms_opt(0, 0, 0).map(|dt| Utc.from_utc_datetime(&dt)),
                date.and_hms_opt(23, 59, 59).map(|dt| Utc.from_utc_datetime(&dt)),
            )
        } else {
            (query.date_from, query.date_to)
        };

    let criteria = AppointmentSearchCriteria {
        patient_id: None,
        practitioner_id: query.practitioner_id,
        date_from,
        date_to,
        status: None,
        appointment_type: None,
        is_urgent: None,
        confirmed: None,
        limit: Some(repository_limit),
    };

    let appointments = state
        .services
        .appointment_service
        .search_appointments(&criteria)
        .await
        .map_err(appointment_service_error_to_response)?;

    let total = appointments.len() as u64;
    let offset = ((page - 1) * limit) as usize;
    let data = appointments
        .into_iter()
        .skip(offset)
        .take(limit as usize)
        .map(appointment_to_response)
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

pub(super) async fn get_appointment(
    State(state): State<ApiState>,
    Extension(context): Extension<AuthContext>,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<AppointmentResponse>), (StatusCode, Json<ApiErrorResponse>)> {
    authorize_read(&context)?;

    let appointment = state
        .services
        .appointment_service
        .find_appointment(id)
        .await
        .map_err(appointment_service_error_to_response)?
        .ok_or_else(|| not_found_response("appointment_not_found", "Appointment not found"))?;

    Ok((StatusCode::OK, Json(appointment_to_response(appointment))))
}

pub(super) async fn get_available_slots(
    State(state): State<ApiState>,
    Extension(context): Extension<AuthContext>,
    Query(query): Query<AppointmentAvailabilityQuery>,
) -> Result<(StatusCode, Json<Vec<String>>), (StatusCode, Json<ApiErrorResponse>)> {
    authorize_read(&context)?;

    if query.duration <= 0 {
        return Err(bad_request_response(
            "validation_error",
            "duration must be greater than zero",
        ));
    }

    let slots = state
        .services
        .availability_service
        .get_available_slots(query.practitioner_id, query.date, query.duration)
        .await
        .map_err(appointment_service_error_to_response)?;

    let response_slots = slots
        .into_iter()
        .map(|time| time.format("%H:%M:%S").to_string())
        .collect();

    Ok((StatusCode::OK, Json(response_slots)))
}

pub(super) async fn create_appointment(
    State(state): State<ApiState>,
    Extension(context): Extension<AuthContext>,
    Json(payload): Json<AppointmentRequest>,
) -> Result<(StatusCode, Json<AppointmentResponse>), (StatusCode, Json<ApiErrorResponse>)> {
    authorize_practitioner_write(&context)?;
    validate_appointment_booking_time(payload.start_time)?;

    let appointment = state
        .services
        .appointment_service
        .create_appointment(appointment_request_to_new_data(payload)?, context.user_id)
        .await
        .map_err(appointment_service_error_to_response)?;

    let appointment_id = appointment.id;
    let audit_entry = AuditEntry::new_created(
        "appointment",
        appointment_id,
        serde_json::to_string(&appointment).unwrap_or_default(),
        context.user_id,
    );
    emit_audit_event_non_blocking(state.audit_emitter.clone(), audit_entry);

    Ok((
        StatusCode::CREATED,
        Json(appointment_to_response(appointment)),
    ))
}

pub(super) async fn update_appointment(
    State(state): State<ApiState>,
    Extension(context): Extension<AuthContext>,
    Path(id): Path<Uuid>,
    Json(payload): Json<AppointmentRequest>,
) -> Result<(StatusCode, Json<AppointmentResponse>), (StatusCode, Json<ApiErrorResponse>)> {
    authorize_practitioner_write(&context)?;
    validate_appointment_booking_time(payload.start_time)?;
    let expected_version = payload.version;

    let appointment = state
        .services
        .appointment_service
        .update_appointment(
            id,
            appointment_request_to_update_data(payload)?,
            expected_version,
            context.user_id,
        )
        .await
        .map_err(appointment_service_error_to_response)?;

    let audit_entry = AuditEntry::new_updated(
        "appointment",
        id,
        "{}",
        serde_json::to_string(&appointment).unwrap_or_default(),
        context.user_id,
    );
    emit_audit_event_non_blocking(state.audit_emitter.clone(), audit_entry);

    Ok((StatusCode::OK, Json(appointment_to_response(appointment))))
}

pub(super) async fn cancel_appointment(
    State(state): State<ApiState>,
    Extension(context): Extension<AuthContext>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<ApiErrorResponse>)> {
    authorize_practitioner_write(&context)?;

    state
        .services
        .appointment_service
        .cancel_appointment(id, "Cancelled via API".to_string(), context.user_id)
        .await
        .map_err(appointment_service_error_to_response)?;

    let audit_entry =
        AuditEntry::new_cancelled("appointment", id, "Cancelled via API", context.user_id);
    emit_audit_event_non_blocking(state.audit_emitter.clone(), audit_entry);

    Ok(StatusCode::NO_CONTENT)
}

pub(super) async fn update_appointment_status(
    State(state): State<ApiState>,
    Extension(context): Extension<AuthContext>,
    Path(id): Path<Uuid>,
    Json(payload): Json<AppointmentStatusActionRequest>,
) -> Result<(StatusCode, Json<AppointmentResponse>), (StatusCode, Json<ApiErrorResponse>)> {
    authorize_practitioner_access(&context)?;

    use opengp_domain::domain::appointment::AppointmentStatus;

    let action = payload.action.trim().to_ascii_lowercase();
    let new_status = match action.as_str() {
        "scheduled" => AppointmentStatus::Scheduled,
        "confirmed" => AppointmentStatus::Confirmed,
        "arrived" => AppointmentStatus::Arrived,
        "in_progress" => AppointmentStatus::InProgress,
        "billing" => AppointmentStatus::Billing,
        "completed" => AppointmentStatus::Completed,
        "cancelled" => AppointmentStatus::Cancelled,
        "no_show" => AppointmentStatus::NoShow,
        "rescheduled" => AppointmentStatus::Rescheduled,
        _ => {
            return Err(bad_request_response(
                "validation_error",
                "Invalid appointment status",
            ));
        }
    };

    let appointment = state
        .services
        .appointment_service
        .set_status(id, new_status, context.user_id)
        .await
        .map_err(appointment_service_error_to_response)?;

    Ok((StatusCode::OK, Json(appointment_to_response(appointment))))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn appointment_list_query_defaults_to_page_1_limit_25() {
        let query = AppointmentListQuery {
            page: None,
            limit: None,
            date: None,
            date_from: None,
            date_to: None,
            practitioner_id: None,
        };

        let page = query.page.unwrap_or(1).max(1);
        let limit = query.limit.unwrap_or(25).clamp(1, 100);

        assert_eq!(page, 1);
        assert_eq!(limit, 25);
    }

    #[test]
    fn appointment_availability_query_rejects_zero_duration() {
        let query = AppointmentAvailabilityQuery {
            practitioner_id: Uuid::new_v4(),
            date: chrono::Local::now().date_naive(),
            duration: 0,
        };

        assert!(query.duration <= 0);
    }

    #[test]
    fn appointment_availability_query_accepts_positive_duration() {
        let query = AppointmentAvailabilityQuery {
            practitioner_id: Uuid::new_v4(),
            date: chrono::Local::now().date_naive(),
            duration: 30,
        };

        assert!(query.duration > 0);
    }

    #[test]
    fn appointment_status_action_normalizes_to_lowercase() {
        let action = "ARRIVED";
        let normalized = action.trim().to_ascii_lowercase();

        assert_eq!(normalized, "arrived");
    }
}

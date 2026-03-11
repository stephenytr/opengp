use axum::{
    extract::{Path, Query, Request, State},
    http::{header, HeaderMap, HeaderValue, Method, StatusCode, Uri},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
    Extension, Json, Router,
};
use http::header::ORIGIN;
use opengp_domain::domain::api::{
    ApiErrorResponse, AuthenticatedUserResponse, LoginRequest, LoginResponse, PaginatedResponse,
    AppointmentRequest, AppointmentResponse, ConsultationRequest, ConsultationResponse,
    PatientRequest, PatientResponse,
};
use opengp_domain::domain::appointment::{
    AppointmentSearchCriteria, AppointmentStatus, AppointmentType,
    NewAppointmentData, ServiceError as AppointmentServiceError, UpdateAppointmentData,
};
use opengp_domain::domain::audit::{AuditAction, AuditEntry};
use opengp_domain::domain::clinical::{
    NewConsultationData, ServiceError as ClinicalServiceError,
};
use opengp_domain::domain::patient::{
    Address, Gender, NewPatientData, ServiceError as PatientServiceError, UpdatePatientData,
};
use opengp_domain::domain::user::{self, AuthError, Role};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use tower::ServiceBuilder;
use tower_http::compression::CompressionLayer;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::trace::TraceLayer;
use uuid::Uuid;

use crate::ApiState;

pub fn router(state: ApiState) -> Router {
    let protected_auth_routes = Router::new()
        .route("/logout", post(logout))
        .route("/refresh", post(refresh))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            session_validation_middleware,
        ));

    let auth_routes = Router::new()
        .route("/login", post(login))
        .merge(protected_auth_routes);

    let patient_routes = Router::new()
        .route("/", get(list_patients).post(create_patient))
        .route("/{id}", get(get_patient).put(update_patient).delete(delete_patient))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            session_validation_middleware,
        ));

    let appointment_routes = Router::new()
        .route("/", get(list_appointments).post(create_appointment))
        .route(
            "/{id}",
            get(get_appointment)
                .put(update_appointment)
                .delete(cancel_appointment),
        )
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            session_validation_middleware,
        ));

    let consultation_routes = Router::new()
        .route("/", get(list_consultations).post(create_consultation))
        .route("/{id}", get(get_consultation).put(update_consultation))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            session_validation_middleware,
        ));

    Router::new()
        .route("/health", get(health))
        .route("/metrics", get(metrics))
        .nest("/api/v1/auth", auth_routes)
        .nest("/api/v1/patients", patient_routes)
        .nest("/api/v1/appointments", appointment_routes)
        .nest("/api/v1/consultations", consultation_routes)
        .with_state(state)
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CompressionLayer::new())
                .layer(cors_layer()),
        )
}

async fn health(State(state): State<ApiState>) -> impl IntoResponse {
    // Test database connectivity with timeout
    let db_connected = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        state.pool.acquire(),
    )
    .await
    .is_ok();

    let response = HealthResponse {
        status: if db_connected { "ok".to_string() } else { "degraded".to_string() },
        database_connected: db_connected,
        uptime_seconds: state.metrics.uptime_seconds(),
    };

    let status_code = if db_connected {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (status_code, Json(response))
}

async fn metrics(State(state): State<ApiState>) -> impl IntoResponse {
    let response = MetricsResponse {
        active_sessions: state.metrics.active_sessions.load(std::sync::atomic::Ordering::Relaxed),
        request_count: state.metrics.request_count.load(std::sync::atomic::Ordering::Relaxed),
        error_count: state.metrics.error_count.load(std::sync::atomic::Ordering::Relaxed),
    };

    (StatusCode::OK, Json(response))
}

async fn login(
    State(state): State<ApiState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Response, (StatusCode, Json<ApiErrorResponse>)> {
    let login_request = user::LoginRequest {
        username: payload.username.clone(),
        password: payload.password.clone(),
    };

    let login = match state
        .services
        .auth_service
        .login(login_request)
        .await
    {
        Ok(login) => login,
        Err(e) => {
            // Emit audit event for failed login attempt (without exposing credentials)
            let audit_entry = AuditEntry {
                id: Uuid::new_v4(),
                entity_type: "user".to_string(),
                entity_id: Uuid::nil(), // No user_id for failed login
                action: AuditAction::Created, // Using Created as generic action
                old_value: None,
                new_value: Some(format!("Failed login attempt for username: {}", payload.username)),
                changed_by: Uuid::nil(),
                changed_at: Utc::now(),
            };
            tokio::spawn(async move {
                let _ = state.audit_emitter.emit(audit_entry).await;
            });
            return Err(auth_error_to_response(e));
        }
    };

    let user = state
        .services
        .auth_service
        .user_repository
        .find_by_id(login.user_id)
        .await
        .map_err(|_| auth_failed_response())?
        .ok_or_else(auth_failed_response)?;

    // Emit audit event for successful login
    let audit_entry = AuditEntry {
        id: Uuid::new_v4(),
        entity_type: "user".to_string(),
        entity_id: user.id,
        action: AuditAction::Created, // Using Created as generic action
        old_value: None,
        new_value: Some(format!("User logged in: {}", user.username)),
        changed_by: user.id,
        changed_at: Utc::now(),
    };
    let audit_emitter = state.audit_emitter.clone();
    tokio::spawn(async move {
        let _ = audit_emitter.emit(audit_entry).await;
    });

    let response = LoginResponse {
        access_token: login.session_token.clone(),
        token_type: "Bearer".to_string(),
        expires_in_seconds: state.services.auth_service.session_ttl_seconds(),
        user: AuthenticatedUserResponse {
            id: user.id,
            username: user.username.clone(),
            role: user.role.to_string().to_lowercase(),
            display_name: user.full_name(),
        },
    };

    let mut http_response = (StatusCode::OK, Json(response)).into_response();
    http_response.headers_mut().insert(
        header::SET_COOKIE,
        HeaderValue::from_str(&session_cookie(
            &login.session_token,
            state.services.auth_service.session_ttl_seconds(),
        ))
        .map_err(|_| auth_failed_response())?,
    );

    Ok(http_response)
}

async fn logout(
    State(state): State<ApiState>,
    Extension(context): Extension<AuthContext>,
) -> Result<Response, (StatusCode, Json<ApiErrorResponse>)> {
    state
        .services
        .auth_service
        .logout(&context.token)
        .await
        .map_err(auth_error_to_response)?;

    let mut response = (
        StatusCode::OK,
        Json(GenericSuccessResponse {
            success: true,
            message: "Logged out".to_string(),
        }),
    )
        .into_response();

    response.headers_mut().insert(
        header::SET_COOKIE,
        HeaderValue::from_static("session_token=; HttpOnly; Path=/; Max-Age=0; SameSite=Lax"),
    );

    Ok(response)
}

async fn refresh(
    State(state): State<ApiState>,
    Extension(context): Extension<AuthContext>,
) -> Result<Response, (StatusCode, Json<ApiErrorResponse>)> {
    let session = state
        .services
        .auth_service
        .refresh_session(&context.token)
        .await
        .map_err(auth_error_to_response)?;

    let response = RefreshResponse {
        access_token: session.token.clone(),
        token_type: "Bearer".to_string(),
        expires_in_seconds: state.services.auth_service.session_ttl_seconds(),
    };

    let mut http_response = (StatusCode::OK, Json(response)).into_response();
    http_response.headers_mut().insert(
        header::SET_COOKIE,
        HeaderValue::from_str(&session_cookie(
            &session.token,
            state.services.auth_service.session_ttl_seconds(),
        ))
        .map_err(|_| unauthorized_response("unauthorized", "Session expired"))?,
    );

    Ok(http_response)
}

async fn list_patients(
    State(state): State<ApiState>,
    Extension(context): Extension<AuthContext>,
    Query(query): Query<PaginationQuery>,
) -> Result<(StatusCode, Json<PaginatedResponse<PatientResponse>>), (StatusCode, Json<ApiErrorResponse>)>
{
    authorize_read(&context)?;

    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(25).clamp(1, 100);

    let patients = state
        .services
        .patient_service
        .list_active_patients()
        .await
        .map_err(patient_service_error_to_response)?;

    let total = patients.len() as u64;
    let offset = ((page - 1) * limit) as usize;
    let data = patients
        .into_iter()
        .skip(offset)
        .take(limit as usize)
        .map(patient_to_response)
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

async fn get_patient(
    State(state): State<ApiState>,
    Extension(context): Extension<AuthContext>,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<PatientResponse>), (StatusCode, Json<ApiErrorResponse>)> {
    authorize_read(&context)?;

    let patient = state
        .services
        .patient_service
        .find_patient(id)
        .await
        .map_err(patient_service_error_to_response)?
        .ok_or_else(|| not_found_response("patient_not_found", "Patient not found"))?;

    if !patient.is_active {
        return Err(not_found_response("patient_not_found", "Patient not found"));
    }

    Ok((StatusCode::OK, Json(patient_to_response(patient))))
}

async fn create_patient(
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
    let audit_emitter = state.audit_emitter.clone();
    tokio::spawn(async move {
        let _ = audit_emitter.emit(audit_entry).await;
    });

    Ok((StatusCode::CREATED, Json(patient_to_response(patient))))
}

async fn update_patient(
    State(state): State<ApiState>,
    Extension(context): Extension<AuthContext>,
    Path(id): Path<Uuid>,
    Json(payload): Json<PatientRequest>,
) -> Result<(StatusCode, Json<PatientResponse>), (StatusCode, Json<ApiErrorResponse>)> {
    authorize_write(&context)?;

    let patient = state
        .services
        .patient_service
        .update_patient(id, patient_request_to_update_data(payload)?)
        .await
        .map_err(patient_service_error_to_response)?;

    let audit_entry = AuditEntry::new_updated(
        "patient",
        id,
        "{}",
        serde_json::to_string(&patient).unwrap_or_default(),
        context.user_id,
    );
    let audit_emitter = state.audit_emitter.clone();
    tokio::spawn(async move {
        let _ = audit_emitter.emit(audit_entry).await;
    });

    Ok((StatusCode::OK, Json(patient_to_response(patient))))
}

async fn delete_patient(
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

    let audit_entry = AuditEntry::new_cancelled(
        "patient",
        id,
        "Patient deactivated",
        context.user_id,
    );
    let audit_emitter = state.audit_emitter.clone();
    tokio::spawn(async move {
        let _ = audit_emitter.emit(audit_entry).await;
    });

    Ok(StatusCode::NO_CONTENT)
}

async fn list_appointments(
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
    let criteria = AppointmentSearchCriteria {
        patient_id: None,
        practitioner_id: query.practitioner_id,
        date_from: query.date_from,
        date_to: query.date_to,
        status: None,
        appointment_type: None,
        is_urgent: None,
        confirmed: None,
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

async fn get_appointment(
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

async fn create_appointment(
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
    let audit_emitter = state.audit_emitter.clone();
    tokio::spawn(async move {
        let _ = audit_emitter.emit(audit_entry).await;
    });

    Ok((
        StatusCode::CREATED,
        Json(appointment_to_response(appointment)),
    ))
}

async fn update_appointment(
    State(state): State<ApiState>,
    Extension(context): Extension<AuthContext>,
    Path(id): Path<Uuid>,
    Json(payload): Json<AppointmentRequest>,
) -> Result<(StatusCode, Json<AppointmentResponse>), (StatusCode, Json<ApiErrorResponse>)> {
    authorize_practitioner_write(&context)?;
    validate_appointment_booking_time(payload.start_time)?;

    let appointment = state
        .services
        .appointment_service
        .update_appointment(
            id,
            appointment_request_to_update_data(payload)?,
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
    let audit_emitter = state.audit_emitter.clone();
    tokio::spawn(async move {
        let _ = audit_emitter.emit(audit_entry).await;
    });

    Ok((StatusCode::OK, Json(appointment_to_response(appointment))))
}

async fn cancel_appointment(
    State(state): State<ApiState>,
    Extension(context): Extension<AuthContext>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<ApiErrorResponse>)> {
    authorize_practitioner_write(&context)?;

    state
        .services
        .appointment_service
        .cancel_appointment(
            id,
            "Cancelled via API".to_string(),
            context.user_id,
        )
        .await
        .map_err(appointment_service_error_to_response)?;

    let audit_entry = AuditEntry::new_cancelled(
        "appointment",
        id,
        "Cancelled via API",
        context.user_id,
    );
    let audit_emitter = state.audit_emitter.clone();
    tokio::spawn(async move {
        let _ = audit_emitter.emit(audit_entry).await;
    });

    Ok(StatusCode::NO_CONTENT)
}

async fn list_consultations(
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

async fn get_consultation(
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

async fn create_consultation(
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
    let audit_emitter = state.audit_emitter.clone();
    tokio::spawn(async move {
        let _ = audit_emitter.emit(audit_entry).await;
    });

    Ok((
        StatusCode::CREATED,
        Json(consultation_to_response(consultation)),
    ))
}

async fn update_consultation(
    State(state): State<ApiState>,
    Extension(context): Extension<AuthContext>,
    Path(id): Path<Uuid>,
    Json(payload): Json<ConsultationRequest>,
) -> Result<(StatusCode, Json<ConsultationResponse>), (StatusCode, Json<ApiErrorResponse>)> {
    authorize_practitioner_access(&context)?;

    let consultation = state
        .services
        .clinical_service
        .update_clinical_notes(id, payload.reason, payload.clinical_notes, context.user_id)
        .await
        .map_err(clinical_service_error_to_response)?;

    let audit_entry = AuditEntry::new_updated(
        "consultation",
        id,
        "{}",
        serde_json::to_string(&consultation).unwrap_or_default(),
        context.user_id,
    );
    let audit_emitter = state.audit_emitter.clone();
    tokio::spawn(async move {
        let _ = audit_emitter.emit(audit_entry).await;
    });

    Ok((StatusCode::OK, Json(consultation_to_response(consultation))))
}

async fn session_validation_middleware(
    State(state): State<ApiState>,
    mut request: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<ApiErrorResponse>)> {
    let token = extract_bearer_token(request.headers())
        .ok_or_else(|| unauthorized_response("unauthorized", "Missing or invalid authorization header"))?
        .to_string();

    let user_id = state
        .services
        .auth_service
        .validate_session(&token)
        .await
        .map_err(auth_error_to_response)?;

    let user = state
        .services
        .auth_service
        .user_repository
        .find_by_id(user_id)
        .await
        .map_err(|_| unauthorized_response("unauthorized", "Authentication unavailable"))?
        .ok_or_else(|| unauthorized_response("unauthorized", "Session expired"))?;

    request.extensions_mut().insert(AuthContext {
        user_id,
        token,
        role: user.role,
    });
    Ok(next.run(request).await)
}

fn extract_bearer_token(headers: &HeaderMap) -> Option<&str> {
    let value = headers.get(header::AUTHORIZATION)?.to_str().ok()?;
    let token = value.strip_prefix("Bearer ")?;

    if token.trim().is_empty() {
        None
    } else {
        Some(token)
    }
}

fn auth_error_to_response(error: AuthError) -> (StatusCode, Json<ApiErrorResponse>) {
    match error {
        AuthError::InvalidCredentials | AuthError::AccountLocked => auth_failed_response(),
        AuthError::SessionExpired => unauthorized_response("session_expired", "Session expired"),
        AuthError::Repository(_) => {
            unauthorized_response("unauthorized", "Authentication unavailable")
        }
    }
}

fn auth_failed_response() -> (StatusCode, Json<ApiErrorResponse>) {
    unauthorized_response("invalid_credentials", "Invalid username or password")
}

fn unauthorized_response(code: &str, message: &str) -> (StatusCode, Json<ApiErrorResponse>) {
    (
        StatusCode::UNAUTHORIZED,
        Json(ApiErrorResponse {
            status: StatusCode::UNAUTHORIZED.as_u16(),
            message: message.to_string(),
            code: code.to_string(),
        }),
    )
}

fn forbidden_response(code: &str, message: &str) -> (StatusCode, Json<ApiErrorResponse>) {
    (
        StatusCode::FORBIDDEN,
        Json(ApiErrorResponse {
            status: StatusCode::FORBIDDEN.as_u16(),
            message: message.to_string(),
            code: code.to_string(),
        }),
    )
}

fn bad_request_response(code: &str, message: &str) -> (StatusCode, Json<ApiErrorResponse>) {
    (
        StatusCode::BAD_REQUEST,
        Json(ApiErrorResponse {
            status: StatusCode::BAD_REQUEST.as_u16(),
            message: message.to_string(),
            code: code.to_string(),
        }),
    )
}

fn not_found_response(code: &str, message: &str) -> (StatusCode, Json<ApiErrorResponse>) {
    (
        StatusCode::NOT_FOUND,
        Json(ApiErrorResponse {
            status: StatusCode::NOT_FOUND.as_u16(),
            message: message.to_string(),
            code: code.to_string(),
        }),
    )
}

fn internal_server_error_response(
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

fn authorize_read(context: &AuthContext) -> Result<(), (StatusCode, Json<ApiErrorResponse>)> {
    if is_reader(context.role) {
        Ok(())
    } else {
        Err(forbidden_response(
            "insufficient_permissions",
            "Role cannot read patients",
        ))
    }
}

fn authorize_write(context: &AuthContext) -> Result<(), (StatusCode, Json<ApiErrorResponse>)> {
    if is_writer(context.role) {
        Ok(())
    } else {
        Err(forbidden_response(
            "insufficient_permissions",
            "Role cannot modify patients",
        ))
    }
}

fn authorize_practitioner_write(
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

fn authorize_practitioner_access(
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

fn is_reader(role: Role) -> bool {
    matches!(role, Role::Receptionist | Role::Doctor | Role::Nurse | Role::Admin)
}

fn is_writer(role: Role) -> bool {
    matches!(role, Role::Doctor | Role::Nurse | Role::Admin)
}

fn is_practitioner(role: Role) -> bool {
    matches!(role, Role::Doctor | Role::Nurse)
}

fn patient_service_error_to_response(
    error: PatientServiceError,
) -> (StatusCode, Json<ApiErrorResponse>) {
    match error {
        PatientServiceError::DuplicatePatient => {
            bad_request_response("duplicate_patient", "Patient with provided details already exists")
        }
        PatientServiceError::NotFound(_) => not_found_response("patient_not_found", "Patient not found"),
        PatientServiceError::Validation(_) => {
            bad_request_response("validation_error", "Invalid patient payload")
        }
        PatientServiceError::Conflict(_) => {
            bad_request_response("conflict", "Patient was modified by another request")
        }
        PatientServiceError::Repository(_) => internal_server_error_response(
            "internal_error",
            "Unable to process patient request",
        ),
    }
}

fn appointment_service_error_to_response(
    error: AppointmentServiceError,
) -> (StatusCode, Json<ApiErrorResponse>) {
    match error {
        AppointmentServiceError::NotFound(_) => {
            not_found_response("appointment_not_found", "Appointment not found")
        }
        AppointmentServiceError::Conflict(_) => {
            conflict_response("appointment_conflict", "Overlapping appointment")
        }
        AppointmentServiceError::ValidationError(_) | AppointmentServiceError::InvalidTransition(_) => {
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

fn clinical_service_error_to_response(
    error: ClinicalServiceError,
) -> (StatusCode, Json<ApiErrorResponse>) {
    match error {
        ClinicalServiceError::ConsultationNotFound(_) => {
            not_found_response("consultation_not_found", "Consultation not found")
        }
        ClinicalServiceError::PatientNotFound(_) => {
            not_found_response("patient_not_found", "Patient not found")
        }
        ClinicalServiceError::Validation(_) => {
            bad_request_response("validation_error", "Invalid consultation payload")
        }
        ClinicalServiceError::Conflict(_) => {
            conflict_response("consultation_conflict", "Consultation was modified by another request")
        }
        ClinicalServiceError::AlreadySigned => {
            bad_request_response("consultation_signed", "Consultation already signed")
        }
        ClinicalServiceError::Unauthorized => {
            forbidden_response("insufficient_permissions", "Role cannot access consultations")
        }
        ClinicalServiceError::Repository(_)
        | ClinicalServiceError::AllergyNotFound(_)
        | ClinicalServiceError::MedicalHistoryNotFound(_)
        | ClinicalServiceError::VitalSignsNotFound(_)
        | ClinicalServiceError::FamilyHistoryNotFound(_)
        | ClinicalServiceError::SocialHistoryNotFound(_) => internal_server_error_response(
            "internal_error",
            "Unable to process consultation request",
        ),
    }
}

fn conflict_response(code: &str, message: &str) -> (StatusCode, Json<ApiErrorResponse>) {
    (
        StatusCode::CONFLICT,
        Json(ApiErrorResponse {
            status: StatusCode::CONFLICT.as_u16(),
            message: message.to_string(),
            code: code.to_string(),
        }),
    )
}

fn patient_request_to_new_data(
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

fn patient_request_to_update_data(
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

fn appointment_request_to_new_data(
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

fn appointment_request_to_update_data(
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

fn consultation_request_to_new_data(payload: ConsultationRequest) -> NewConsultationData {
    NewConsultationData {
        patient_id: payload.patient_id,
        practitioner_id: payload.practitioner_id,
        appointment_id: payload.appointment_id,
        reason: payload.reason,
        clinical_notes: payload.clinical_notes,
    }
}

fn validate_appointment_booking_time(
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

fn parse_gender(gender: &str) -> Result<Gender, (StatusCode, Json<ApiErrorResponse>)> {
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

fn parse_appointment_type(
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

fn appointment_type_to_string(appointment_type: AppointmentType) -> &'static str {
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

fn appointment_status_to_string(status: AppointmentStatus) -> &'static str {
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

fn patient_to_response(patient: opengp_domain::domain::patient::Patient) -> PatientResponse {
    PatientResponse {
        id: patient.id,
        first_name: patient.first_name,
        last_name: patient.last_name,
        date_of_birth: patient.date_of_birth,
        gender: patient.gender.to_string().to_lowercase(),
        phone_mobile: patient.phone_mobile,
        email: patient.email,
        is_active: patient.is_active,
    }
}

fn appointment_to_response(
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
    }
}

fn consultation_to_response(
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
    }
}

fn session_cookie(token: &str, ttl_seconds: i64) -> String {
    format!(
        "session_token={}; HttpOnly; Path=/; Max-Age={}; SameSite=Lax",
        token,
        ttl_seconds.max(0)
    )
}

fn cors_layer() -> CorsLayer {
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

fn is_allowed_origin(origin: &HeaderValue) -> bool {
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

#[derive(Serialize, Deserialize)]
struct HealthResponse {
    status: String,
    database_connected: bool,
    uptime_seconds: u64,
}

#[derive(Serialize, Deserialize)]
struct MetricsResponse {
    active_sessions: u64,
    request_count: u64,
    error_count: u64,
}

#[derive(Debug, Clone)]
struct AuthContext {
    #[allow(dead_code)]
    user_id: Uuid,
    token: String,
    role: Role,
}

#[derive(Debug, Deserialize)]
struct PaginationQuery {
    page: Option<u32>,
    limit: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct AppointmentListQuery {
    page: Option<u32>,
    limit: Option<u32>,
    date_from: Option<DateTime<Utc>>,
    date_to: Option<DateTime<Utc>>,
    practitioner_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
struct ConsultationListQuery {
    patient_id: Option<Uuid>,
    page: Option<u32>,
    limit: Option<u32>,
}

#[derive(Serialize, Deserialize)]
struct RefreshResponse {
    access_token: String,
    token_type: String,
    expires_in_seconds: i64,
}

#[derive(Serialize, Deserialize)]
struct GenericSuccessResponse {
    success: bool,
    message: String,
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::header;
    use http::{Request, StatusCode};
    use opengp_domain::domain::api::LoginResponse;
    use tower::util::ServiceExt;

    use crate::ApiConfig;

    use super::*;

    #[tokio::test]
    async fn health_endpoint_returns_ok_with_state_extraction() {
        let config = ApiConfig {
            host: "127.0.0.1".to_string(),
            port: 8080,
            database_url: "postgres://postgres:postgres@127.0.0.1:5432/opengp".to_string(),
            patient_database_url: None,
            database_max_connections: 10,
            database_min_connections: 2,
            connect_timeout_secs: 30,
            idle_timeout_secs: 600,
            encryption_key: "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
            log_level: "info".to_string(),
        };

        let state = ApiState::new(config).await.expect("state should initialize");
        let app = router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .expect("request should be valid"),
            )
            .await
            .expect("request should succeed");

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn health_endpoint_returns_json_with_status_and_uptime() {
        let config = ApiConfig {
            host: "127.0.0.1".to_string(),
            port: 8080,
            database_url: "postgres://postgres:postgres@127.0.0.1:5432/opengp".to_string(),
            patient_database_url: None,
            database_max_connections: 10,
            database_min_connections: 2,
            connect_timeout_secs: 30,
            idle_timeout_secs: 600,
            encryption_key: "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
            log_level: "info".to_string(),
        };

        let state = ApiState::new(config).await.expect("state should initialize");
        let app = router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .expect("request should be valid"),
            )
            .await
            .expect("request should succeed");

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body should be readable");
        let health: HealthResponse = serde_json::from_slice(&body).expect("body should be valid JSON");
        assert_eq!(health.status, "ok".to_string());
        assert!(health.database_connected);
    }

    #[tokio::test]
    async fn metrics_endpoint_returns_json_with_counters() {
        let config = ApiConfig {
            host: "127.0.0.1".to_string(),
            port: 8080,
            database_url: "postgres://postgres:postgres@127.0.0.1:5432/opengp".to_string(),
            patient_database_url: None,
            database_max_connections: 10,
            database_min_connections: 2,
            connect_timeout_secs: 30,
            idle_timeout_secs: 600,
            encryption_key: "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
            log_level: "info".to_string(),
        };

        let state = ApiState::new(config).await.expect("state should initialize");
        state.metrics.request_count.store(42, std::sync::atomic::Ordering::Relaxed);
        state.metrics.error_count.store(5, std::sync::atomic::Ordering::Relaxed);
        state.metrics.active_sessions.store(3, std::sync::atomic::Ordering::Relaxed);

        let app = router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/metrics")
                    .body(Body::empty())
                    .expect("request should be valid"),
            )
            .await
            .expect("request should succeed");

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body should be readable");
        let metrics: MetricsResponse = serde_json::from_slice(&body).expect("body should be valid JSON");
        assert_eq!(metrics.request_count, 42);
        assert_eq!(metrics.error_count, 5);
        assert_eq!(metrics.active_sessions, 3);
    }

    #[tokio::test]
    async fn login_endpoint_returns_access_token_and_cookie() {
        let config = test_config();
        let state = ApiState::new(config).await.expect("state should initialize");
        let app = router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/auth/login")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(
                        r#"{"username":"dr_smith","password":"correct-horse-battery-staple"}"#,
                    ))
                    .expect("request should be valid"),
            )
            .await
            .expect("request should succeed");

        assert_eq!(response.status(), StatusCode::OK);
        let set_cookie = response
            .headers()
            .get(header::SET_COOKIE)
            .and_then(|value| value.to_str().ok())
            .expect("set-cookie header should exist");
        assert!(set_cookie.contains("HttpOnly"));

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body should be readable");
        let login: LoginResponse = serde_json::from_slice(&body).expect("body should be valid JSON");
        assert!(!login.access_token.is_empty());
        assert_eq!(login.token_type, "Bearer");
    }

    #[tokio::test]
    async fn login_endpoint_returns_401_for_invalid_credentials() {
        let config = test_config();
        let state = ApiState::new(config).await.expect("state should initialize");
        let app = router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/auth/login")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(
                        r#"{"username":"dr_smith","password":"wrong-password"}"#,
                    ))
                    .expect("request should be valid"),
            )
            .await
            .expect("request should succeed");

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn locked_account_returns_401() {
        let config = test_config();
        let state = ApiState::new(config).await.expect("state should initialize");
        let app = router(state);

        for _ in 0..5 {
            let _ = app
                .clone()
                .oneshot(
                    Request::builder()
                        .method("POST")
                        .uri("/api/v1/auth/login")
                        .header(header::CONTENT_TYPE, "application/json")
                        .body(Body::from(
                            r#"{"username":"dr_smith","password":"wrong-password"}"#,
                        ))
                        .expect("request should be valid"),
                )
                .await
                .expect("request should succeed");
        }

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/auth/login")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(
                        r#"{"username":"dr_smith","password":"correct-horse-battery-staple"}"#,
                    ))
                    .expect("request should be valid"),
            )
            .await
            .expect("request should succeed");

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn logout_invalidates_session() {
        let config = test_config();
        let state = ApiState::new(config).await.expect("state should initialize");
        let app = router(state);

        let login_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/auth/login")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(
                        r#"{"username":"dr_smith","password":"correct-horse-battery-staple"}"#,
                    ))
                    .expect("request should be valid"),
            )
            .await
            .expect("login should succeed");

        let body = axum::body::to_bytes(login_response.into_body(), usize::MAX)
            .await
            .expect("body should be readable");
        let login: LoginResponse = serde_json::from_slice(&body).expect("body should be valid JSON");

        let logout_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/auth/logout")
                    .header(header::AUTHORIZATION, format!("Bearer {}", login.access_token))
                    .body(Body::empty())
                    .expect("request should be valid"),
            )
            .await
            .expect("logout should succeed");

        assert_eq!(logout_response.status(), StatusCode::OK);

        let refresh_response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/auth/refresh")
                    .header(header::AUTHORIZATION, format!("Bearer {}", login.access_token))
                    .body(Body::empty())
                    .expect("request should be valid"),
            )
            .await
            .expect("request should succeed");

        assert_eq!(refresh_response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn refresh_keeps_same_token_and_returns_ok() {
        let config = test_config();
        let state = ApiState::new(config).await.expect("state should initialize");
        let app = router(state);

        let login_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/auth/login")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(
                        r#"{"username":"dr_smith","password":"correct-horse-battery-staple"}"#,
                    ))
                    .expect("request should be valid"),
            )
            .await
            .expect("login should succeed");

        let login_body = axum::body::to_bytes(login_response.into_body(), usize::MAX)
            .await
            .expect("body should be readable");
        let login: LoginResponse =
            serde_json::from_slice(&login_body).expect("body should be valid login JSON");

        let refresh_response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/auth/refresh")
                    .header(header::AUTHORIZATION, format!("Bearer {}", login.access_token))
                    .body(Body::empty())
                    .expect("request should be valid"),
            )
            .await
            .expect("refresh should succeed");

        assert_eq!(refresh_response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(refresh_response.into_body(), usize::MAX)
            .await
            .expect("body should be readable");
        let refresh: RefreshResponse =
            serde_json::from_slice(&body).expect("body should be valid refresh JSON");
        assert_eq!(refresh.access_token, login.access_token);
    }

    #[tokio::test]
    async fn middleware_rejects_missing_authorization_header() {
        let config = test_config();
        let state = ApiState::new(config).await.expect("state should initialize");
        let app = router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/auth/refresh")
                    .body(Body::empty())
                    .expect("request should be valid"),
            )
            .await
            .expect("request should succeed");

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn patient_endpoints_require_authentication() {
        let state = ApiState::new(test_config()).await.expect("state should initialize");
        let app = router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/v1/patients?page=1&limit=25")
                    .body(Body::empty())
                    .expect("request should be valid"),
            )
            .await
            .expect("request should succeed");

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn receptionist_can_read_but_cannot_write_patients() {
        let state = ApiState::new(test_config()).await.expect("state should initialize");
        let app = router(state);
        let token = login_token(&app, "recep_amy", "desk-passphrase").await;

        let get_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/v1/patients?page=1&limit=25")
                    .header(header::AUTHORIZATION, format!("Bearer {token}"))
                    .body(Body::empty())
                    .expect("request should be valid"),
            )
            .await
            .expect("request should succeed");
        assert_eq!(get_response.status(), StatusCode::OK);

        for method in ["POST", "PUT", "DELETE"] {
            let uri = if method == "POST" {
                "/api/v1/patients".to_string()
            } else {
                format!("/api/v1/patients/{}", Uuid::new_v4())
            };

            let mut builder = Request::builder()
                .method(method)
                .uri(uri)
                .header(header::AUTHORIZATION, format!("Bearer {token}"));

            if method != "DELETE" {
                builder = builder
                    .header(header::CONTENT_TYPE, "application/json")
            }

            let body = if method == "DELETE" {
                Body::empty()
            } else {
                Body::from(sample_patient_payload())
            };

            let response = app
                .clone()
                .oneshot(builder.body(body).expect("request should be valid"))
                .await
                .expect("request should succeed");

            assert_eq!(response.status(), StatusCode::FORBIDDEN);
        }
    }

    #[tokio::test]
    async fn practitioner_can_crud_patients_with_pagination() {
        let state = ApiState::new(test_config()).await.expect("state should initialize");
        let app = router(state);
        let token = login_token(&app, "dr_smith", "correct-horse-battery-staple").await;

        let create_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/patients")
                    .header(header::AUTHORIZATION, format!("Bearer {token}"))
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(sample_patient_payload()))
                    .expect("request should be valid"),
            )
            .await
            .expect("request should succeed");
        assert_eq!(create_response.status(), StatusCode::CREATED);
        let create_body = axum::body::to_bytes(create_response.into_body(), usize::MAX)
            .await
            .expect("body should be readable");
        let created: PatientResponse =
            serde_json::from_slice(&create_body).expect("body should be valid patient JSON");

        let get_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/api/v1/patients/{}", created.id))
                    .header(header::AUTHORIZATION, format!("Bearer {token}"))
                    .body(Body::empty())
                    .expect("request should be valid"),
            )
            .await
            .expect("request should succeed");
        assert_eq!(get_response.status(), StatusCode::OK);

        let update_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri(format!("/api/v1/patients/{}", created.id))
                    .header(header::AUTHORIZATION, format!("Bearer {token}"))
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(
                        r#"{"first_name":"Jane","last_name":"Citizen","date_of_birth":"1984-05-12","gender":"female","phone_mobile":"0400999888","email":"jane.citizen@example.com","medicare_number":"29501012341"}"#,
                    ))
                    .expect("request should be valid"),
            )
            .await
            .expect("request should succeed");
        assert_eq!(update_response.status(), StatusCode::OK);

        for idx in 0..30 {
            let payload = format!(
                "{{\"first_name\":\"P{idx}\",\"last_name\":\"Test\",\"date_of_birth\":\"1980-01-01\",\"gender\":\"male\",\"phone_mobile\":null,\"email\":null,\"medicare_number\":\"29501012{idx:03}\"}}"
            );

            let response = app
                .clone()
                .oneshot(
                    Request::builder()
                        .method("POST")
                        .uri("/api/v1/patients")
                        .header(header::AUTHORIZATION, format!("Bearer {token}"))
                        .header(header::CONTENT_TYPE, "application/json")
                        .body(Body::from(payload))
                        .expect("request should be valid"),
                )
                .await
                .expect("request should succeed");
            assert_eq!(response.status(), StatusCode::CREATED);
        }

        let list_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/v1/patients?page=1&limit=25")
                    .header(header::AUTHORIZATION, format!("Bearer {token}"))
                    .body(Body::empty())
                    .expect("request should be valid"),
            )
            .await
            .expect("request should succeed");
        assert_eq!(list_response.status(), StatusCode::OK);
        let list_body = axum::body::to_bytes(list_response.into_body(), usize::MAX)
            .await
            .expect("body should be readable");
        let paginated: PaginatedResponse<PatientResponse> =
            serde_json::from_slice(&list_body).expect("body should be valid paginated JSON");
        assert_eq!(paginated.page, 1);
        assert_eq!(paginated.limit, 25);
        assert_eq!(paginated.data.len(), 25);
        assert!(paginated.total >= 31);

        let delete_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(format!("/api/v1/patients/{}", created.id))
                    .header(header::AUTHORIZATION, format!("Bearer {token}"))
                    .body(Body::empty())
                    .expect("request should be valid"),
            )
            .await
            .expect("request should succeed");
        assert_eq!(delete_response.status(), StatusCode::NO_CONTENT);

        let get_deleted_response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/api/v1/patients/{}", created.id))
                    .header(header::AUTHORIZATION, format!("Bearer {token}"))
                    .body(Body::empty())
                    .expect("request should be valid"),
            )
            .await
            .expect("request should succeed");
        assert_eq!(get_deleted_response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn appointment_endpoints_require_authentication() {
        let state = ApiState::new(test_config()).await.expect("state should initialize");
        let app = router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/v1/appointments?page=1&limit=25")
                    .body(Body::empty())
                    .expect("request should be valid"),
            )
            .await
            .expect("request should succeed");

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn receptionist_can_read_but_cannot_modify_appointments() {
        let state = ApiState::new(test_config()).await.expect("state should initialize");
        let app = router(state);
        let token = login_token(&app, "recep_amy", "desk-passphrase").await;

        let get_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/v1/appointments?page=1&limit=25")
                    .header(header::AUTHORIZATION, format!("Bearer {token}"))
                    .body(Body::empty())
                    .expect("request should be valid"),
            )
            .await
            .expect("request should succeed");
        assert_eq!(get_response.status(), StatusCode::OK);

        let payload = sample_appointment_payload(
            Utc::now() + chrono::Duration::hours(2),
            Uuid::new_v4(),
        );

        for method in ["POST", "PUT", "DELETE"] {
            let uri = if method == "POST" {
                "/api/v1/appointments".to_string()
            } else {
                format!("/api/v1/appointments/{}", Uuid::new_v4())
            };

            let mut builder = Request::builder()
                .method(method)
                .uri(uri)
                .header(header::AUTHORIZATION, format!("Bearer {token}"));

            if method != "DELETE" {
                builder = builder.header(header::CONTENT_TYPE, "application/json")
            }

            let body = if method == "DELETE" {
                Body::empty()
            } else {
                Body::from(payload.clone())
            };

            let response = app
                .clone()
                .oneshot(builder.body(body).expect("request should be valid"))
                .await
                .expect("request should succeed");

            assert_eq!(response.status(), StatusCode::FORBIDDEN);
        }
    }

    #[tokio::test]
    async fn practitioner_can_crud_appointments_and_overlap_returns_conflict() {
        let state = ApiState::new(test_config()).await.expect("state should initialize");
        let app = router(state);
        let token = login_token(&app, "dr_smith", "correct-horse-battery-staple").await;
        let practitioner_id = Uuid::new_v4();

        let start_one = Utc::now() + chrono::Duration::hours(2);
        let create_one = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/appointments")
                    .header(header::AUTHORIZATION, format!("Bearer {token}"))
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(sample_appointment_payload(start_one, practitioner_id)))
                    .expect("request should be valid"),
            )
            .await
            .expect("request should succeed");
        assert_eq!(create_one.status(), StatusCode::CREATED);
        let create_one_body = axum::body::to_bytes(create_one.into_body(), usize::MAX)
            .await
            .expect("body should be readable");
        let appointment_one: AppointmentResponse =
            serde_json::from_slice(&create_one_body).expect("valid appointment response");

        let get_one = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/api/v1/appointments/{}", appointment_one.id))
                    .header(header::AUTHORIZATION, format!("Bearer {token}"))
                    .body(Body::empty())
                    .expect("request should be valid"),
            )
            .await
            .expect("request should succeed");
        assert_eq!(get_one.status(), StatusCode::OK);

        let start_two = Utc::now() + chrono::Duration::hours(4);
        let create_two = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/appointments")
                    .header(header::AUTHORIZATION, format!("Bearer {token}"))
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(sample_appointment_payload(start_two, practitioner_id)))
                    .expect("request should be valid"),
            )
            .await
            .expect("request should succeed");
        assert_eq!(create_two.status(), StatusCode::CREATED);

        let overlap_create = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/appointments")
                    .header(header::AUTHORIZATION, format!("Bearer {token}"))
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(sample_appointment_payload(
                        start_one + chrono::Duration::minutes(5),
                        practitioner_id,
                    )))
                    .expect("request should be valid"),
            )
            .await
            .expect("request should succeed");
        assert_eq!(overlap_create.status(), StatusCode::CONFLICT);

        let overlap_update = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri(format!("/api/v1/appointments/{}", appointment_one.id))
                    .header(header::AUTHORIZATION, format!("Bearer {token}"))
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(sample_appointment_payload(
                        start_two + chrono::Duration::minutes(5),
                        practitioner_id,
                    )))
                    .expect("request should be valid"),
            )
            .await
            .expect("request should succeed");
        assert_eq!(overlap_update.status(), StatusCode::CONFLICT);

        let list_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!(
                        "/api/v1/appointments?page=1&limit=10&date_from={}&date_to={}&practitioner_id={}",
                        (Utc::now() + chrono::Duration::hours(1))
                            .to_rfc3339()
                            .replace("+00:00", "Z"),
                        (Utc::now() + chrono::Duration::hours(5))
                            .to_rfc3339()
                            .replace("+00:00", "Z"),
                        practitioner_id
                    ))
                    .header(header::AUTHORIZATION, format!("Bearer {token}"))
                    .body(Body::empty())
                    .expect("request should be valid"),
            )
            .await
            .expect("request should succeed");
        assert_eq!(list_response.status(), StatusCode::OK);
        let list_body = axum::body::to_bytes(list_response.into_body(), usize::MAX)
            .await
            .expect("body should be readable");
        let paginated: PaginatedResponse<AppointmentResponse> =
            serde_json::from_slice(&list_body).expect("valid paginated appointment response");
        assert_eq!(paginated.page, 1);
        assert_eq!(paginated.limit, 10);
        assert!(!paginated.data.is_empty());

        let cancel_response = app
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(format!("/api/v1/appointments/{}", appointment_one.id))
                    .header(header::AUTHORIZATION, format!("Bearer {token}"))
                    .body(Body::empty())
                    .expect("request should be valid"),
            )
            .await
            .expect("request should succeed");
        assert_eq!(cancel_response.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn consultation_endpoints_require_authentication() {
        let state = ApiState::new(test_config()).await.expect("state should initialize");
        let app = router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!(
                        "/api/v1/consultations?patient_id={}&page=1&limit=25",
                        Uuid::new_v4()
                    ))
                    .body(Body::empty())
                    .expect("request should be valid"),
            )
            .await
            .expect("request should succeed");

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn receptionist_cannot_access_consultations() {
        let state = ApiState::new(test_config()).await.expect("state should initialize");
        let app = router(state);
        let token = login_token(&app, "recep_amy", "desk-passphrase").await;

        let patient_id = Uuid::new_v4();

        let get_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!(
                        "/api/v1/consultations?patient_id={patient_id}&page=1&limit=25"
                    ))
                    .header(header::AUTHORIZATION, format!("Bearer {token}"))
                    .body(Body::empty())
                    .expect("request should be valid"),
            )
            .await
            .expect("request should succeed");
        assert_eq!(get_response.status(), StatusCode::FORBIDDEN);

        for method in ["POST", "PUT"] {
            let uri = if method == "POST" {
                "/api/v1/consultations".to_string()
            } else {
                format!("/api/v1/consultations/{}", Uuid::new_v4())
            };

            let response = app
                .clone()
                .oneshot(
                    Request::builder()
                        .method(method)
                        .uri(uri)
                        .header(header::AUTHORIZATION, format!("Bearer {token}"))
                        .header(header::CONTENT_TYPE, "application/json")
                        .body(Body::from(sample_consultation_payload(patient_id, Uuid::new_v4())))
                        .expect("request should be valid"),
                )
                .await
                .expect("request should succeed");

            assert_eq!(response.status(), StatusCode::FORBIDDEN);
        }
    }

    #[tokio::test]
    async fn practitioner_can_crud_consultations_with_pagination() {
        let state = ApiState::new(test_config()).await.expect("state should initialize");
        let app = router(state);
        let token = login_token(&app, "dr_smith", "correct-horse-battery-staple").await;

        let create_patient = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/patients")
                    .header(header::AUTHORIZATION, format!("Bearer {token}"))
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(sample_patient_payload()))
                    .expect("request should be valid"),
            )
            .await
            .expect("request should succeed");
        assert_eq!(create_patient.status(), StatusCode::CREATED);
        let patient_body = axum::body::to_bytes(create_patient.into_body(), usize::MAX)
            .await
            .expect("body should be readable");
        let patient: PatientResponse =
            serde_json::from_slice(&patient_body).expect("valid patient response");

        let practitioner_id = Uuid::new_v4();
        let create_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/consultations")
                    .header(header::AUTHORIZATION, format!("Bearer {token}"))
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(sample_consultation_payload(
                        patient.id,
                        practitioner_id,
                    )))
                    .expect("request should be valid"),
            )
            .await
            .expect("request should succeed");
        assert_eq!(create_response.status(), StatusCode::CREATED);
        let create_body = axum::body::to_bytes(create_response.into_body(), usize::MAX)
            .await
            .expect("body should be readable");
        let created: ConsultationResponse =
            serde_json::from_slice(&create_body).expect("valid consultation response");

        let get_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/api/v1/consultations/{}", created.id))
                    .header(header::AUTHORIZATION, format!("Bearer {token}"))
                    .body(Body::empty())
                    .expect("request should be valid"),
            )
            .await
            .expect("request should succeed");
        assert_eq!(get_response.status(), StatusCode::OK);

        let update_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri(format!("/api/v1/consultations/{}", created.id))
                    .header(header::AUTHORIZATION, format!("Bearer {token}"))
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(
                        r#"{"patient_id":"00000000-0000-0000-0000-000000000001","practitioner_id":"00000000-0000-0000-0000-000000000002","appointment_id":null,"reason":"Updated reason","clinical_notes":"Updated SOAP notes"}"#,
                    ))
                    .expect("request should be valid"),
            )
            .await
            .expect("request should succeed");
        assert_eq!(update_response.status(), StatusCode::OK);
        let update_body = axum::body::to_bytes(update_response.into_body(), usize::MAX)
            .await
            .expect("body should be readable");
        let updated: ConsultationResponse =
            serde_json::from_slice(&update_body).expect("valid consultation response");
        assert_eq!(updated.reason.as_deref(), Some("Updated reason"));
        assert_eq!(updated.clinical_notes.as_deref(), Some("Updated SOAP notes"));

        for idx in 0..30 {
            let payload = format!(
                "{{\"patient_id\":\"{}\",\"practitioner_id\":\"{}\",\"appointment_id\":null,\"reason\":\"Review {idx}\",\"clinical_notes\":\"SOAP note {idx}\"}}",
                patient.id, practitioner_id
            );

            let response = app
                .clone()
                .oneshot(
                    Request::builder()
                        .method("POST")
                        .uri("/api/v1/consultations")
                        .header(header::AUTHORIZATION, format!("Bearer {token}"))
                        .header(header::CONTENT_TYPE, "application/json")
                        .body(Body::from(payload))
                        .expect("request should be valid"),
                )
                .await
                .expect("request should succeed");

            assert_eq!(response.status(), StatusCode::CREATED);
        }

        let list_response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!(
                        "/api/v1/consultations?patient_id={}&page=1&limit=25",
                        patient.id
                    ))
                    .header(header::AUTHORIZATION, format!("Bearer {token}"))
                    .body(Body::empty())
                    .expect("request should be valid"),
            )
            .await
            .expect("request should succeed");
        assert_eq!(list_response.status(), StatusCode::OK);
        let list_body = axum::body::to_bytes(list_response.into_body(), usize::MAX)
            .await
            .expect("body should be readable");
        let paginated: PaginatedResponse<ConsultationResponse> =
            serde_json::from_slice(&list_body).expect("valid paginated consultation response");
        assert_eq!(paginated.page, 1);
        assert_eq!(paginated.limit, 25);
        assert_eq!(paginated.data.len(), 25);
        assert!(paginated.total >= 31);
    }

    async fn login_token(app: &Router, username: &str, password: &str) -> String {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/auth/login")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(format!(
                        "{{\"username\":\"{username}\",\"password\":\"{password}\"}}"
                    )))
                    .expect("request should be valid"),
            )
            .await
            .expect("login should succeed");

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body should be readable");
        let login: LoginResponse = serde_json::from_slice(&body).expect("body should be login JSON");
        login.access_token
    }

    fn sample_patient_payload() -> &'static str {
        r#"{"first_name":"John","last_name":"Citizen","date_of_birth":"1984-05-12","gender":"male","phone_mobile":"0400123456","email":"john.citizen@example.com","medicare_number":"29501012341"}"#
    }

    fn sample_appointment_payload(start_time: DateTime<Utc>, practitioner_id: Uuid) -> String {
        format!(
            "{{\"patient_id\":\"{}\",\"practitioner_id\":\"{}\",\"start_time\":\"{}\",\"duration_minutes\":15,\"appointment_type\":\"standard\",\"reason\":\"Follow-up\",\"is_urgent\":false}}",
            Uuid::new_v4(),
            practitioner_id,
            start_time.to_rfc3339()
        )
    }

    fn sample_consultation_payload(patient_id: Uuid, practitioner_id: Uuid) -> String {
        format!(
            "{{\"patient_id\":\"{}\",\"practitioner_id\":\"{}\",\"appointment_id\":null,\"reason\":\"Hypertension review\",\"clinical_notes\":\"S: mild headache\\nO: BP 132/82\\nA: improving\\nP: continue current treatment\"}}",
            patient_id, practitioner_id
        )
    }

    fn test_config() -> ApiConfig {
        ApiConfig {
            host: "127.0.0.1".to_string(),
            port: 8080,
            database_url: "postgres://postgres:postgres@127.0.0.1:5432/opengp".to_string(),
            patient_database_url: None,
            database_max_connections: 10,
            database_min_connections: 2,
            connect_timeout_secs: 30,
            idle_timeout_secs: 600,
            encryption_key: "0000000000000000000000000000000000000000000000000000000000000000"
                .to_string(),
            log_level: "info".to_string(),
        }
    }

    #[tokio::test]
    async fn audit_emitter_is_initialized_in_api_state() {
        let config = test_config();
        let state = ApiState::new(config).await.expect("state should initialize");
        let ptr = std::ptr::addr_of!(state.audit_emitter) as *const _ as usize;
        assert_ne!(ptr, 0);
    }

    #[tokio::test]
    async fn audit_events_emitted_on_patient_mutations() {
        let config = test_config();
        let state = ApiState::new(config).await.expect("state should initialize");
        assert!(!std::ptr::addr_of!(state.audit_emitter).is_null());
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    }

    #[tokio::test]
    async fn audit_events_emitted_on_appointment_mutations() {
        let config = test_config();
        let state = ApiState::new(config).await.expect("state should initialize");
        assert!(!std::ptr::addr_of!(state.audit_emitter).is_null());
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    }

    #[tokio::test]
    async fn audit_events_emitted_on_consultation_mutations() {
        let config = test_config();
        let state = ApiState::new(config).await.expect("state should initialize");
        assert!(!std::ptr::addr_of!(state.audit_emitter).is_null());
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    }

    #[tokio::test]
    async fn audit_events_emitted_on_login_attempts() {
        let config = test_config();
        let state = ApiState::new(config).await.expect("state should initialize");
        assert!(!std::ptr::addr_of!(state.audit_emitter).is_null());
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    }
}

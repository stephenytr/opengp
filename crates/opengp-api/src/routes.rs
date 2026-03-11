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
    PatientRequest, PatientResponse,
};
use opengp_domain::domain::patient::{
    Address, Gender, NewPatientData, ServiceError as PatientServiceError, UpdatePatientData,
};
use opengp_domain::domain::user::{self, AuthError, Role};
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

    Router::new()
        .route("/health", get(health))
        .route("/metrics", get(metrics))
        .nest("/api/v1/auth", auth_routes)
        .nest("/api/v1/patients", patient_routes)
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
        username: payload.username,
        password: payload.password,
    };

    let login = state
        .services
        .auth_service
        .login(login_request)
        .await
        .map_err(auth_error_to_response)?;

    let user = state
        .services
        .auth_service
        .user_repository
        .find_by_id(login.user_id)
        .await
        .map_err(|_| auth_failed_response())?
        .ok_or_else(auth_failed_response)?;

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

    Ok(StatusCode::NO_CONTENT)
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

fn is_reader(role: Role) -> bool {
    matches!(role, Role::Receptionist | Role::Doctor | Role::Nurse | Role::Admin)
}

fn is_writer(role: Role) -> bool {
    matches!(role, Role::Doctor | Role::Nurse | Role::Admin)
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
}

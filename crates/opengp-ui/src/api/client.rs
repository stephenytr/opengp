use std::sync::Arc;

use chrono::{DateTime, Utc};
use opengp_domain::domain::api::{
    ApiErrorResponse, AppointmentRequest, AppointmentResponse, ConsultationRequest,
    ConsultationResponse, LoginRequest, LoginResponse, PaginatedResponse, PatientRequest,
    PatientResponse,
};
use reqwest::{Method, StatusCode};
use tokio::sync::Mutex;
use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub enum ApiClientError {
    #[error("Unable to reach the server. Please check your connection and try again.")]
    Network,
    #[error("Authentication error: {0}")]
    Authentication(String),
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Server error: {0}")]
    Server(String),
    #[error("Unexpected API response. Please try again.")]
    Unexpected,
}

pub struct ApiClient {
    pub base_url: String,
    pub http_client: reqwest::Client,
    pub session_token: Arc<Mutex<Option<String>>>,
}

impl ApiClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            http_client: reqwest::Client::new(),
            session_token: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn login(&self, username: String, password: String) -> Result<LoginResponse, ApiClientError> {
        let request = LoginRequest { username, password };
        let response = self
            .http_client
            .post(self.endpoint("/api/v1/auth/login"))
            .json(&request)
            .send()
            .await
            .map_err(Self::map_request_error)?;

        if !response.status().is_success() {
            return Err(Self::map_error_response(response).await);
        }

        let login_response = response
            .json::<LoginResponse>()
            .await
            .map_err(|_| ApiClientError::Unexpected)?;

        self.update_session_token(Some(login_response.access_token.clone()))
            .await;

        Ok(login_response)
    }

    pub async fn logout(&self) -> Result<(), ApiClientError> {
        let response = self
            .authenticated_request(Method::POST, "/api/v1/auth/logout")
            .await?
            .send()
            .await
            .map_err(Self::map_request_error)?;

        if !response.status().is_success() {
            return Err(Self::map_error_response(response).await);
        }

        self.update_session_token(None).await;
        Ok(())
    }

    pub async fn get_patients(
        &self,
        page: u32,
        limit: u32,
    ) -> Result<PaginatedResponse<PatientResponse>, ApiClientError> {
        let response = self
            .authenticated_request(Method::GET, "/api/v1/patients")
            .await?
            .query(&[("page", page), ("limit", limit)])
            .send()
            .await
            .map_err(Self::map_request_error)?;

        Self::parse_json_response(response).await
    }

    pub async fn create_patient(&self, request: &PatientRequest) -> Result<PatientResponse, ApiClientError> {
        let response = self
            .authenticated_request(Method::POST, "/api/v1/patients")
            .await?
            .json(request)
            .send()
            .await
            .map_err(Self::map_request_error)?;

        Self::parse_json_response(response).await
    }

    pub async fn get_appointments(
        &self,
        page: u32,
        limit: u32,
        date_from: Option<DateTime<Utc>>,
        date_to: Option<DateTime<Utc>>,
        practitioner_id: Option<Uuid>,
    ) -> Result<PaginatedResponse<AppointmentResponse>, ApiClientError> {
        let mut query_params = vec![("page".to_string(), page.to_string()), ("limit".to_string(), limit.to_string())];

        if let Some(from) = date_from {
            query_params.push(("date_from".to_string(), from.to_rfc3339()));
        }

        if let Some(to) = date_to {
            query_params.push(("date_to".to_string(), to.to_rfc3339()));
        }

        if let Some(practitioner) = practitioner_id {
            query_params.push(("practitioner_id".to_string(), practitioner.to_string()));
        }

        let response = self
            .authenticated_request(Method::GET, "/api/v1/appointments")
            .await?
            .query(&query_params)
            .send()
            .await
            .map_err(Self::map_request_error)?;

        Self::parse_json_response(response).await
    }

    pub async fn create_appointment(
        &self,
        request: &AppointmentRequest,
    ) -> Result<AppointmentResponse, ApiClientError> {
        let response = self
            .authenticated_request(Method::POST, "/api/v1/appointments")
            .await?
            .json(request)
            .send()
            .await
            .map_err(Self::map_request_error)?;

        Self::parse_json_response(response).await
    }

    pub async fn get_consultations(
        &self,
        patient_id: Uuid,
        page: u32,
        limit: u32,
    ) -> Result<PaginatedResponse<ConsultationResponse>, ApiClientError> {
        let response = self
            .authenticated_request(Method::GET, "/api/v1/consultations")
            .await?
            .query(&[
                ("patient_id", patient_id.to_string()),
                ("page", page.to_string()),
                ("limit", limit.to_string()),
            ])
            .send()
            .await
            .map_err(Self::map_request_error)?;

        Self::parse_json_response(response).await
    }

    pub async fn create_consultation(
        &self,
        request: &ConsultationRequest,
    ) -> Result<ConsultationResponse, ApiClientError> {
        let response = self
            .authenticated_request(Method::POST, "/api/v1/consultations")
            .await?
            .json(request)
            .send()
            .await
            .map_err(Self::map_request_error)?;

        Self::parse_json_response(response).await
    }

    pub async fn current_session_token(&self) -> Option<String> {
        self.session_token.lock().await.clone()
    }

    async fn parse_json_response<T: serde::de::DeserializeOwned>(
        response: reqwest::Response,
    ) -> Result<T, ApiClientError> {
        if !response.status().is_success() {
            return Err(Self::map_error_response(response).await);
        }

        response
            .json::<T>()
            .await
            .map_err(|_| ApiClientError::Unexpected)
    }

    async fn authenticated_request(
        &self,
        method: Method,
        path: &str,
    ) -> Result<reqwest::RequestBuilder, ApiClientError> {
        let token = self
            .current_session_token()
            .await
            .ok_or_else(|| ApiClientError::Authentication("Please log in to continue".to_string()))?;

        Ok(self
            .http_client
            .request(method, self.endpoint(path))
            .bearer_auth(token))
    }

    fn endpoint(&self, path: &str) -> String {
        format!(
            "{}/{}",
            self.base_url.trim_end_matches('/'),
            path.trim_start_matches('/')
        )
    }

    pub async fn set_session_token(&self, token: Option<String>) {
        self.update_session_token(token).await;
    }

    async fn update_session_token(&self, token: Option<String>) {
        *self.session_token.lock().await = token;
    }

    fn map_request_error(error: reqwest::Error) -> ApiClientError {
        if error.is_connect() || error.is_timeout() {
            ApiClientError::Network
        } else {
            ApiClientError::Server("Unable to complete request".to_string())
        }
    }

    async fn map_error_response(response: reqwest::Response) -> ApiClientError {
        let status = response.status();
        let fallback_message = status
            .canonical_reason()
            .unwrap_or("Request failed")
            .to_string();

        let message = response
            .json::<ApiErrorResponse>()
            .await
            .map(|e| e.message)
            .unwrap_or(fallback_message);

        match status {
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => ApiClientError::Authentication(message),
            StatusCode::BAD_REQUEST | StatusCode::UNPROCESSABLE_ENTITY => {
                ApiClientError::Validation(message)
            }
            _ => ApiClientError::Server(message),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::net::SocketAddr;

    use axum::{
        extract::{Query, State},
        http::{HeaderMap, StatusCode},
        routing::{get, post},
        Json, Router,
    };
    use chrono::{NaiveDate, TimeZone};
    use serde::{Deserialize, Serialize};

    use super::*;

    #[derive(Clone, Default)]
    struct TestState {
        seen_auth_headers: Arc<Mutex<Vec<String>>>,
    }

    #[tokio::test]
    async fn login_stores_token_and_authorization_header_is_sent() {
        let state = TestState::default();
        let app = Router::new()
            .route("/api/v1/auth/login", post(login_handler))
            .route("/api/v1/patients", get(patients_handler))
            .with_state(state.clone());

        let (base_url, _server_task) = spawn_server(app).await;
        let client = ApiClient::new(base_url);

        let _ = client
            .login("dr_smith".to_string(), "correct-horse-battery-staple".to_string())
            .await
            .expect("login should succeed");

        let token = client.current_session_token().await;
        assert_eq!(token.as_deref(), Some("session-token"));

        let patients = client
            .get_patients(1, 25)
            .await
            .expect("get_patients should succeed");

        assert_eq!(patients.data.len(), 1);
        let headers = state.seen_auth_headers.lock().await;
        assert_eq!(headers.as_slice(), ["Bearer session-token"]);
    }

    #[tokio::test]
    async fn logout_clears_token_only_after_success() {
        let state = TestState::default();
        let app = Router::new()
            .route("/api/v1/auth/login", post(login_handler))
            .route("/api/v1/auth/logout", post(logout_success_handler))
            .with_state(state);

        let (base_url, _server_task) = spawn_server(app).await;
        let client = ApiClient::new(base_url);

        client
            .login("dr_smith".to_string(), "correct-horse-battery-staple".to_string())
            .await
            .expect("login should succeed");

        client.logout().await.expect("logout should succeed");
        assert!(client.current_session_token().await.is_none());
    }

    #[tokio::test]
    async fn logout_error_does_not_clear_existing_token() {
        let app = Router::new()
            .route("/api/v1/auth/login", post(login_handler))
            .route("/api/v1/auth/logout", post(logout_failure_handler));

        let (base_url, _server_task) = spawn_server(app).await;
        let client = ApiClient::new(base_url);

        client
            .login("dr_smith".to_string(), "correct-horse-battery-staple".to_string())
            .await
            .expect("login should succeed");

        let result = client.logout().await;
        assert!(matches!(result, Err(ApiClientError::Server(_))));
        assert_eq!(client.current_session_token().await.as_deref(), Some("session-token"));
    }

    #[tokio::test]
    async fn maps_validation_and_network_errors() {
        let app = Router::new().route("/api/v1/auth/login", post(login_handler)).route(
            "/api/v1/patients",
            post(|| async {
                (
                    StatusCode::BAD_REQUEST,
                    Json(ApiErrorResponse {
                        status: 400,
                        message: "Missing required first_name".to_string(),
                        code: "validation_error".to_string(),
                    }),
                )
            }),
        );

        let (base_url, _server_task) = spawn_server(app).await;
        let client = ApiClient::new(base_url);

        client
            .login("dr_smith".to_string(), "correct-horse-battery-staple".to_string())
            .await
            .expect("login should succeed");

        let validation_result = client
            .create_patient(&PatientRequest {
                first_name: String::new(),
                last_name: "Citizen".to_string(),
                date_of_birth: NaiveDate::from_ymd_opt(1984, 5, 12).expect("valid date"),
                gender: "male".to_string(),
                phone_mobile: None,
                email: None,
                medicare_number: None,
            })
            .await;

        assert!(matches!(validation_result, Err(ApiClientError::Validation(_))));

        let network_client = ApiClient::new("http://127.0.0.1:9");
        let network_result = network_client
            .login("dr_smith".to_string(), "correct-horse-battery-staple".to_string())
            .await;
        assert!(matches!(network_result, Err(ApiClientError::Network)));
    }

    #[tokio::test]
    async fn endpoint_methods_return_expected_payloads() {
        let state = TestState::default();
        let app = Router::new()
            .route("/api/v1/auth/login", post(login_handler))
            .route("/api/v1/patients", get(patients_handler).post(create_patient_handler))
            .route(
                "/api/v1/appointments",
                get(appointments_handler).post(create_appointment_handler),
            )
            .route(
                "/api/v1/consultations",
                get(consultations_handler).post(create_consultation_handler),
            )
            .with_state(state.clone());

        let (base_url, _server_task) = spawn_server(app).await;
        let client = ApiClient::new(base_url);

        client
            .login("dr_smith".to_string(), "correct-horse-battery-staple".to_string())
            .await
            .expect("login should succeed");

        let created_patient = client
            .create_patient(&sample_patient_request())
            .await
            .expect("create_patient should succeed");
        assert_eq!(created_patient.first_name, "John");

        let appointments = client
            .get_appointments(1, 10, None, None, None)
            .await
            .expect("get_appointments should succeed");
        assert_eq!(appointments.data.len(), 1);

        let created_appointment = client
            .create_appointment(&sample_appointment_request())
            .await
            .expect("create_appointment should succeed");
        assert_eq!(created_appointment.status, "scheduled");

        let consultations = client
            .get_consultations(sample_patient_id(), 1, 25)
            .await
            .expect("get_consultations should succeed");
        assert_eq!(consultations.data.len(), 1);

        let created_consultation = client
            .create_consultation(&sample_consultation_request())
            .await
            .expect("create_consultation should succeed");
        assert!(!created_consultation.is_signed);

        let headers = state.seen_auth_headers.lock().await;
        assert!(headers.iter().all(|header| header == "Bearer session-token"));
    }

    async fn login_handler(Json(_payload): Json<LoginRequest>) -> Json<LoginResponse> {
        Json(LoginResponse {
            access_token: "session-token".to_string(),
            token_type: "Bearer".to_string(),
            expires_in_seconds: 3600,
            user: opengp_domain::domain::api::AuthenticatedUserResponse {
                id: Uuid::new_v4(),
                username: "dr_smith".to_string(),
                role: "doctor".to_string(),
                display_name: "Dr Smith".to_string(),
            },
        })
    }

    async fn logout_success_handler(headers: HeaderMap) -> (StatusCode, Json<LogoutResponse>) {
        assert_eq!(
            headers
                .get("authorization")
                .and_then(|h| h.to_str().ok()),
            Some("Bearer session-token")
        );

        (
            StatusCode::OK,
            Json(LogoutResponse {
                success: true,
                message: "Logged out".to_string(),
            }),
        )
    }

    async fn logout_failure_handler() -> (StatusCode, Json<ApiErrorResponse>) {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiErrorResponse {
                status: 500,
                message: "Could not log out".to_string(),
                code: "logout_failed".to_string(),
            }),
        )
    }

    async fn patients_handler(
        State(state): State<TestState>,
        headers: HeaderMap,
    ) -> (StatusCode, Json<PaginatedResponse<PatientResponse>>) {
        capture_header(state, headers).await;

        (
            StatusCode::OK,
            Json(PaginatedResponse {
                data: vec![sample_patient_response()],
                total: 1,
                page: 1,
                limit: 25,
            }),
        )
    }

    async fn create_patient_handler(
        State(state): State<TestState>,
        headers: HeaderMap,
        Json(_payload): Json<PatientRequest>,
    ) -> (StatusCode, Json<PatientResponse>) {
        capture_header(state, headers).await;
        (StatusCode::CREATED, Json(sample_patient_response()))
    }

    async fn appointments_handler(
        State(state): State<TestState>,
        headers: HeaderMap,
        Query(_query): Query<AppointmentsQuery>,
    ) -> (StatusCode, Json<PaginatedResponse<AppointmentResponse>>) {
        capture_header(state, headers).await;

        (
            StatusCode::OK,
            Json(PaginatedResponse {
                data: vec![sample_appointment_response()],
                total: 1,
                page: 1,
                limit: 10,
            }),
        )
    }

    async fn create_appointment_handler(
        State(state): State<TestState>,
        headers: HeaderMap,
        Json(_payload): Json<AppointmentRequest>,
    ) -> (StatusCode, Json<AppointmentResponse>) {
        capture_header(state, headers).await;
        (StatusCode::CREATED, Json(sample_appointment_response()))
    }

    async fn consultations_handler(
        State(state): State<TestState>,
        headers: HeaderMap,
        Query(_query): Query<ConsultationsQuery>,
    ) -> (StatusCode, Json<PaginatedResponse<ConsultationResponse>>) {
        capture_header(state, headers).await;

        (
            StatusCode::OK,
            Json(PaginatedResponse {
                data: vec![sample_consultation_response()],
                total: 1,
                page: 1,
                limit: 25,
            }),
        )
    }

    async fn create_consultation_handler(
        State(state): State<TestState>,
        headers: HeaderMap,
        Json(_payload): Json<ConsultationRequest>,
    ) -> (StatusCode, Json<ConsultationResponse>) {
        capture_header(state, headers).await;
        (StatusCode::CREATED, Json(sample_consultation_response()))
    }

    async fn capture_header(state: TestState, headers: HeaderMap) {
        if let Some(auth) = headers
            .get("authorization")
            .and_then(|header| header.to_str().ok())
        {
            state.seen_auth_headers.lock().await.push(auth.to_string());
        }
    }

    async fn spawn_server(app: Router) -> (String, tokio::task::JoinHandle<()>) {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("listener should bind");
        let addr = listener
            .local_addr()
            .expect("listener should have local address");
        let server_task = tokio::spawn(async move {
            axum::serve(listener, app)
                .await
                .expect("test server should run")
        });

        (format!("http://{}", socket_addr_to_host_port(addr)), server_task)
    }

    fn socket_addr_to_host_port(addr: SocketAddr) -> String {
        format!("{}:{}", addr.ip(), addr.port())
    }

    #[derive(Debug, Deserialize)]
    struct AppointmentsQuery {
        _page: Option<u32>,
        _limit: Option<u32>,
        _date_from: Option<String>,
        _date_to: Option<String>,
        _practitioner_id: Option<String>,
    }

    #[derive(Debug, Deserialize)]
    struct ConsultationsQuery {
        _patient_id: Option<String>,
        _page: Option<u32>,
        _limit: Option<u32>,
    }

    #[derive(Debug, Serialize)]
    struct LogoutResponse {
        success: bool,
        message: String,
    }

    fn sample_patient_id() -> Uuid {
        Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").expect("valid uuid")
    }

    fn sample_practitioner_id() -> Uuid {
        Uuid::parse_str("0f8fad5b-d9cb-469f-a165-70867728950e").expect("valid uuid")
    }

    fn sample_appointment_id() -> Uuid {
        Uuid::parse_str("27a88f3f-7f5a-4d9a-9f6a-5a2a4a9e1f80").expect("valid uuid")
    }

    fn sample_consultation_id() -> Uuid {
        Uuid::parse_str("1b31d6e0-a532-426f-9bf2-9b84f0f8c1bb").expect("valid uuid")
    }

    fn sample_patient_request() -> PatientRequest {
        PatientRequest {
            first_name: "John".to_string(),
            last_name: "Citizen".to_string(),
            date_of_birth: NaiveDate::from_ymd_opt(1984, 5, 12).expect("valid date"),
            gender: "male".to_string(),
            phone_mobile: Some("0400123456".to_string()),
            email: Some("john.citizen@example.com".to_string()),
            medicare_number: Some("29501012341".to_string()),
        }
    }

    fn sample_patient_response() -> PatientResponse {
        PatientResponse {
            id: sample_patient_id(),
            first_name: "John".to_string(),
            last_name: "Citizen".to_string(),
            date_of_birth: NaiveDate::from_ymd_opt(1984, 5, 12).expect("valid date"),
            gender: "male".to_string(),
            phone_mobile: Some("0400123456".to_string()),
            email: Some("john.citizen@example.com".to_string()),
            is_active: true,
        }
    }

    fn sample_appointment_request() -> AppointmentRequest {
        AppointmentRequest {
            patient_id: sample_patient_id(),
            practitioner_id: sample_practitioner_id(),
            start_time: Utc
                .with_ymd_and_hms(2026, 3, 11, 9, 30, 0)
                .single()
                .expect("valid datetime"),
            duration_minutes: 15,
            appointment_type: "standard".to_string(),
            reason: Some("Medication review".to_string()),
            is_urgent: false,
        }
    }

    fn sample_appointment_response() -> AppointmentResponse {
        AppointmentResponse {
            id: sample_appointment_id(),
            patient_id: sample_patient_id(),
            practitioner_id: sample_practitioner_id(),
            start_time: Utc
                .with_ymd_and_hms(2026, 3, 11, 9, 30, 0)
                .single()
                .expect("valid datetime"),
            end_time: Utc
                .with_ymd_and_hms(2026, 3, 11, 9, 45, 0)
                .single()
                .expect("valid datetime"),
            status: "scheduled".to_string(),
            appointment_type: "standard".to_string(),
            is_urgent: false,
            reason: Some("Medication review".to_string()),
        }
    }

    fn sample_consultation_request() -> ConsultationRequest {
        ConsultationRequest {
            patient_id: sample_patient_id(),
            practitioner_id: sample_practitioner_id(),
            appointment_id: Some(sample_appointment_id()),
            reason: Some("Follow-up for hypertension".to_string()),
            clinical_notes: Some("BP stable. Continue current ACE inhibitor dose.".to_string()),
        }
    }

    fn sample_consultation_response() -> ConsultationResponse {
        ConsultationResponse {
            id: sample_consultation_id(),
            patient_id: sample_patient_id(),
            practitioner_id: sample_practitioner_id(),
            appointment_id: Some(sample_appointment_id()),
            consultation_date: Utc
                .with_ymd_and_hms(2026, 3, 11, 10, 0, 0)
                .single()
                .expect("valid datetime"),
            reason: Some("Follow-up for hypertension".to_string()),
            clinical_notes: Some("BP stable. Continue current ACE inhibitor dose.".to_string()),
            is_signed: false,
        }
    }
}

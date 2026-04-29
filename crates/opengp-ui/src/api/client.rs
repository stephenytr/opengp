use std::sync::Arc;

use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use opengp_domain::domain::api::{
    AllergyRequest, AllergyResponse, ApiErrorResponse, AppointmentRequest, AppointmentResponse,
    ConsultationRequest, ConsultationResponse, FamilyHistoryRequest, FamilyHistoryResponse,
    LoginRequest, LoginResponse, MedicalHistoryRequest, MedicalHistoryResponse, PaginatedResponse,
    PatientRequest, PatientResponse, PractitionerResponse, SocialHistoryRequest,
    SocialHistoryResponse, VitalSignsRequest, VitalSignsResponse,
};
use reqwest::{Method, StatusCode};
use tokio::sync::Mutex;
use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub enum ApiClientError {
    #[error("Cannot connect to server: {0}")]
    ServerUnavailable(String),
    #[error("Session expired. Please log in again.")]
    Unauthorized,
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

    pub async fn login(
        &self,
        username: String,
        password: String,
    ) -> Result<LoginResponse, ApiClientError> {
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

    pub async fn get_practitioners(&self) -> Result<Vec<PractitionerResponse>, ApiClientError> {
        let response = self
            .authenticated_request(Method::GET, "/api/v1/practitioners")
            .await?
            .send()
            .await
            .map_err(Self::map_request_error)?;

        Self::parse_json_response(response).await
    }

    pub async fn create_patient(
        &self,
        request: &PatientRequest,
    ) -> Result<PatientResponse, ApiClientError> {
        let response = self
            .authenticated_request(Method::POST, "/api/v1/patients")
            .await?
            .json(request)
            .send()
            .await
            .map_err(Self::map_request_error)?;

        Self::parse_json_response(response).await
    }

    pub async fn get_patient(&self, id: Uuid) -> Result<PatientResponse, ApiClientError> {
        let response = self
            .authenticated_request(Method::GET, &format!("/api/v1/patients/{id}"))
            .await?
            .send()
            .await
            .map_err(Self::map_request_error)?;

        Self::parse_json_response(response).await
    }

    pub async fn update_patient(
        &self,
        id: Uuid,
        request: PatientRequest,
    ) -> Result<PatientResponse, ApiClientError> {
        let response = self
            .authenticated_request(Method::PUT, &format!("/api/v1/patients/{id}"))
            .await?
            .json(&request)
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
        let mut query_params = vec![
            ("page".to_string(), page.to_string()),
            ("limit".to_string(), limit.to_string()),
        ];

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

    pub async fn update_appointment(
        &self,
        id: Uuid,
        request: &AppointmentRequest,
    ) -> Result<AppointmentResponse, ApiClientError> {
        let response = self
            .authenticated_request(Method::PUT, &format!("/api/v1/appointments/{id}"))
            .await?
            .json(request)
            .send()
            .await
            .map_err(Self::map_request_error)?;

        Self::parse_json_response(response).await
    }

    pub async fn update_appointment_status(
        &self,
        id: Uuid,
        status: &str,
    ) -> Result<AppointmentResponse, ApiClientError> {
        let response = self
            .authenticated_request(Method::POST, &format!("/api/v1/appointments/{id}/status"))
            .await?
            .json(&serde_json::json!({ "action": status }))
            .send()
            .await
            .map_err(Self::map_request_error)?;

        Self::parse_json_response(response).await
    }

    pub async fn get_available_slots(
        &self,
        practitioner_id: Uuid,
        date: NaiveDate,
        duration: i64,
    ) -> Result<Vec<NaiveTime>, ApiClientError> {
        let response = self
            .authenticated_request(Method::GET, "/api/v1/appointments/available-slots")
            .await?
            .query(&[
                ("practitioner_id", practitioner_id.to_string()),
                ("date", date.to_string()),
                ("duration", duration.to_string()),
            ])
            .send()
            .await
            .map_err(Self::map_request_error)?;

        let slot_strings = Self::parse_json_response::<Vec<String>>(response).await?;
        slot_strings
            .into_iter()
            .map(|slot| {
                NaiveTime::parse_from_str(&slot, "%H:%M:%S").map_err(|_| ApiClientError::Unexpected)
            })
            .collect()
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

    pub async fn get_allergies(
        &self,
        patient_id: Uuid,
    ) -> Result<Vec<AllergyResponse>, ApiClientError> {
        let response = self
            .authenticated_request(
                Method::GET,
                &format!("/api/v1/patients/{patient_id}/allergies"),
            )
            .await?
            .send()
            .await
            .map_err(Self::map_request_error)?;

        Self::parse_json_response(response).await
    }

    pub async fn create_allergy(
        &self,
        patient_id: Uuid,
        request: &AllergyRequest,
    ) -> Result<AllergyResponse, ApiClientError> {
        let response = self
            .authenticated_request(
                Method::POST,
                &format!("/api/v1/patients/{patient_id}/allergies"),
            )
            .await?
            .json(request)
            .send()
            .await
            .map_err(Self::map_request_error)?;

        Self::parse_json_response(response).await
    }

    pub async fn get_medical_history(
        &self,
        patient_id: Uuid,
    ) -> Result<Vec<MedicalHistoryResponse>, ApiClientError> {
        let response = self
            .authenticated_request(
                Method::GET,
                &format!("/api/v1/patients/{patient_id}/medical-history"),
            )
            .await?
            .send()
            .await
            .map_err(Self::map_request_error)?;

        Self::parse_json_response(response).await
    }

    pub async fn create_medical_history(
        &self,
        patient_id: Uuid,
        request: &MedicalHistoryRequest,
    ) -> Result<MedicalHistoryResponse, ApiClientError> {
        let response = self
            .authenticated_request(
                Method::POST,
                &format!("/api/v1/patients/{patient_id}/medical-history"),
            )
            .await?
            .json(request)
            .send()
            .await
            .map_err(Self::map_request_error)?;

        Self::parse_json_response(response).await
    }

    pub async fn get_family_history(
        &self,
        patient_id: Uuid,
    ) -> Result<Vec<FamilyHistoryResponse>, ApiClientError> {
        let response = self
            .authenticated_request(
                Method::GET,
                &format!("/api/v1/patients/{patient_id}/family-history"),
            )
            .await?
            .send()
            .await
            .map_err(Self::map_request_error)?;

        Self::parse_json_response(response).await
    }

    pub async fn create_family_history(
        &self,
        patient_id: Uuid,
        request: &FamilyHistoryRequest,
    ) -> Result<FamilyHistoryResponse, ApiClientError> {
        let response = self
            .authenticated_request(
                Method::POST,
                &format!("/api/v1/patients/{patient_id}/family-history"),
            )
            .await?
            .json(request)
            .send()
            .await
            .map_err(Self::map_request_error)?;

        Self::parse_json_response(response).await
    }

    pub async fn get_social_history(
        &self,
        patient_id: Uuid,
    ) -> Result<SocialHistoryResponse, ApiClientError> {
        let response = self
            .authenticated_request(
                Method::GET,
                &format!("/api/v1/patients/{patient_id}/social-history"),
            )
            .await?
            .send()
            .await
            .map_err(Self::map_request_error)?;

        Self::parse_json_response(response).await
    }

    pub async fn update_social_history(
        &self,
        patient_id: Uuid,
        request: &SocialHistoryRequest,
    ) -> Result<SocialHistoryResponse, ApiClientError> {
        let response = self
            .authenticated_request(
                Method::PUT,
                &format!("/api/v1/patients/{patient_id}/social-history"),
            )
            .await?
            .json(request)
            .send()
            .await
            .map_err(Self::map_request_error)?;

        Self::parse_json_response(response).await
    }

    pub async fn get_vitals(
        &self,
        patient_id: Uuid,
    ) -> Result<Vec<VitalSignsResponse>, ApiClientError> {
        let response = self
            .authenticated_request(
                Method::GET,
                &format!("/api/v1/patients/{patient_id}/vitals"),
            )
            .await?
            .send()
            .await
            .map_err(Self::map_request_error)?;

        Self::parse_json_response(response).await
    }

    pub async fn create_vitals(
        &self,
        patient_id: Uuid,
        request: &VitalSignsRequest,
    ) -> Result<VitalSignsResponse, ApiClientError> {
        let response = self
            .authenticated_request(
                Method::POST,
                &format!("/api/v1/patients/{patient_id}/vitals"),
            )
            .await?
            .json(request)
            .send()
            .await
            .map_err(Self::map_request_error)?;

        Self::parse_json_response(response).await
    }

    pub async fn delete_allergy(
        &self,
        patient_id: Uuid,
        allergy_id: Uuid,
    ) -> Result<(), ApiClientError> {
        let response = self
            .authenticated_request(
                Method::DELETE,
                &format!("/api/v1/patients/{patient_id}/allergies/{allergy_id}"),
            )
            .await?
            .send()
            .await
            .map_err(Self::map_request_error)?;

        if !response.status().is_success() {
            return Err(Self::map_error_response(response).await);
        }

        Ok(())
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
        let token = self.current_session_token().await.ok_or_else(|| {
            ApiClientError::Authentication("Please log in to continue".to_string())
        })?;

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
            ApiClientError::ServerUnavailable(Self::server_unavailable_message(&error))
        } else {
            ApiClientError::Server("Unable to complete request".to_string())
        }
    }

    fn server_unavailable_message(error: &reqwest::Error) -> String {
        if error.is_timeout() {
            return "Cannot connect to server (request timed out)".to_string();
        }

        let lower = error.to_string().to_lowercase();
        if lower.contains("dns")
            || lower.contains("resolve")
            || lower.contains("name or service not known")
            || lower.contains("no such host")
        {
            "Cannot connect to server (DNS resolution failed)".to_string()
        } else if lower.contains("connection refused") {
            "Cannot connect to server (connection refused)".to_string()
        } else if lower.contains("network is unreachable") || lower.contains("host is unreachable")
        {
            "Cannot connect to server (network unreachable)".to_string()
        } else {
            "Cannot connect to server".to_string()
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
            StatusCode::UNAUTHORIZED => ApiClientError::Unauthorized,
            StatusCode::FORBIDDEN => ApiClientError::Authentication(message),
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
        extract::{Path, Query, State},
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
        seen_patient_update_versions: Arc<Mutex<Vec<i32>>>,
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
            .login(
                "dr_smith".to_string(),
                "correct-horse-battery-staple".to_string(),
            )
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
            .login(
                "dr_smith".to_string(),
                "correct-horse-battery-staple".to_string(),
            )
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
            .login(
                "dr_smith".to_string(),
                "correct-horse-battery-staple".to_string(),
            )
            .await
            .expect("login should succeed");

        let result = client.logout().await;
        assert!(matches!(result, Err(ApiClientError::Server(_))));
        assert_eq!(
            client.current_session_token().await.as_deref(),
            Some("session-token")
        );
    }

    #[tokio::test]
    async fn maps_validation_and_server_unavailable_errors() {
        let app = Router::new()
            .route("/api/v1/auth/login", post(login_handler))
            .route(
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
            .login(
                "dr_smith".to_string(),
                "correct-horse-battery-staple".to_string(),
            )
            .await
            .expect("login should succeed");

        let validation_result = client
            .create_patient(&PatientRequest {
                first_name: String::new(),
                last_name: "Citizen".to_string(),
                date_of_birth: NaiveDate::from_ymd_opt(1984, 5, 12).expect("valid date"),
                gender: "male".to_string(),
                title: None,
                middle_name: None,
                preferred_name: None,
                phone_home: None,
                phone_mobile: None,
                email: None,
                address_line1: None,
                address_line2: None,
                suburb: None,
                state: None,
                postcode: None,
                country: None,
                medicare_number: None,
                medicare_irn: None,
                medicare_expiry: None,
                ihi: None,
                emergency_contact_name: None,
                emergency_contact_phone: None,
                emergency_contact_relationship: None,
                concession_type: None,
                concession_number: None,
                preferred_language: None,
                interpreter_required: None,
                atsi_status: None,
                version: 1,
            })
            .await;

        assert!(matches!(
            validation_result,
            Err(ApiClientError::Validation(_))
        ));

        let network_client = ApiClient::new("http://127.0.0.1:9");
        let network_result = network_client
            .login(
                "dr_smith".to_string(),
                "correct-horse-battery-staple".to_string(),
            )
            .await;
        assert!(matches!(
            network_result,
            Err(ApiClientError::ServerUnavailable(_))
        ));
    }

    #[tokio::test]
    async fn maps_unauthorized_errors_to_dedicated_variant() {
        let app = Router::new()
            .route("/api/v1/auth/login", post(login_handler))
            .route(
                "/api/v1/patients",
                get(|| async {
                    (
                        StatusCode::UNAUTHORIZED,
                        Json(ApiErrorResponse {
                            status: 401,
                            message: "Session expired".to_string(),
                            code: "unauthorized".to_string(),
                        }),
                    )
                }),
            );

        let (base_url, _server_task) = spawn_server(app).await;
        let client = ApiClient::new(base_url);

        client
            .login(
                "dr_smith".to_string(),
                "correct-horse-battery-staple".to_string(),
            )
            .await
            .expect("login should succeed");

        let result = client.get_patients(1, 25).await;
        assert!(matches!(result, Err(ApiClientError::Unauthorized)));
    }

    #[tokio::test]
    async fn endpoint_methods_return_expected_payloads() {
        let state = TestState::default();
        let app = Router::new()
            .route("/api/v1/auth/login", post(login_handler))
            .route(
                "/api/v1/patients",
                get(patients_handler).post(create_patient_handler),
            )
            .route(
                "/api/v1/patients/{id}",
                get(get_patient_handler).put(update_patient_handler),
            )
            .route(
                "/api/v1/patients/{id}/allergies",
                get(allergies_handler).post(create_allergy_handler),
            )
            .route(
                "/api/v1/patients/{id}/medical-history",
                get(medical_history_handler).post(create_medical_history_handler),
            )
            .route(
                "/api/v1/patients/{id}/family-history",
                get(family_history_handler).post(create_family_history_handler),
            )
            .route(
                "/api/v1/patients/{id}/vitals",
                get(vitals_handler).post(create_vitals_handler),
            )
            .route(
                "/api/v1/patients/{id}/social-history",
                get(social_history_handler).put(update_social_history_handler),
            )
            .route(
                "/api/v1/patients/{id}/allergies/{allergy_id}",
                axum::routing::delete(delete_allergy_handler),
            )
            .route("/api/v1/practitioners", get(practitioners_handler))
            .route(
                "/api/v1/appointments",
                get(appointments_handler).post(create_appointment_handler),
            )
            .route(
                "/api/v1/appointments/{id}/status",
                post(update_appointment_status_handler),
            )
            .route(
                "/api/v1/appointments/available-slots",
                get(available_slots_handler),
            )
            .route(
                "/api/v1/consultations",
                get(consultations_handler).post(create_consultation_handler),
            )
            .with_state(state.clone());

        let (base_url, _server_task) = spawn_server(app).await;
        let client = ApiClient::new(base_url);

        client
            .login(
                "dr_smith".to_string(),
                "correct-horse-battery-staple".to_string(),
            )
            .await
            .expect("login should succeed");

        let created_patient = client
            .create_patient(&sample_patient_request())
            .await
            .expect("create_patient should succeed");
        assert_eq!(created_patient.first_name, "John");

        let fetched_patient = client
            .get_patient(sample_patient_id())
            .await
            .expect("get_patient should succeed");
        assert_eq!(fetched_patient.id, sample_patient_id());
        assert_eq!(fetched_patient.version, 1);

        let mut patient_update = sample_patient_request();
        patient_update.version = 7;
        let updated_patient = client
            .update_patient(sample_patient_id(), patient_update)
            .await
            .expect("update_patient should succeed");
        assert_eq!(updated_patient.version, 8);

        let practitioners = client
            .get_practitioners()
            .await
            .expect("get_practitioners should succeed");
        assert_eq!(practitioners.len(), 1);
        assert_eq!(practitioners[0].specialty, "General Practice");

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

        let status_updated_appointment = client
            .update_appointment_status(sample_appointment_id(), "arrived")
            .await
            .expect("update_appointment_status should succeed");
        assert_eq!(status_updated_appointment.status, "arrived");

        let available_slots = client
            .get_available_slots(
                sample_practitioner_id(),
                NaiveDate::from_ymd_opt(2026, 3, 11).expect("valid date"),
                15,
            )
            .await
            .expect("get_available_slots should succeed");
        assert_eq!(available_slots.len(), 2);
        assert_eq!(
            available_slots[0],
            NaiveTime::from_hms_opt(9, 0, 0).expect("valid time")
        );

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

        let allergies = client
            .get_allergies(sample_patient_id())
            .await
            .expect("get_allergies should succeed");
        assert_eq!(allergies.len(), 1);
        assert_eq!(allergies[0].allergy_type, "drug");

        let created_allergy = client
            .create_allergy(sample_patient_id(), &sample_allergy_request())
            .await
            .expect("create_allergy should succeed");
        assert_eq!(created_allergy.severity, "severe");

        let medical_history = client
            .get_medical_history(sample_patient_id())
            .await
            .expect("get_medical_history should succeed");
        assert_eq!(medical_history.len(), 1);
        assert_eq!(medical_history[0].status, "chronic");

        let created_medical_history = client
            .create_medical_history(sample_patient_id(), &sample_medical_history_request())
            .await
            .expect("create_medical_history should succeed");
        assert_eq!(created_medical_history.condition, "Type 2 diabetes");

        let family_history = client
            .get_family_history(sample_patient_id())
            .await
            .expect("get_family_history should succeed");
        assert_eq!(family_history.len(), 1);
        assert_eq!(family_history[0].condition, "Breast cancer");

        let created_family_history = client
            .create_family_history(sample_patient_id(), &sample_family_history_request())
            .await
            .expect("create_family_history should succeed");
        assert_eq!(created_family_history.relative_relationship, "Mother");

        let social_history = client
            .get_social_history(sample_patient_id())
            .await
            .expect("get_social_history should succeed");
        assert_eq!(social_history.smoking_status, "never_smoked");

        let updated_social_history = client
            .update_social_history(sample_patient_id(), &sample_social_history_request())
            .await
            .expect("update_social_history should succeed");
        assert_eq!(updated_social_history.alcohol_status, "occasional");

        let vitals = client
            .get_vitals(sample_patient_id())
            .await
            .expect("get_vitals should succeed");
        assert_eq!(vitals.len(), 1);
        assert_eq!(vitals[0].heart_rate, Some(72));

        let created_vitals = client
            .create_vitals(sample_patient_id(), &sample_vitals_request())
            .await
            .expect("create_vitals should succeed");
        assert_eq!(created_vitals.systolic_bp, Some(124));

        client
            .delete_allergy(sample_patient_id(), sample_allergy_id())
            .await
            .expect("delete_allergy should succeed");

        let headers = state.seen_auth_headers.lock().await;
        assert!(headers
            .iter()
            .all(|header| header == "Bearer session-token"));

        let seen_versions = state.seen_patient_update_versions.lock().await;
        assert_eq!(seen_versions.as_slice(), &[7]);
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
            headers.get("authorization").and_then(|h| h.to_str().ok()),
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

    async fn get_patient_handler(
        State(state): State<TestState>,
        headers: HeaderMap,
        Path(_id): Path<Uuid>,
    ) -> (StatusCode, Json<PatientResponse>) {
        capture_header(state, headers).await;
        (StatusCode::OK, Json(sample_patient_response()))
    }

    async fn update_patient_handler(
        State(state): State<TestState>,
        headers: HeaderMap,
        Path(_id): Path<Uuid>,
        Json(payload): Json<PatientRequest>,
    ) -> (StatusCode, Json<PatientResponse>) {
        capture_header(state.clone(), headers).await;
        state
            .seen_patient_update_versions
            .lock()
            .await
            .push(payload.version);

        let mut response = sample_patient_response();
        response.version = payload.version + 1;
        (StatusCode::OK, Json(response))
    }

    async fn practitioners_handler(
        State(state): State<TestState>,
        headers: HeaderMap,
    ) -> (StatusCode, Json<Vec<PractitionerResponse>>) {
        capture_header(state, headers).await;
        (StatusCode::OK, Json(vec![sample_practitioner_response()]))
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

    async fn update_appointment_status_handler(
        State(state): State<TestState>,
        headers: HeaderMap,
        Path(_id): Path<Uuid>,
        Json(payload): Json<AppointmentStatusActionPayload>,
    ) -> (StatusCode, Json<AppointmentResponse>) {
        capture_header(state, headers).await;

        let mut response = sample_appointment_response();
        response.status = payload.action;
        (StatusCode::OK, Json(response))
    }

    async fn available_slots_handler(
        State(state): State<TestState>,
        headers: HeaderMap,
        Query(_query): Query<AvailableSlotsQuery>,
    ) -> (StatusCode, Json<Vec<String>>) {
        capture_header(state, headers).await;
        (
            StatusCode::OK,
            Json(vec!["09:00:00".to_string(), "09:15:00".to_string()]),
        )
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

    async fn allergies_handler(
        State(state): State<TestState>,
        headers: HeaderMap,
        Path(_id): Path<Uuid>,
    ) -> (StatusCode, Json<Vec<AllergyResponse>>) {
        capture_header(state, headers).await;
        (StatusCode::OK, Json(vec![sample_allergy_response()]))
    }

    async fn create_allergy_handler(
        State(state): State<TestState>,
        headers: HeaderMap,
        Path(_id): Path<Uuid>,
        Json(_payload): Json<AllergyRequest>,
    ) -> (StatusCode, Json<AllergyResponse>) {
        capture_header(state, headers).await;
        (StatusCode::CREATED, Json(sample_allergy_response()))
    }

    async fn medical_history_handler(
        State(state): State<TestState>,
        headers: HeaderMap,
        Path(_id): Path<Uuid>,
    ) -> (StatusCode, Json<Vec<MedicalHistoryResponse>>) {
        capture_header(state, headers).await;
        (
            StatusCode::OK,
            Json(vec![sample_medical_history_response()]),
        )
    }

    async fn create_medical_history_handler(
        State(state): State<TestState>,
        headers: HeaderMap,
        Path(_id): Path<Uuid>,
        Json(_payload): Json<MedicalHistoryRequest>,
    ) -> (StatusCode, Json<MedicalHistoryResponse>) {
        capture_header(state, headers).await;
        (StatusCode::CREATED, Json(sample_medical_history_response()))
    }

    async fn family_history_handler(
        State(state): State<TestState>,
        headers: HeaderMap,
        Path(_id): Path<Uuid>,
    ) -> (StatusCode, Json<Vec<FamilyHistoryResponse>>) {
        capture_header(state, headers).await;
        (StatusCode::OK, Json(vec![sample_family_history_response()]))
    }

    async fn create_family_history_handler(
        State(state): State<TestState>,
        headers: HeaderMap,
        Path(_id): Path<Uuid>,
        Json(_payload): Json<FamilyHistoryRequest>,
    ) -> (StatusCode, Json<FamilyHistoryResponse>) {
        capture_header(state, headers).await;
        (StatusCode::CREATED, Json(sample_family_history_response()))
    }

    async fn social_history_handler(
        State(state): State<TestState>,
        headers: HeaderMap,
        Path(_id): Path<Uuid>,
    ) -> (StatusCode, Json<SocialHistoryResponse>) {
        capture_header(state, headers).await;
        (StatusCode::OK, Json(sample_social_history_response()))
    }

    async fn update_social_history_handler(
        State(state): State<TestState>,
        headers: HeaderMap,
        Path(_id): Path<Uuid>,
        Json(_payload): Json<SocialHistoryRequest>,
    ) -> (StatusCode, Json<SocialHistoryResponse>) {
        capture_header(state, headers).await;
        (StatusCode::OK, Json(sample_social_history_response()))
    }

    async fn vitals_handler(
        State(state): State<TestState>,
        headers: HeaderMap,
        Path(_id): Path<Uuid>,
    ) -> (StatusCode, Json<Vec<VitalSignsResponse>>) {
        capture_header(state, headers).await;
        (StatusCode::OK, Json(vec![sample_vitals_response()]))
    }

    async fn create_vitals_handler(
        State(state): State<TestState>,
        headers: HeaderMap,
        Path(_id): Path<Uuid>,
        Json(_payload): Json<VitalSignsRequest>,
    ) -> (StatusCode, Json<VitalSignsResponse>) {
        capture_header(state, headers).await;
        (StatusCode::CREATED, Json(sample_vitals_response()))
    }

    async fn delete_allergy_handler(
        State(state): State<TestState>,
        headers: HeaderMap,
        Path((_id, _allergy_id)): Path<(Uuid, Uuid)>,
    ) -> StatusCode {
        capture_header(state, headers).await;
        StatusCode::NO_CONTENT
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

        (
            format!("http://{}", socket_addr_to_host_port(addr)),
            server_task,
        )
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
    struct AvailableSlotsQuery {
        _practitioner_id: Option<String>,
        _date: Option<String>,
        _duration: Option<String>,
    }

    #[derive(Debug, Deserialize)]
    struct AppointmentStatusActionPayload {
        action: String,
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

    fn sample_practitioner_response() -> PractitionerResponse {
        PractitionerResponse {
            id: sample_practitioner_id(),
            name: "Sarah Smith".to_string(),
            title: "Dr".to_string(),
            specialty: "General Practice".to_string(),
        }
    }

    fn sample_consultation_id() -> Uuid {
        Uuid::parse_str("1b31d6e0-a532-426f-9bf2-9b84f0f8c1bb").expect("valid uuid")
    }

    fn sample_allergy_id() -> Uuid {
        Uuid::parse_str("6e8c10be-f9f6-4db4-8d51-4235e31af1fa").expect("valid uuid")
    }

    fn sample_patient_request() -> PatientRequest {
        PatientRequest {
            first_name: "John".to_string(),
            last_name: "Citizen".to_string(),
            date_of_birth: NaiveDate::from_ymd_opt(1984, 5, 12).expect("valid date"),
            gender: "male".to_string(),
            title: None,
            middle_name: None,
            preferred_name: None,
            phone_home: None,
            phone_mobile: Some("0400123456".to_string()),
            email: Some("john.citizen@example.com".to_string()),
            address_line1: None,
            address_line2: None,
            suburb: None,
            state: None,
            postcode: None,
            country: None,
            medicare_number: Some("29501012341".to_string()),
            medicare_irn: None,
            medicare_expiry: None,
            ihi: None,
            emergency_contact_name: None,
            emergency_contact_phone: None,
            emergency_contact_relationship: None,
            concession_type: None,
            concession_number: None,
            preferred_language: None,
            interpreter_required: None,
            atsi_status: None,
            version: 1,
        }
    }

    fn sample_patient_response() -> PatientResponse {
        PatientResponse {
            id: sample_patient_id(),
            first_name: "John".to_string(),
            last_name: "Citizen".to_string(),
            date_of_birth: NaiveDate::from_ymd_opt(1984, 5, 12).expect("valid date"),
            gender: "male".to_string(),
            title: None,
            middle_name: None,
            preferred_name: None,
            phone_home: None,
            phone_mobile: Some("0400123456".to_string()),
            email: Some("john.citizen@example.com".to_string()),
            address_line1: None,
            address_line2: None,
            suburb: None,
            state: None,
            postcode: None,
            country: None,
            medicare_number: None,
            medicare_irn: None,
            medicare_expiry: None,
            ihi: None,
            emergency_contact_name: None,
            emergency_contact_phone: None,
            emergency_contact_relationship: None,
            concession_type: None,
            concession_number: None,
            preferred_language: None,
            interpreter_required: None,
            atsi_status: None,
            is_active: true,
            version: 1,
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
            version: 1,
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
            version: 1,
        }
    }

    fn sample_consultation_request() -> ConsultationRequest {
        ConsultationRequest {
            patient_id: sample_patient_id(),
            practitioner_id: sample_practitioner_id(),
            appointment_id: Some(sample_appointment_id()),
            reason: Some("Follow-up for hypertension".to_string()),
            clinical_notes: Some("BP stable. Continue current ACE inhibitor dose.".to_string()),
            version: 1,
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
            version: 1,
            consultation_started_at: Some(
                Utc.with_ymd_and_hms(2026, 3, 11, 10, 0, 0)
                    .single()
                    .expect("valid datetime"),
            ),
            consultation_ended_at: Some(
                Utc.with_ymd_and_hms(2026, 3, 11, 10, 25, 0)
                    .single()
                    .expect("valid datetime"),
            ),
        }
    }

    fn sample_allergy_request() -> AllergyRequest {
        AllergyRequest {
            allergen: "Penicillin".to_string(),
            allergy_type: "drug".to_string(),
            severity: "severe".to_string(),
            reaction: Some("Rash and wheeze".to_string()),
            onset_date: NaiveDate::from_ymd_opt(2024, 1, 12),
            notes: Some("Confirmed in ED".to_string()),
        }
    }

    fn sample_allergy_response() -> AllergyResponse {
        AllergyResponse {
            id: sample_allergy_id(),
            patient_id: sample_patient_id(),
            allergen: "Penicillin".to_string(),
            allergy_type: "drug".to_string(),
            severity: "severe".to_string(),
            reaction: Some("Rash and wheeze".to_string()),
            onset_date: NaiveDate::from_ymd_opt(2024, 1, 12),
            notes: Some("Confirmed in ED".to_string()),
            is_active: true,
        }
    }

    fn sample_medical_history_id() -> Uuid {
        Uuid::parse_str("f6588cad-58d8-4f68-b534-9f228de8c2a3").expect("valid uuid")
    }

    fn sample_medical_history_request() -> MedicalHistoryRequest {
        MedicalHistoryRequest {
            condition: "Type 2 diabetes".to_string(),
            diagnosis_date: NaiveDate::from_ymd_opt(2019, 4, 3),
            status: "chronic".to_string(),
            severity: Some("moderate".to_string()),
            notes: Some("Managed with metformin".to_string()),
        }
    }

    fn sample_medical_history_response() -> MedicalHistoryResponse {
        MedicalHistoryResponse {
            id: sample_medical_history_id(),
            patient_id: sample_patient_id(),
            condition: "Type 2 diabetes".to_string(),
            diagnosis_date: NaiveDate::from_ymd_opt(2019, 4, 3),
            status: "chronic".to_string(),
            severity: Some("moderate".to_string()),
            notes: Some("Managed with metformin".to_string()),
            is_active: true,
        }
    }

    fn sample_family_history_id() -> Uuid {
        Uuid::parse_str("9d2164e4-c8f7-4eb2-a927-35fd3e068aba").expect("valid uuid")
    }

    fn sample_family_history_request() -> FamilyHistoryRequest {
        FamilyHistoryRequest {
            relative_relationship: "Mother".to_string(),
            condition: "Breast cancer".to_string(),
            age_at_diagnosis: Some(52),
            notes: Some("Diagnosed post-menopause".to_string()),
        }
    }

    fn sample_family_history_response() -> FamilyHistoryResponse {
        FamilyHistoryResponse {
            id: sample_family_history_id(),
            patient_id: sample_patient_id(),
            relative_relationship: "Mother".to_string(),
            condition: "Breast cancer".to_string(),
            age_at_diagnosis: Some(52),
            notes: Some("Diagnosed post-menopause".to_string()),
            created_at: Utc
                .with_ymd_and_hms(2026, 3, 11, 10, 30, 0)
                .single()
                .expect("valid datetime"),
            created_by: sample_practitioner_id(),
        }
    }

    fn sample_social_history_id() -> Uuid {
        Uuid::parse_str("8f670ba9-35da-444f-a44d-970ef6847c56").expect("valid uuid")
    }

    fn sample_social_history_request() -> SocialHistoryRequest {
        SocialHistoryRequest {
            smoking_status: "never_smoked".to_string(),
            cigarettes_per_day: Some(0),
            smoking_quit_date: NaiveDate::from_ymd_opt(2021, 2, 28),
            alcohol_status: "occasional".to_string(),
            standard_drinks_per_week: Some(3),
            exercise_frequency: Some("three_to_five_times".to_string()),
            occupation: Some("Teacher".to_string()),
            living_situation: Some("Lives with partner and two children".to_string()),
            support_network: Some("Strong family support".to_string()),
            notes: Some("Working on smoking cessation plan".to_string()),
        }
    }

    fn sample_social_history_response() -> SocialHistoryResponse {
        SocialHistoryResponse {
            id: sample_social_history_id(),
            patient_id: sample_patient_id(),
            smoking_status: "never_smoked".to_string(),
            cigarettes_per_day: Some(0),
            smoking_quit_date: NaiveDate::from_ymd_opt(2021, 2, 28),
            alcohol_status: "occasional".to_string(),
            standard_drinks_per_week: Some(3),
            exercise_frequency: Some("three_to_five_times".to_string()),
            occupation: Some("Teacher".to_string()),
            living_situation: Some("Lives with partner and two children".to_string()),
            support_network: Some("Strong family support".to_string()),
            notes: Some("Working on smoking cessation plan".to_string()),
            updated_at: Utc
                .with_ymd_and_hms(2026, 3, 11, 10, 30, 0)
                .single()
                .expect("valid datetime"),
            updated_by: sample_practitioner_id(),
        }
    }

    fn sample_vitals_id() -> Uuid {
        Uuid::parse_str("2c8ee9f9-04cb-4a97-a05b-c997377a9f40").expect("valid uuid")
    }

    fn sample_vitals_request() -> VitalSignsRequest {
        VitalSignsRequest {
            consultation_id: Some(sample_consultation_id()),
            systolic_bp: Some(124),
            diastolic_bp: Some(78),
            heart_rate: Some(72),
            respiratory_rate: Some(16),
            temperature: Some(36.8),
            oxygen_saturation: Some(98),
            height_cm: Some(178),
            weight_kg: Some(82.4),
            notes: Some("Patient reports mild headache".to_string()),
        }
    }

    fn sample_vitals_response() -> VitalSignsResponse {
        VitalSignsResponse {
            id: sample_vitals_id(),
            patient_id: sample_patient_id(),
            consultation_id: Some(sample_consultation_id()),
            measured_at: Utc
                .with_ymd_and_hms(2026, 3, 11, 10, 5, 0)
                .single()
                .expect("valid datetime"),
            systolic_bp: Some(124),
            diastolic_bp: Some(78),
            heart_rate: Some(72),
            respiratory_rate: Some(16),
            temperature: Some(36.8),
            oxygen_saturation: Some(98),
            height_cm: Some(178),
            weight_kg: Some(82.4),
            bmi: Some(26.0),
            notes: Some("Patient reports mild headache".to_string()),
        }
    }
}

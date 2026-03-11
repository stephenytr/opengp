use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// Request payload for `/api/v1/auth/login`.
///
/// Validation hints:
/// - `username` should be 3-50 characters.
/// - `password` should be at least 8 characters.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub struct LoginRequest {
    /// Unique username used for authentication.
    #[schema(min_length = 3, max_length = 50, example = "dr_smith")]
    pub username: String,

    /// Plain text password provided by the client.
    #[schema(min_length = 8, example = "correct-horse-battery-staple")]
    pub password: String,
}

/// User identity returned after successful authentication.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub struct AuthenticatedUserResponse {
    /// User identifier.
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub id: Uuid,

    /// Username used to login.
    #[schema(example = "dr_smith")]
    pub username: String,

    /// Role label for client-side authorisation checks.
    #[schema(example = "doctor")]
    pub role: String,

    /// Display name for UI surfaces.
    #[schema(example = "Sarah Smith")]
    pub display_name: String,
}

/// Response payload for `/api/v1/auth/login`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub struct LoginResponse {
    /// JWT access token for authenticated requests.
    #[schema(example = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...")]
    pub access_token: String,

    /// Token scheme expected in the `Authorization` header.
    #[schema(example = "Bearer")]
    pub token_type: String,

    /// Access token expiry in seconds.
    #[schema(example = 3600)]
    pub expires_in_seconds: i64,

    /// Authenticated user profile safe for client consumption.
    pub user: AuthenticatedUserResponse,
}

/// Request payload for creating or updating a patient via `/api/v1/patients`.
///
/// Validation hints:
/// - `first_name` and `last_name` must be non-empty.
/// - `date_of_birth` must not be in the future.
/// - `medicare_number` should contain only digits.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub struct PatientRequest {
    #[schema(example = "John")]
    pub first_name: String,
    #[schema(example = "Citizen")]
    pub last_name: String,
    #[schema(example = "1984-05-12")]
    pub date_of_birth: NaiveDate,
    #[schema(example = "male")]
    pub gender: String,
    #[schema(example = "0400123456")]
    pub phone_mobile: Option<String>,
    #[schema(example = "john.citizen@example.com")]
    pub email: Option<String>,
    #[schema(example = "29501012341")]
    pub medicare_number: Option<String>,
}

/// Response payload for `/api/v1/patients/{id}`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub struct PatientResponse {
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub id: Uuid,
    #[schema(example = "John")]
    pub first_name: String,
    #[schema(example = "Citizen")]
    pub last_name: String,
    #[schema(example = "1984-05-12")]
    pub date_of_birth: NaiveDate,
    #[schema(example = "male")]
    pub gender: String,
    #[schema(example = "0400123456")]
    pub phone_mobile: Option<String>,
    #[schema(example = "john.citizen@example.com")]
    pub email: Option<String>,
    #[schema(example = true)]
    pub is_active: bool,
}

/// Request payload for `/api/v1/appointments`.
///
/// Validation hints:
/// - `duration_minutes` should be between 5 and 120.
/// - `start_time` should be in the future for new bookings.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub struct AppointmentRequest {
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub patient_id: Uuid,
    #[schema(example = "0f8fad5b-d9cb-469f-a165-70867728950e")]
    pub practitioner_id: Uuid,
    #[schema(example = "2026-03-11T09:30:00Z")]
    pub start_time: DateTime<Utc>,
    #[schema(minimum = 5, maximum = 120, example = 15)]
    pub duration_minutes: i64,
    #[schema(example = "standard")]
    pub appointment_type: String,
    #[schema(example = "Medication review")]
    pub reason: Option<String>,
    #[schema(example = false)]
    pub is_urgent: bool,
}

/// Response payload for `/api/v1/appointments/{id}`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub struct AppointmentResponse {
    #[schema(example = "27a88f3f-7f5a-4d9a-9f6a-5a2a4a9e1f80")]
    pub id: Uuid,
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub patient_id: Uuid,
    #[schema(example = "0f8fad5b-d9cb-469f-a165-70867728950e")]
    pub practitioner_id: Uuid,
    #[schema(example = "2026-03-11T09:30:00Z")]
    pub start_time: DateTime<Utc>,
    #[schema(example = "2026-03-11T09:45:00Z")]
    pub end_time: DateTime<Utc>,
    #[schema(example = "scheduled")]
    pub status: String,
    #[schema(example = "standard")]
    pub appointment_type: String,
    #[schema(example = false)]
    pub is_urgent: bool,
    #[schema(example = "Medication review")]
    pub reason: Option<String>,
}

/// Request payload for `/api/v1/consultations`.
///
/// Validation hints:
/// - `clinical_notes` should be provided for completed consultations.
/// - `appointment_id` is optional for walk-in consultations.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub struct ConsultationRequest {
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub patient_id: Uuid,
    #[schema(example = "0f8fad5b-d9cb-469f-a165-70867728950e")]
    pub practitioner_id: Uuid,
    #[schema(example = "27a88f3f-7f5a-4d9a-9f6a-5a2a4a9e1f80")]
    pub appointment_id: Option<Uuid>,
    #[schema(example = "Follow-up for hypertension")]
    pub reason: Option<String>,
    #[schema(example = "BP stable. Continue current ACE inhibitor dose.")]
    pub clinical_notes: Option<String>,
}

/// Response payload for `/api/v1/consultations/{id}`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub struct ConsultationResponse {
    #[schema(example = "1b31d6e0-a532-426f-9bf2-9b84f0f8c1bb")]
    pub id: Uuid,
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub patient_id: Uuid,
    #[schema(example = "0f8fad5b-d9cb-469f-a165-70867728950e")]
    pub practitioner_id: Uuid,
    #[schema(example = "27a88f3f-7f5a-4d9a-9f6a-5a2a4a9e1f80")]
    pub appointment_id: Option<Uuid>,
    #[schema(example = "2026-03-11T10:00:00Z")]
    pub consultation_date: DateTime<Utc>,
    #[schema(example = "Follow-up for hypertension")]
    pub reason: Option<String>,
    #[schema(example = "BP stable. Continue current ACE inhibitor dose.")]
    pub clinical_notes: Option<String>,
    #[schema(example = true)]
    pub is_signed: bool,
}

/// Standard API error format used across `/api/v1/*` routes.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub struct ApiErrorResponse {
    /// HTTP status code emitted by the server.
    #[schema(example = 404)]
    pub status: u16,

    /// Human-readable error message suitable for UI display.
    #[schema(example = "Patient not found")]
    pub message: String,

    /// Stable machine-friendly error code for client handling.
    #[schema(example = "patient_not_found")]
    pub code: String,
}

/// Generic pagination wrapper for list endpoints.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
#[schema(bound = "T: ToSchema")]
pub struct PaginatedResponse<T> {
    /// Current page of result items.
    pub data: Vec<T>,

    /// Total number of items across all pages.
    #[schema(example = 250)]
    pub total: u64,

    /// Current page number, starting at 1.
    #[schema(example = 1)]
    pub page: u32,

    /// Maximum number of items returned per page.
    #[schema(example = 25)]
    pub limit: u32,
}

use axum::body::{to_bytes, Body};
use axum::http::{header, Request, StatusCode};
use chrono::Utc;
use opengp_domain::user::PasswordHasher;
use opengp_api::{router, ApiConfig, ApiState};
use opengp_domain::domain::api::{
    ApiErrorResponse, AppointmentResponse, LoginResponse, PaginatedResponse, PatientResponse,
};
use opengp_infrastructure::infrastructure::crypto::password::BcryptPasswordHasher;
use sqlx::PgPool;
use tower::util::ServiceExt;
use uuid::Uuid;

fn test_config() -> ApiConfig {
    let mut config = ApiConfig::from_env().expect("test config should load from environment");
    config.log_level = "warn".to_string();
    config
}

async fn align_appointment_schema(pool: &PgPool) {
    sqlx::query(
        r#"
        ALTER TABLE appointments
            ALTER COLUMN start_time TYPE TEXT USING start_time::text,
            ALTER COLUMN end_time TYPE TEXT USING end_time::text,
            ALTER COLUMN created_at TYPE TEXT USING created_at::text,
            ALTER COLUMN updated_at TYPE TEXT USING updated_at::text
        "#,
    )
    .execute(pool)
    .await
    .expect("appointment schema alignment should succeed");
}

async fn login(app: &axum::Router, username: &str, password: &str) -> LoginResponse {
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
        .expect("login request should succeed");

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should be readable");
    serde_json::from_slice(&body).expect("body should be valid login response")
}

async fn seed_login_user(pool: &PgPool, role: &str) -> (String, String) {
    let user_id = Uuid::new_v4();
    let username = format!("{}_{}", role.to_lowercase(), Uuid::new_v4().simple());
    let password = format!("pw-{}", Uuid::new_v4().simple());
    let password_hash = BcryptPasswordHasher::new()
        .hash_password(&password)
        .expect("password hashing should succeed");
    let now = Utc::now();

    sqlx::query(
        r#"
        INSERT INTO users (
            id,
            username,
            password_hash,
            first_name,
            last_name,
            email,
            role,
            is_active,
            is_locked,
            failed_login_attempts,
            last_login,
            password_changed_at,
            additional_permissions,
            created_at,
            updated_at
        ) VALUES (
            $1, $2, $3, $4, $5, $6, $7, TRUE, FALSE, 0, NULL, $8, '[]', $9, $10
        )
        "#,
    )
    .bind(user_id)
    .bind(&username)
    .bind(&password_hash)
    .bind("Integration")
    .bind("Tester")
    .bind(format!("{username}@example.com"))
    .bind(role)
    .bind(now)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await
    .expect("test user insert should succeed");

    (username, password)
}

#[tokio::test]
async fn full_workflow_login_view_patients_create_appointment_logout_then_unauthorized() {
    let state = ApiState::new(test_config())
        .await
        .expect("state should initialize");
    align_appointment_schema(&state.pool).await;
    let (username, password) = seed_login_user(&state.pool, "Doctor").await;
    let app = router(state);

    let login = login(&app, &username, &password).await;
    let token = login.access_token;

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
        .expect("patient list request should succeed");

    assert_eq!(list_response.status(), StatusCode::OK);
    let list_body = to_bytes(list_response.into_body(), usize::MAX)
        .await
        .expect("body should be readable");
    let patients: PaginatedResponse<PatientResponse> =
        serde_json::from_slice(&list_body).expect("patients list should be valid JSON");
    assert_eq!(patients.page, 1);
    assert_eq!(patients.limit, 25);

    let patient_id = patients
        .data
        .first()
        .map(|p| p.id)
        .unwrap_or_else(Uuid::new_v4);
    let start_time = Utc::now() + chrono::Duration::hours(2);

    let appointment_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/appointments")
                .header(header::AUTHORIZATION, format!("Bearer {token}"))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "patient_id": patient_id,
                        "practitioner_id": login.user.id,
                        "start_time": start_time,
                        "duration_minutes": 15,
                        "appointment_type": "standard",
                        "reason": "Integration test appointment",
                        "is_urgent": false
                    })
                    .to_string(),
                ))
                .expect("request should be valid"),
        )
        .await
        .expect("appointment create request should succeed");

    assert_eq!(appointment_response.status(), StatusCode::CREATED);
    let appointment_body = to_bytes(appointment_response.into_body(), usize::MAX)
        .await
        .expect("body should be readable");
    let appointment: AppointmentResponse =
        serde_json::from_slice(&appointment_body).expect("appointment response should be valid");
    assert_eq!(appointment.patient_id, patient_id);
    assert_eq!(appointment.practitioner_id, login.user.id);

    let logout_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/logout")
                .header(header::AUTHORIZATION, format!("Bearer {token}"))
                .body(Body::empty())
                .expect("request should be valid"),
        )
        .await
        .expect("logout request should succeed");
    assert_eq!(logout_response.status(), StatusCode::OK);

    let post_logout_response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/patients?page=1&limit=25")
                .header(header::AUTHORIZATION, format!("Bearer {token}"))
                .body(Body::empty())
                .expect("request should be valid"),
        )
        .await
        .expect("post-logout request should succeed");

    assert_eq!(post_logout_response.status(), StatusCode::UNAUTHORIZED);
    let unauthorized_body = to_bytes(post_logout_response.into_body(), usize::MAX)
        .await
        .expect("body should be readable");
    let unauthorized: ApiErrorResponse =
        serde_json::from_slice(&unauthorized_body).expect("unauthorized response should be valid");
    assert_eq!(unauthorized.status, 401);
    assert_eq!(unauthorized.code, "session_expired");
}

#[tokio::test]
async fn receptionist_can_view_patients_but_is_denied_clinical_endpoints() {
    let state = ApiState::new(test_config())
        .await
        .expect("state should initialize");
    align_appointment_schema(&state.pool).await;
    let (username, password) = seed_login_user(&state.pool, "Receptionist").await;
    let app = router(state);

    let login = login(&app, &username, &password).await;
    let token = login.access_token;

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
        .expect("patient list request should succeed");
    assert_eq!(list_response.status(), StatusCode::OK);

    let clinical_get_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/consultations/{}", Uuid::new_v4()))
                .header(header::AUTHORIZATION, format!("Bearer {token}"))
                .body(Body::empty())
                .expect("request should be valid"),
        )
        .await
        .expect("clinical GET request should succeed");
    assert_eq!(clinical_get_response.status(), StatusCode::FORBIDDEN);
    let forbidden_get_body = to_bytes(clinical_get_response.into_body(), usize::MAX)
        .await
        .expect("body should be readable");
    let forbidden_get: ApiErrorResponse =
        serde_json::from_slice(&forbidden_get_body).expect("forbidden response should be valid");
    assert_eq!(forbidden_get.status, 403);
    assert_eq!(forbidden_get.code, "insufficient_permissions");

    let clinical_post_response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/consultations")
                .header(header::AUTHORIZATION, format!("Bearer {token}"))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "patient_id": Uuid::new_v4(),
                        "practitioner_id": Uuid::new_v4(),
                        "appointment_id": null,
                        "reason": "Receptionist should be denied",
                        "clinical_notes": "Should not be accepted"
                    })
                    .to_string(),
                ))
                .expect("request should be valid"),
        )
        .await
        .expect("clinical POST request should succeed");
    assert_eq!(clinical_post_response.status(), StatusCode::FORBIDDEN);
    let forbidden_post_body = to_bytes(clinical_post_response.into_body(), usize::MAX)
        .await
        .expect("body should be readable");
    let forbidden_post: ApiErrorResponse = serde_json::from_slice(&forbidden_post_body)
        .expect("forbidden response body should be valid");
    assert_eq!(forbidden_post.status, 403);
    assert_eq!(forbidden_post.code, "insufficient_permissions");
}

#[tokio::test]
async fn concurrent_appointment_creates_return_one_created_and_one_conflict() {
    let state = ApiState::new(test_config())
        .await
        .expect("state should initialize");
    align_appointment_schema(&state.pool).await;
    let (username, password) = seed_login_user(&state.pool, "Doctor").await;
    let app = router(state);

    let login = login(&app, &username, &password).await;
    let token = login.access_token;
    let practitioner_id = login.user.id;
    let patient_list_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/patients?page=1&limit=1")
                .header(header::AUTHORIZATION, format!("Bearer {token}"))
                .body(Body::empty())
                .expect("request should be valid"),
        )
        .await
        .expect("patient list request should succeed");
    assert_eq!(patient_list_response.status(), StatusCode::OK);
    let patient_list_body = to_bytes(patient_list_response.into_body(), usize::MAX)
        .await
        .expect("patient list body should be readable");
    let patient_list: PaginatedResponse<PatientResponse> =
        serde_json::from_slice(&patient_list_body).expect("patient list should be valid");
    let patient_id = patient_list
        .data
        .first()
        .map(|p| p.id)
        .expect("at least one patient is required for appointment tests");
    let start_time = Utc::now() + chrono::Duration::hours(3);

    let request_body = serde_json::json!({
        "patient_id": patient_id,
        "practitioner_id": practitioner_id,
        "start_time": start_time,
        "duration_minutes": 30,
        "appointment_type": "standard",
        "reason": "Concurrent conflict test",
        "is_urgent": false
    })
    .to_string();

    let app_one = app.clone();
    let token_one = token.clone();
    let body_one = request_body.clone();
    let first = tokio::spawn(async move {
        app_one
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/appointments")
                    .header(header::AUTHORIZATION, format!("Bearer {token_one}"))
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(body_one))
                    .expect("request should be valid"),
            )
            .await
            .expect("first request should succeed")
    });

    let app_two = app.clone();
    let token_two = token.clone();
    let body_two = request_body.clone();
    let second = tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;
        app_two
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/appointments")
                    .header(header::AUTHORIZATION, format!("Bearer {token_two}"))
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(body_two))
                    .expect("request should be valid"),
            )
            .await
            .expect("second request should succeed")
    });

    let first_response = first.await.expect("first task should join");
    let second_response = second.await.expect("second task should join");
    let statuses = [first_response.status(), second_response.status()];

    let created_count = statuses
        .iter()
        .filter(|status| **status == StatusCode::CREATED)
        .count();
    let conflict_count = statuses
        .iter()
        .filter(|status| **status == StatusCode::CONFLICT)
        .count();

    assert_eq!(created_count, 1, "exactly one request should create");
    assert_eq!(conflict_count, 1, "exactly one request should conflict");

    let conflict_response = if first_response.status() == StatusCode::CONFLICT {
        first_response
    } else {
        second_response
    };

    let conflict_body = to_bytes(conflict_response.into_body(), usize::MAX)
        .await
        .expect("body should be readable");
    let conflict_error: ApiErrorResponse =
        serde_json::from_slice(&conflict_body).expect("conflict error should be valid");
    assert_eq!(conflict_error.status, 409);
    assert_eq!(conflict_error.code, "appointment_conflict");
}

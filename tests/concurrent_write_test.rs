use axum::body::{to_bytes, Body};
use axum::http::{header, Request, StatusCode};
use chrono::Utc;
use opengp_api::{router, ApiState};
use opengp_config::Config;
use opengp_domain::domain::api::{
    ApiErrorResponse, LoginResponse, PaginatedResponse, PatientResponse,
};
use opengp_domain::user::PasswordHasher;
use opengp_infrastructure::infrastructure::crypto::password::BcryptPasswordHasher;
use sqlx::PgPool;
use tower::util::ServiceExt;
use uuid::Uuid;

fn test_config() -> Config {
    let mut config = Config::from_env().expect("test config should load from environment");
    config.app.logging.level = "warn".to_string();
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

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "login must succeed before booking"
    );
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("login body should be readable");
    serde_json::from_slice(&body).expect("login response should be valid JSON")
}

async fn first_patient_id(app: &axum::Router, token: &str) -> Uuid {
    let response = app
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

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("patient list body should be readable");
    let patients: PaginatedResponse<PatientResponse> =
        serde_json::from_slice(&body).expect("patient list response should be valid");

    patients
        .data
        .first()
        .map(|p| p.id)
        .expect("at least one patient is required for appointment tests")
}

async fn seed_login_user(pool: &PgPool) -> (String, String) {
    let user_id = Uuid::new_v4();
    let username = format!("dr_{}", Uuid::new_v4().simple());
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
    .bind("Concurrent")
    .bind("Tester")
    .bind(format!("{username}@example.com"))
    .bind("Doctor")
    .bind(now)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await
    .expect("test user insert should succeed");

    (username, password)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires running PostgreSQL instance"]
async fn concurrent_booking_returns_conflict_for_second_writer() {
    let state = ApiState::new(test_config())
        .await
        .expect("state should initialize");
    align_appointment_schema(&state.pool).await;
    let (username, password) = seed_login_user(&state.pool).await;
    let app = router(state);

    let login = login(&app, &username, &password).await;
    let token = login.access_token;
    let patient_id = first_patient_id(&app, &token).await;

    let appointment_payload = serde_json::json!({
        "patient_id": patient_id,
        "practitioner_id": login.user.id,
        "start_time": (Utc::now() + chrono::Duration::hours(2)).to_rfc3339(),
        "duration_minutes": 15,
        "appointment_type": "standard",
        "reason": "Concurrent booking test",
        "is_urgent": false
    })
    .to_string();

    let app_one = app.clone();
    let token_one = token.clone();
    let body_one = appointment_payload.clone();
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
    let token_two = token;
    let body_two = appointment_payload;
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

    let first_response = first.await.expect("first task should complete");
    let second_response = second.await.expect("second task should complete");
    let statuses = [first_response.status(), second_response.status()];

    let created_count = statuses
        .iter()
        .filter(|status| **status == StatusCode::CREATED)
        .count();
    let conflict_count = statuses
        .iter()
        .filter(|status| **status == StatusCode::CONFLICT)
        .count();

    assert_eq!(created_count, 1, "exactly one booking should succeed");
    assert_eq!(
        conflict_count, 1,
        "exactly one booking should return conflict"
    );

    let conflict_response = if first_response.status() == StatusCode::CONFLICT {
        first_response
    } else {
        second_response
    };

    let conflict_body = to_bytes(conflict_response.into_body(), usize::MAX)
        .await
        .expect("conflict response body should be readable");
    let conflict: ApiErrorResponse =
        serde_json::from_slice(&conflict_body).expect("conflict response should be valid JSON");

    assert_eq!(conflict.status, 409);
    assert_eq!(conflict.code, "appointment_conflict");
    assert!(
        conflict.message.to_lowercase().contains("overlapping"),
        "conflict message should clearly describe overlap, got: {}",
        conflict.message
    );
}

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
#[cfg(test)]
use chrono::{DateTime, Duration, Utc};
#[cfg(test)]
use opengp_domain::domain::api::{
    ApiErrorResponse, AppointmentResponse, ConsultationResponse, PaginatedResponse, PatientResponse,
};
use serde::{Deserialize, Serialize};
use tower::ServiceBuilder;
use tower_http::compression::CompressionLayer;
use tower_http::trace::TraceLayer;
#[cfg(test)]
use uuid::Uuid;

use crate::ApiState;

mod appointments;
mod auth;
mod consultations;
mod middleware;
mod patients;
mod practitioners;

use self::{
    appointments::*, auth::*, consultations::*, middleware::*, patients::*, practitioners::*,
};

pub fn router(state: ApiState) -> Router {
    let protected_auth_routes = Router::new()
        .route("/logout", post(logout))
        .route("/refresh", post(refresh))
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            session_validation_middleware,
        ));

    let auth_routes = Router::new()
        .route("/login", post(login))
        .merge(protected_auth_routes);

    let patient_routes = Router::new()
        .route("/", get(list_patients).post(create_patient))
        .route(
            "/{id}",
            get(get_patient).put(update_patient).delete(delete_patient),
        )
        .route("/{id}/allergies", get(list_allergies).post(create_allergy))
        .route(
            "/{id}/medical-history",
            get(list_medical_history).post(create_medical_history),
        )
        .route(
            "/{id}/family-history",
            get(list_family_history).post(create_family_history),
        )
        .route(
            "/{id}/social-history",
            get(get_social_history).put(update_social_history),
        )
        .route("/{id}/vitals", get(list_vitals).post(create_vitals))
        .route(
            "/{id}/allergies/{allergy_id}",
            axum::routing::delete(delete_allergy),
        )
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            session_validation_middleware,
        ));

    let appointment_routes = Router::new()
        .route("/", get(list_appointments).post(create_appointment))
        .route("/available-slots", get(get_available_slots))
        .route("/{id}/status", post(update_appointment_status))
        .route(
            "/{id}",
            get(get_appointment)
                .put(update_appointment)
                .delete(cancel_appointment),
        )
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            session_validation_middleware,
        ));

    let consultation_routes = Router::new()
        .route("/", get(list_consultations).post(create_consultation))
        .route("/{id}", get(get_consultation).put(update_consultation))
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            session_validation_middleware,
        ));

    let practitioner_routes = Router::new()
        .route("/", get(list_practitioners))
        .route_layer(axum::middleware::from_fn_with_state(
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
        .nest("/api/v1/practitioners", practitioner_routes)
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
    let db_connected =
        tokio::time::timeout(std::time::Duration::from_secs(5), state.pool.acquire())
            .await
            .is_ok();

    let response = HealthResponse {
        status: if db_connected {
            "ok".to_string()
        } else {
            "degraded".to_string()
        },
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
        active_sessions: state
            .metrics
            .active_sessions
            .load(std::sync::atomic::Ordering::Relaxed),
        request_count: state
            .metrics
            .request_count
            .load(std::sync::atomic::Ordering::Relaxed),
        error_count: state
            .metrics
            .error_count
            .load(std::sync::atomic::Ordering::Relaxed),
    };

    (StatusCode::OK, Json(response))
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

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_trait::async_trait;
    use axum::body::Body;
    use axum::http::header;
    use http::{HeaderMap, HeaderValue, Request, StatusCode};
    use opengp_domain::domain::api::LoginResponse;
    use opengp_domain::domain::audit::{AuditAction, AuditEmitter, AuditEmitterError, AuditEntry};
    use opengp_domain::domain::user::Session;
    use tokio::sync::RwLock;
    use tower::util::ServiceExt;

    use crate::ApiConfig;

    use super::*;

    #[tokio::test]
    #[ignore] // Requires PostgreSQL database connection
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
            encryption_key: "0000000000000000000000000000000000000000000000000000000000000000"
                .to_string(),
            session_timeout_minutes: 480,
            log_level: "info".to_string(),
        };

        let state = ApiState::new(config)
            .await
            .expect("state should initialize");
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
    #[ignore] // Requires PostgreSQL database connection
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
            encryption_key: "0000000000000000000000000000000000000000000000000000000000000000"
                .to_string(),
            session_timeout_minutes: 480,
            log_level: "info".to_string(),
        };

        let state = ApiState::new(config)
            .await
            .expect("state should initialize");
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
        let health: HealthResponse =
            serde_json::from_slice(&body).expect("body should be valid JSON");
        assert_eq!(health.status, "ok".to_string());
        assert!(health.database_connected);
    }

    #[tokio::test]
    #[ignore] // Requires PostgreSQL database connection
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
            encryption_key: "0000000000000000000000000000000000000000000000000000000000000000"
                .to_string(),
            session_timeout_minutes: 480,
            log_level: "info".to_string(),
        };

        let state = ApiState::new(config)
            .await
            .expect("state should initialize");
        state
            .metrics
            .request_count
            .store(42, std::sync::atomic::Ordering::Relaxed);
        state
            .metrics
            .error_count
            .store(5, std::sync::atomic::Ordering::Relaxed);
        state
            .metrics
            .active_sessions
            .store(3, std::sync::atomic::Ordering::Relaxed);

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
        let metrics: MetricsResponse =
            serde_json::from_slice(&body).expect("body should be valid JSON");
        assert_eq!(metrics.request_count, 42);
        assert_eq!(metrics.error_count, 5);
        assert_eq!(metrics.active_sessions, 3);
    }

    #[tokio::test]
    #[ignore] // Requires PostgreSQL database connection
    async fn login_endpoint_returns_access_token_and_cookie() {
        let config = test_config();
        let state = ApiState::new(config)
            .await
            .expect("state should initialize");
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
        let login: LoginResponse =
            serde_json::from_slice(&body).expect("body should be valid JSON");
        assert!(!login.access_token.is_empty());
        assert_eq!(login.token_type, "Bearer");
    }

    #[tokio::test]
    #[ignore] // Requires PostgreSQL database connection
    async fn login_endpoint_returns_401_for_invalid_credentials() {
        let config = test_config();
        let state = ApiState::new(config)
            .await
            .expect("state should initialize");
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
    #[ignore] // Requires PostgreSQL database connection
    async fn locked_account_returns_401() {
        let config = test_config();
        let state = ApiState::new(config)
            .await
            .expect("state should initialize");
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
    #[ignore] // Requires PostgreSQL database connection
    async fn logout_invalidates_session() {
        let config = test_config();
        let state = ApiState::new(config)
            .await
            .expect("state should initialize");
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
        let login: LoginResponse =
            serde_json::from_slice(&body).expect("body should be valid JSON");

        let logout_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/auth/logout")
                    .header(
                        header::AUTHORIZATION,
                        format!("Bearer {}", login.access_token),
                    )
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
                    .header(
                        header::AUTHORIZATION,
                        format!("Bearer {}", login.access_token),
                    )
                    .body(Body::empty())
                    .expect("request should be valid"),
            )
            .await
            .expect("request should succeed");

        assert_eq!(refresh_response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    #[ignore] // Requires PostgreSQL database connection
    async fn refresh_keeps_same_token_and_returns_ok() {
        let config = test_config();
        let state = ApiState::new(config)
            .await
            .expect("state should initialize");
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
                    .header(
                        header::AUTHORIZATION,
                        format!("Bearer {}", login.access_token),
                    )
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

    #[test]
    fn session_token_extraction_supports_cookie_as_header_alternative() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::COOKIE,
            HeaderValue::from_static("theme=dark; session_token=abc123"),
        );

        let token = extract_session_token(&headers);
        assert_eq!(token.as_deref(), Some("abc123"));

        headers.insert(
            header::AUTHORIZATION,
            HeaderValue::from_static("Bearer bearer-token"),
        );
        let preferred = extract_session_token(&headers);
        assert_eq!(preferred.as_deref(), Some("bearer-token"));
    }

    #[test]
    fn health_response_status_field_indicates_ok_or_degraded() {
        let health = HealthResponse {
            status: "ok".to_string(),
            database_connected: true,
            uptime_seconds: 3600,
        };

        assert_eq!(health.status, "ok");
        assert!(health.database_connected);
    }

    #[test]
    fn metrics_response_contains_counter_values() {
        let metrics = MetricsResponse {
            active_sessions: 5,
            request_count: 100,
            error_count: 2,
        };

        assert_eq!(metrics.active_sessions, 5);
        assert_eq!(metrics.request_count, 100);
        assert_eq!(metrics.error_count, 2);
    }

    #[tokio::test]
    #[ignore] // Requires PostgreSQL database connection
    async fn middleware_rejects_missing_authorization_header() {
        let config = test_config();
        let state = ApiState::new(config)
            .await
            .expect("state should initialize");
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
    #[ignore] // Requires PostgreSQL database connection
    async fn middleware_rejects_expired_session_with_401_and_cleans_it_up() {
        let state = ApiState::new(test_config())
            .await
            .expect("state should initialize");
        let app = router(state.clone());

        let token = login_token(&app, "dr_smith", "correct-horse-battery-staple").await;
        let user_id = state
            .services
            .auth_service
            .validate_session(&token)
            .await
            .expect("session should be valid before expiring");

        state
            .services
            .auth_service
            .session_repository
            .delete_by_token(&token)
            .await
            .expect("existing session should be deletable");
        state
            .services
            .auth_service
            .session_repository
            .create(Session {
                id: Uuid::new_v4(),
                user_id,
                created_at: Utc::now() - Duration::hours(2),
                expires_at: Utc::now() - Duration::hours(1),
                token: token.clone(),
            })
            .await
            .expect("expired session should be inserted");

        let response = app
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

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        let stored = state
            .services
            .auth_service
            .session_repository
            .find_by_token(&token)
            .await
            .expect("session lookup should succeed");
        assert!(stored.is_none(), "expired session should be removed");
    }

    #[tokio::test]
    #[ignore] // Requires PostgreSQL database connection
    async fn patient_endpoints_require_authentication() {
        let state = ApiState::new(test_config())
            .await
            .expect("state should initialize");
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
    #[ignore] // Requires PostgreSQL database connection
    async fn receptionist_can_read_but_cannot_write_patients() {
        let state = ApiState::new(test_config())
            .await
            .expect("state should initialize");
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
                builder = builder.header(header::CONTENT_TYPE, "application/json")
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
    #[ignore] // Requires PostgreSQL database connection
    async fn practitioner_can_crud_patients_with_pagination() {
        let state = ApiState::new(test_config())
            .await
            .expect("state should initialize");
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
        assert_eq!(created.version, 1);

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
                    .body(Body::from(format!(
                        "{{\"first_name\":\"Jane\",\"last_name\":\"Citizen\",\"date_of_birth\":\"1984-05-12\",\"gender\":\"female\",\"phone_mobile\":\"0400999888\",\"email\":\"jane.citizen@example.com\",\"medicare_number\":\"29501012341\",\"version\":{}}}",
                        created.version
                    )))
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
    #[ignore] // Requires PostgreSQL database connection
    async fn patient_post_and_put_with_invalid_payload_return_400() {
        let state = ApiState::new(test_config())
            .await
            .expect("state should initialize");
        let app = router(state);
        let token = login_token(&app, "dr_smith", "correct-horse-battery-staple").await;

        let invalid_gender_payload = r#"{"first_name":"John","last_name":"Citizen","date_of_birth":"1984-05-12","gender":"unknown","phone_mobile":"0400123456","email":"john.citizen@example.com","medicare_number":"29501012341"}"#;

        let invalid_create = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/patients")
                    .header(header::AUTHORIZATION, format!("Bearer {token}"))
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(invalid_gender_payload))
                    .expect("request should be valid"),
            )
            .await
            .expect("request should succeed");
        assert_eq!(invalid_create.status(), StatusCode::BAD_REQUEST);

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

        let invalid_update_payload = format!(
            "{{\"first_name\":\"Jane\",\"last_name\":\"Citizen\",\"date_of_birth\":\"1984-05-12\",\"gender\":\"invalid\",\"phone_mobile\":\"0400999888\",\"email\":\"jane.citizen@example.com\",\"medicare_number\":\"29501012341\",\"version\":{}}}",
            created.version
        );

        let invalid_update = app
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri(format!("/api/v1/patients/{}", created.id))
                    .header(header::AUTHORIZATION, format!("Bearer {token}"))
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(invalid_update_payload))
                    .expect("request should be valid"),
            )
            .await
            .expect("request should succeed");
        assert_eq!(invalid_update.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    #[ignore] // Requires PostgreSQL database connection
    async fn patient_update_with_stale_version_returns_409_and_clear_message() {
        let state = ApiState::new(test_config())
            .await
            .expect("state should initialize");
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
            serde_json::from_slice(&create_body).expect("valid patient response");

        let current_payload = format!(
            "{{\"first_name\":\"Version\",\"last_name\":\"Current\",\"date_of_birth\":\"1984-05-12\",\"gender\":\"male\",\"phone_mobile\":\"0400123456\",\"email\":\"current@example.com\",\"medicare_number\":\"29501012341\",\"version\":{}}}",
            created.version
        );

        let first_update = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri(format!("/api/v1/patients/{}", created.id))
                    .header(header::AUTHORIZATION, format!("Bearer {token}"))
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(current_payload))
                    .expect("request should be valid"),
            )
            .await
            .expect("request should succeed");
        assert_eq!(first_update.status(), StatusCode::OK);

        let stale_version = created.version - 1;
        let stale_payload = format!(
            "{{\"first_name\":\"Version\",\"last_name\":\"Stale\",\"date_of_birth\":\"1984-05-12\",\"gender\":\"male\",\"phone_mobile\":\"0400123456\",\"email\":\"stale@example.com\",\"medicare_number\":\"29501012341\",\"version\":{}}}",
            stale_version
        );

        let stale_update = app
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri(format!("/api/v1/patients/{}", created.id))
                    .header(header::AUTHORIZATION, format!("Bearer {token}"))
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(stale_payload))
                    .expect("request should be valid"),
            )
            .await
            .expect("request should succeed");

        assert_eq!(stale_update.status(), StatusCode::CONFLICT);
        let stale_body = axum::body::to_bytes(stale_update.into_body(), usize::MAX)
            .await
            .expect("body should be readable");
        let error: ApiErrorResponse =
            serde_json::from_slice(&stale_body).expect("valid api error response");
        assert_eq!(
            error.message,
            "Resource was modified. Please refresh and try again."
        );
    }

    #[tokio::test]
    #[ignore] // Requires PostgreSQL database connection
    async fn appointment_endpoints_require_authentication() {
        let state = ApiState::new(test_config())
            .await
            .expect("state should initialize");
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
    #[ignore] // Requires PostgreSQL database connection
    async fn receptionist_can_read_but_cannot_modify_appointments() {
        let state = ApiState::new(test_config())
            .await
            .expect("state should initialize");
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

        let payload =
            sample_appointment_payload(Utc::now() + chrono::Duration::hours(2), Uuid::new_v4());

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
    #[ignore] // Requires PostgreSQL database connection
    async fn practitioner_can_crud_appointments_and_overlap_returns_conflict() {
        let state = ApiState::new(test_config())
            .await
            .expect("state should initialize");
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
                    .body(Body::from(sample_appointment_payload(
                        start_one,
                        practitioner_id,
                    )))
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
                    .body(Body::from(sample_appointment_payload(
                        start_two,
                        practitioner_id,
                    )))
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

        let past_create = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/appointments")
                    .header(header::AUTHORIZATION, format!("Bearer {token}"))
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(sample_appointment_payload(
                        Utc::now() - chrono::Duration::hours(1),
                        practitioner_id,
                    )))
                    .expect("request should be valid"),
            )
            .await
            .expect("request should succeed");
        assert_eq!(past_create.status(), StatusCode::BAD_REQUEST);

        let list_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!(
                        "/api/v1/appointments?page=1&limit=10&date={}&practitioner={}",
                        (Utc::now() + chrono::Duration::hours(1)).date_naive(),
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
    #[ignore] // Requires PostgreSQL database connection
    async fn consultation_endpoints_require_authentication() {
        let state = ApiState::new(test_config())
            .await
            .expect("state should initialize");
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
    #[ignore] // Requires PostgreSQL database connection
    async fn receptionist_cannot_access_consultations() {
        let state = ApiState::new(test_config())
            .await
            .expect("state should initialize");
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
                        .body(Body::from(sample_consultation_payload(
                            patient_id,
                            Uuid::new_v4(),
                        )))
                        .expect("request should be valid"),
                )
                .await
                .expect("request should succeed");

            assert_eq!(response.status(), StatusCode::FORBIDDEN);
        }
    }

    #[tokio::test]
    #[ignore] // Requires PostgreSQL database connection
    async fn practitioner_can_crud_consultations_with_pagination() {
        let state = ApiState::new(test_config())
            .await
            .expect("state should initialize");
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
        assert_eq!(
            updated.clinical_notes.as_deref(),
            Some("Updated SOAP notes")
        );

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
        let login: LoginResponse =
            serde_json::from_slice(&body).expect("body should be login JSON");
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
            session_timeout_minutes: 480,
            log_level: "info".to_string(),
        }
    }

    #[tokio::test]
    #[ignore] // Requires PostgreSQL database connection
    async fn audit_emitter_is_initialized_in_api_state() {
        let config = test_config();
        let state = ApiState::new(config)
            .await
            .expect("state should initialize");
        let ptr = std::ptr::addr_of!(state.audit_emitter) as *const _ as usize;
        assert_ne!(ptr, 0);
    }

    #[tokio::test]
    #[ignore] // Requires PostgreSQL database connection
    async fn audit_events_emitted_on_patient_mutations() {
        let recorder = Arc::new(RecordingAuditEmitter::default());
        let state = state_with_audit_emitter(recorder.clone()).await;
        let app = router(state.clone());
        let token = login_token(&app, "dr_smith", "correct-horse-battery-staple").await;
        let user_id = state
            .services
            .auth_service
            .validate_session(&token)
            .await
            .expect("session should resolve user");
        recorder.clear().await;

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
            .expect("create request should succeed");
        assert_eq!(create_response.status(), StatusCode::CREATED);
        let create_body = axum::body::to_bytes(create_response.into_body(), usize::MAX)
            .await
            .expect("body should be readable");
        let created: PatientResponse = serde_json::from_slice(&create_body).expect("valid patient");

        let update_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri(format!("/api/v1/patients/{}", created.id))
                    .header(header::AUTHORIZATION, format!("Bearer {token}"))
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(format!(
                        "{{\"first_name\":\"Jane\",\"last_name\":\"Citizen\",\"date_of_birth\":\"1984-05-12\",\"gender\":\"female\",\"phone_mobile\":\"0400999888\",\"email\":\"jane.citizen@example.com\",\"medicare_number\":\"29501012341\",\"version\":{}}}",
                        created.version
                    )))
                    .expect("request should be valid"),
            )
            .await
            .expect("update request should succeed");
        assert_eq!(update_response.status(), StatusCode::OK);

        let delete_response = app
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(format!("/api/v1/patients/{}", created.id))
                    .header(header::AUTHORIZATION, format!("Bearer {token}"))
                    .body(Body::empty())
                    .expect("request should be valid"),
            )
            .await
            .expect("delete request should succeed");
        assert_eq!(delete_response.status(), StatusCode::NO_CONTENT);

        wait_for_audit_entries(&recorder, 3).await;
        let entries = recorder.entries().await;
        let patient_entries: Vec<_> = entries
            .into_iter()
            .filter(|entry| entry.entity_type == "patient" && entry.entity_id == created.id)
            .collect();

        assert_eq!(patient_entries.len(), 3);
        assert!(patient_entries
            .iter()
            .any(|entry| entry.action == AuditAction::Created));
        assert!(patient_entries
            .iter()
            .any(|entry| entry.action == AuditAction::Updated));
        assert!(patient_entries
            .iter()
            .any(|entry| matches!(entry.action, AuditAction::Cancelled { .. })));
        assert!(patient_entries
            .iter()
            .all(|entry| entry.changed_by == user_id));
        assert!(patient_entries
            .iter()
            .all(|entry| entry.changed_at <= Utc::now()));
    }

    #[tokio::test]
    #[ignore] // Requires PostgreSQL database connection
    async fn audit_events_emitted_on_appointment_mutations() {
        let recorder = Arc::new(RecordingAuditEmitter::default());
        let state = state_with_audit_emitter(recorder.clone()).await;
        let app = router(state.clone());
        let token = login_token(&app, "dr_smith", "correct-horse-battery-staple").await;
        let user_id = state
            .services
            .auth_service
            .validate_session(&token)
            .await
            .expect("session should resolve user");
        recorder.clear().await;

        let practitioner_id = Uuid::new_v4();
        let start_time = Utc::now() + chrono::Duration::hours(2);
        let create_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/appointments")
                    .header(header::AUTHORIZATION, format!("Bearer {token}"))
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(sample_appointment_payload(
                        start_time,
                        practitioner_id,
                    )))
                    .expect("request should be valid"),
            )
            .await
            .expect("create request should succeed");
        assert_eq!(create_response.status(), StatusCode::CREATED);
        let create_body = axum::body::to_bytes(create_response.into_body(), usize::MAX)
            .await
            .expect("body should be readable");
        let created: AppointmentResponse =
            serde_json::from_slice(&create_body).expect("valid appointment response");

        let update_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri(format!("/api/v1/appointments/{}", created.id))
                    .header(header::AUTHORIZATION, format!("Bearer {token}"))
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(format!(
                        "{{\"patient_id\":\"{}\",\"practitioner_id\":\"{}\",\"start_time\":\"{}\",\"duration_minutes\":15,\"appointment_type\":\"standard\",\"reason\":\"Follow-up\",\"is_urgent\":false,\"version\":{}}}",
                        created.patient_id,
                        created.practitioner_id,
                        (Utc::now() + chrono::Duration::hours(3)).to_rfc3339(),
                        created.version
                    )))
                    .expect("request should be valid"),
            )
            .await
            .expect("update request should succeed");
        assert_eq!(update_response.status(), StatusCode::OK);

        let delete_response = app
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(format!("/api/v1/appointments/{}", created.id))
                    .header(header::AUTHORIZATION, format!("Bearer {token}"))
                    .body(Body::empty())
                    .expect("request should be valid"),
            )
            .await
            .expect("delete request should succeed");
        assert_eq!(delete_response.status(), StatusCode::NO_CONTENT);

        wait_for_audit_entries(&recorder, 3).await;
        let entries = recorder.entries().await;
        let appointment_entries: Vec<_> = entries
            .into_iter()
            .filter(|entry| entry.entity_type == "appointment" && entry.entity_id == created.id)
            .collect();

        assert_eq!(appointment_entries.len(), 3);
        assert!(appointment_entries
            .iter()
            .any(|entry| entry.action == AuditAction::Created));
        assert!(appointment_entries
            .iter()
            .any(|entry| entry.action == AuditAction::Updated));
        assert!(appointment_entries
            .iter()
            .any(|entry| matches!(entry.action, AuditAction::Cancelled { .. })));
        assert!(appointment_entries
            .iter()
            .all(|entry| entry.changed_by == user_id));
        assert!(appointment_entries
            .iter()
            .all(|entry| entry.changed_at <= Utc::now()));
    }

    #[tokio::test]
    #[ignore] // Requires PostgreSQL database connection
    async fn audit_events_emitted_on_consultation_mutations() {
        let recorder = Arc::new(RecordingAuditEmitter::default());
        let state = state_with_audit_emitter(recorder.clone()).await;
        let app = router(state.clone());
        let token = login_token(&app, "dr_smith", "correct-horse-battery-staple").await;
        let user_id = state
            .services
            .auth_service
            .validate_session(&token)
            .await
            .expect("session should resolve user");
        recorder.clear().await;

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
            serde_json::from_slice(&patient_body).expect("valid patient");

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

        let update_response = app
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri(format!("/api/v1/consultations/{}", created.id))
                    .header(header::AUTHORIZATION, format!("Bearer {token}"))
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(format!(
                        "{{\"patient_id\":\"{}\",\"practitioner_id\":\"{}\",\"appointment_id\":null,\"reason\":\"Updated reason\",\"clinical_notes\":\"Updated SOAP notes\",\"version\":{}}}",
                        patient.id,
                        practitioner_id,
                        created.version
                    )))
                    .expect("request should be valid"),
            )
            .await
            .expect("request should succeed");
        assert_eq!(update_response.status(), StatusCode::OK);

        wait_for_audit_entries(&recorder, 2).await;
        let entries = recorder.entries().await;
        let consultation_entries: Vec<_> = entries
            .into_iter()
            .filter(|entry| entry.entity_type == "consultation" && entry.entity_id == created.id)
            .collect();

        assert_eq!(consultation_entries.len(), 2);
        assert!(consultation_entries
            .iter()
            .any(|entry| entry.action == AuditAction::Created));
        assert!(consultation_entries
            .iter()
            .any(|entry| entry.action == AuditAction::Updated));
        assert!(consultation_entries
            .iter()
            .all(|entry| entry.changed_by == user_id));
        assert!(consultation_entries
            .iter()
            .all(|entry| entry.changed_at <= Utc::now()));
    }

    #[tokio::test]
    #[ignore] // Requires PostgreSQL database connection
    async fn audit_events_emitted_on_login_attempts() {
        let recorder = Arc::new(RecordingAuditEmitter::default());
        let state = state_with_audit_emitter(recorder.clone()).await;
        let app = router(state);

        let failed_response = app
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
        assert_eq!(failed_response.status(), StatusCode::UNAUTHORIZED);

        wait_for_audit_entries(&recorder, 1).await;
        let entries = recorder.entries().await;
        let failed_auth = entries
            .iter()
            .find(|entry| entry.entity_type == "auth")
            .expect("failed auth audit event should exist");

        assert_eq!(failed_auth.entity_id, Uuid::nil());
        assert_eq!(failed_auth.changed_by, Uuid::nil());
        assert_eq!(failed_auth.action, AuditAction::Created);
        let details = failed_auth
            .new_value
            .clone()
            .expect("details should be populated");
        assert!(details.contains("Failed auth attempt"));
        assert!(!details.contains("wrong-password"));
    }

    #[tokio::test]
    #[ignore] // Requires PostgreSQL database connection
    async fn mutation_requests_do_not_fail_when_audit_emission_fails() {
        let mut state = ApiState::new(test_config())
            .await
            .expect("state should initialize");
        state.audit_emitter = Arc::new(FailingAuditEmitter);
        let app = router(state);
        let token = login_token(&app, "dr_smith", "correct-horse-battery-staple").await;

        let response = app
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

        assert_eq!(response.status(), StatusCode::CREATED);
    }

    #[derive(Default)]
    struct RecordingAuditEmitter {
        entries: RwLock<Vec<AuditEntry>>,
    }

    #[async_trait]
    impl AuditEmitter for RecordingAuditEmitter {
        async fn emit(&self, entry: AuditEntry) -> Result<(), AuditEmitterError> {
            self.entries.write().await.push(entry);
            Ok(())
        }
    }

    impl RecordingAuditEmitter {
        async fn entries(&self) -> Vec<AuditEntry> {
            self.entries.read().await.clone()
        }

        async fn clear(&self) {
            self.entries.write().await.clear();
        }
    }

    struct FailingAuditEmitter;

    #[async_trait]
    impl AuditEmitter for FailingAuditEmitter {
        async fn emit(&self, _entry: AuditEntry) -> Result<(), AuditEmitterError> {
            Err(AuditEmitterError::Emit(
                "intentional test failure".to_string(),
            ))
        }
    }

    async fn state_with_audit_emitter(recorder: Arc<RecordingAuditEmitter>) -> ApiState {
        let mut state = ApiState::new(test_config())
            .await
            .expect("state should initialize");
        state.audit_emitter = recorder;
        state
    }

    async fn wait_for_audit_entries(recorder: &RecordingAuditEmitter, min_count: usize) {
        for _ in 0..30 {
            if recorder.entries.read().await.len() >= min_count {
                return;
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;
        }

        panic!("Timed out waiting for {min_count} audit entries");
    }
}

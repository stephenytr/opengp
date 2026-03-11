use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use http::{header::ORIGIN, HeaderValue, Method, Uri};
use serde::{Deserialize, Serialize};
use tower::ServiceBuilder;
use tower_http::compression::CompressionLayer;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::trace::TraceLayer;

use crate::ApiState;

pub fn router(state: ApiState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/metrics", get(metrics))
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
        .allow_headers([ORIGIN, http::header::CONTENT_TYPE, http::header::ACCEPT])
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

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use http::{Request, StatusCode};
    use tower::util::ServiceExt;

    use crate::ApiConfig;

    use super::*;

    #[tokio::test]
    async fn health_endpoint_returns_ok_with_state_extraction() {
        let config = ApiConfig {
            host: "127.0.0.1".to_string(),
            port: 8080,
            database_url: "postgres://postgres:postgres@127.0.0.1:5432/opengp".to_string(),
            database_max_connections: 10,
            database_min_connections: 2,
            connect_timeout_secs: 30,
            idle_timeout_secs: 600,
            encryption_key: "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
            log_level: "info".to_string(),
        };

        let state = ApiState::new(config).expect("state should initialize");
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
            database_max_connections: 10,
            database_min_connections: 2,
            connect_timeout_secs: 30,
            idle_timeout_secs: 600,
            encryption_key: "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
            log_level: "info".to_string(),
        };

        let state = ApiState::new(config).expect("state should initialize");
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
            database_max_connections: 10,
            database_min_connections: 2,
            connect_timeout_secs: 30,
            idle_timeout_secs: 600,
            encryption_key: "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
            log_level: "info".to_string(),
        };

        let state = ApiState::new(config).expect("state should initialize");
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
}

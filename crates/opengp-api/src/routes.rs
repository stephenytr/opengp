use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use http::{header::ORIGIN, HeaderValue, Method, Uri};
use serde::Serialize;
use tower::ServiceBuilder;
use tower_http::compression::CompressionLayer;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::trace::TraceLayer;

use crate::ApiState;

pub fn router(state: ApiState) -> Router {
    Router::new()
        .route("/health", get(health))
        .with_state(state)
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CompressionLayer::new())
                .layer(cors_layer()),
        )
}

async fn health(State(state): State<ApiState>) -> impl IntoResponse {
    let response = HealthResponse {
        status: "ok",
        db_pool_size: state.pool.size(),
        port: state.config.port,
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

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    db_pool_size: u32,
    port: u16,
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
}

use axum::{
    extract::{Request, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Extension, Json,
};
use chrono::Utc;
use opengp_domain::domain::api::{
    ApiErrorResponse, AuthenticatedUserResponse, LoginRequest, LoginResponse,
};
use opengp_domain::domain::audit::{AuditAction, AuditEntry};
use opengp_domain::domain::user::{self, AuthError};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::ApiState;

use super::middleware::{emit_audit_event_non_blocking, unauthorized_response, AuthContext};

pub(super) async fn login(
    State(state): State<ApiState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Response, (StatusCode, Json<ApiErrorResponse>)> {
    let login = match state.services.auth_service.login(payload.clone()).await {
        Ok(login) => login,
        Err(e) => {
            emit_auth_failure_audit(
                &state,
                Some(payload.username.as_str()),
                "invalid_credentials",
            );
            return Err(auth_error_to_response(e));
        }
    };

    let audit_entry = AuditEntry {
        id: Uuid::new_v4(),
        entity_type: "user".to_string(),
        entity_id: login.user.id,
        action: AuditAction::Created,
        old_value: None,
        new_value: Some(format!("User logged in: {}", login.user.username)),
        changed_by: login.user.id,
        changed_at: Utc::now(),
    };
    emit_audit_event_non_blocking(state.audit_emitter.clone(), audit_entry);

    let access_token = login.access_token.clone();
    let ttl_seconds = state.services.auth_service.session_ttl_seconds();

    let mut http_response = (StatusCode::OK, Json(login)).into_response();
    http_response.headers_mut().insert(
        header::SET_COOKIE,
        HeaderValue::from_str(&session_cookie(&access_token, ttl_seconds))
            .map_err(|_| auth_failed_response())?,
    );

    Ok(http_response)
}

pub(super) async fn logout(
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

pub(super) async fn refresh(
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

pub(super) async fn session_validation_middleware(
    State(state): State<ApiState>,
    mut request: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<ApiErrorResponse>)> {
    let token = match extract_session_token(request.headers()) {
        Some(token) => token,
        None => {
            emit_auth_failure_audit(&state, None, "missing_or_invalid_token");
            return Err(unauthorized_response(
                "unauthorized",
                "Missing or invalid authentication token",
            ));
        }
    };

    let now = Utc::now();
    let session = state
        .services
        .auth_service
        .session_repository
        .find_by_token(&token)
        .await
        .map_err(|err| auth_error_to_response(AuthError::Repository(err)))?;

    if let Some(session) = session {
        if session.is_expired_at(now) {
            state
                .services
                .auth_service
                .session_repository
                .delete_by_token(&token)
                .await
                .map_err(|err| auth_error_to_response(AuthError::Repository(err)))?;

            emit_auth_failure_audit(&state, None, "session_expired");
            return Err(unauthorized_response("unauthorized", "Session expired"));
        }
    }

    let user_id = state
        .services
        .auth_service
        .validate_session(&token)
        .await
        .map_err(|error| {
            emit_auth_failure_audit(&state, None, "session_validation_failed");
            auth_error_to_response(error)
        })?;

    let user = state
        .services
        .auth_service
        .user_repository
        .find_by_id(user_id)
        .await
        .map_err(|_| {
            emit_auth_failure_audit(&state, None, "user_lookup_failed");
            unauthorized_response("unauthorized", "Authentication unavailable")
        })?
        .ok_or_else(|| {
            emit_auth_failure_audit(&state, None, "session_expired");
            unauthorized_response("unauthorized", "Session expired")
        })?;

    request.extensions_mut().insert(AuthContext {
        user_id,
        token,
        role: user.role,
    });
    Ok(next.run(request).await)
}

pub(super) fn auth_error_to_response(error: AuthError) -> (StatusCode, Json<ApiErrorResponse>) {
    match error {
        AuthError::InvalidCredentials | AuthError::AccountLocked => auth_failed_response(),
        AuthError::SessionExpired => unauthorized_response("session_expired", "Session expired"),
        AuthError::Repository(_) => {
            unauthorized_response("unauthorized", "Authentication unavailable")
        }
    }
}

pub(super) fn auth_failed_response() -> (StatusCode, Json<ApiErrorResponse>) {
    unauthorized_response("invalid_credentials", "Invalid username or password")
}

pub(super) fn emit_auth_failure_audit(state: &ApiState, username: Option<&str>, reason: &str) {
    let details = match username {
        Some(name) => format!("Failed auth attempt for username: {name}; reason: {reason}"),
        None => format!("Failed auth attempt; reason: {reason}"),
    };

    let audit_entry = AuditEntry {
        id: Uuid::new_v4(),
        entity_type: "auth".to_string(),
        entity_id: Uuid::nil(),
        action: AuditAction::Created,
        old_value: None,
        new_value: Some(details),
        changed_by: Uuid::nil(),
        changed_at: Utc::now(),
    };

    emit_audit_event_non_blocking(state.audit_emitter.clone(), audit_entry);
}

pub(super) fn extract_session_token(headers: &HeaderMap) -> Option<String> {
    extract_bearer_token(headers)
        .or_else(|| extract_session_cookie(headers))
        .map(std::string::ToString::to_string)
}

pub(super) fn extract_bearer_token(headers: &HeaderMap) -> Option<&str> {
    let value = headers.get(header::AUTHORIZATION)?.to_str().ok()?;
    let token = value.strip_prefix("Bearer ")?.trim();
    (!token.is_empty()).then_some(token)
}

pub(super) fn extract_session_cookie(headers: &HeaderMap) -> Option<&str> {
    let raw_cookie = headers.get(header::COOKIE)?.to_str().ok()?;

    raw_cookie.split(';').find_map(|part| {
        let mut pair = part.trim().splitn(2, '=');
        let name = pair.next()?.trim();
        let value = pair.next()?.trim();
        if name == "session_token" && !value.is_empty() {
            Some(value)
        } else {
            None
        }
    })
}

pub(super) fn session_cookie(token: &str, ttl_seconds: i64) -> String {
    format!(
        "session_token={}; HttpOnly; Path=/; Max-Age={}; SameSite=Lax",
        token,
        ttl_seconds.max(0)
    )
}

#[derive(Serialize, Deserialize)]
pub(super) struct RefreshResponse {
    pub(super) access_token: String,
    pub(super) token_type: String,
    pub(super) expires_in_seconds: i64,
}

#[derive(Serialize, Deserialize)]
pub(super) struct GenericSuccessResponse {
    pub(super) success: bool,
    pub(super) message: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::header;
    use http::{Request, StatusCode};
    use tower::util::ServiceExt;

    #[test]
    fn extract_bearer_token_from_authorization_header() {
        let mut headers = http::HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            http::HeaderValue::from_static("Bearer valid-token-123"),
        );

        let token = extract_bearer_token(&headers);
        assert_eq!(token, Some("valid-token-123"));
    }

    #[test]
    fn extract_bearer_token_returns_none_for_missing_header() {
        let headers = http::HeaderMap::new();
        let token = extract_bearer_token(&headers);
        assert_eq!(token, None);
    }

    #[test]
    fn extract_bearer_token_returns_none_for_invalid_format() {
        let mut headers = http::HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            http::HeaderValue::from_static("Invalid token-123"),
        );

        let token = extract_bearer_token(&headers);
        assert_eq!(token, None);
    }

    #[test]
    fn session_cookie_format_is_correct() {
        let cookie = session_cookie("test-token-123", 3600);

        assert!(cookie.contains("session_token=test-token-123"));
        assert!(cookie.contains("HttpOnly"));
        assert!(cookie.contains("Path=/"));
        assert!(cookie.contains("Max-Age=3600"));
        assert!(cookie.contains("SameSite=Lax"));
    }

    #[test]
    fn extract_session_token_prefers_bearer_over_cookie() {
        let mut headers = http::HeaderMap::new();
        headers.insert(
            header::COOKIE,
            http::HeaderValue::from_static("session_token=cookie-token"),
        );
        headers.insert(
            header::AUTHORIZATION,
            http::HeaderValue::from_static("Bearer bearer-token"),
        );

        let token = extract_session_token(&headers);
        assert_eq!(token.as_deref(), Some("bearer-token"));
    }
}

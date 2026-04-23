use axum::{extract::State, http::StatusCode, Extension, Json};
use opengp_domain::domain::api::{ApiErrorResponse, PractitionerResponse};

use crate::ApiState;

use super::middleware::{
    authorize_read, internal_server_error_response, practitioner_specialty, AuthContext,
};

pub(super) async fn list_practitioners(
    State(state): State<ApiState>,
    Extension(context): Extension<AuthContext>,
) -> Result<(StatusCode, Json<Vec<PractitionerResponse>>), (StatusCode, Json<ApiErrorResponse>)> {
    authorize_read(&context)?;

    let users = state
        .services
        .auth_service
        .user_repository
        .find_all()
        .await
        .map_err(|_| {
            internal_server_error_response("internal_error", "Unable to load practitioners")
        })?;

    let practitioners = users
        .into_iter()
        .filter(|user| user.is_active)
        .filter(|user| matches!(user.role, opengp_domain::domain::user::Role::Doctor | opengp_domain::domain::user::Role::Nurse))
        .map(|user| {
            let title = match user.role {
                opengp_domain::domain::user::Role::Doctor => "Dr".to_string(),
                opengp_domain::domain::user::Role::Nurse => "Nurse".to_string(),
                _ => String::new(),
            };
            PractitionerResponse {
                id: user.id,
                name: user.full_name(),
                title,
                specialty: practitioner_specialty(user.role).to_string(),
            }
        })
        .collect();

    Ok((StatusCode::OK, Json(practitioners)))
}

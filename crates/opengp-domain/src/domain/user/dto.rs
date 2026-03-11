use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::model::{Permission, Role};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewUserData {
    pub username: String,
    pub email: Option<String>,
    pub first_name: String,
    pub last_name: String,
    pub role: Role,
    pub additional_permissions: Option<Vec<Permission>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub user_id: Uuid,
    pub session_token: String,
}

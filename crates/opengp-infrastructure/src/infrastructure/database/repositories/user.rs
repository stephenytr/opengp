use async_trait::async_trait;
use chrono::Utc;
use sqlx::{FromRow, SqlitePool};
use uuid::Uuid;

use opengp_domain::domain::user::{Permission, RepositoryError, Role, User, UserRepository};
use crate::infrastructure::database::helpers as db_helpers;
use crate::infrastructure::database::helpers::{
    bytes_to_uuid, datetime_to_string, string_to_datetime, uuid_to_bytes, DbUuid,
};
use crate::infrastructure::database::sqlx_to_user_error;

#[derive(Debug, FromRow)]
struct UserRow {
    id: DbUuid,
    username: String,
    password_hash_new: Option<String>,
    email: Option<String>,
    first_name: String,
    last_name: String,
    role: String,
    additional_permissions: Option<String>,
    is_active: bool,
    is_locked: bool,
    failed_login_attempts: i64,
    last_login: Option<String>,
    password_changed_at: String,
    created_at: String,
    updated_at: String,
}

impl UserRow {
    fn into_user(self) -> Result<User, RepositoryError> {
        let additional_permissions = match self.additional_permissions {
            Some(json_str) if !json_str.trim().is_empty() => {
                serde_json::from_str::<Vec<Permission>>(&json_str).map_err(|e| {
                    RepositoryError::ConstraintViolation(format!(
                        "Invalid permissions JSON: {}",
                        e
                    ))
                })?
            }
            _ => Vec::new(),
        };

        Ok(User {
            id: bytes_to_uuid(&self.id).map_err(|_| {
                RepositoryError::ConstraintViolation("Invalid UUID bytes".to_string())
            })?,
            username: self.username,
            password_hash: self.password_hash_new,
            email: self.email,
            first_name: self.first_name,
            last_name: self.last_name,
            role: self.role.parse::<Role>().map_err(|_| {
                RepositoryError::ConstraintViolation(format!("Invalid role: {}", self.role))
            })?,
            additional_permissions,
            is_active: self.is_active,
            is_locked: self.is_locked,
            failed_login_attempts: self.failed_login_attempts as u8,
            last_login: self.last_login.map(|s| string_to_datetime(&s)),
            password_changed_at: string_to_datetime(&self.password_changed_at),
            created_at: string_to_datetime(&self.created_at),
            updated_at: string_to_datetime(&self.updated_at),
        })
    }
}

const USER_SELECT_QUERY: &str = r#"
SELECT 
    id, username, password_hash_new, email,
    first_name, last_name,
    role, additional_permissions,
    is_active, is_locked, failed_login_attempts,
    last_login, password_changed_at,
    created_at, updated_at
FROM users
"#;

pub struct SqlxUserRepository {
    pool: SqlitePool,
}

impl SqlxUserRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for SqlxUserRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, RepositoryError> {
        let id_bytes = uuid_to_bytes(&id);

        let row = sqlx::query_as::<_, UserRow>(&db_helpers::sql_with_placeholders(&format!("{}WHERE id = ?", USER_SELECT_QUERY)))
            .bind(id_bytes)
            .fetch_optional(&self.pool)
            .await
            .map_err(sqlx_to_user_error)?;

        match row {
            Some(r) => Ok(Some(r.into_user()?)),
            None => Ok(None),
        }
    }

    async fn find_by_username(&self, username: &str) -> Result<Option<User>, RepositoryError> {
        let row = sqlx::query_as::<_, UserRow>(&db_helpers::sql_with_placeholders(&format!("{}WHERE username = ?", USER_SELECT_QUERY)))
            .bind(username)
            .fetch_optional(&self.pool)
            .await
            .map_err(sqlx_to_user_error)?;

        match row {
            Some(r) => Ok(Some(r.into_user()?)),
            None => Ok(None),
        }
    }

    async fn find_all(&self) -> Result<Vec<User>, RepositoryError> {
        let rows = sqlx::query_as::<_, UserRow>(&db_helpers::sql_with_placeholders(&format!(
            "{}ORDER BY last_name, first_name",
            USER_SELECT_QUERY
        )))
        .fetch_all(&self.pool)
        .await
        .map_err(sqlx_to_user_error)?;

        rows.into_iter().map(|r| r.into_user()).collect()
    }

    async fn find_by_role(&self, role: Role) -> Result<Vec<User>, RepositoryError> {
        let role_str = role.to_string();

        let rows = sqlx::query_as::<_, UserRow>(&db_helpers::sql_with_placeholders(&format!(
            "{}WHERE role = ? ORDER BY last_name, first_name",
            USER_SELECT_QUERY
        )))
        .bind(role_str)
        .fetch_all(&self.pool)
        .await
        .map_err(sqlx_to_user_error)?;

        rows.into_iter().map(|r| r.into_user()).collect()
    }

    async fn create(&self, user: User) -> Result<User, RepositoryError> {
        let id_bytes = uuid_to_bytes(&user.id);
        let role_str = user.role.to_string();
        let additional_permissions_json = if user.additional_permissions.is_empty() {
            "[]".to_string()
        } else {
            serde_json::to_string(&user.additional_permissions).map_err(|e| {
                RepositoryError::ConstraintViolation(format!(
                    "Failed to serialize permissions: {}",
                    e
                ))
            })?
        };
        let failed_login_attempts_i64 = user.failed_login_attempts as i64;
        let last_login_str = user.last_login.map(|dt| datetime_to_string(&dt));
        let password_changed_at_str = datetime_to_string(&user.password_changed_at);
        let created_at_str = datetime_to_string(&user.created_at);
        let updated_at_str = datetime_to_string(&user.updated_at);

        let result = sqlx::query(&db_helpers::sql_with_placeholders(&r#"
        INSERT INTO users (
            id, username, password_hash_new, email,
            first_name, last_name,
            role, additional_permissions,
            is_active, is_locked, failed_login_attempts,
            last_login, password_changed_at,
            created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#))
        .bind(id_bytes)
        .bind(&user.username)
        .bind(&user.password_hash)
        .bind(&user.email)
        .bind(&user.first_name)
        .bind(&user.last_name)
        .bind(role_str)
        .bind(additional_permissions_json)
        .bind(user.is_active)
        .bind(user.is_locked)
        .bind(failed_login_attempts_i64)
        .bind(last_login_str)
        .bind(password_changed_at_str)
        .bind(created_at_str)
        .bind(updated_at_str)
        .execute(&self.pool)
        .await;

        match result {
            Ok(_) => Ok(user),
            Err(sqlx::Error::Database(db_err)) => {
                let err_msg = db_err.message();
                if err_msg.contains("UNIQUE constraint") && err_msg.contains("username") {
                    Err(RepositoryError::ConstraintViolation(
                        "Username already exists".to_string(),
                    ))
                } else if err_msg.contains("NOT NULL constraint") {
                    Err(RepositoryError::ConstraintViolation(
                        "Required field is missing".to_string(),
                    ))
                } else if err_msg.contains("CHECK constraint") {
                    Err(RepositoryError::ConstraintViolation(
                        "Invalid value for field".to_string(),
                    ))
                } else if err_msg.contains("FOREIGN KEY constraint") {
                    Err(RepositoryError::ConstraintViolation(
                        "Referenced record does not exist".to_string(),
                    ))
                } else {
                    Err(RepositoryError::Database(db_err.to_string()))
                }
            }
            Err(e) => Err(RepositoryError::Database(e.to_string())),
        }
    }

    async fn update(&self, user: User) -> Result<User, RepositoryError> {
        let id_bytes = uuid_to_bytes(&user.id);
        let role_str = user.role.to_string();
        let additional_permissions_json = if user.additional_permissions.is_empty() {
            "[]".to_string()
        } else {
            serde_json::to_string(&user.additional_permissions).map_err(|e| {
                RepositoryError::ConstraintViolation(format!(
                    "Failed to serialize permissions: {}",
                    e
                ))
            })?
        };
        let failed_login_attempts_i64 = user.failed_login_attempts as i64;
        let last_login_str = user.last_login.map(|dt| datetime_to_string(&dt));
        let password_changed_at_str = datetime_to_string(&user.password_changed_at);
        let updated_at_str = datetime_to_string(&user.updated_at);

        let result = sqlx::query(&db_helpers::sql_with_placeholders(&r#"
        UPDATE users
        SET username = ?,
            password_hash_new = ?,
            email = ?,
            first_name = ?,
            last_name = ?,
            role = ?,
            additional_permissions = ?,
            is_active = ?,
            is_locked = ?,
            failed_login_attempts = ?,
            last_login = ?,
            password_changed_at = ?,
            updated_at = ?
        WHERE id = ?
        "#))
        .bind(&user.username)
        .bind(&user.password_hash)
        .bind(&user.email)
        .bind(&user.first_name)
        .bind(&user.last_name)
        .bind(role_str)
        .bind(additional_permissions_json)
        .bind(user.is_active)
        .bind(user.is_locked)
        .bind(failed_login_attempts_i64)
        .bind(last_login_str)
        .bind(password_changed_at_str)
        .bind(updated_at_str)
        .bind(id_bytes)
        .execute(&self.pool)
        .await;

        match result {
            Ok(query_result) => {
                if query_result.rows_affected() == 0 {
                    Err(RepositoryError::NotFound)
                } else {
                    Ok(user)
                }
            }
            Err(sqlx::Error::Database(db_err)) => {
                let err_msg = db_err.message();
                if err_msg.contains("UNIQUE constraint") && err_msg.contains("username") {
                    Err(RepositoryError::ConstraintViolation(
                        "Username already exists".to_string(),
                    ))
                } else if err_msg.contains("CHECK constraint") {
                    Err(RepositoryError::ConstraintViolation(
                        "Invalid value for field".to_string(),
                    ))
                } else {
                    Err(RepositoryError::Database(db_err.to_string()))
                }
            }
            Err(e) => Err(RepositoryError::Database(e.to_string())),
        }
    }

    async fn delete(&self, id: Uuid) -> Result<(), RepositoryError> {
        let id_bytes = uuid_to_bytes(&id);
        let updated_at_str = datetime_to_string(&Utc::now());

        let result = sqlx::query(&db_helpers::sql_with_placeholders(&r#"
        UPDATE users
        SET is_active = FALSE,
            updated_at = ?
        WHERE id = ?
        "#))
        .bind(updated_at_str)
        .bind(id_bytes)
        .execute(&self.pool)
        .await;

        match result {
            Ok(query_result) => {
                if query_result.rows_affected() == 0 {
                    Err(RepositoryError::NotFound)
                } else {
                    Ok(())
                }
            }
            Err(e) => Err(RepositoryError::Database(e.to_string())),
        }
    }
}

use async_trait::async_trait;
use chrono::Utc;
use sqlx::{postgres::PgPool, FromRow};
use uuid::Uuid;

use crate::infrastructure::database::helpers::{datetime_to_string, string_to_datetime};
use crate::infrastructure::database::sqlx_to_user_error;
use opengp_domain::domain::user::{Permission, RepositoryError, Role, User, UserRepository};

#[derive(Debug, FromRow)]
struct UserRow {
    id: Uuid,
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
                    RepositoryError::ConstraintViolation(format!("Invalid permissions JSON: {}", e))
                })?
            }
            _ => Vec::new(),
        };

        Ok(User {
            id: self.id,
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

/// SQLx-backed user repository for PostgreSQL
///
/// Implements the domain `UserRepository` trait using the `users`
/// table to store staff accounts, roles, and additional permissions.
pub struct SqlxUserRepository {
    pool: PgPool,
}

impl SqlxUserRepository {
    /// Create a new user repository backed by a PostgreSQL pool
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for SqlxUserRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, RepositoryError> {
        let row = sqlx::query_as::<_, UserRow>(&format!("{}WHERE id = $1", USER_SELECT_QUERY))
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(sqlx_to_user_error)?;

        match row {
            Some(r) => Ok(Some(r.into_user()?)),
            None => Ok(None),
        }
    }

    async fn find_by_username(&self, username: &str) -> Result<Option<User>, RepositoryError> {
        let row =
            sqlx::query_as::<_, UserRow>(&format!("{}WHERE username = $1", USER_SELECT_QUERY))
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
        let rows = sqlx::query_as::<_, UserRow>(&format!(
            "{}ORDER BY last_name, first_name",
            USER_SELECT_QUERY
        ))
        .fetch_all(&self.pool)
        .await
        .map_err(sqlx_to_user_error)?;

        rows.into_iter().map(|r| r.into_user()).collect()
    }

    async fn find_by_role(&self, role: Role) -> Result<Vec<User>, RepositoryError> {
        let role_str = role.to_string();

        let rows = sqlx::query_as::<_, UserRow>(&format!(
            "{}WHERE role = $1 ORDER BY last_name, first_name",
            USER_SELECT_QUERY
        ))
        .bind(role_str)
        .fetch_all(&self.pool)
        .await
        .map_err(sqlx_to_user_error)?;

        rows.into_iter().map(|r| r.into_user()).collect()
    }

    async fn create(&self, user: User) -> Result<User, RepositoryError> {
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

        let result = sqlx::query(
            r#"
        INSERT INTO users (
            id, username, password_hash_new, email,
            first_name, last_name,
            role, additional_permissions,
            is_active, is_locked, failed_login_attempts,
            last_login, password_changed_at,
            created_at, updated_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
        "#,
        )
        .bind(user.id)
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

        let result = sqlx::query(
            r#"
        UPDATE users
        SET username = $1,
            password_hash_new = $2,
            email = $3,
            first_name = $4,
            last_name = $5,
            role = $6,
            additional_permissions = $7,
            is_active = $8,
            is_locked = $9,
            failed_login_attempts = $10,
            last_login = $11,
            password_changed_at = $12,
            updated_at = $13
        WHERE id = $14
        "#,
        )
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
        .bind(user.id)
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
        let updated_at_str = datetime_to_string(&Utc::now());

        let result = sqlx::query(
            r#"
        UPDATE users
        SET is_active = FALSE,
            updated_at = $1
        WHERE id = $2
        "#,
        )
        .bind(updated_at_str)
        .bind(id)
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

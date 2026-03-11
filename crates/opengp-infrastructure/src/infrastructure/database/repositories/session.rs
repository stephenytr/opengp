use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{FromRow, SqlitePool};
use tokio::sync::RwLock;

use opengp_domain::domain::user::{RepositoryError, Session, SessionRepository};

use crate::infrastructure::database::helpers as db_helpers;
use crate::infrastructure::database::helpers::{
    bytes_to_uuid, datetime_to_string, string_to_datetime, uuid_to_bytes, DbUuid,
};

pub struct InMemorySessionRepository {
    sessions: RwLock<Vec<Session>>,
}

impl InMemorySessionRepository {
    pub fn new() -> Self {
        Self {
            sessions: RwLock::new(Vec::new()),
        }
    }
}

impl Default for InMemorySessionRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SessionRepository for InMemorySessionRepository {
    async fn create(&self, session: Session) -> Result<Session, RepositoryError> {
        let mut sessions = self.sessions.write().await;

        if sessions.iter().any(|existing| existing.token == session.token) {
            return Err(RepositoryError::ConstraintViolation(
                "Session token already exists".to_string(),
            ));
        }

        sessions.push(session.clone());
        Ok(session)
    }

    async fn find_by_token(&self, token: &str) -> Result<Option<Session>, RepositoryError> {
        let sessions = self.sessions.read().await;
        Ok(sessions.iter().find(|s| s.token == token).cloned())
    }

    async fn delete_by_token(&self, token: &str) -> Result<(), RepositoryError> {
        let mut sessions = self.sessions.write().await;
        let before = sessions.len();
        sessions.retain(|s| s.token != token);

        if before == sessions.len() {
            Err(RepositoryError::NotFound)
        } else {
            Ok(())
        }
    }

    async fn cleanup_expired(&self, now: DateTime<Utc>) -> Result<u64, RepositoryError> {
        let mut sessions = self.sessions.write().await;
        let before = sessions.len();
        sessions.retain(|session| !session.is_expired_at(now));
        Ok((before - sessions.len()) as u64)
    }
}

#[derive(Debug, FromRow)]
struct SessionRow {
    id: DbUuid,
    user_id: DbUuid,
    created_at: String,
    expires_at: String,
    token: String,
}

impl SessionRow {
    fn into_session(self) -> Result<Session, RepositoryError> {
        Ok(Session {
            id: bytes_to_uuid(&self.id)
                .map_err(|_| RepositoryError::ConstraintViolation("Invalid UUID bytes".to_string()))?,
            user_id: bytes_to_uuid(&self.user_id)
                .map_err(|_| RepositoryError::ConstraintViolation("Invalid user_id bytes".to_string()))?,
            created_at: string_to_datetime(&self.created_at),
            expires_at: string_to_datetime(&self.expires_at),
            token: self.token,
        })
    }
}

const SESSION_SELECT_QUERY: &str = r#"
SELECT id, user_id, created_at, expires_at, token
FROM sessions
"#;

pub struct SqlxSessionRepository {
    pool: SqlitePool,
}

impl SqlxSessionRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SessionRepository for SqlxSessionRepository {
    async fn create(&self, session: Session) -> Result<Session, RepositoryError> {
        let result = sqlx::query(&db_helpers::sql_with_placeholders(&r#"
        INSERT INTO sessions (id, user_id, created_at, expires_at, token)
        VALUES (?, ?, ?, ?, ?)
        "#))
        .bind(uuid_to_bytes(&session.id))
        .bind(uuid_to_bytes(&session.user_id))
        .bind(datetime_to_string(&session.created_at))
        .bind(datetime_to_string(&session.expires_at))
        .bind(&session.token)
        .execute(&self.pool)
        .await;

        match result {
            Ok(_) => Ok(session),
            Err(sqlx::Error::Database(db_err)) => {
                let err_msg = db_err.message();
                if err_msg.contains("UNIQUE constraint") && err_msg.contains("token") {
                    Err(RepositoryError::ConstraintViolation(
                        "Session token already exists".to_string(),
                    ))
                } else {
                    Err(RepositoryError::Database(db_err.to_string()))
                }
            }
            Err(e) => Err(RepositoryError::Database(e.to_string())),
        }
    }

    async fn find_by_token(&self, token: &str) -> Result<Option<Session>, RepositoryError> {
        let row = sqlx::query_as::<_, SessionRow>(&db_helpers::sql_with_placeholders(&format!("{}WHERE token = ?", SESSION_SELECT_QUERY)))
            .bind(token)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        match row {
            Some(r) => Ok(Some(r.into_session()?)),
            None => Ok(None),
        }
    }

    async fn delete_by_token(&self, token: &str) -> Result<(), RepositoryError> {
        let result = sqlx::query(&db_helpers::sql_with_placeholders(&"DELETE FROM sessions WHERE token = ?"))
            .bind(token)
            .execute(&self.pool)
            .await
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        if result.rows_affected() == 0 {
            Err(RepositoryError::NotFound)
        } else {
            Ok(())
        }
    }

    async fn cleanup_expired(&self, now: DateTime<Utc>) -> Result<u64, RepositoryError> {
        let result = sqlx::query(&db_helpers::sql_with_placeholders(&"DELETE FROM sessions WHERE expires_at <= ?"))
            .bind(datetime_to_string(&now))
            .execute(&self.pool)
            .await
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(result.rows_affected())
    }
}

#[cfg(test)]
mod tests {
    use chrono::Duration;
    use uuid::Uuid;

    use super::*;
    use crate::infrastructure::database::helpers::{datetime_to_string, uuid_to_bytes};
    use crate::infrastructure::database::test_utils::create_test_pool;

    fn build_session(user_id: Uuid, token: &str, expires_in_minutes: i64) -> Session {
        let now = Utc::now();
        Session {
            id: Uuid::new_v4(),
            user_id,
            created_at: now,
            expires_at: now + Duration::minutes(expires_in_minutes),
            token: token.to_string(),
        }
    }

    #[tokio::test]
    async fn in_memory_repository_supports_create_find_and_cleanup() {
        let repo = InMemorySessionRepository::new();
        let user_id = Uuid::new_v4();

        repo.create(build_session(user_id, "active-token", 60))
            .await
            .expect("create should succeed");
        repo.create(build_session(user_id, "expired-token", -5))
            .await
            .expect("create should succeed");

        assert!(repo
            .find_by_token("active-token")
            .await
            .expect("query should succeed")
            .is_some());

        let removed = repo
            .cleanup_expired(Utc::now())
            .await
            .expect("cleanup should succeed");
        assert_eq!(removed, 1);

        assert!(repo
            .find_by_token("expired-token")
            .await
            .expect("query should succeed")
            .is_none());
    }

    #[tokio::test]
    async fn sqlx_repository_creates_finds_and_deletes_session() {
        let pool = create_test_pool().await.expect("pool should initialize");
        let user_id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query(&db_helpers::sql_with_placeholders(&r#"
        INSERT INTO users (
            id, username, password_hash, role, is_active, created_at, updated_at,
            first_name, last_name
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#))
        .bind(uuid_to_bytes(&user_id))
        .bind("session-test-user")
        .bind("hash")
        .bind("Doctor")
        .bind(true)
        .bind(datetime_to_string(&now))
        .bind(datetime_to_string(&now))
        .bind("Test")
        .bind("User")
        .execute(&pool)
        .await
        .expect("user insert should succeed");

        let repo = SqlxSessionRepository::new(pool);
        let session = build_session(user_id, "db-token", 30);

        repo.create(session.clone())
            .await
            .expect("session create should succeed");

        let found = repo
            .find_by_token("db-token")
            .await
            .expect("query should succeed")
            .expect("session should exist");
        assert_eq!(found.token, session.token);
        assert_eq!(found.user_id, session.user_id);

        repo.delete_by_token("db-token")
            .await
            .expect("delete should succeed");
        assert!(repo
            .find_by_token("db-token")
            .await
            .expect("query should succeed")
            .is_none());
    }
}

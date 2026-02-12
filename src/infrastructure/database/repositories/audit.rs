use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{FromRow, SqlitePool};
use uuid::Uuid;

use crate::domain::audit::{AuditAction, AuditEntry, AuditRepository, RepositoryError};

#[derive(Debug, FromRow)]
struct AuditLogRow {
    id: Vec<u8>,
    entity_type: String,
    entity_id: Vec<u8>,
    action: String,
    old_value: Option<String>,
    new_value: Option<String>,
    changed_by: Vec<u8>,
    changed_at: String,
}

impl AuditLogRow {
    fn into_audit_entry(self) -> Result<AuditEntry, RepositoryError> {
        let action: AuditAction = serde_json::from_str(&self.action).map_err(|e| {
            RepositoryError::ConstraintViolation(format!(
                "Failed to deserialize AuditAction: {}",
                e
            ))
        })?;

        Ok(AuditEntry {
            id: Uuid::from_slice(&self.id).map_err(|e| {
                RepositoryError::ConstraintViolation(format!("Invalid UUID: {}", e))
            })?,
            entity_type: self.entity_type,
            entity_id: Uuid::from_slice(&self.entity_id).map_err(|e| {
                RepositoryError::ConstraintViolation(format!("Invalid entity UUID: {}", e))
            })?,
            action,
            old_value: self.old_value,
            new_value: self.new_value,
            changed_by: Uuid::from_slice(&self.changed_by).map_err(|e| {
                RepositoryError::ConstraintViolation(format!("Invalid user UUID: {}", e))
            })?,
            changed_at: DateTime::parse_from_rfc3339(&self.changed_at)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
        })
    }
}

pub struct SqlxAuditRepository {
    pool: SqlitePool,
}

impl SqlxAuditRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AuditRepository for SqlxAuditRepository {
    async fn create(&self, entry: AuditEntry) -> Result<AuditEntry, RepositoryError> {
        let id_bytes = entry.id.as_bytes().to_vec();
        let entity_id_bytes = entry.entity_id.as_bytes().to_vec();
        let changed_by_bytes = entry.changed_by.as_bytes().to_vec();
        let changed_at_str = entry.changed_at.to_rfc3339();

        let action_json = serde_json::to_string(&entry.action).map_err(|e| {
            RepositoryError::ConstraintViolation(format!("Failed to serialize AuditAction: {}", e))
        })?;

        let result = sqlx::query(
            r#"
            INSERT INTO audit_logs (
                id, entity_type, entity_id, action,
                old_value, new_value,
                changed_by, changed_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(id_bytes)
        .bind(&entry.entity_type)
        .bind(entity_id_bytes)
        .bind(action_json)
        .bind(&entry.old_value)
        .bind(&entry.new_value)
        .bind(changed_by_bytes)
        .bind(changed_at_str)
        .execute(&self.pool)
        .await;

        match result {
            Ok(_) => Ok(entry),
            Err(sqlx::Error::Database(db_err)) => {
                let err_msg = db_err.message();
                if err_msg.contains("FOREIGN KEY constraint") {
                    Err(RepositoryError::ConstraintViolation(
                        "User does not exist".to_string(),
                    ))
                } else if err_msg.contains("NOT NULL constraint") {
                    Err(RepositoryError::ConstraintViolation(
                        "Required field is missing".to_string(),
                    ))
                } else {
                    Err(RepositoryError::Database(sqlx::Error::Database(db_err)))
                }
            }
            Err(e) => Err(RepositoryError::Database(e)),
        }
    }

    async fn find_by_entity(
        &self,
        entity_type: &str,
        entity_id: Uuid,
    ) -> Result<Vec<AuditEntry>, RepositoryError> {
        let entity_id_bytes = entity_id.as_bytes().to_vec();

        let rows = sqlx::query_as::<_, AuditLogRow>(
            r#"
            SELECT
                id, entity_type, entity_id, action,
                old_value, new_value,
                changed_by, changed_at
            FROM audit_logs
            WHERE entity_type = ? AND entity_id = ?
            ORDER BY changed_at ASC
            "#,
        )
        .bind(entity_type)
        .bind(entity_id_bytes)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(|r| r.into_audit_entry()).collect()
    }

    async fn find_by_user(&self, user_id: Uuid) -> Result<Vec<AuditEntry>, RepositoryError> {
        let user_id_bytes = user_id.as_bytes().to_vec();

        let rows = sqlx::query_as::<_, AuditLogRow>(
            r#"
            SELECT
                id, entity_type, entity_id, action,
                old_value, new_value,
                changed_by, changed_at
            FROM audit_logs
            WHERE changed_by = ?
            ORDER BY changed_at DESC
            "#,
        )
        .bind(user_id_bytes)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(|r| r.into_audit_entry()).collect()
    }

    async fn find_by_time_range(
        &self,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<Vec<AuditEntry>, RepositoryError> {
        let start_time_str = start_time.to_rfc3339();
        let end_time_str = end_time.to_rfc3339();

        let rows = sqlx::query_as::<_, AuditLogRow>(
            r#"
            SELECT
                id, entity_type, entity_id, action,
                old_value, new_value,
                changed_by, changed_at
            FROM audit_logs
            WHERE changed_at >= ? AND changed_at <= ?
            ORDER BY changed_at ASC
            "#,
        )
        .bind(start_time_str)
        .bind(end_time_str)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(|r| r.into_audit_entry()).collect()
    }

    async fn find_by_entity_and_time_range(
        &self,
        entity_type: &str,
        entity_id: Uuid,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<Vec<AuditEntry>, RepositoryError> {
        let entity_id_bytes = entity_id.as_bytes().to_vec();
        let start_time_str = start_time.to_rfc3339();
        let end_time_str = end_time.to_rfc3339();

        let rows = sqlx::query_as::<_, AuditLogRow>(
            r#"
            SELECT
                id, entity_type, entity_id, action,
                old_value, new_value,
                changed_by, changed_at
            FROM audit_logs
            WHERE entity_type = ? 
              AND entity_id = ? 
              AND changed_at >= ? 
              AND changed_at <= ?
            ORDER BY changed_at ASC
            "#,
        )
        .bind(entity_type)
        .bind(entity_id_bytes)
        .bind(start_time_str)
        .bind(end_time_str)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(|r| r.into_audit_entry()).collect()
    }
}

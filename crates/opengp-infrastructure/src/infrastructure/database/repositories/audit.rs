use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{FromRow, SqlitePool};
use uuid::Uuid;

use opengp_domain::domain::error::RepositoryError as BaseRepositoryError;
use opengp_domain::domain::audit::{AuditAction, AuditEntry, AuditRepository, AuditRepositoryError};
use crate::infrastructure::database::helpers as db_helpers;
use crate::infrastructure::database::sqlx_to_audit_error;

fn bytes_to_uuid(bytes: &db_helpers::DbUuid) -> Result<Uuid, AuditRepositoryError> {
    db_helpers::bytes_to_uuid(bytes).map_err(|_| {
        AuditRepositoryError::Base(BaseRepositoryError::ConstraintViolation(
            "Invalid UUID bytes".to_string(),
        ))
    })
}

fn string_to_datetime(s: &str) -> DateTime<Utc> {
    db_helpers::string_to_datetime(s)
}

const AUDIT_SELECT_QUERY: &str = r#"
    SELECT
        id, entity_type, entity_id, action,
        old_value, new_value,
        changed_by, changed_at
    FROM audit_logs
"#;

#[derive(Debug, FromRow)]
struct AuditLogRow {
    id: db_helpers::DbUuid,
    entity_type: String,
    entity_id: db_helpers::DbUuid,
    action: String,
    old_value: Option<String>,
    new_value: Option<String>,
    changed_by: db_helpers::DbUuid,
    changed_at: String,
}

impl AuditLogRow {
    fn into_audit_entry(self) -> Result<AuditEntry, AuditRepositoryError> {
        let action: AuditAction = serde_json::from_str(&self.action).map_err(|e| {
            AuditRepositoryError::Base(BaseRepositoryError::ConstraintViolation(format!(
                "Failed to deserialize AuditAction: {}",
                e
            )))
        })?;

        Ok(AuditEntry {
            id: bytes_to_uuid(&self.id)?,
            entity_type: self.entity_type,
            entity_id: bytes_to_uuid(&self.entity_id)?,
            action,
            old_value: self.old_value,
            new_value: self.new_value,
            changed_by: bytes_to_uuid(&self.changed_by)?,
            changed_at: string_to_datetime(&self.changed_at),
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
    async fn create(&self, entry: AuditEntry) -> Result<AuditEntry, AuditRepositoryError> {
        let id_bytes = db_helpers::uuid_to_bytes(&entry.id);
        let entity_id_bytes = db_helpers::uuid_to_bytes(&entry.entity_id);
        let changed_by_bytes = db_helpers::uuid_to_bytes(&entry.changed_by);
        let changed_at_str = entry.changed_at.to_rfc3339();

        let action_json = serde_json::to_string(&entry.action).map_err(|e| {
            AuditRepositoryError::Base(BaseRepositoryError::ConstraintViolation(format!(
                "Failed to serialize AuditAction: {}",
                e
            )))
        })?;

        let result = sqlx::query(&db_helpers::sql_with_placeholders(&r#"
        INSERT INTO audit_logs (
            id, entity_type, entity_id, action,
            old_value, new_value,
            changed_by, changed_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        "#))
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
                    Err(AuditRepositoryError::Base(BaseRepositoryError::ConstraintViolation(
                        "User does not exist".to_string(),
                    )))
                } else if err_msg.contains("NOT NULL constraint") {
                    Err(AuditRepositoryError::Base(BaseRepositoryError::ConstraintViolation(
                        "Required field is missing".to_string(),
                    )))
                } else {
                    Err(AuditRepositoryError::Base(BaseRepositoryError::Database(
                        db_err.to_string(),
                    )))
                }
            }
            Err(e) => Err(AuditRepositoryError::Base(BaseRepositoryError::Database(
                e.to_string(),
            ))),
        }
    }

    async fn find_by_entity(
        &self,
        entity_type: &str,
        entity_id: Uuid,
    ) -> Result<Vec<AuditEntry>, AuditRepositoryError> {
        let entity_id_bytes = db_helpers::uuid_to_bytes(&entity_id);

        let rows = sqlx::query_as::<_, AuditLogRow>(&db_helpers::sql_with_placeholders(&format!(
            "{}WHERE entity_type = ? AND entity_id = ? ORDER BY changed_at ASC",
            AUDIT_SELECT_QUERY
        )))
        .bind(entity_type)
        .bind(entity_id_bytes)
        .fetch_all(&self.pool)
        .await
            .map_err(sqlx_to_audit_error)?;

        rows.into_iter().map(|r| r.into_audit_entry()).collect()
    }

    async fn find_by_user(&self, user_id: Uuid) -> Result<Vec<AuditEntry>, AuditRepositoryError> {
        let user_id_bytes = db_helpers::uuid_to_bytes(&user_id);

        let rows = sqlx::query_as::<_, AuditLogRow>(&db_helpers::sql_with_placeholders(&format!(
            "{}WHERE changed_by = ? ORDER BY changed_at DESC",
            AUDIT_SELECT_QUERY
        )))
        .bind(user_id_bytes)
        .fetch_all(&self.pool)
        .await
            .map_err(sqlx_to_audit_error)?;

        rows.into_iter().map(|r| r.into_audit_entry()).collect()
    }

    async fn find_by_time_range(
        &self,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<Vec<AuditEntry>, AuditRepositoryError> {
        let start_time_str = start_time.to_rfc3339();
        let end_time_str = end_time.to_rfc3339();

        let rows = sqlx::query_as::<_, AuditLogRow>(&db_helpers::sql_with_placeholders(&format!(
            "{}WHERE changed_at >= ? AND changed_at <= ? ORDER BY changed_at ASC",
            AUDIT_SELECT_QUERY
        )))
        .bind(start_time_str)
        .bind(end_time_str)
        .fetch_all(&self.pool)
        .await
            .map_err(sqlx_to_audit_error)?;

        rows.into_iter().map(|r| r.into_audit_entry()).collect()
    }

    async fn find_by_entity_and_time_range(
        &self,
        entity_type: &str,
        entity_id: Uuid,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<Vec<AuditEntry>, AuditRepositoryError> {
        let entity_id_bytes = db_helpers::uuid_to_bytes(&entity_id);
        let start_time_str = start_time.to_rfc3339();
        let end_time_str = end_time.to_rfc3339();

        let rows = sqlx::query_as::<_, AuditLogRow>(&db_helpers::sql_with_placeholders(&format!(
            "{}WHERE entity_type = ? AND entity_id = ? AND changed_at >= ? AND changed_at <= ? ORDER BY changed_at ASC",
            AUDIT_SELECT_QUERY
        )))
        .bind(entity_type)
        .bind(entity_id_bytes)
        .bind(start_time_str)
        .bind(end_time_str)
        .fetch_all(&self.pool)
        .await
            .map_err(sqlx_to_audit_error)?;

        rows.into_iter().map(|r| r.into_audit_entry()).collect()
    }
}

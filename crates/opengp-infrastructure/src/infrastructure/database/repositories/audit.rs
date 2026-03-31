use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::infrastructure::database::sqlx_to_audit_error;
use opengp_domain::domain::audit::{
    AuditAction, AuditEntry, AuditRepository, AuditRepositoryError,
};
use opengp_domain::domain::error::RepositoryError as BaseRepositoryError;

const AUDIT_SELECT_QUERY: &str = r#"
    SELECT
        id, entity_type, entity_id, action,
        old_value, new_value,
        changed_by, changed_at, source
    FROM audit_logs
"#;

#[derive(Debug, FromRow)]
struct AuditLogRow {
    id: Uuid,
    entity_type: String,
    entity_id: Uuid,
    action: String,
    old_value: Option<String>,
    new_value: Option<String>,
    changed_by: Uuid,
    changed_at: DateTime<Utc>,
    source: String,
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
            id: self.id,
            entity_type: self.entity_type,
            entity_id: self.entity_id,
            action,
            old_value: self.old_value,
            new_value: self.new_value,
            changed_by: self.changed_by,
            changed_at: self.changed_at,
            source: self.source,
        })
    }
}

/// SQLx-backed audit repository for PostgreSQL
///
/// Persists `AuditEntry` records in the `audit_logs` table so
/// user and system actions can be reviewed for compliance.
pub struct SqlxAuditRepository {
    pool: PgPool,
}

impl SqlxAuditRepository {
    /// Create a new audit repository backed by a PostgreSQL pool
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AuditRepository for SqlxAuditRepository {
    async fn create(&self, entry: AuditEntry) -> Result<AuditEntry, AuditRepositoryError> {
        let action_json = serde_json::to_string(&entry.action).map_err(|e| {
            AuditRepositoryError::Base(BaseRepositoryError::ConstraintViolation(format!(
                "Failed to serialize AuditAction: {}",
                e
            )))
        })?;

        let result = sqlx::query(
            r#"
        INSERT INTO audit_logs (
            id, entity_type, entity_id, action,
            old_value, new_value,
            changed_by, changed_at, source
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        "#,
        )
        .bind(entry.id)
        .bind(&entry.entity_type)
        .bind(entry.entity_id)
        .bind(action_json)
        .bind(&entry.old_value)
        .bind(&entry.new_value)
        .bind(entry.changed_by)
        .bind(entry.changed_at)
        .bind(&entry.source)
        .execute(&self.pool)
        .await;

        match result {
            Ok(_) => Ok(entry),
            Err(sqlx::Error::Database(db_err)) => {
                let err_msg = db_err.message();
                if err_msg.contains("FOREIGN KEY constraint") {
                    Err(AuditRepositoryError::Base(
                        BaseRepositoryError::ConstraintViolation("User does not exist".to_string()),
                    ))
                } else if err_msg.contains("NOT NULL constraint") {
                    Err(AuditRepositoryError::Base(
                        BaseRepositoryError::ConstraintViolation(
                            "Required field is missing".to_string(),
                        ),
                    ))
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
        let rows = sqlx::query_as::<_, AuditLogRow>(&format!(
            "{}WHERE entity_type = $1 AND entity_id = $2 ORDER BY changed_at ASC",
            AUDIT_SELECT_QUERY
        ))
        .bind(entity_type)
        .bind(entity_id)
        .fetch_all(&self.pool)
        .await
        .map_err(sqlx_to_audit_error)?;

        rows.into_iter().map(|r| r.into_audit_entry()).collect()
    }

    async fn find_by_user(&self, user_id: Uuid) -> Result<Vec<AuditEntry>, AuditRepositoryError> {
        let rows = sqlx::query_as::<_, AuditLogRow>(&format!(
            "{}WHERE changed_by = $1 ORDER BY changed_at DESC",
            AUDIT_SELECT_QUERY
        ))
        .bind(user_id)
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
        let rows = sqlx::query_as::<_, AuditLogRow>(&format!(
            "{}WHERE changed_at >= $1 AND changed_at <= $2 ORDER BY changed_at ASC",
            AUDIT_SELECT_QUERY
        ))
        .bind(start_time)
        .bind(end_time)
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
        let rows = sqlx::query_as::<_, AuditLogRow>(&format!(
            "{}WHERE entity_type = $1 AND entity_id = $2 AND changed_at >= $3 AND changed_at <= $4 ORDER BY changed_at ASC",
            AUDIT_SELECT_QUERY
        ))
        .bind(entity_type)
        .bind(entity_id)
        .bind(start_time)
        .bind(end_time)
        .fetch_all(&self.pool)
        .await
            .map_err(sqlx_to_audit_error)?;

        rows.into_iter().map(|r| r.into_audit_entry()).collect()
    }
}

use async_trait::async_trait;
use sqlx::{FromRow, SqlitePool};
use uuid::Uuid;

use opengp_domain::domain::user::{RepositoryError, WorkingHours, WorkingHoursRepository};
use crate::infrastructure::database::helpers as db_helpers;
use crate::infrastructure::database::helpers::{
    bytes_to_uuid, datetime_to_string, string_to_datetime, uuid_to_bytes, DbUuid,
};

#[derive(Debug, FromRow)]
struct WorkingHoursRow {
    id: DbUuid,
    practitioner_id: DbUuid,
    day_of_week: i64,
    start_time: String,
    end_time: String,
    is_active: bool,
    created_at: String,
    updated_at: String,
}

impl WorkingHoursRow {
    fn into_working_hours(self) -> Result<WorkingHours, RepositoryError> {
        let start_time = chrono::NaiveTime::parse_from_str(&self.start_time, "%H:%M:%S")
            .map_err(|_| {
                RepositoryError::ConstraintViolation(format!("Invalid start_time format: {}", self.start_time))
            })?;

        let end_time = chrono::NaiveTime::parse_from_str(&self.end_time, "%H:%M:%S")
            .map_err(|_| {
                RepositoryError::ConstraintViolation(format!("Invalid end_time format: {}", self.end_time))
            })?;

        Ok(WorkingHours {
            id: bytes_to_uuid(&self.id).map_err(|_| {
                RepositoryError::ConstraintViolation("Invalid UUID bytes".to_string())
            })?,
            practitioner_id: bytes_to_uuid(&self.practitioner_id).map_err(|_| {
                RepositoryError::ConstraintViolation("Invalid practitioner UUID bytes".to_string())
            })?,
            day_of_week: self.day_of_week as u8,
            start_time,
            end_time,
            is_active: self.is_active,
            created_at: string_to_datetime(&self.created_at),
            updated_at: string_to_datetime(&self.updated_at),
        })
    }
}

const WORKING_HOURS_SELECT_QUERY: &str = r#"
SELECT 
    id, practitioner_id, day_of_week,
    start_time, end_time,
    is_active,
    created_at, updated_at
FROM working_hours
"#;

pub struct SqlxWorkingHoursRepository {
    pool: SqlitePool,
}

impl SqlxWorkingHoursRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl WorkingHoursRepository for SqlxWorkingHoursRepository {
    async fn find_by_practitioner(
        &self,
        practitioner_id: Uuid,
    ) -> Result<Vec<WorkingHours>, RepositoryError> {
        let practitioner_id_bytes = uuid_to_bytes(&practitioner_id);

        let rows = sqlx::query_as::<_, WorkingHoursRow>(&db_helpers::sql_with_placeholders(&format!(
            "{}WHERE practitioner_id = ? ORDER BY day_of_week",
            WORKING_HOURS_SELECT_QUERY
        )))
        .bind(practitioner_id_bytes)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        rows.into_iter().map(|r| r.into_working_hours()).collect()
    }

    async fn find_for_day(
        &self,
        practitioner_id: Uuid,
        day_of_week: u8,
    ) -> Result<Option<WorkingHours>, RepositoryError> {
        let practitioner_id_bytes = uuid_to_bytes(&practitioner_id);
        let day_of_week_i64 = day_of_week as i64;

        let row = sqlx::query_as::<_, WorkingHoursRow>(&db_helpers::sql_with_placeholders(&format!(
            "{}WHERE practitioner_id = ? AND day_of_week = ?",
            WORKING_HOURS_SELECT_QUERY
        )))
        .bind(practitioner_id_bytes)
        .bind(day_of_week_i64)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        match row {
            Some(r) => Ok(Some(r.into_working_hours()?)),
            None => Ok(None),
        }
    }

    async fn save(&self, working_hours: WorkingHours) -> Result<WorkingHours, RepositoryError> {
        let id_bytes = uuid_to_bytes(&working_hours.id);
        let practitioner_id_bytes = uuid_to_bytes(&working_hours.practitioner_id);
        let day_of_week_i64 = working_hours.day_of_week as i64;
        let start_time_str = working_hours.start_time.format("%H:%M:%S").to_string();
        let end_time_str = working_hours.end_time.format("%H:%M:%S").to_string();
        let created_at_str = datetime_to_string(&working_hours.created_at);
        let updated_at_str = datetime_to_string(&working_hours.updated_at);

        let existing = self
            .find_for_day(working_hours.practitioner_id, working_hours.day_of_week)
            .await?;

        if existing.is_some() {
            let result = sqlx::query(&db_helpers::sql_with_placeholders(&r#"
            UPDATE working_hours
            SET start_time = ?,
                end_time = ?,
                is_active = ?,
                updated_at = ?
            WHERE practitioner_id = ? AND day_of_week = ?
            "#))
            .bind(&start_time_str)
            .bind(&end_time_str)
            .bind(working_hours.is_active)
            .bind(&updated_at_str)
            .bind(&practitioner_id_bytes)
            .bind(day_of_week_i64)
            .execute(&self.pool)
            .await;

            match result {
                Ok(_) => Ok(working_hours),
                Err(sqlx::Error::Database(db_err)) => {
                    let err_msg = db_err.message();
                    if err_msg.contains("FOREIGN KEY constraint") {
                        Err(RepositoryError::ConstraintViolation(
                            "Referenced practitioner does not exist".to_string(),
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
        } else {
            let result = sqlx::query(&db_helpers::sql_with_placeholders(&r#"
            INSERT INTO working_hours (
                id, practitioner_id, day_of_week,
                start_time, end_time,
                is_active,
                created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#))
            .bind(id_bytes)
            .bind(practitioner_id_bytes)
            .bind(day_of_week_i64)
            .bind(&start_time_str)
            .bind(&end_time_str)
            .bind(working_hours.is_active)
            .bind(&created_at_str)
            .bind(&updated_at_str)
            .execute(&self.pool)
            .await;

            match result {
                Ok(_) => Ok(working_hours),
                Err(sqlx::Error::Database(db_err)) => {
                    let err_msg = db_err.message();
                    if err_msg.contains("UNIQUE constraint") {
                        Err(RepositoryError::ConstraintViolation(
                            "Working hours already exists for this practitioner on this day"
                                .to_string(),
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
                            "Referenced practitioner does not exist".to_string(),
                        ))
                    } else {
                        Err(RepositoryError::Database(db_err.to_string()))
                    }
                }
                Err(e) => Err(RepositoryError::Database(e.to_string())),
            }
        }
    }

    async fn delete(&self, id: Uuid) -> Result<(), RepositoryError> {
        let id_bytes = uuid_to_bytes(&id);

        let result = sqlx::query(&db_helpers::sql_with_placeholders(&r#"
        DELETE FROM working_hours
        WHERE id = ?
        "#))
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

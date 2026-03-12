use async_trait::async_trait;
use chrono::{DateTime, NaiveTime, Utc};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use opengp_domain::domain::user::{RepositoryError, WorkingHours, WorkingHoursRepository};

#[derive(Debug, FromRow)]
struct WorkingHoursRow {
    id: Uuid,
    practitioner_id: Uuid,
    day_of_week: i64,
    start_time: NaiveTime,
    end_time: NaiveTime,
    is_active: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl WorkingHoursRow {
    fn into_working_hours(self) -> Result<WorkingHours, RepositoryError> {
        Ok(WorkingHours {
            id: self.id,
            practitioner_id: self.practitioner_id,
            day_of_week: self.day_of_week as u8,
            start_time: self.start_time,
            end_time: self.end_time,
            is_active: self.is_active,
            created_at: self.created_at,
            updated_at: self.updated_at,
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
    pool: PgPool,
}

impl SqlxWorkingHoursRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl WorkingHoursRepository for SqlxWorkingHoursRepository {
    async fn find_by_practitioner(
        &self,
        practitioner_id: Uuid,
    ) -> Result<Vec<WorkingHours>, RepositoryError> {
        let rows =
            sqlx::query_as::<_, WorkingHoursRow>(&format!(
                "{}WHERE practitioner_id = $1 ORDER BY day_of_week",
                WORKING_HOURS_SELECT_QUERY
            ))
            .bind(practitioner_id)
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
        let day_of_week_i64 = day_of_week as i64;

        let row =
            sqlx::query_as::<_, WorkingHoursRow>(&format!(
                "{}WHERE practitioner_id = $1 AND day_of_week = $2",
                WORKING_HOURS_SELECT_QUERY
            ))
            .bind(practitioner_id)
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
        let day_of_week_i64 = working_hours.day_of_week as i64;

        let existing = self
            .find_for_day(working_hours.practitioner_id, working_hours.day_of_week)
            .await?;

        if existing.is_some() {
            let result = sqlx::query(
                r#"
            UPDATE working_hours
            SET start_time = $1,
                end_time = $2,
                is_active = $3,
                updated_at = $4
            WHERE practitioner_id = $5 AND day_of_week = $6
            "#,
            )
            .bind(working_hours.start_time)
            .bind(working_hours.end_time)
            .bind(working_hours.is_active)
            .bind(working_hours.updated_at)
            .bind(working_hours.practitioner_id)
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
            let result = sqlx::query(
                r#"
            INSERT INTO working_hours (
                id, practitioner_id, day_of_week,
                start_time, end_time,
                is_active,
                created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
            )
            .bind(working_hours.id)
            .bind(working_hours.practitioner_id)
            .bind(day_of_week_i64)
            .bind(working_hours.start_time)
            .bind(working_hours.end_time)
            .bind(working_hours.is_active)
            .bind(working_hours.created_at)
            .bind(working_hours.updated_at)
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
        let result = sqlx::query(
            r#"
        DELETE FROM working_hours
        WHERE id = $1
        "#,
        )
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

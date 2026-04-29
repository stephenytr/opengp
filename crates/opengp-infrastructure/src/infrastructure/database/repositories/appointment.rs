use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{postgres::PgPool, FromRow, Postgres, QueryBuilder};
use uuid::Uuid;

use crate::infrastructure::database::sqlx_to_appointment_error;
use opengp_domain::domain::appointment::{
    Appointment, AppointmentCalendarQuery, AppointmentRepository, AppointmentSearchCriteria,
    AppointmentStatus, AppointmentType, CalendarAppointment, RepositoryError,
};

#[derive(Debug, FromRow)]
struct AppointmentRow {
    id: Uuid,
    patient_id: Uuid,
    practitioner_id: Uuid,
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
    appointment_type: String,
    status: String,
    reason: Option<String>,
    notes: Option<String>,
    is_urgent: bool,
    reminder_sent: bool,
    confirmed: bool,
    cancellation_reason: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    version: i32,
    created_by: Option<Uuid>,
    updated_by: Option<Uuid>,
}

impl AppointmentRow {
    fn into_appointment(self) -> Result<Appointment, RepositoryError> {
        Ok(Appointment {
            id: self.id,
            patient_id: self.patient_id,
            practitioner_id: self.practitioner_id,
            start_time: self.start_time,
            end_time: self.end_time,
            appointment_type: self
                .appointment_type
                .parse::<AppointmentType>()
                .unwrap_or(AppointmentType::Standard),
            status: self
                .status
                .parse::<AppointmentStatus>()
                .unwrap_or(AppointmentStatus::Scheduled),
            reason: self.reason,
            notes: self.notes,
            is_urgent: self.is_urgent,
            reminder_sent: self.reminder_sent,
            confirmed: self.confirmed,
            cancellation_reason: self.cancellation_reason,
            created_at: self.created_at,
            updated_at: self.updated_at,
            version: self.version,
            created_by: self.created_by,
            updated_by: self.updated_by,
        })
    }
}

#[derive(Debug, FromRow)]
struct CalendarAppointmentRow {
    id: Uuid,
    patient_id: Uuid,
    practitioner_id: Uuid,
    patient_name: Option<String>,
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
    appointment_type: String,
    status: String,
    is_urgent: bool,
    #[allow(dead_code)]
    confirmed: bool,
    reason: Option<String>,
    notes: Option<String>,
}

impl CalendarAppointmentRow {
    fn into_calendar_appointment(self) -> Result<CalendarAppointment, RepositoryError> {
        let start_time = self.start_time;
        let end_time = self.end_time;

        // Calculate slot_span: number of 15-minute slots
        let duration_minutes = (end_time - start_time).num_minutes();
        let slot_span = ((duration_minutes as f64 / 15.0).ceil() as u8).max(1);

        Ok(CalendarAppointment {
            id: self.id,
            patient_id: self.patient_id,
            practitioner_id: self.practitioner_id,
            patient_name: self
                .patient_name
                .unwrap_or_else(|| "Unknown Patient".to_string()),
            start_time,
            end_time,
            appointment_type: self
                .appointment_type
                .parse::<AppointmentType>()
                .unwrap_or(AppointmentType::Standard),
            status: self
                .status
                .parse::<AppointmentStatus>()
                .unwrap_or(AppointmentStatus::Scheduled),
            is_urgent: self.is_urgent,
            slot_span,
            reason: self.reason,
            notes: self.notes,
            is_overlapping: false,
        })
    }
}

const APPOINTMENT_SELECT_QUERY: &str = r#"
SELECT 
    id, patient_id, practitioner_id,
    start_time, end_time,
    appointment_type, status,
    reason, notes,
    is_urgent, reminder_sent, confirmed,
    cancellation_reason,
    created_at, updated_at,
    version,
    created_by, updated_by
FROM appointments
"#;

const CALENDAR_APPOINTMENT_SELECT_QUERY: &str = r#"
SELECT 
    a.id,
    a.patient_id,
    a.practitioner_id,
    COALESCE(p.preferred_name, p.first_name) || ' ' || p.last_name as patient_name,
    a.start_time,
    a.end_time,
    a.appointment_type,
    a.status,
    a.is_urgent,
    a.confirmed,
    a.reason,
    a.notes
FROM appointments a
LEFT JOIN patients p ON a.patient_id = p.id
"#;

/// SQLx-backed appointment repository for PostgreSQL
///
/// Stores appointments for Australian general practice workflows,
/// including status, type, urgency, and audit fields.
pub struct SqlxAppointmentRepository {
    pool: PgPool,
}

impl SqlxAppointmentRepository {
    /// Create a new appointment repository backed by a PostgreSQL pool
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn appointment_type_to_str(appointment_type: AppointmentType) -> &'static str {
        match appointment_type {
            AppointmentType::Standard => "Standard",
            AppointmentType::Long => "Long",
            AppointmentType::Brief => "Brief",
            AppointmentType::NewPatient => "NewPatient",
            AppointmentType::HealthAssessment => "HealthAssessment",
            AppointmentType::ChronicDiseaseReview => "ChronicDiseaseReview",
            AppointmentType::MentalHealthPlan => "MentalHealthPlan",
            AppointmentType::Immunisation => "Immunisation",
            AppointmentType::Procedure => "Procedure",
            AppointmentType::Telephone => "Telephone",
            AppointmentType::Telehealth => "Telehealth",
            AppointmentType::HomeVisit => "HomeVisit",
            AppointmentType::Emergency => "Emergency",
        }
    }

    fn appointment_status_to_str(status: AppointmentStatus) -> &'static str {
        match status {
            AppointmentStatus::Scheduled => "Scheduled",
            AppointmentStatus::Confirmed => "Confirmed",
            AppointmentStatus::Arrived => "Arrived",
            AppointmentStatus::InProgress => "InProgress",
            AppointmentStatus::Billing => "Billing",
            AppointmentStatus::Completed => "Completed",
            AppointmentStatus::NoShow => "NoShow",
            AppointmentStatus::Cancelled => "Cancelled",
            AppointmentStatus::Rescheduled => "Rescheduled",
        }
    }
}

#[async_trait]
impl AppointmentRepository for SqlxAppointmentRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Appointment>, RepositoryError> {
        let row = sqlx::query_as::<_, AppointmentRow>(&format!(
            "{}WHERE id = $1",
            APPOINTMENT_SELECT_QUERY
        ))
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(sqlx_to_appointment_error)?;

        match row {
            Some(r) => Ok(Some(r.into_appointment()?)),
            None => Ok(None),
        }
    }

    async fn create(&self, appointment: Appointment) -> Result<Appointment, RepositoryError> {
        let appointment_type_str = match appointment.appointment_type {
            AppointmentType::Standard => "Standard",
            AppointmentType::Long => "Long",
            AppointmentType::Brief => "Brief",
            AppointmentType::NewPatient => "NewPatient",
            AppointmentType::HealthAssessment => "HealthAssessment",
            AppointmentType::ChronicDiseaseReview => "ChronicDiseaseReview",
            AppointmentType::MentalHealthPlan => "MentalHealthPlan",
            AppointmentType::Immunisation => "Immunisation",
            AppointmentType::Procedure => "Procedure",
            AppointmentType::Telephone => "Telephone",
            AppointmentType::Telehealth => "Telehealth",
            AppointmentType::HomeVisit => "HomeVisit",
            AppointmentType::Emergency => "Emergency",
        };
        let status_str = match appointment.status {
            AppointmentStatus::Scheduled => "Scheduled",
            AppointmentStatus::Confirmed => "Confirmed",
            AppointmentStatus::Arrived => "Arrived",
            AppointmentStatus::InProgress => "InProgress",
            AppointmentStatus::Billing => "Billing",
            AppointmentStatus::Completed => "Completed",
            AppointmentStatus::NoShow => "NoShow",
            AppointmentStatus::Cancelled => "Cancelled",
            AppointmentStatus::Rescheduled => "Rescheduled",
        };

        let result = sqlx::query(
            r#"
        INSERT INTO appointments (
            id, patient_id, practitioner_id,
            start_time, end_time,
            appointment_type, status,
            reason, notes,
            is_urgent, reminder_sent, confirmed,
            cancellation_reason,
            created_at, updated_at,
            version,
            created_by, updated_by
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18)
        "#,
        )
        .bind(appointment.id)
        .bind(appointment.patient_id)
        .bind(appointment.practitioner_id)
        .bind(appointment.start_time)
        .bind(appointment.end_time)
        .bind(appointment_type_str)
        .bind(status_str)
        .bind(&appointment.reason)
        .bind(&appointment.notes)
        .bind(appointment.is_urgent)
        .bind(appointment.reminder_sent)
        .bind(appointment.confirmed)
        .bind(&appointment.cancellation_reason)
        .bind(appointment.created_at)
        .bind(appointment.updated_at)
        .bind(appointment.version)
        .bind(appointment.created_by)
        .bind(appointment.updated_by)
        .execute(&self.pool)
        .await;

        match result {
            Ok(_) => Ok(appointment),
            Err(sqlx::Error::Database(db_err)) => {
                let err_msg = db_err.message();
                if err_msg.contains("FOREIGN KEY constraint") {
                    Err(RepositoryError::ConstraintViolation(
                        "Patient or practitioner does not exist".to_string(),
                    ))
                } else if err_msg.contains("NOT NULL constraint") {
                    Err(RepositoryError::ConstraintViolation(
                        "Required field is missing".to_string(),
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

    async fn update(&self, appointment: Appointment) -> Result<Appointment, RepositoryError> {
        let appointment_type_str = match appointment.appointment_type {
            AppointmentType::Standard => "Standard",
            AppointmentType::Long => "Long",
            AppointmentType::Brief => "Brief",
            AppointmentType::NewPatient => "NewPatient",
            AppointmentType::HealthAssessment => "HealthAssessment",
            AppointmentType::ChronicDiseaseReview => "ChronicDiseaseReview",
            AppointmentType::MentalHealthPlan => "MentalHealthPlan",
            AppointmentType::Immunisation => "Immunisation",
            AppointmentType::Procedure => "Procedure",
            AppointmentType::Telephone => "Telephone",
            AppointmentType::Telehealth => "Telehealth",
            AppointmentType::HomeVisit => "HomeVisit",
            AppointmentType::Emergency => "Emergency",
        };
        let status_str = match appointment.status {
            AppointmentStatus::Scheduled => "Scheduled",
            AppointmentStatus::Confirmed => "Confirmed",
            AppointmentStatus::Arrived => "Arrived",
            AppointmentStatus::InProgress => "InProgress",
            AppointmentStatus::Billing => "Billing",
            AppointmentStatus::Completed => "Completed",
            AppointmentStatus::NoShow => "NoShow",
            AppointmentStatus::Cancelled => "Cancelled",
            AppointmentStatus::Rescheduled => "Rescheduled",
        };

        let current_version =
            sqlx::query_scalar::<_, i32>("SELECT version FROM appointments WHERE id = $1")
                .bind(appointment.id)
                .fetch_optional(&self.pool)
                .await
                .map_err(sqlx_to_appointment_error)?;

        let current_version = match current_version {
            Some(version) => version,
            None => return Err(RepositoryError::NotFound),
        };

        if current_version != appointment.version {
            return Err(RepositoryError::Conflict(
                "Appointment was modified by another user".to_string(),
            ));
        }

        let new_version = appointment.version + 1;

        let result = sqlx::query(
            r#"
        UPDATE appointments
        SET start_time = $1,
            end_time = $2,
            appointment_type = $3,
            status = $4,
            reason = $5,
            notes = $6,
            is_urgent = $7,
            reminder_sent = $8,
            confirmed = $9,
            cancellation_reason = $10,
            updated_at = $11,
            updated_by = $12,
            version = $13
        WHERE id = $14 AND version = $15
        "#,
        )
        .bind(appointment.start_time)
        .bind(appointment.end_time)
        .bind(appointment_type_str)
        .bind(status_str)
        .bind(&appointment.reason)
        .bind(&appointment.notes)
        .bind(appointment.is_urgent)
        .bind(appointment.reminder_sent)
        .bind(appointment.confirmed)
        .bind(&appointment.cancellation_reason)
        .bind(appointment.updated_at)
        .bind(appointment.updated_by)
        .bind(new_version)
        .bind(appointment.id)
        .bind(appointment.version)
        .execute(&self.pool)
        .await;

        match result {
            Ok(query_result) => {
                if query_result.rows_affected() == 0 {
                    Err(RepositoryError::Conflict(
                        "Appointment was modified by another user".to_string(),
                    ))
                } else {
                    let mut updated_appointment = appointment;
                    updated_appointment.version = new_version;
                    Ok(updated_appointment)
                }
            }
            Err(sqlx::Error::Database(db_err)) => {
                let err_msg = db_err.message();
                if err_msg.contains("CHECK constraint") {
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
        let result = sqlx::query(
            r#"
        DELETE FROM appointments
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

    async fn find_by_criteria(
        &self,
        criteria: &AppointmentSearchCriteria,
    ) -> Result<Vec<Appointment>, RepositoryError> {
        let mut query_builder = QueryBuilder::<Postgres>::new(APPOINTMENT_SELECT_QUERY);
        let mut has_condition = false;

        if let Some(patient_id) = criteria.patient_id {
            query_builder.push(if has_condition { " AND " } else { " WHERE " });
            query_builder.push("patient_id = ").push_bind(patient_id);
            has_condition = true;
        }

        if let Some(practitioner_id) = criteria.practitioner_id {
            query_builder.push(if has_condition { " AND " } else { " WHERE " });
            query_builder
                .push("practitioner_id = ")
                .push_bind(practitioner_id);
            has_condition = true;
        }

        if let Some(date_from) = criteria.date_from {
            query_builder.push(if has_condition { " AND " } else { " WHERE " });
            query_builder.push("start_time >= ").push_bind(date_from);
            has_condition = true;
        }

        if let Some(date_to) = criteria.date_to {
            query_builder.push(if has_condition { " AND " } else { " WHERE " });
            query_builder.push("start_time < ").push_bind(date_to);
            has_condition = true;
        }

        if let Some(appointment_type) = criteria.appointment_type {
            query_builder.push(if has_condition { " AND " } else { " WHERE " });
            query_builder
                .push("appointment_type = ")
                .push_bind(Self::appointment_type_to_str(appointment_type));
            has_condition = true;
        }

        if let Some(status) = criteria.status {
            query_builder.push(if has_condition { " AND " } else { " WHERE " });
            query_builder
                .push("status = ")
                .push_bind(Self::appointment_status_to_str(status));
        }

        let limit = criteria.limit.unwrap_or(100).max(1);
        query_builder
            .push(" ORDER BY start_time LIMIT ")
            .push_bind(limit);

        let rows = query_builder
            .build_query_as::<AppointmentRow>()
            .fetch_all(&self.pool)
            .await
            .map_err(sqlx_to_appointment_error)?;

        rows.into_iter().map(|r| r.into_appointment()).collect()
    }

    async fn find_overlapping(
        &self,
        practitioner_id: Uuid,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<Vec<Appointment>, RepositoryError> {
        let rows = sqlx::query_as::<_, AppointmentRow>(&format!(
            "{}WHERE practitioner_id = $1 AND start_time < $2 AND end_time > $3 AND status NOT IN ('Cancelled', 'NoShow') ORDER BY start_time",
            APPOINTMENT_SELECT_QUERY
        ))
        .bind(practitioner_id)
        .bind(end_time)
        .bind(start_time)
        .fetch_all(&self.pool)
        .await
            .map_err(sqlx_to_appointment_error)?;

        rows.into_iter().map(|r| r.into_appointment()).collect()
    }
}

#[async_trait]
impl AppointmentCalendarQuery for SqlxAppointmentRepository {
    async fn find_calendar_appointments(
        &self,
        criteria: &AppointmentSearchCriteria,
    ) -> Result<Vec<CalendarAppointment>, RepositoryError> {
        let mut query_builder = QueryBuilder::<Postgres>::new(CALENDAR_APPOINTMENT_SELECT_QUERY);
        let mut has_condition = false;

        if let Some(patient_id) = criteria.patient_id {
            query_builder.push(if has_condition { " AND " } else { " WHERE " });
            query_builder.push("a.patient_id = ").push_bind(patient_id);
            has_condition = true;
        }

        if let Some(practitioner_id) = criteria.practitioner_id {
            query_builder.push(if has_condition { " AND " } else { " WHERE " });
            query_builder
                .push("a.practitioner_id = ")
                .push_bind(practitioner_id);
            has_condition = true;
        }

        if let Some(date_from) = criteria.date_from {
            query_builder.push(if has_condition { " AND " } else { " WHERE " });
            query_builder.push("a.start_time >= ").push_bind(date_from);
            has_condition = true;
        }

        if let Some(date_to) = criteria.date_to {
            query_builder.push(if has_condition { " AND " } else { " WHERE " });
            query_builder.push("a.start_time < ").push_bind(date_to);
            has_condition = true;
        }

        if let Some(appointment_type) = criteria.appointment_type {
            query_builder.push(if has_condition { " AND " } else { " WHERE " });
            query_builder
                .push("a.appointment_type = ")
                .push_bind(Self::appointment_type_to_str(appointment_type));
            has_condition = true;
        }

        if let Some(status) = criteria.status {
            query_builder.push(if has_condition { " AND " } else { " WHERE " });
            query_builder
                .push("a.status = ")
                .push_bind(Self::appointment_status_to_str(status));
            has_condition = true;
        } else {
            query_builder.push(if has_condition { " AND " } else { " WHERE " });
            query_builder.push("a.status NOT IN ('Cancelled', 'NoShow')");
            has_condition = true;
        }

        if let Some(is_urgent) = criteria.is_urgent {
            query_builder.push(if has_condition { " AND " } else { " WHERE " });
            query_builder.push("a.is_urgent = ").push_bind(is_urgent);
            has_condition = true;
        }

        if let Some(confirmed) = criteria.confirmed {
            query_builder.push(if has_condition { " AND " } else { " WHERE " });
            query_builder.push("a.confirmed = ").push_bind(confirmed);
        }

        let limit = criteria.limit.unwrap_or(100).max(1);
        query_builder
            .push(" ORDER BY a.start_time LIMIT ")
            .push_bind(limit);

        let rows = query_builder
            .build_query_as::<CalendarAppointmentRow>()
            .fetch_all(&self.pool)
            .await
            .map_err(sqlx_to_appointment_error)?;

        rows.into_iter()
            .map(|r| r.into_calendar_appointment())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::database::test_utils::create_test_pool;
    use chrono::Duration;

    #[tokio::test]
    #[ignore] // Requires PostgreSQL database connection
    async fn update_uses_optimistic_locking_and_increments_version() {
        let pool = create_test_pool().await.expect("pool should initialize");
        let repo = SqlxAppointmentRepository::new(pool.clone());

        let mut appointment = Appointment::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Utc::now() + Duration::hours(1),
            Duration::minutes(15),
            AppointmentType::Standard,
            Some(Uuid::new_v4()),
        );
        appointment.reason = Some("Initial reason".to_string());

        let created = repo
            .create(appointment.clone())
            .await
            .expect("create appointment");
        assert_eq!(created.version, 1);

        let mut update_a = created.clone();
        update_a.reason = Some("First update".to_string());
        update_a.updated_at = Utc::now();

        let updated = repo
            .update(update_a)
            .await
            .expect("first update should succeed");
        assert_eq!(updated.version, 2);

        let mut stale_update = created.clone();
        stale_update.reason = Some("Stale update".to_string());
        stale_update.updated_at = Utc::now();

        let err = repo
            .update(stale_update)
            .await
            .expect_err("stale update should conflict");

        assert!(matches!(err, RepositoryError::Conflict(_)));

        let latest = repo
            .find_by_id(created.id)
            .await
            .expect("find latest")
            .expect("appointment exists");
        assert_eq!(latest.version, 2);
    }
}

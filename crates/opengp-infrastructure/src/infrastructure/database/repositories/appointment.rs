use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{FromRow, SqlitePool};
use uuid::Uuid;

use crate::infrastructure::database::helpers as db_helpers;
use crate::infrastructure::database::sqlx_to_appointment_error;
use opengp_domain::domain::appointment::{
    Appointment, AppointmentCalendarQuery, AppointmentRepository, AppointmentSearchCriteria,
    AppointmentStatus, AppointmentType, CalendarAppointment, RepositoryError,
};

fn bytes_to_uuid(bytes: &db_helpers::DbUuid) -> Result<Uuid, RepositoryError> {
    db_helpers::bytes_to_uuid(bytes)
        .map_err(|_| RepositoryError::ConstraintViolation("Invalid UUID bytes".to_string()))
}

fn string_to_datetime(s: &str) -> DateTime<Utc> {
    db_helpers::string_to_datetime(s)
}

#[derive(Debug, FromRow)]
struct AppointmentRow {
    id: db_helpers::DbUuid,
    patient_id: db_helpers::DbUuid,
    practitioner_id: db_helpers::DbUuid,
    start_time: String,
    end_time: String,
    appointment_type: String,
    status: String,
    reason: Option<String>,
    notes: Option<String>,
    is_urgent: bool,
    reminder_sent: bool,
    confirmed: bool,
    cancellation_reason: Option<String>,
    created_at: String,
    updated_at: String,
    version: i32,
    created_by: Option<db_helpers::DbUuid>,
    updated_by: Option<db_helpers::DbUuid>,
}

impl AppointmentRow {
    fn into_appointment(self) -> Result<Appointment, RepositoryError> {
        Ok(Appointment {
            id: bytes_to_uuid(&self.id)?,
            patient_id: bytes_to_uuid(&self.patient_id)?,
            practitioner_id: bytes_to_uuid(&self.practitioner_id)?,
            start_time: string_to_datetime(&self.start_time),
            end_time: string_to_datetime(&self.end_time),
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
            created_at: string_to_datetime(&self.created_at),
            updated_at: string_to_datetime(&self.updated_at),
            version: self.version,
            created_by: self.created_by.and_then(|bytes| bytes_to_uuid(&bytes).ok()),
            updated_by: self.updated_by.and_then(|bytes| bytes_to_uuid(&bytes).ok()),
        })
    }
}

#[derive(Debug, FromRow)]
struct CalendarAppointmentRow {
    id: db_helpers::DbUuid,
    patient_id: db_helpers::DbUuid,
    practitioner_id: db_helpers::DbUuid,
    patient_name: Option<String>,
    start_time: String,
    end_time: String,
    appointment_type: String,
    status: String,
    is_urgent: bool,
    confirmed: bool,
    reason: Option<String>,
    notes: Option<String>,
}

impl CalendarAppointmentRow {
    fn into_calendar_appointment(self) -> Result<CalendarAppointment, RepositoryError> {
        let start_time = string_to_datetime(&self.start_time);
        let end_time = string_to_datetime(&self.end_time);

        // Calculate slot_span: number of 15-minute slots
        let duration_minutes = (end_time - start_time).num_minutes();
        let slot_span = ((duration_minutes as f64 / 15.0).ceil() as u8).max(1);

        Ok(CalendarAppointment {
            id: bytes_to_uuid(&self.id)?,
            patient_id: bytes_to_uuid(&self.patient_id)?,
            practitioner_id: bytes_to_uuid(&self.practitioner_id)?,
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

pub struct SqlxAppointmentRepository {
    pool: SqlitePool,
}

impl SqlxAppointmentRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AppointmentRepository for SqlxAppointmentRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Appointment>, RepositoryError> {
        let id_bytes = db_helpers::uuid_to_bytes(&id);

        let row = sqlx::query_as::<_, AppointmentRow>(&db_helpers::sql_with_placeholders(
            &format!("{}WHERE id = ?", APPOINTMENT_SELECT_QUERY),
        ))
        .bind(id_bytes)
        .fetch_optional(&self.pool)
        .await
        .map_err(sqlx_to_appointment_error)?;

        match row {
            Some(r) => Ok(Some(r.into_appointment()?)),
            None => Ok(None),
        }
    }

    async fn create(&self, appointment: Appointment) -> Result<Appointment, RepositoryError> {
        let id_bytes = db_helpers::uuid_to_bytes(&appointment.id);
        let patient_id_bytes = db_helpers::uuid_to_bytes(&appointment.patient_id);
        let practitioner_id_bytes = db_helpers::uuid_to_bytes(&appointment.practitioner_id);
        let start_time_str = appointment.start_time.to_rfc3339();
        let end_time_str = appointment.end_time.to_rfc3339();
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
            AppointmentStatus::Completed => "Completed",
            AppointmentStatus::NoShow => "NoShow",
            AppointmentStatus::Cancelled => "Cancelled",
            AppointmentStatus::Rescheduled => "Rescheduled",
        };
        let created_at_str = appointment.created_at.to_rfc3339();
        let updated_at_str = appointment.updated_at.to_rfc3339();
        let created_by_bytes = appointment
            .created_by
            .map(|id| db_helpers::uuid_to_bytes(&id));
        let updated_by_bytes = appointment
            .updated_by
            .map(|id| db_helpers::uuid_to_bytes(&id));

        let result = sqlx::query(&db_helpers::sql_with_placeholders(
            &r#"
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
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
        ))
        .bind(id_bytes)
        .bind(patient_id_bytes)
        .bind(practitioner_id_bytes)
        .bind(start_time_str)
        .bind(end_time_str)
        .bind(appointment_type_str)
        .bind(status_str)
        .bind(&appointment.reason)
        .bind(&appointment.notes)
        .bind(appointment.is_urgent)
        .bind(appointment.reminder_sent)
        .bind(appointment.confirmed)
        .bind(&appointment.cancellation_reason)
        .bind(created_at_str)
        .bind(updated_at_str)
        .bind(appointment.version)
        .bind(created_by_bytes)
        .bind(updated_by_bytes)
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
        let id_bytes = db_helpers::uuid_to_bytes(&appointment.id);
        let start_time_str = appointment.start_time.to_rfc3339();
        let end_time_str = appointment.end_time.to_rfc3339();
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
            AppointmentStatus::Completed => "Completed",
            AppointmentStatus::NoShow => "NoShow",
            AppointmentStatus::Cancelled => "Cancelled",
            AppointmentStatus::Rescheduled => "Rescheduled",
        };
        let updated_at_str = appointment.updated_at.to_rfc3339();
        let updated_by_bytes = appointment
            .updated_by
            .map(|id| db_helpers::uuid_to_bytes(&id));

        let current_version = sqlx::query_scalar::<_, i32>(&db_helpers::sql_with_placeholders(
            "SELECT version FROM appointments WHERE id = ?",
        ))
        .bind(id_bytes.clone())
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

        let result = sqlx::query(&db_helpers::sql_with_placeholders(
            &r#"
        UPDATE appointments
        SET start_time = ?,
            end_time = ?,
            appointment_type = ?,
            status = ?,
            reason = ?,
            notes = ?,
            is_urgent = ?,
            reminder_sent = ?,
            confirmed = ?,
            cancellation_reason = ?,
            updated_at = ?,
            updated_by = ?,
            version = ?
        WHERE id = ? AND version = ?
        "#,
        ))
        .bind(start_time_str)
        .bind(end_time_str)
        .bind(appointment_type_str)
        .bind(status_str)
        .bind(&appointment.reason)
        .bind(&appointment.notes)
        .bind(appointment.is_urgent)
        .bind(appointment.reminder_sent)
        .bind(appointment.confirmed)
        .bind(&appointment.cancellation_reason)
        .bind(updated_at_str)
        .bind(updated_by_bytes)
        .bind(new_version)
        .bind(id_bytes)
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
        let id_bytes = db_helpers::uuid_to_bytes(&id);

        let result = sqlx::query(&db_helpers::sql_with_placeholders(
            &r#"
        DELETE FROM appointments
        WHERE id = ?
        "#,
        ))
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

    async fn find_by_criteria(
        &self,
        criteria: &AppointmentSearchCriteria,
    ) -> Result<Vec<Appointment>, RepositoryError> {
        let mut query = String::from(
            r#"
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
            WHERE 1=1
            "#,
        );

        if let Some(patient_id) = criteria.patient_id {
            query.push_str(" AND patient_id = ?");
        }

        if let Some(practitioner_id) = criteria.practitioner_id {
            query.push_str(" AND practitioner_id = ?");
        }

        if let Some(date_from) = criteria.date_from {
            query.push_str(" AND start_time >= ?");
        }

        if let Some(date_to) = criteria.date_to {
            query.push_str(" AND start_time < ?");
        }

        if let Some(appointment_type) = criteria.appointment_type {
            let _type_str = match appointment_type {
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
            query.push_str(" AND appointment_type = ?");
        }

        if let Some(status) = criteria.status {
            let status_str = match status {
                AppointmentStatus::Scheduled => "Scheduled",
                AppointmentStatus::Confirmed => "Confirmed",
                AppointmentStatus::Arrived => "Arrived",
                AppointmentStatus::InProgress => "InProgress",
                AppointmentStatus::Completed => "Completed",
                AppointmentStatus::NoShow => "NoShow",
                AppointmentStatus::Cancelled => "Cancelled",
                AppointmentStatus::Rescheduled => "Rescheduled",
            };
            query.push_str(" AND status = ?");
        }

        query.push_str(" ORDER BY start_time");

        let patient_id_bytes = criteria.patient_id.map(|id| db_helpers::uuid_to_bytes(&id));
        let practitioner_id_bytes = criteria
            .practitioner_id
            .map(|id| db_helpers::uuid_to_bytes(&id));
        let date_from_str = criteria.date_from.map(|dt| dt.to_rfc3339());
        let date_to_str = criteria.date_to.map(|dt| dt.to_rfc3339());

        let all_rows = sqlx::query_as::<_, AppointmentRow>(&db_helpers::sql_with_placeholders(
            &format!("{}ORDER BY start_time", APPOINTMENT_SELECT_QUERY),
        ))
        .fetch_all(&self.pool)
        .await
        .map_err(sqlx_to_appointment_error)?;

        let rows: Vec<AppointmentRow> = all_rows
            .into_iter()
            .filter(|row| {
                if let Some(ref pid_bytes) = patient_id_bytes {
                    if &row.patient_id != pid_bytes {
                        return false;
                    }
                }
                if let Some(ref prid_bytes) = practitioner_id_bytes {
                    if &row.practitioner_id != prid_bytes {
                        return false;
                    }
                }
                if let Some(ref start) = date_from_str {
                    if &row.start_time < start {
                        return false;
                    }
                }
                if let Some(ref end) = date_to_str {
                    if &row.start_time >= end {
                        return false;
                    }
                }
                if let Some(atype) = criteria.appointment_type {
                    let type_str = match atype {
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
                    if row.appointment_type != type_str {
                        return false;
                    }
                }
                if let Some(st) = criteria.status {
                    let status_str = match st {
                        AppointmentStatus::Scheduled => "Scheduled",
                        AppointmentStatus::Confirmed => "Confirmed",
                        AppointmentStatus::Arrived => "Arrived",
                        AppointmentStatus::InProgress => "InProgress",
                        AppointmentStatus::Completed => "Completed",
                        AppointmentStatus::NoShow => "NoShow",
                        AppointmentStatus::Cancelled => "Cancelled",
                        AppointmentStatus::Rescheduled => "Rescheduled",
                    };
                    if row.status != status_str {
                        return false;
                    }
                }
                true
            })
            .collect();

        rows.into_iter().map(|r| r.into_appointment()).collect()
    }

    async fn find_overlapping(
        &self,
        practitioner_id: Uuid,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<Vec<Appointment>, RepositoryError> {
        let practitioner_id_bytes = db_helpers::uuid_to_bytes(&practitioner_id);
        let start_time_str = start_time.to_rfc3339();
        let end_time_str = end_time.to_rfc3339();

        let rows = sqlx::query_as::<_, AppointmentRow>(&db_helpers::sql_with_placeholders(&format!(
            "{}WHERE practitioner_id = ? AND start_time < ? AND end_time > ? AND status NOT IN ('Cancelled', 'NoShow') ORDER BY start_time",
            APPOINTMENT_SELECT_QUERY
        )))
        .bind(practitioner_id_bytes)
        .bind(end_time_str)
        .bind(start_time_str)
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
        let mut query = String::from(
            r#"
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
            WHERE 1=1
            "#,
        );

        if let Some(patient_id) = criteria.patient_id {
            query.push_str(" AND a.patient_id = ?");
        }

        if let Some(practitioner_id) = criteria.practitioner_id {
            query.push_str(" AND a.practitioner_id = ?");
        }

        if let Some(date_from) = criteria.date_from {
            query.push_str(" AND a.start_time >= ?");
        }

        if let Some(date_to) = criteria.date_to {
            query.push_str(" AND a.start_time < ?");
        }

        if let Some(appointment_type) = criteria.appointment_type {
            let _type_str = match appointment_type {
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
            query.push_str(" AND a.appointment_type = ?");
        }

        if let Some(_status) = criteria.status {
            let _status_str = match _status {
                AppointmentStatus::Scheduled => "Scheduled",
                AppointmentStatus::Confirmed => "Confirmed",
                AppointmentStatus::Arrived => "Arrived",
                AppointmentStatus::InProgress => "InProgress",
                AppointmentStatus::Completed => "Completed",
                AppointmentStatus::NoShow => "NoShow",
                AppointmentStatus::Cancelled => "Cancelled",
                AppointmentStatus::Rescheduled => "Rescheduled",
            };
            query.push_str(" AND a.status = ?");
        }

        if let Some(_is_urgent) = criteria.is_urgent {
            query.push_str(" AND a.is_urgent = ?");
        }

        if let Some(_confirmed) = criteria.confirmed {
            query.push_str(" AND a.confirmed = ?");
        }

        query.push_str(" ORDER BY a.start_time");

        let rows = if criteria.patient_id.is_some()
            || criteria.practitioner_id.is_some()
            || criteria.date_from.is_some()
            || criteria.date_to.is_some()
            || criteria.appointment_type.is_some()
            || criteria.status.is_some()
            || criteria.is_urgent.is_some()
            || criteria.confirmed.is_some()
        {
            let patient_id_bytes = criteria.patient_id.map(|id| db_helpers::uuid_to_bytes(&id));
            let practitioner_id_bytes = criteria
                .practitioner_id
                .map(|id| db_helpers::uuid_to_bytes(&id));
            let date_from_str = criteria.date_from.map(|dt| dt.to_rfc3339());
            let date_to_str = criteria.date_to.map(|dt| dt.to_rfc3339());

            let all_rows =
                sqlx::query_as::<_, CalendarAppointmentRow>(&db_helpers::sql_with_placeholders(
                    &format!("{}ORDER BY a.start_time", CALENDAR_APPOINTMENT_SELECT_QUERY),
                ))
                .fetch_all(&self.pool)
                .await
                .map_err(sqlx_to_appointment_error)?;

            all_rows
                .into_iter()
                .filter(|row| {
                    if let Some(ref pid_bytes) = patient_id_bytes {
                        if &row.patient_id != pid_bytes {
                            return false;
                        }
                    }
                    if let Some(ref prid_bytes) = practitioner_id_bytes {
                        if &row.practitioner_id != prid_bytes {
                            return false;
                        }
                    }
                    if let Some(ref start) = date_from_str {
                        if &row.start_time < start {
                            return false;
                        }
                    }
                    if let Some(ref end) = date_to_str {
                        if &row.start_time >= end {
                            return false;
                        }
                    }
                    if let Some(atype) = criteria.appointment_type {
                        let type_str = match atype {
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
                        if row.appointment_type != type_str {
                            return false;
                        }
                    }
                    if let Some(st) = criteria.status {
                        let status_str = match st {
                            AppointmentStatus::Scheduled => "Scheduled",
                            AppointmentStatus::Confirmed => "Confirmed",
                            AppointmentStatus::Arrived => "Arrived",
                            AppointmentStatus::InProgress => "InProgress",
                            AppointmentStatus::Completed => "Completed",
                            AppointmentStatus::NoShow => "NoShow",
                            AppointmentStatus::Cancelled => "Cancelled",
                            AppointmentStatus::Rescheduled => "Rescheduled",
                        };
                        if row.status != status_str {
                            return false;
                        }
                    }
                    if let Some(urgent) = criteria.is_urgent {
                        if row.is_urgent != urgent {
                            return false;
                        }
                    }
                    if let Some(conf) = criteria.confirmed {
                        if row.confirmed != conf {
                            return false;
                        }
                    }
                    true
                })
                .collect()
        } else {
            sqlx::query_as::<_, CalendarAppointmentRow>(&db_helpers::sql_with_placeholders(
                &format!("{}ORDER BY a.start_time", CALENDAR_APPOINTMENT_SELECT_QUERY),
            ))
            .fetch_all(&self.pool)
            .await
            .map_err(sqlx_to_appointment_error)?
        };

        rows.into_iter()
            .map(|r| r.into_calendar_appointment())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn setup_test_pool() -> SqlitePool {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("in-memory sqlite pool");

        sqlx::query(
            r#"
            CREATE TABLE appointments (
                id BLOB PRIMARY KEY,
                patient_id BLOB NOT NULL,
                practitioner_id BLOB NOT NULL,
                start_time TEXT NOT NULL,
                end_time TEXT NOT NULL,
                appointment_type TEXT NOT NULL,
                status TEXT NOT NULL,
                reason TEXT,
                notes TEXT,
                is_urgent BOOLEAN NOT NULL,
                reminder_sent BOOLEAN NOT NULL,
                confirmed BOOLEAN NOT NULL,
                cancellation_reason TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                version INTEGER NOT NULL DEFAULT 1,
                created_by BLOB,
                updated_by BLOB
            )
            "#,
        )
        .execute(&pool)
        .await
        .expect("create appointments table");

        pool
    }

    #[tokio::test]
    async fn update_uses_optimistic_locking_and_increments_version() {
        let pool = setup_test_pool().await;
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

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{FromRow, SqlitePool};
use uuid::Uuid;

use crate::domain::appointment::{
    Appointment, AppointmentCalendarQuery, AppointmentRepository, AppointmentSearchCriteria,
    AppointmentStatus, AppointmentType, CalendarAppointment, RepositoryError,
};

#[derive(Debug, FromRow)]
struct AppointmentRow {
    id: Vec<u8>,
    patient_id: Vec<u8>,
    practitioner_id: Vec<u8>,
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
    created_by: Option<Vec<u8>>,
    updated_by: Option<Vec<u8>>,
}

impl AppointmentRow {
    fn into_appointment(self) -> Result<Appointment, RepositoryError> {
        Ok(Appointment {
            id: Uuid::from_slice(&self.id).map_err(|e| {
                RepositoryError::ConstraintViolation(format!("Invalid UUID: {}", e))
            })?,
            patient_id: Uuid::from_slice(&self.patient_id).map_err(|e| {
                RepositoryError::ConstraintViolation(format!("Invalid patient UUID: {}", e))
            })?,
            practitioner_id: Uuid::from_slice(&self.practitioner_id).map_err(|e| {
                RepositoryError::ConstraintViolation(format!("Invalid practitioner UUID: {}", e))
            })?,
            start_time: DateTime::parse_from_rfc3339(&self.start_time)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            end_time: DateTime::parse_from_rfc3339(&self.end_time)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            appointment_type: match self.appointment_type.as_str() {
                "Standard" => AppointmentType::Standard,
                "Long" => AppointmentType::Long,
                "Brief" => AppointmentType::Brief,
                "NewPatient" => AppointmentType::NewPatient,
                "HealthAssessment" => AppointmentType::HealthAssessment,
                "ChronicDiseaseReview" => AppointmentType::ChronicDiseaseReview,
                "MentalHealthPlan" => AppointmentType::MentalHealthPlan,
                "Immunisation" => AppointmentType::Immunisation,
                "Procedure" => AppointmentType::Procedure,
                "Telephone" => AppointmentType::Telephone,
                "Telehealth" => AppointmentType::Telehealth,
                "HomeVisit" => AppointmentType::HomeVisit,
                "Emergency" => AppointmentType::Emergency,
                _ => AppointmentType::Standard,
            },
            status: match self.status.as_str() {
                "Scheduled" => AppointmentStatus::Scheduled,
                "Confirmed" => AppointmentStatus::Confirmed,
                "Arrived" => AppointmentStatus::Arrived,
                "InProgress" => AppointmentStatus::InProgress,
                "Completed" => AppointmentStatus::Completed,
                "NoShow" => AppointmentStatus::NoShow,
                "Cancelled" => AppointmentStatus::Cancelled,
                "Rescheduled" => AppointmentStatus::Rescheduled,
                _ => AppointmentStatus::Scheduled,
            },
            reason: self.reason,
            notes: self.notes,
            is_urgent: self.is_urgent,
            reminder_sent: self.reminder_sent,
            confirmed: self.confirmed,
            cancellation_reason: self.cancellation_reason,
            created_at: DateTime::parse_from_rfc3339(&self.created_at)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            updated_at: DateTime::parse_from_rfc3339(&self.updated_at)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            created_by: self
                .created_by
                .and_then(|bytes| Uuid::from_slice(&bytes).ok()),
            updated_by: self
                .updated_by
                .and_then(|bytes| Uuid::from_slice(&bytes).ok()),
        })
    }
}

#[derive(Debug, FromRow)]
struct CalendarAppointmentRow {
    id: Vec<u8>,
    patient_id: Vec<u8>,
    practitioner_id: Vec<u8>,
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
        let start_time = DateTime::parse_from_rfc3339(&self.start_time)
            .map(|dt| dt.with_timezone(&Utc))
            .map_err(|e| {
                RepositoryError::ConstraintViolation(format!("Invalid start_time: {}", e))
            })?;

        let end_time = DateTime::parse_from_rfc3339(&self.end_time)
            .map(|dt| dt.with_timezone(&Utc))
            .map_err(|e| {
                RepositoryError::ConstraintViolation(format!("Invalid end_time: {}", e))
            })?;

        // Calculate slot_span: number of 15-minute slots
        let duration_minutes = (end_time - start_time).num_minutes();
        let slot_span = ((duration_minutes as f64 / 15.0).ceil() as u8).max(1);

        Ok(CalendarAppointment {
            id: Uuid::from_slice(&self.id).map_err(|e| {
                RepositoryError::ConstraintViolation(format!("Invalid UUID: {}", e))
            })?,
            patient_id: Uuid::from_slice(&self.patient_id).map_err(|e| {
                RepositoryError::ConstraintViolation(format!("Invalid patient UUID: {}", e))
            })?,
            practitioner_id: Uuid::from_slice(&self.practitioner_id).map_err(|e| {
                RepositoryError::ConstraintViolation(format!("Invalid practitioner UUID: {}", e))
            })?,
            patient_name: self
                .patient_name
                .unwrap_or_else(|| "Unknown Patient".to_string()),
            start_time,
            end_time,
            appointment_type: match self.appointment_type.as_str() {
                "Standard" => AppointmentType::Standard,
                "Long" => AppointmentType::Long,
                "Brief" => AppointmentType::Brief,
                "NewPatient" => AppointmentType::NewPatient,
                "HealthAssessment" => AppointmentType::HealthAssessment,
                "ChronicDiseaseReview" => AppointmentType::ChronicDiseaseReview,
                "MentalHealthPlan" => AppointmentType::MentalHealthPlan,
                "Immunisation" => AppointmentType::Immunisation,
                "Procedure" => AppointmentType::Procedure,
                "Telephone" => AppointmentType::Telephone,
                "Telehealth" => AppointmentType::Telehealth,
                "HomeVisit" => AppointmentType::HomeVisit,
                "Emergency" => AppointmentType::Emergency,
                _ => AppointmentType::Standard,
            },
            status: match self.status.as_str() {
                "Scheduled" => AppointmentStatus::Scheduled,
                "Confirmed" => AppointmentStatus::Confirmed,
                "Arrived" => AppointmentStatus::Arrived,
                "InProgress" => AppointmentStatus::InProgress,
                "Completed" => AppointmentStatus::Completed,
                "NoShow" => AppointmentStatus::NoShow,
                "Cancelled" => AppointmentStatus::Cancelled,
                "Rescheduled" => AppointmentStatus::Rescheduled,
                _ => AppointmentStatus::Scheduled,
            },
            is_urgent: self.is_urgent,
            slot_span,
            reason: self.reason,
            notes: self.notes,
        })
    }
}

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
        let id_bytes = id.as_bytes().to_vec();

        let row = sqlx::query_as::<_, AppointmentRow>(
            r#"
            SELECT 
                id, patient_id, practitioner_id,
                start_time, end_time,
                appointment_type, status,
                reason, notes,
                is_urgent, reminder_sent, confirmed,
                cancellation_reason,
                created_at, updated_at,
                created_by, updated_by
            FROM appointments
            WHERE id = ?
            "#,
        )
        .bind(id_bytes)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(r) => Ok(Some(r.into_appointment()?)),
            None => Ok(None),
        }
    }

    async fn create(&self, appointment: Appointment) -> Result<Appointment, RepositoryError> {
        let id_bytes = appointment.id.as_bytes().to_vec();
        let patient_id_bytes = appointment.patient_id.as_bytes().to_vec();
        let practitioner_id_bytes = appointment.practitioner_id.as_bytes().to_vec();
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
        let created_by_bytes = appointment.created_by.map(|id| id.as_bytes().to_vec());
        let updated_by_bytes = appointment.updated_by.map(|id| id.as_bytes().to_vec());

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
                created_by, updated_by
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
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
                    Err(RepositoryError::Database(sqlx::Error::Database(db_err)))
                }
            }
            Err(e) => Err(RepositoryError::Database(e)),
        }
    }

    async fn update(&self, appointment: Appointment) -> Result<Appointment, RepositoryError> {
        let id_bytes = appointment.id.as_bytes().to_vec();
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
        let updated_by_bytes = appointment.updated_by.map(|id| id.as_bytes().to_vec());

        let result = sqlx::query(
            r#"
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
                updated_by = ?
            WHERE id = ?
            "#,
        )
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
        .bind(id_bytes)
        .execute(&self.pool)
        .await;

        match result {
            Ok(query_result) => {
                if query_result.rows_affected() == 0 {
                    Err(RepositoryError::NotFound)
                } else {
                    Ok(appointment)
                }
            }
            Err(sqlx::Error::Database(db_err)) => {
                let err_msg = db_err.message();
                if err_msg.contains("CHECK constraint") {
                    Err(RepositoryError::ConstraintViolation(
                        "Invalid value for field".to_string(),
                    ))
                } else {
                    Err(RepositoryError::Database(sqlx::Error::Database(db_err)))
                }
            }
            Err(e) => Err(RepositoryError::Database(e)),
        }
    }

    async fn delete(&self, id: Uuid) -> Result<(), RepositoryError> {
        let id_bytes = id.as_bytes().to_vec();

        let result = sqlx::query(
            r#"
            DELETE FROM appointments
            WHERE id = ?
            "#,
        )
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
            Err(e) => Err(RepositoryError::Database(e)),
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
                created_by, updated_by
            FROM appointments
            WHERE 1=1
            "#,
        );

        let mut bindings: Vec<Box<dyn sqlx::Encode<'_, sqlx::Sqlite> + Send>> = Vec::new();

        if let Some(patient_id) = criteria.patient_id {
            query.push_str(" AND patient_id = ?");
            bindings.push(Box::new(patient_id.as_bytes().to_vec()));
        }

        if let Some(practitioner_id) = criteria.practitioner_id {
            query.push_str(" AND practitioner_id = ?");
            bindings.push(Box::new(practitioner_id.as_bytes().to_vec()));
        }

        if let Some(date_from) = criteria.date_from {
            query.push_str(" AND start_time >= ?");
            bindings.push(Box::new(date_from.to_rfc3339()));
        }

        if let Some(date_to) = criteria.date_to {
            query.push_str(" AND start_time < ?");
            bindings.push(Box::new(date_to.to_rfc3339()));
        }

        if let Some(appointment_type) = criteria.appointment_type {
            let type_str = match appointment_type {
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
            bindings.push(Box::new(type_str.to_string()));
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
            bindings.push(Box::new(status_str.to_string()));
        }

        query.push_str(" ORDER BY start_time");

        let rows = if criteria.patient_id.is_some()
            || criteria.practitioner_id.is_some()
            || criteria.date_from.is_some()
            || criteria.date_to.is_some()
            || criteria.appointment_type.is_some()
            || criteria.status.is_some()
        {
            let patient_id_bytes = criteria.patient_id.map(|id| id.as_bytes().to_vec());
            let practitioner_id_bytes = criteria.practitioner_id.map(|id| id.as_bytes().to_vec());
            let date_from_str = criteria.date_from.map(|dt| dt.to_rfc3339());
            let date_to_str = criteria.date_to.map(|dt| dt.to_rfc3339());

            let all_rows = sqlx::query_as::<_, AppointmentRow>(
                r#"
                SELECT 
                    id, patient_id, practitioner_id,
                    start_time, end_time,
                    appointment_type, status,
                    reason, notes,
                    is_urgent, reminder_sent, confirmed,
                    cancellation_reason,
                    created_at, updated_at,
                    created_by, updated_by
                FROM appointments
                ORDER BY start_time
                "#,
            )
            .fetch_all(&self.pool)
            .await?;

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
                    true
                })
                .collect()
        } else {
            sqlx::query_as::<_, AppointmentRow>(
                r#"
                SELECT 
                    id, patient_id, practitioner_id,
                    start_time, end_time,
                    appointment_type, status,
                    reason, notes,
                    is_urgent, reminder_sent, confirmed,
                    cancellation_reason,
                    created_at, updated_at,
                    created_by, updated_by
                FROM appointments
                ORDER BY start_time
                "#,
            )
            .fetch_all(&self.pool)
            .await?
        };

        rows.into_iter().map(|r| r.into_appointment()).collect()
    }

    async fn find_overlapping(
        &self,
        practitioner_id: Uuid,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<Vec<Appointment>, RepositoryError> {
        let practitioner_id_bytes = practitioner_id.as_bytes().to_vec();
        let start_time_str = start_time.to_rfc3339();
        let end_time_str = end_time.to_rfc3339();

        let rows = sqlx::query_as::<_, AppointmentRow>(
            r#"
            SELECT 
                id, patient_id, practitioner_id,
                start_time, end_time,
                appointment_type, status,
                reason, notes,
                is_urgent, reminder_sent, confirmed,
                cancellation_reason,
                created_at, updated_at,
                created_by, updated_by
            FROM appointments
            WHERE practitioner_id = ?
              AND start_time < ?
              AND end_time > ?
              AND status NOT IN ('Cancelled', 'NoShow')
            ORDER BY start_time
            "#,
        )
        .bind(practitioner_id_bytes)
        .bind(end_time_str)
        .bind(start_time_str)
        .fetch_all(&self.pool)
        .await?;

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

        let mut bindings: Vec<Box<dyn sqlx::Encode<'_, sqlx::Sqlite> + Send>> = Vec::new();

        if let Some(patient_id) = criteria.patient_id {
            query.push_str(" AND a.patient_id = ?");
            bindings.push(Box::new(patient_id.as_bytes().to_vec()));
        }

        if let Some(practitioner_id) = criteria.practitioner_id {
            query.push_str(" AND a.practitioner_id = ?");
            bindings.push(Box::new(practitioner_id.as_bytes().to_vec()));
        }

        if let Some(date_from) = criteria.date_from {
            query.push_str(" AND a.start_time >= ?");
            bindings.push(Box::new(date_from.to_rfc3339()));
        }

        if let Some(date_to) = criteria.date_to {
            query.push_str(" AND a.start_time < ?");
            bindings.push(Box::new(date_to.to_rfc3339()));
        }

        if let Some(appointment_type) = criteria.appointment_type {
            let type_str = match appointment_type {
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
            bindings.push(Box::new(type_str.to_string()));
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
            query.push_str(" AND a.status = ?");
            bindings.push(Box::new(status_str.to_string()));
        }

        if let Some(is_urgent) = criteria.is_urgent {
            query.push_str(" AND a.is_urgent = ?");
            bindings.push(Box::new(is_urgent));
        }

        if let Some(confirmed) = criteria.confirmed {
            query.push_str(" AND a.confirmed = ?");
            bindings.push(Box::new(confirmed));
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
            let patient_id_bytes = criteria.patient_id.map(|id| id.as_bytes().to_vec());
            let practitioner_id_bytes = criteria.practitioner_id.map(|id| id.as_bytes().to_vec());
            let date_from_str = criteria.date_from.map(|dt| dt.to_rfc3339());
            let date_to_str = criteria.date_to.map(|dt| dt.to_rfc3339());

            let all_rows = sqlx::query_as::<_, CalendarAppointmentRow>(
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
                ORDER BY a.start_time
                "#,
            )
            .fetch_all(&self.pool)
            .await?;

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
            sqlx::query_as::<_, CalendarAppointmentRow>(
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
                ORDER BY a.start_time
                "#,
            )
            .fetch_all(&self.pool)
            .await?
        };

        rows.into_iter()
            .map(|r| r.into_calendar_appointment())
            .collect()
    }
}

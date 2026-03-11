//! Appointment UI Service
//!
//! Bridge between UI components and domain layer for appointment operations.

use std::sync::Arc;

use chrono::{NaiveDate, TimeZone, Utc};

use opengp_domain::domain::appointment::{
    AppointmentSearchCriteria, AppointmentCalendarQuery, AppointmentRepository,
    AppointmentService, AvailabilityService, CalendarAppointment, CalendarDayView, NewAppointmentData, PractitionerSchedule,
};
use opengp_domain::domain::user::{Practitioner, PractitionerRepository};
use opengp_domain::domain::error::RepositoryError;
use chrono::NaiveTime;

/// Result type for UI operations
pub type UiResult<T> = Result<T, UiServiceError>;

/// UI Service errors
#[derive(Debug)]
pub enum UiServiceError {
    /// Repository error
    Repository(String),
    /// Unknown error
    Unknown(String),
}

impl std::fmt::Display for UiServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UiServiceError::Repository(msg) => write!(f, "Repository error: {}", msg),
            UiServiceError::Unknown(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for UiServiceError {}

impl From<RepositoryError> for UiServiceError {
    fn from(err: RepositoryError) -> Self {
        UiServiceError::Repository(err.to_string())
    }
}

/// Appointment UI Service - bridges UI to domain layer
pub struct AppointmentUiService {
    /// Practitioner repository
    practitioner_repo: Arc<dyn PractitionerRepository>,
    /// Calendar query for appointments
    calendar_query: Arc<dyn AppointmentCalendarQuery>,
    /// Appointment repository for creating/updating appointments
    #[allow(dead_code)]
    appointment_repo: Arc<dyn AppointmentRepository>,
    /// Domain appointment service for status transitions
    domain_service: Arc<AppointmentService>,
    /// Availability service for checking slot availability
    availability_service: Arc<AvailabilityService>,
}

impl AppointmentUiService {
    /// Create a new appointment UI service
    pub fn new(
        practitioner_repo: Arc<dyn PractitionerRepository>,
        calendar_query: Arc<dyn AppointmentCalendarQuery>,
        appointment_repo: Arc<dyn AppointmentRepository>,
        domain_service: Arc<AppointmentService>,
        availability_service: Arc<AvailabilityService>,
    ) -> Self {
        Self {
            practitioner_repo,
            calendar_query,
            appointment_repo,
            domain_service,
            availability_service,
        }
    }

    pub async fn create_appointment(&self, data: NewAppointmentData, user_id: uuid::Uuid) -> UiResult<()> {
        self.domain_service
            .create_appointment(data, user_id)
            .await
            .map(|_| ())
            .map_err(|e| UiServiceError::Unknown(e.to_string()))
    }

    /// List all active practitioners
    pub async fn get_practitioners(&self) -> UiResult<Vec<Practitioner>> {
        self.practitioner_repo
            .list_active()
            .await
            .map_err(|e| UiServiceError::Repository(e.to_string()))
    }

    /// Get schedule for a specific date
    ///
    /// Fetches all appointments for the given date and groups them by practitioner
    /// to create a CalendarDayView for calendar rendering.
    pub async fn get_schedule(&self, date: NaiveDate) -> UiResult<CalendarDayView> {
        // Build date range for the day
        let start_of_day = Utc.from_utc_datetime(&date.and_hms_opt(0, 0, 0).unwrap());
        let end_of_day = Utc.from_utc_datetime(&date.and_hms_opt(23, 59, 59).unwrap());

        // Build search criteria for the date
        let criteria = AppointmentSearchCriteria {
            patient_id: None,
            practitioner_id: None,
            date_from: Some(start_of_day),
            date_to: Some(end_of_day),
            status: None,
            appointment_type: None,
            is_urgent: None,
            confirmed: None,
        };

        // Fetch appointments for the date
        let appointments = self
            .calendar_query
            .find_calendar_appointments(&criteria)
            .await
            .map_err(|e| UiServiceError::Repository(e.to_string()))?;

        // Group appointments by practitioner
        let mut appointments_by_practitioner: std::collections::HashMap<
            uuid::Uuid,
            Vec<CalendarAppointment>,
        > = std::collections::HashMap::new();

        for appointment in appointments {
            appointments_by_practitioner
                .entry(appointment.practitioner_id)
                .or_default()
                .push(appointment);
        }

        // Build practitioner schedules
        let practitioners = self
            .practitioner_repo
            .list_active()
            .await
            .map_err(|e| UiServiceError::Repository(e.to_string()))?;

        let schedules: Vec<PractitionerSchedule> = practitioners
            .into_iter()
            .map(|p| {
                let practitioner_appointments = appointments_by_practitioner
                    .remove(&p.id)
                    .unwrap_or_default();

                PractitionerSchedule {
                    practitioner_id: p.id,
                    practitioner_name: format!("{} {}", p.title, p.full_name()),
                    appointments: practitioner_appointments,
                }
            })
            .collect();

        Ok(CalendarDayView { date, practitioners: schedules })
    }

    pub async fn mark_arrived(&self, appointment_id: uuid::Uuid, user_id: uuid::Uuid) -> UiResult<()> {
        self.domain_service
            .mark_arrived(appointment_id, user_id)
            .await
            .map(|_| ())
            .map_err(|e| UiServiceError::Unknown(e.to_string()))
    }

    pub async fn mark_in_progress(&self, appointment_id: uuid::Uuid, user_id: uuid::Uuid) -> UiResult<()> {
        self.domain_service
            .mark_in_progress(appointment_id, user_id)
            .await
            .map(|_| ())
            .map_err(|e| UiServiceError::Unknown(e.to_string()))
    }

    pub async fn mark_completed(&self, appointment_id: uuid::Uuid, user_id: uuid::Uuid) -> UiResult<()> {
        self.domain_service
            .mark_completed(appointment_id, user_id)
            .await
            .map(|_| ())
            .map_err(|e| UiServiceError::Unknown(e.to_string()))
    }

    pub async fn get_available_slots(
        &self,
        practitioner_id: uuid::Uuid,
        date: NaiveDate,
        duration: u32,
    ) -> UiResult<Vec<NaiveTime>> {
        self.availability_service
            .get_available_slots(practitioner_id, date, duration as i64)
            .await
            .map_err(|e| UiServiceError::Unknown(e.to_string()))
    }
}

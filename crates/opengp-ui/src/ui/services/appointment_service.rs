//! Appointment UI Service
//!
//! Bridge between UI components and domain layer for appointment operations.

use std::sync::Arc;

use chrono::{Datelike, NaiveDate, TimeZone, Utc, Weekday};

use chrono::NaiveTime;
use super::shared::{ToUiError, UiResult};
use opengp_domain::domain::appointment::{
    AppointmentCalendarQuery, AppointmentRepository, AppointmentSearchCriteria, AppointmentService,
    AvailabilityService, CalendarAppointment, CalendarDayView, NewAppointmentData,
    PractitionerSchedule,
};
use opengp_domain::domain::user::{Practitioner, PractitionerRepository, WorkingHoursRepository};

#[cfg(test)]
use super::shared::UiServiceError;
#[cfg(test)]
use opengp_domain::domain::error::RepositoryError;

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
    /// Working hours repository for fetching practitioner schedules
    working_hours_repo: Arc<dyn WorkingHoursRepository>,
}

impl AppointmentUiService {
    /// Creates a new appointment UI service.
    pub fn new(
        practitioner_repo: Arc<dyn PractitionerRepository>,
        calendar_query: Arc<dyn AppointmentCalendarQuery>,
        appointment_repo: Arc<dyn AppointmentRepository>,
        domain_service: Arc<AppointmentService>,
        availability_service: Arc<AvailabilityService>,
        working_hours_repo: Arc<dyn WorkingHoursRepository>,
    ) -> Self {
        Self {
            practitioner_repo,
            calendar_query,
            appointment_repo,
            domain_service,
            availability_service,
            working_hours_repo,
        }
    }

    /// Creates a new appointment using the domain service.
    pub async fn create_appointment(
        &self,
        data: NewAppointmentData,
        user_id: uuid::Uuid,
    ) -> UiResult<()> {
        self.domain_service
            .create_appointment(data, user_id)
            .await
            .map(|_| ())
            .map_err(|e| e.to_ui_error())
    }

    /// Lists all active practitioners.
    pub async fn get_practitioners(&self) -> UiResult<Vec<Practitioner>> {
        self.practitioner_repo
            .list_active()
            .await
            .map_err(|e| e.to_ui_repository_error())
    }

    /// Get schedule for a specific date
    ///
    /// Fetches all appointments for the given date and groups them by practitioner
    /// to create a CalendarDayView for calendar rendering.
    pub async fn get_schedule(&self, date: NaiveDate) -> UiResult<CalendarDayView> {
         // Build date range for the day
         // SAFETY: Valid hours (0, 23) and valid seconds (0, 59) will always produce Some
         #[allow(clippy::unwrap_used)]
         let start_of_day = Utc.from_utc_datetime(&date.and_hms_opt(0, 0, 0).unwrap());
         // SAFETY: Valid hours (23) and valid seconds (59) will always produce Some
         #[allow(clippy::unwrap_used)]
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
            limit: Some(1000),
        };

        // Fetch appointments for the date
        let appointments = self
            .calendar_query
            .find_calendar_appointments(&criteria)
            .await
            .map_err(|e| e.to_ui_repository_error())?;

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
            .map_err(|e| e.to_ui_repository_error())?;

        let mut schedules: Vec<PractitionerSchedule> = Vec::new();
        for p in practitioners {
            let mut practitioner_appointments = appointments_by_practitioner
                .remove(&p.id)
                .unwrap_or_default();

            // Sort by start time for overlap detection
            practitioner_appointments.sort_by_key(|a| a.start_time);
            
            // Detect overlaps: for each pair where start_i < end_j and end_i > start_j
            for i in 0..practitioner_appointments.len() {
                for j in (i + 1)..practitioner_appointments.len() {
                    if practitioner_appointments[j].start_time < practitioner_appointments[i].end_time {
                        practitioner_appointments[i].is_overlapping = true;
                        practitioner_appointments[j].is_overlapping = true;
                    } else {
                        break;
                    }
                }
            }

            // Fetch working hours for this practitioner on this day
            let day_of_week = match date.weekday() {
                Weekday::Mon => 0,
                Weekday::Tue => 1,
                Weekday::Wed => 2,
                Weekday::Thu => 3,
                Weekday::Fri => 4,
                Weekday::Sat => 5,
                Weekday::Sun => 6,
            };
            let working_hours = self
                .working_hours_repo
                .find_for_day(p.id, day_of_week as u8)
                .await
                .ok()
                .and_then(|wh| wh);

            schedules.push(PractitionerSchedule {
                practitioner_id: p.id,
                practitioner_name: format!("{} {}", p.title, p.full_name()),
                appointments: practitioner_appointments,
                working_hours,
            });
        }

        Ok(CalendarDayView {
            date,
            practitioners: schedules,
        })
    }

    /// Marks an appointment as arrived.
    pub async fn mark_arrived(
        &self,
        appointment_id: uuid::Uuid,
        user_id: uuid::Uuid,
    ) -> UiResult<()> {
        self.domain_service
            .mark_arrived(appointment_id, user_id)
            .await
            .map(|_| ())
            .map_err(|e| e.to_ui_error())
    }

    /// Marks an appointment as in progress.
    pub async fn mark_in_progress(
        &self,
        appointment_id: uuid::Uuid,
        user_id: uuid::Uuid,
    ) -> UiResult<()> {
        self.domain_service
            .mark_in_progress(appointment_id, user_id)
            .await
            .map(|_| ())
            .map_err(|e| e.to_ui_error())
    }

    /// Marks an appointment as completed.
    pub async fn mark_completed(
        &self,
        appointment_id: uuid::Uuid,
        user_id: uuid::Uuid,
    ) -> UiResult<()> {
        self.domain_service
            .mark_completed(appointment_id, user_id)
            .await
            .map(|_| ())
            .map_err(|e| e.to_ui_error())
    }

    /// Returns the available time slots for a practitioner on a given date.
    pub async fn get_available_slots(
        &self,
        practitioner_id: uuid::Uuid,
        date: NaiveDate,
        duration: u32,
    ) -> UiResult<Vec<NaiveTime>> {
        self.availability_service
            .get_available_slots(practitioner_id, date, duration as i64)
            .await
            .map_err(|e| e.to_ui_error())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ui_service_error_display_repository() {
        let err = UiServiceError::Repository("db error".to_string());
        assert_eq!(err.to_string(), "Repository error: db error");
    }

    #[test]
    fn test_ui_service_error_display_unknown() {
        let err = UiServiceError::Unknown("something failed".to_string());
        assert_eq!(err.to_string(), "Error: something failed");
    }

    #[test]
    fn test_ui_service_error_from_repository_error() {
        let repo_err = RepositoryError::Database("connection lost".to_string());
        let ui_err = repo_err.to_ui_repository_error();
        match ui_err {
            UiServiceError::Repository(msg) => {
                assert!(msg.contains("connection lost"));
            }
            _ => panic!("Expected Repository error"),
        }
    }

    #[test]
    fn test_ui_service_error_is_error_trait() {
        let err: Box<dyn std::error::Error> = Box::new(UiServiceError::Unknown("test".to_string()));
        assert_eq!(err.to_string(), "Error: test");
    }

    #[test]
    fn test_overlap_detection_two_overlapping() {
        use chrono::{Duration, Utc};
        use opengp_domain::domain::appointment::{AppointmentStatus, AppointmentType};
        use uuid::Uuid;

        let practitioner_id = Uuid::new_v4();
        let base = Utc::now();
        
        let a = CalendarAppointment {
            id: Uuid::new_v4(),
            patient_id: Uuid::new_v4(),
            practitioner_id,
            patient_name: "A".to_string(),
            start_time: base,
            end_time: base + Duration::minutes(30),
            appointment_type: AppointmentType::Standard,
            status: AppointmentStatus::Scheduled,
            is_urgent: false,
            slot_span: 2,
            reason: None,
            notes: None,
            is_overlapping: false,
        };
        let b = CalendarAppointment {
            id: Uuid::new_v4(),
            patient_id: Uuid::new_v4(),
            practitioner_id,
            patient_name: "B".to_string(),
            start_time: base + Duration::minutes(15),
            end_time: base + Duration::minutes(45),
            appointment_type: AppointmentType::Standard,
            status: AppointmentStatus::Scheduled,
            is_urgent: false,
            slot_span: 2,
            reason: None,
            notes: None,
            is_overlapping: false,
        };
        
        let mut appts = vec![a, b];
        appts.sort_by_key(|a| a.start_time);
        for i in 0..appts.len() {
            for j in (i + 1)..appts.len() {
                if appts[j].start_time < appts[i].end_time {
                    appts[i].is_overlapping = true;
                    appts[j].is_overlapping = true;
                } else {
                    break;
                }
            }
        }
        assert!(appts[0].is_overlapping);
        assert!(appts[1].is_overlapping);
    }

    #[test]
    fn test_overlap_detection_non_overlapping() {
        use chrono::{Duration, Utc};
        use opengp_domain::domain::appointment::{AppointmentStatus, AppointmentType};
        use uuid::Uuid;

        let practitioner_id = Uuid::new_v4();
        let base = Utc::now();
        
        let a = CalendarAppointment {
            id: Uuid::new_v4(),
            patient_id: Uuid::new_v4(),
            practitioner_id,
            patient_name: "A".to_string(),
            start_time: base,
            end_time: base + Duration::minutes(30),
            appointment_type: AppointmentType::Standard,
            status: AppointmentStatus::Scheduled,
            is_urgent: false,
            slot_span: 2,
            reason: None,
            notes: None,
            is_overlapping: false,
        };
        let b = CalendarAppointment {
            id: Uuid::new_v4(),
            patient_id: Uuid::new_v4(),
            practitioner_id,
            patient_name: "B".to_string(),
            start_time: base + Duration::minutes(60),
            end_time: base + Duration::minutes(90),
            appointment_type: AppointmentType::Standard,
            status: AppointmentStatus::Scheduled,
            is_urgent: false,
            slot_span: 2,
            reason: None,
            notes: None,
            is_overlapping: false,
        };
        
        let mut appts = vec![a, b];
        appts.sort_by_key(|a| a.start_time);
        for i in 0..appts.len() {
            for j in (i + 1)..appts.len() {
                if appts[j].start_time < appts[i].end_time {
                    appts[i].is_overlapping = true;
                    appts[j].is_overlapping = true;
                } else {
                    break;
                }
            }
        }
        assert!(!appts[0].is_overlapping);
        assert!(!appts[1].is_overlapping);
    }
}

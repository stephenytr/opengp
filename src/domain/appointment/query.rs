use async_trait::async_trait;

use super::dto::{AppointmentSearchCriteria, CalendarAppointment};
use super::error::RepositoryError;

/// Query interface for reading appointment calendar data
///
/// This trait defines the read model interface for fetching calendar appointments
/// with denormalized patient names. It is separate from the domain repository
/// (`AppointmentRepository`) which returns domain entities.
///
/// This separation follows CQRS principles:
/// - **Commands** use domain entities (`Appointment`) via `AppointmentRepository`
/// - **Queries** use optimized read models (`CalendarAppointment`) via this interface
///
/// The read model includes denormalized patient names for efficient calendar rendering
/// without requiring joins at the application layer.
#[async_trait]
pub trait AppointmentCalendarQuery: Send + Sync {
    /// Find calendar appointments matching the given criteria
    ///
    /// Returns a list of simplified appointment records optimized for calendar display.
    /// Patient names are denormalized in the result for performance.
    ///
    /// # Arguments
    /// * `criteria` - Search criteria (all fields optional)
    ///
    /// # Returns
    /// * `Ok(appointments)` - List of calendar appointments matching criteria
    /// * `Err(RepositoryError)` - Database error
    ///
    /// # Example
    /// ```ignore
    /// let criteria = AppointmentSearchCriteria {
    ///     practitioner_id: Some(practitioner_id),
    ///     date_from: Some(start_of_day),
    ///     date_to: Some(end_of_day),
    ///     ..Default::default()
    /// };
    /// let appointments = query.find_calendar_appointments(&criteria).await?;
    /// ```
    async fn find_calendar_appointments(
        &self,
        criteria: &AppointmentSearchCriteria,
    ) -> Result<Vec<CalendarAppointment>, RepositoryError>;
}

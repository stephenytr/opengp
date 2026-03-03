use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::dto::AppointmentSearchCriteria;
use super::error::RepositoryError;
use super::model::Appointment;

/// Repository trait for appointment persistence
///
/// Defines the interface for storing and retrieving appointments from the database.
/// Implementations must handle all database operations and return appropriate errors.
#[async_trait]
pub trait AppointmentRepository: Send + Sync {
    /// Find an appointment by its ID
    ///
    /// # Arguments
    /// * `id` - The appointment ID
    ///
    /// # Returns
    /// * `Ok(Some(appointment))` - Appointment found
    /// * `Ok(None)` - Appointment not found
    /// * `Err(RepositoryError)` - Database error
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Appointment>, RepositoryError>;

    /// Create a new appointment
    ///
    /// # Arguments
    /// * `appointment` - The appointment to create
    ///
    /// # Returns
    /// * `Ok(appointment)` - Successfully created appointment
    /// * `Err(RepositoryError)` - Database error or constraint violation
    async fn create(&self, appointment: Appointment) -> Result<Appointment, RepositoryError>;

    /// Update an existing appointment
    ///
    /// # Arguments
    /// * `appointment` - The appointment with updated fields
    ///
    /// # Returns
    /// * `Ok(appointment)` - Successfully updated appointment
    /// * `Err(RepositoryError)` - Database error or appointment not found
    async fn update(&self, appointment: Appointment) -> Result<Appointment, RepositoryError>;

    /// Delete an appointment (soft delete - sets is_active to false)
    ///
    /// # Arguments
    /// * `id` - The appointment ID to delete
    ///
    /// # Returns
    /// * `Ok(())` - Successfully deleted
    /// * `Err(RepositoryError)` - Database error or appointment not found
    async fn delete(&self, id: Uuid) -> Result<(), RepositoryError>;

    /// Search appointments using criteria
    ///
    /// # Arguments
    /// * `criteria` - Search criteria (all fields optional)
    ///
    /// # Returns
    /// * `Ok(appointments)` - List of matching appointments
    /// * `Err(RepositoryError)` - Database error
    async fn find_by_criteria(
        &self,
        criteria: &AppointmentSearchCriteria,
    ) -> Result<Vec<Appointment>, RepositoryError>;

    /// Find appointments that overlap with a given time range for a practitioner
    ///
    /// Used to detect scheduling conflicts and prevent double-booking.
    /// Returns all active appointments for the practitioner that overlap with the given time range.
    ///
    /// # Arguments
    /// * `practitioner_id` - The practitioner ID
    /// * `start_time` - Start of the time range
    /// * `end_time` - End of the time range
    ///
    /// # Returns
    /// * `Ok(appointments)` - List of overlapping appointments
    /// * `Err(RepositoryError)` - Database error
    async fn find_overlapping(
        &self,
        practitioner_id: Uuid,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<Vec<Appointment>, RepositoryError>;
}

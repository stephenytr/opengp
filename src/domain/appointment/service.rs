use std::sync::Arc;
use uuid::Uuid;
use tracing::{info, error};
use chrono::Utc;

use super::dto::{NewAppointmentData, UpdateAppointmentData, AppointmentSearchCriteria};
use super::error::ServiceError;
use super::model::Appointment;
use super::repository::AppointmentRepository;

/// Service layer for appointment business logic
///
/// Handles appointment creation, updates, cancellations, and searches.
/// Enforces business rules such as overlap checking to prevent double-booking.
pub struct AppointmentService {
    repository: Arc<dyn AppointmentRepository>,
}

impl AppointmentService {
    pub fn new(repository: Arc<dyn AppointmentRepository>) -> Self {
        Self { repository }
    }

    /// Create a new appointment with overlap checking
    ///
    /// # Arguments
    /// * `data` - New appointment data
    /// * `user_id` - ID of user creating the appointment
    ///
    /// # Returns
    /// * `Ok(Appointment)` - Successfully created appointment
    /// * `Err(ServiceError::Conflict)` - Overlapping appointment found (double-booking)
    /// * `Err(ServiceError::Validation)` - Invalid appointment data
    /// * `Err(ServiceError::Repository)` - Database error
    pub async fn create_appointment(
        &self,
        data: NewAppointmentData,
        user_id: Uuid,
    ) -> Result<Appointment, ServiceError> {
        info!(
            "Creating appointment for patient {} with practitioner {}",
            data.patient_id, data.practitioner_id
        );

        // Calculate end time
        let end_time = data.start_time + data.duration;

        // Critical: Check for overlapping appointments (prevent double-booking)
        info!(
            "Checking for overlapping appointments for practitioner {} between {:?} and {:?}",
            data.practitioner_id, data.start_time, end_time
        );

        let overlapping = self
            .repository
            .find_overlapping(data.practitioner_id, data.start_time, end_time)
            .await?;

        if !overlapping.is_empty() {
            error!(
                "Overlapping appointment(s) found for practitioner {}: {} conflict(s)",
                data.practitioner_id,
                overlapping.len()
            );
            return Err(ServiceError::Conflict(format!(
                "Practitioner has {} overlapping appointment(s) during this time",
                overlapping.len()
            )));
        }

        info!("No conflicts found, creating appointment domain model");

        // Create appointment domain model
        let mut appointment = Appointment::new(
            data.patient_id,
            data.practitioner_id,
            data.start_time,
            data.duration,
            data.appointment_type,
            Some(user_id),
        );

        // Set optional fields from data
        appointment.reason = data.reason;
        appointment.is_urgent = data.is_urgent;

        info!("Saving appointment to database with ID: {}", appointment.id);

        // Save to repository
        match self.repository.create(appointment.clone()).await {
            Ok(saved) => {
                info!(
                    "Appointment saved successfully: {} at {:?}",
                    saved.id, saved.start_time
                );
                Ok(saved)
            }
            Err(e) => {
                error!("Failed to save appointment to database: {}", e);
                Err(e.into())
            }
        }
    }

    /// Update an existing appointment
    ///
    /// # Arguments
    /// * `id` - Appointment ID
    /// * `data` - Update data (only provided fields are updated)
    /// * `user_id` - ID of user updating the appointment
    ///
    /// # Returns
    /// * `Ok(Appointment)` - Successfully updated appointment
    /// * `Err(ServiceError::NotFound)` - Appointment not found
    /// * `Err(ServiceError::Repository)` - Database error
    pub async fn update_appointment(
        &self,
        id: Uuid,
        data: UpdateAppointmentData,
        user_id: Uuid,
    ) -> Result<Appointment, ServiceError> {
        info!("Updating appointment: {}", id);

        // Load existing appointment
        let mut appointment = self
            .repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| ServiceError::NotFound(id))?;

        // Apply updates (only provided fields)
        if let Some(status) = data.status {
            info!("Updating status to: {:?}", status);
            appointment.status = status;
        }

        if let Some(appointment_type) = data.appointment_type {
            info!("Updating type to: {:?}", appointment_type);
            appointment.appointment_type = appointment_type;
        }

        if let Some(reason) = data.reason {
            info!("Updating reason");
            appointment.reason = Some(reason);
        }

        if let Some(notes) = data.notes {
            info!("Updating notes");
            appointment.notes = Some(notes);
        }

        if let Some(is_urgent) = data.is_urgent {
            info!("Updating urgent flag to: {}", is_urgent);
            appointment.is_urgent = is_urgent;
        }

        if let Some(confirmed) = data.confirmed {
            info!("Updating confirmed flag to: {}", confirmed);
            appointment.confirmed = confirmed;
        }

        if let Some(reminder_sent) = data.reminder_sent {
            info!("Updating reminder_sent flag to: {}", reminder_sent);
            appointment.reminder_sent = reminder_sent;
        }

        if let Some(cancellation_reason) = data.cancellation_reason {
            info!("Updating cancellation reason");
            appointment.cancellation_reason = Some(cancellation_reason);
        }

        // Update audit fields
        appointment.updated_at = Utc::now();
        appointment.updated_by = Some(user_id);

        // Save changes
        match self.repository.update(appointment.clone()).await {
            Ok(updated) => {
                info!("Appointment updated successfully: {}", updated.id);
                Ok(updated)
            }
            Err(e) => {
                error!("Failed to update appointment in database: {}", e);
                Err(e.into())
            }
        }
    }

    /// Cancel an appointment
    ///
    /// Uses the domain model's cancel method to ensure business rules are enforced.
    ///
    /// # Arguments
    /// * `id` - Appointment ID
    /// * `reason` - Cancellation reason
    /// * `user_id` - ID of user cancelling the appointment
    ///
    /// # Returns
    /// * `Ok(Appointment)` - Successfully cancelled appointment
    /// * `Err(ServiceError::NotFound)` - Appointment not found
    /// * `Err(ServiceError::Repository)` - Database error
    pub async fn cancel_appointment(
        &self,
        id: Uuid,
        reason: String,
        user_id: Uuid,
    ) -> Result<Appointment, ServiceError> {
        info!("Cancelling appointment: {} with reason: {}", id, reason);

        // Load existing appointment
        let mut appointment = self
            .repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| ServiceError::NotFound(id))?;

        // Use domain method to cancel (enforces business rules)
        appointment.cancel(reason, user_id);

        // Save changes
        match self.repository.update(appointment.clone()).await {
            Ok(cancelled) => {
                info!("Appointment cancelled successfully: {}", cancelled.id);
                Ok(cancelled)
            }
            Err(e) => {
                error!("Failed to cancel appointment in database: {}", e);
                Err(e.into())
            }
        }
    }

    /// Find an appointment by ID
    ///
    /// # Arguments
    /// * `id` - Appointment ID
    ///
    /// # Returns
    /// * `Ok(Some(Appointment))` - Appointment found
    /// * `Ok(None)` - Appointment not found
    /// * `Err(ServiceError::Repository)` - Database error
    pub async fn find_appointment(&self, id: Uuid) -> Result<Option<Appointment>, ServiceError> {
        let appointment = self.repository.find_by_id(id).await?;
        Ok(appointment)
    }

    /// Search appointments using criteria
    ///
    /// # Arguments
    /// * `criteria` - Search criteria (all fields optional)
    ///
    /// # Returns
    /// * `Ok(Vec<Appointment>)` - List of matching appointments
    /// * `Err(ServiceError::Repository)` - Database error
    pub async fn search_appointments(
        &self,
        criteria: &AppointmentSearchCriteria,
    ) -> Result<Vec<Appointment>, ServiceError> {
        info!("Searching appointments with criteria: {:?}", criteria);

        let appointments = self.repository.find_by_criteria(criteria).await?;

        info!("Found {} appointments", appointments.len());
        Ok(appointments)
    }

    /// Fetch appointments for a specific date
    ///
    /// # Arguments
    /// * `date` - The date to fetch appointments for
    /// * `practitioner_ids` - Optional list of practitioner IDs to filter by
    ///
    /// # Returns
    /// * `Ok(Vec<Appointment>)` - List of appointments for the date
    /// * `Err(ServiceError)` - Database error
    pub async fn get_day_appointments(
        &self,
        date: chrono::NaiveDate,
        practitioner_ids: Option<Vec<Uuid>>,
    ) -> Result<Vec<Appointment>, ServiceError> {
        use chrono::TimeZone;
        
        info!("Fetching appointments for date: {}", date);
        
        // Convert date to UTC datetime range (these times are always valid)
        let start_of_day = chrono::Utc.from_utc_datetime(
            &date.and_hms_opt(0, 0, 0).expect("00:00:00 is always valid")
        );
        let end_of_day = chrono::Utc.from_utc_datetime(
            &date.and_hms_opt(23, 59, 59).expect("23:59:59 is always valid")
        );
        
        // Build search criteria
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
        
        let mut appointments = self.search_appointments(&criteria).await?;
        
        // Filter by practitioner IDs if provided
        if let Some(ids) = practitioner_ids {
            appointments.retain(|a| ids.contains(&a.practitioner_id));
        }
        
        info!("Found {} appointments for date {}", appointments.len(), date);
        Ok(appointments)
    }
}

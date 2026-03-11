use chrono::{Datelike, NaiveTime, TimeZone, Utc, Weekday};
use std::sync::Arc;
use tracing::{error, info};
use uuid::Uuid;

use crate::service;

use super::dto::{
    AppointmentSearchCriteria, CalendarAppointment, NewAppointmentData, UpdateAppointmentData,
};
use super::error::ServiceError;
use super::model::{Appointment, AppointmentStatus};
use super::query::AppointmentCalendarQuery;
use super::repository::AppointmentRepository;
use crate::domain::audit::{AuditEmitter, AuditEntry};
use crate::domain::error::RepositoryError as BaseRepositoryError;
use crate::domain::user::WorkingHoursRepository;

service! {
    AppointmentService {
        repository: Arc<dyn AppointmentRepository>,
        audit_service: Arc<dyn AuditEmitter>,
        calendar_query: Arc<dyn AppointmentCalendarQuery>,
    }
}

service! {
    AvailabilityService {
        appointment_repository: Arc<dyn AppointmentRepository>,
        working_hours_repository: Arc<dyn WorkingHoursRepository>,
    }
}

impl AppointmentService {
    fn map_repository_error(error: BaseRepositoryError) -> ServiceError {
        match error {
            BaseRepositoryError::Conflict(message) => ServiceError::Conflict(message),
            other => ServiceError::Repository(other),
        }
    }

    /// Check for overlapping appointments for a practitioner
    ///
    /// # Arguments
    /// * `practitioner_id` - ID of the practitioner
    /// * `start_time` - Start time of the appointment slot
    /// * `end_time` - End time of the appointment slot
    /// * `exclude_id` - Optional appointment ID to exclude from overlap check (used for rescheduling)
    ///
    /// # Returns
    /// * `Ok(())` - No overlapping appointments found
    /// * `Err(ServiceError::Conflict)` - Overlapping appointment(s) found
    /// * `Err(ServiceError::Repository)` - Database error
    async fn check_no_overlap(
        &self,
        practitioner_id: Uuid,
        start_time: chrono::DateTime<Utc>,
        end_time: chrono::DateTime<Utc>,
        exclude_id: Option<Uuid>,
    ) -> Result<(), ServiceError> {
        info!(
            "Checking for overlapping appointments for practitioner {} between {:?} and {:?}",
            practitioner_id, start_time, end_time
        );

        let overlapping = self
            .repository
            .find_overlapping(practitioner_id, start_time, end_time)
            .await?;

        let conflicts: Vec<&Appointment> = overlapping
            .iter()
            .filter(|a| exclude_id.is_none() || a.id != exclude_id.unwrap())
            .collect();

        if !conflicts.is_empty() {
            error!(
                "Overlapping appointment(s) found for practitioner {}: {} conflict(s)",
                practitioner_id,
                conflicts.len()
            );
            return Err(ServiceError::Conflict(format!(
                "Practitioner has {} overlapping appointment(s) during this time",
                conflicts.len()
            )));
        }

        Ok(())
    }

    /// Validate status transition using state machine rules
    ///
    /// Checks if the appointment can transition from its current status to the target status.
    /// Uses the domain model's `can_transition_to()` method to enforce business rules.
    ///
    /// # Arguments
    /// * `appointment` - The appointment to validate
    /// * `target_status` - The target status to transition to
    ///
    /// # Returns
    /// * `Ok(())` - Transition is valid
    /// * `Err(ServiceError::InvalidTransition)` - Transition is not allowed
    fn validate_transition(
        &self,
        appointment: &Appointment,
        target_status: AppointmentStatus,
    ) -> Result<(), ServiceError> {
        appointment.can_transition_to(target_status).map_err(|e| {
            tracing::warn!("Invalid transition blocked: {}", e);
            ServiceError::InvalidTransition(e)
        })
    }

    /// Log an audit entry for appointment operations
    ///
    /// # Arguments
    /// * `entry` - The audit entry to log
    ///
    /// # Returns
    /// * `Ok(())` - Successfully logged
    /// * `Err(ServiceError)` - Failed to log audit entry
    async fn audit_log(&self, entry: AuditEntry) -> Result<(), ServiceError> {
        self.audit_service.emit(entry).await.ok();
        Ok(())
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
        self.check_no_overlap(data.practitioner_id, data.start_time, end_time, None)
            .await?;

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
        expected_version: i32,
        user_id: Uuid,
    ) -> Result<Appointment, ServiceError> {
        info!("Updating appointment: {}", id);

        // Load existing appointment
        let mut appointment = self
            .repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| ServiceError::NotFound(id))?;

        if appointment.version != expected_version {
            return Err(ServiceError::Conflict(
                "Resource was modified. Please refresh and try again.".to_string(),
            ));
        }

        let original_practitioner_id = appointment.practitioner_id;
        let original_start_time = appointment.start_time;
        let original_end_time = appointment.end_time;

        let mut effective_practitioner_id = original_practitioner_id;
        let mut effective_start_time = original_start_time;
        let mut effective_end_time = original_end_time;

        if let Some(patient_id) = data.patient_id {
            appointment.patient_id = patient_id;
        }

        if let Some(practitioner_id) = data.practitioner_id {
            effective_practitioner_id = practitioner_id;
            appointment.practitioner_id = practitioner_id;
        }

        if let Some(start_time) = data.start_time {
            let duration = effective_end_time - effective_start_time;
            effective_start_time = start_time;
            effective_end_time = start_time + duration;
            appointment.start_time = effective_start_time;
            appointment.end_time = effective_end_time;
        }

        if let Some(duration) = data.duration {
            effective_end_time = effective_start_time + duration;
            appointment.end_time = effective_end_time;
        }

        if effective_practitioner_id != original_practitioner_id
            || effective_start_time != original_start_time
            || effective_end_time != original_end_time
        {
            self.check_no_overlap(
                effective_practitioner_id,
                effective_start_time,
                effective_end_time,
                Some(id),
            )
            .await?;
        }

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
        match self
            .repository
            .update(appointment.clone())
            .await
            .map_err(Self::map_repository_error)
        {
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
        match self
            .repository
            .update(appointment.clone())
            .await
            .map_err(Self::map_repository_error)
        {
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

    /// Get calendar appointments with denormalized patient names
    ///
    /// This method returns a read model optimized for calendar display.
    /// It includes patient names denormalized from the patient table for efficient rendering.
    ///
    /// # Arguments
    /// * `criteria` - Search criteria (all fields optional)
    ///
    /// # Returns
    /// * `Ok(Vec<CalendarAppointment>)` - List of calendar appointments with patient names
    /// * `Err(ServiceError::Repository)` - Database error
    pub async fn get_calendar_appointments(
        &self,
        criteria: &AppointmentSearchCriteria,
    ) -> Result<Vec<CalendarAppointment>, ServiceError> {
        info!(
            "Fetching calendar appointments with criteria: {:?}",
            criteria
        );

        let appointments = self
            .calendar_query
            .find_calendar_appointments(criteria)
            .await?;

        info!("Found {} calendar appointments", appointments.len());
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
        let start_of_day = chrono::Utc
            .from_utc_datetime(&date.and_hms_opt(0, 0, 0).expect("00:00:00 is always valid"));
        let end_of_day = chrono::Utc.from_utc_datetime(
            &date
                .and_hms_opt(23, 59, 59)
                .expect("23:59:59 is always valid"),
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

        info!(
            "Found {} appointments for date {}",
            appointments.len(),
            date
        );
        Ok(appointments)
    }

    /// Mark appointment as arrived
    ///
    /// # Arguments
    /// * `appointment_id` - Appointment ID
    /// * `user_id` - ID of user marking the appointment as arrived
    ///
    /// # Returns
    /// * `Ok(Appointment)` - Updated appointment
    /// * `Err(ServiceError::NotFound)` - Appointment not found
    /// * `Err(ServiceError::InvalidTransition)` - Invalid status transition
    /// * `Err(ServiceError::Repository)` - Database error
    pub async fn mark_arrived(
        &self,
        appointment_id: Uuid,
        user_id: Uuid,
    ) -> Result<Appointment, ServiceError> {
        info!(
            "Marking appointment {} as arrived by user {}",
            appointment_id, user_id
        );

        let mut appointment = self
            .repository
            .find_by_id(appointment_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound(appointment_id))?;

        let old_status = appointment.status;

        self.validate_transition(&appointment, AppointmentStatus::Arrived)?;

        appointment.mark_arrived(user_id);

        let updated = self
            .repository
            .update(appointment)
            .await
            .map_err(Self::map_repository_error)?;
        info!("Appointment {} marked as arrived", appointment_id);

        let audit_entry = AuditEntry::new_status_changed(
            "appointment",
            appointment_id,
            format!("{:?}", old_status),
            format!("{:?}", updated.status),
            user_id,
        );
        self.audit_log(audit_entry).await?;

        Ok(updated)
    }

    /// Mark appointment as in progress
    ///
    /// # Arguments
    /// * `appointment_id` - Appointment ID
    /// * `user_id` - ID of user marking the appointment as in progress
    ///
    /// # Returns
    /// * `Ok(Appointment)` - Updated appointment
    /// * `Err(ServiceError::NotFound)` - Appointment not found
    /// * `Err(ServiceError::InvalidTransition)` - Invalid status transition
    /// * `Err(ServiceError::Repository)` - Database error
    pub async fn mark_in_progress(
        &self,
        appointment_id: Uuid,
        user_id: Uuid,
    ) -> Result<Appointment, ServiceError> {
        info!(
            "Marking appointment {} as in progress by user {}",
            appointment_id, user_id
        );

        let mut appointment = self
            .repository
            .find_by_id(appointment_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound(appointment_id))?;

        let old_status = appointment.status;

        self.validate_transition(&appointment, AppointmentStatus::InProgress)?;

        appointment.mark_in_progress(user_id);

        let updated = self
            .repository
            .update(appointment)
            .await
            .map_err(Self::map_repository_error)?;
        info!("Appointment {} marked as in progress", appointment_id);

        let audit_entry = AuditEntry::new_status_changed(
            "appointment",
            appointment_id,
            format!("{:?}", old_status),
            format!("{:?}", updated.status),
            user_id,
        );
        self.audit_log(audit_entry).await?;

        Ok(updated)
    }

    /// Mark appointment as completed
    ///
    /// # Arguments
    /// * `appointment_id` - Appointment ID
    /// * `user_id` - ID of user marking the appointment as completed
    ///
    /// # Returns
    /// * `Ok(Appointment)` - Updated appointment
    /// * `Err(ServiceError::NotFound)` - Appointment not found
    /// * `Err(ServiceError::InvalidTransition)` - Invalid status transition
    /// * `Err(ServiceError::Repository)` - Database error
    pub async fn mark_completed(
        &self,
        appointment_id: Uuid,
        user_id: Uuid,
    ) -> Result<Appointment, ServiceError> {
        info!(
            "Marking appointment {} as completed by user {}",
            appointment_id, user_id
        );

        let mut appointment = self
            .repository
            .find_by_id(appointment_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound(appointment_id))?;

        let old_status = appointment.status;

        self.validate_transition(&appointment, AppointmentStatus::Completed)?;

        appointment.mark_completed(user_id);

        let updated = self
            .repository
            .update(appointment)
            .await
            .map_err(Self::map_repository_error)?;
        info!("Appointment {} marked as completed", appointment_id);

        let audit_entry = AuditEntry::new_status_changed(
            "appointment",
            appointment_id,
            format!("{:?}", old_status),
            format!("{:?}", updated.status),
            user_id,
        );
        self.audit_log(audit_entry).await?;

        Ok(updated)
    }

    /// Mark appointment as no show
    ///
    /// # Arguments
    /// * `appointment_id` - Appointment ID
    /// * `user_id` - ID of user marking the appointment as no show
    ///
    /// # Returns
    /// * `Ok(Appointment)` - Updated appointment
    /// * `Err(ServiceError::NotFound)` - Appointment not found
    /// * `Err(ServiceError::InvalidTransition)` - Invalid status transition
    /// * `Err(ServiceError::Repository)` - Database error
    pub async fn mark_no_show(
        &self,
        appointment_id: Uuid,
        user_id: Uuid,
    ) -> Result<Appointment, ServiceError> {
        info!(
            "Marking appointment {} as no show by user {}",
            appointment_id, user_id
        );

        let mut appointment = self
            .repository
            .find_by_id(appointment_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound(appointment_id))?;

        let old_status = appointment.status;

        self.validate_transition(&appointment, AppointmentStatus::NoShow)?;

        appointment.status = AppointmentStatus::NoShow;
        appointment.updated_at = Utc::now();
        appointment.updated_by = Some(user_id);

        let updated = self
            .repository
            .update(appointment)
            .await
            .map_err(Self::map_repository_error)?;
        info!("Appointment {} marked as no show", appointment_id);

        let audit_entry = AuditEntry::new_status_changed(
            "appointment",
            appointment_id,
            format!("{:?}", old_status),
            format!("{:?}", updated.status),
            user_id,
        );
        self.audit_log(audit_entry).await?;

        Ok(updated)
    }

    pub async fn reschedule_appointment(
        &self,
        appointment_id: Uuid,
        new_start_time: chrono::DateTime<Utc>,
        new_duration_minutes: i64,
        user_id: Uuid,
    ) -> Result<Appointment, ServiceError> {
        info!(
            "Rescheduling appointment {} to {} with duration {} minutes",
            appointment_id, new_start_time, new_duration_minutes
        );

        let mut appointment = self
            .repository
            .find_by_id(appointment_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound(appointment_id))?;

        let old_start_time = appointment.start_time;
        let new_end_time = new_start_time + chrono::Duration::minutes(new_duration_minutes);

        // Check for overlapping appointments, excluding the current appointment being rescheduled
        self.check_no_overlap(
            appointment.practitioner_id,
            new_start_time,
            new_end_time,
            Some(appointment_id),
        )
        .await?;

        appointment.start_time = new_start_time;
        appointment.end_time = new_end_time;
        appointment.updated_at = Utc::now();
        appointment.updated_by = Some(user_id);

        let updated = self
            .repository
            .update(appointment)
            .await
            .map_err(Self::map_repository_error)?;
        info!("Appointment {} rescheduled successfully", appointment_id);

        let audit_entry =
            AuditEntry::new_rescheduled(appointment_id, old_start_time, new_start_time, user_id);
        self.audit_log(audit_entry).await?;

        Ok(updated)
    }
}

impl AvailabilityService {
    /// Get available appointment slots for a practitioner on a specific date
    ///
    /// Combines working hours with existing appointments to determine available time slots.
    /// Generates 15-minute slots and filters out times where an appointment of the specified
    /// duration would overlap with existing appointments.
    ///
    /// # Arguments
    /// * `practitioner_id` - ID of the practitioner
    /// * `date` - Date to check availability for
    /// * `duration_minutes` - Duration of the appointment in minutes
    ///
    /// # Returns
    /// * `Ok(Vec<NaiveTime>)` - List of available start times (15-minute intervals)
    /// * `Err(ServiceError)` - Database error or no working hours defined for that day
    pub async fn get_available_slots(
        &self,
        practitioner_id: Uuid,
        date: chrono::NaiveDate,
        duration_minutes: i64,
    ) -> Result<Vec<NaiveTime>, ServiceError> {
        info!(
            "Getting available slots for practitioner {} on {} with duration {} minutes",
            practitioner_id, date, duration_minutes
        );

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
            .working_hours_repository
            .find_for_day(practitioner_id, day_of_week as u8)
            .await?;

        let working_hours = match working_hours {
            Some(wh) => wh,
            None => {
                info!(
                    "No working hours defined for practitioner {} on day {}",
                    practitioner_id, day_of_week
                );
                return Ok(vec![]);
            }
        };

        let start_of_day = Utc.from_utc_datetime(&date.and_hms_opt(0, 0, 0).unwrap());
        let end_of_day = Utc.from_utc_datetime(&date.and_hms_opt(23, 59, 59).unwrap());

        let appointments = self
            .appointment_repository
            .find_by_criteria(&AppointmentSearchCriteria {
                patient_id: None,
                practitioner_id: Some(practitioner_id),
                date_from: Some(start_of_day),
                date_to: Some(end_of_day),
                status: None,
                appointment_type: None,
                is_urgent: None,
                confirmed: None,
            })
            .await?;

        let mut available_slots = Vec::new();
        let mut current_time = working_hours.start_time;
        let slot_duration = chrono::Duration::minutes(15);
        let appointment_duration = chrono::Duration::minutes(duration_minutes);

        while current_time < working_hours.end_time {
            let appointment_end_time = current_time + appointment_duration;
            if appointment_end_time > working_hours.end_time {
                break;
            }

            let naive_dt = date.and_time(current_time);
            let slot_start_datetime = Utc.from_utc_datetime(&naive_dt);
            let slot_end_datetime = slot_start_datetime + appointment_duration;

            let overlapping = appointments.iter().any(|apt| {
                slot_start_datetime < apt.end_time && slot_end_datetime > apt.start_time
            });

            if !overlapping {
                available_slots.push(current_time);
            }

            current_time = current_time + slot_duration;
        }

        info!(
            "Found {} available slots for practitioner {} on {}",
            available_slots.len(),
            practitioner_id,
            date
        );
        Ok(available_slots)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::appointment::{AppointmentType, RepositoryError};
    use crate::domain::audit::AuditEmitterError;
    use async_trait::async_trait;
    use chrono::Duration;
    use std::sync::Mutex;

    struct NoOpAuditEmitter;

    #[async_trait]
    impl AuditEmitter for NoOpAuditEmitter {
        async fn emit(&self, _entry: AuditEntry) -> Result<(), AuditEmitterError> {
            Ok(())
        }
    }

    struct MockAppointmentRepository {
        appointments: Vec<Appointment>,
        overlapping: Vec<Appointment>,
        created: Mutex<Vec<Appointment>>,
        updated: Mutex<Vec<Appointment>>,
        update_error: Mutex<Option<RepositoryError>>,
    }

    #[async_trait]
    impl AppointmentRepository for MockAppointmentRepository {
        async fn find_by_id(&self, id: Uuid) -> Result<Option<Appointment>, RepositoryError> {
            Ok(self.appointments.iter().find(|a| a.id == id).cloned())
        }

        async fn create(&self, appointment: Appointment) -> Result<Appointment, RepositoryError> {
            self.created
                .lock()
                .expect("created lock poisoned")
                .push(appointment.clone());
            Ok(appointment)
        }

        async fn update(&self, appointment: Appointment) -> Result<Appointment, RepositoryError> {
            if let Some(err) = self
                .update_error
                .lock()
                .expect("update_error lock poisoned")
                .take()
            {
                return Err(err);
            }
            self.updated
                .lock()
                .expect("updated lock poisoned")
                .push(appointment.clone());
            Ok(appointment)
        }

        async fn delete(&self, _id: Uuid) -> Result<(), RepositoryError> {
            Ok(())
        }

        async fn find_by_criteria(
            &self,
            _criteria: &AppointmentSearchCriteria,
        ) -> Result<Vec<Appointment>, RepositoryError> {
            Ok(self.appointments.clone())
        }

        async fn find_overlapping(
            &self,
            _practitioner_id: Uuid,
            _start_time: chrono::DateTime<Utc>,
            _end_time: chrono::DateTime<Utc>,
        ) -> Result<Vec<Appointment>, RepositoryError> {
            Ok(self.overlapping.clone())
        }
    }

    struct MockCalendarQuery;

    #[async_trait]
    impl AppointmentCalendarQuery for MockCalendarQuery {
        async fn find_calendar_appointments(
            &self,
            _criteria: &AppointmentSearchCriteria,
        ) -> Result<Vec<CalendarAppointment>, RepositoryError> {
            Ok(vec![])
        }
    }

    fn new_service(
        appointments: Vec<Appointment>,
        overlapping: Vec<Appointment>,
    ) -> AppointmentService {
        AppointmentService::new(
            Arc::new(MockAppointmentRepository {
                appointments,
                overlapping,
                created: Mutex::new(vec![]),
                updated: Mutex::new(vec![]),
                update_error: Mutex::new(None),
            }),
            Arc::new(NoOpAuditEmitter),
            Arc::new(MockCalendarQuery),
        )
    }

    fn test_new_appointment_data(
        practitioner_id: Uuid,
        start_time: chrono::DateTime<Utc>,
    ) -> NewAppointmentData {
        NewAppointmentData {
            patient_id: Uuid::new_v4(),
            practitioner_id,
            start_time,
            duration: Duration::minutes(15),
            appointment_type: AppointmentType::Standard,
            reason: Some("Checkup".to_string()),
            is_urgent: false,
        }
    }

    #[tokio::test]
    async fn test_create_appointment_prevents_double_booking() {
        let practitioner_id = Uuid::new_v4();
        let start_time = Utc::now() + Duration::hours(1);
        let overlapping = Appointment::new(
            Uuid::new_v4(),
            practitioner_id,
            start_time,
            Duration::minutes(15),
            AppointmentType::Standard,
            Some(Uuid::new_v4()),
        );

        let service = new_service(vec![], vec![overlapping]);

        let result = service
            .create_appointment(
                test_new_appointment_data(practitioner_id, start_time),
                Uuid::new_v4(),
            )
            .await;

        assert!(matches!(result, Err(ServiceError::Conflict(_))));
    }

    #[tokio::test]
    async fn test_reschedule_allows_existing_appointment_when_excluded() {
        let practitioner_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let original_start = Utc::now() + Duration::hours(2);

        let existing = Appointment::new(
            Uuid::new_v4(),
            practitioner_id,
            original_start,
            Duration::minutes(15),
            AppointmentType::Standard,
            Some(user_id),
        );

        let service = new_service(vec![existing.clone()], vec![existing.clone()]);

        let new_start = original_start + Duration::hours(1);
        let result = service
            .reschedule_appointment(existing.id, new_start, 30, user_id)
            .await;

        assert!(result.is_ok());
        let updated = result.expect("reschedule should succeed when only self overlaps");
        assert_eq!(updated.start_time, new_start);
        assert_eq!(updated.end_time, new_start + Duration::minutes(30));
    }

    use crate::domain::user::WorkingHours;
    use crate::domain::user::WorkingHoursRepository;

    struct MockWorkingHoursRepository {
        working_hours: Option<WorkingHours>,
    }

    #[async_trait]
    impl WorkingHoursRepository for MockWorkingHoursRepository {
        async fn find_by_practitioner(
            &self,
            _practitioner_id: Uuid,
        ) -> Result<Vec<WorkingHours>, RepositoryError> {
            Ok(self.working_hours.iter().cloned().collect())
        }

        async fn find_for_day(
            &self,
            _practitioner_id: Uuid,
            _day_of_week: u8,
        ) -> Result<Option<WorkingHours>, RepositoryError> {
            Ok(self.working_hours.clone())
        }

        async fn save(
            &self,
            _working_hours: WorkingHours,
        ) -> Result<WorkingHours, RepositoryError> {
            Err(RepositoryError::Database("not implemented".to_string()))
        }

        async fn delete(&self, _id: Uuid) -> Result<(), RepositoryError> {
            Err(RepositoryError::Database("not implemented".to_string()))
        }
    }

    fn new_availability_service(
        appointments: Vec<Appointment>,
        working_hours: Option<WorkingHours>,
    ) -> AvailabilityService {
        AvailabilityService::new(
            Arc::new(MockAppointmentRepository {
                appointments,
                overlapping: vec![],
                created: Mutex::new(vec![]),
                updated: Mutex::new(vec![]),
                update_error: Mutex::new(None),
            }),
            Arc::new(MockWorkingHoursRepository { working_hours }),
        )
    }

    #[tokio::test]
    async fn test_update_appointment_returns_conflict_for_concurrent_modification() {
        let practitioner_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let appointment = Appointment::new(
            Uuid::new_v4(),
            practitioner_id,
            Utc::now() + Duration::hours(1),
            Duration::minutes(15),
            AppointmentType::Standard,
            Some(user_id),
        );

        let service = AppointmentService::new(
            Arc::new(MockAppointmentRepository {
                appointments: vec![appointment.clone()],
                overlapping: vec![],
                created: Mutex::new(vec![]),
                updated: Mutex::new(vec![]),
                update_error: Mutex::new(Some(RepositoryError::Conflict(
                    "Appointment was modified by another user".to_string(),
                ))),
            }),
            Arc::new(NoOpAuditEmitter),
            Arc::new(MockCalendarQuery),
        );

        let result = service
            .update_appointment(
                appointment.id,
                UpdateAppointmentData {
                    patient_id: None,
                    practitioner_id: None,
                    start_time: None,
                    duration: None,
                    status: Some(AppointmentStatus::Confirmed),
                    appointment_type: None,
                    reason: None,
                    notes: None,
                    is_urgent: None,
                    reminder_sent: None,
                    confirmed: None,
                    cancellation_reason: None,
                },
                1,
                user_id,
            )
            .await;

        assert!(matches!(result, Err(ServiceError::Conflict(_))));
    }

    #[tokio::test]
    async fn test_available_slots_with_no_working_hours() {
        let practitioner_id = Uuid::new_v4();
        let service = new_availability_service(vec![], None);
        let date = chrono::NaiveDate::from_ymd_opt(2026, 3, 11).unwrap();

        let result = service.get_available_slots(practitioner_id, date, 15).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![]);
    }

    #[tokio::test]
    async fn test_available_slots_generates_15_minute_intervals() {
        let practitioner_id = Uuid::new_v4();
        let wh = WorkingHours::new(
            practitioner_id,
            2,
            NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
        )
        .unwrap();

        let service = new_availability_service(vec![], Some(wh));
        let date = chrono::NaiveDate::from_ymd_opt(2026, 3, 11).unwrap();

        let result = service.get_available_slots(practitioner_id, date, 15).await;

        assert!(result.is_ok());
        let slots = result.unwrap();
        assert_eq!(slots.len(), 4);
        assert_eq!(slots[0], NaiveTime::from_hms_opt(9, 0, 0).unwrap());
        assert_eq!(slots[1], NaiveTime::from_hms_opt(9, 15, 0).unwrap());
        assert_eq!(slots[2], NaiveTime::from_hms_opt(9, 30, 0).unwrap());
        assert_eq!(slots[3], NaiveTime::from_hms_opt(9, 45, 0).unwrap());
    }

    #[tokio::test]
    async fn test_available_slots_filters_overlapping_appointments() {
        let practitioner_id = Uuid::new_v4();
        let date = chrono::NaiveDate::from_ymd_opt(2026, 3, 11).unwrap();

        let wh = WorkingHours::new(
            practitioner_id,
            2,
            NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
        )
        .unwrap();

        let start_of_slot_1 = Utc.from_utc_datetime(&date.and_hms_opt(9, 15, 0).unwrap());
        let appointment = Appointment::new(
            Uuid::new_v4(),
            practitioner_id,
            start_of_slot_1,
            Duration::minutes(15),
            AppointmentType::Standard,
            Some(Uuid::new_v4()),
        );

        let service = new_availability_service(vec![appointment], Some(wh));

        let result = service.get_available_slots(practitioner_id, date, 15).await;

        assert!(result.is_ok());
        let slots = result.unwrap();
        assert_eq!(slots.len(), 3);
        assert_eq!(slots[0], NaiveTime::from_hms_opt(9, 0, 0).unwrap());
        assert_eq!(slots[1], NaiveTime::from_hms_opt(9, 30, 0).unwrap());
        assert_eq!(slots[2], NaiveTime::from_hms_opt(9, 45, 0).unwrap());
    }

    #[tokio::test]
    async fn test_available_slots_respects_appointment_duration() {
        let practitioner_id = Uuid::new_v4();
        let date = chrono::NaiveDate::from_ymd_opt(2026, 3, 11).unwrap();

        let wh = WorkingHours::new(
            practitioner_id,
            2,
            NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
        )
        .unwrap();

        let service = new_availability_service(vec![], Some(wh));

        let result = service.get_available_slots(practitioner_id, date, 30).await;

        assert!(result.is_ok());
        let slots = result.unwrap();
        assert_eq!(slots.len(), 3);
        assert_eq!(slots[0], NaiveTime::from_hms_opt(9, 0, 0).unwrap());
        assert_eq!(slots[1], NaiveTime::from_hms_opt(9, 15, 0).unwrap());
        assert_eq!(slots[2], NaiveTime::from_hms_opt(9, 30, 0).unwrap());
    }

    #[tokio::test]
    async fn test_available_slots_no_overlap_with_partial_appointment() {
        let practitioner_id = Uuid::new_v4();
        let date = chrono::NaiveDate::from_ymd_opt(2026, 3, 11).unwrap();

        let wh = WorkingHours::new(
            practitioner_id,
            2,
            NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
        )
        .unwrap();

        let start_of_slot = Utc.from_utc_datetime(&date.and_hms_opt(9, 20, 0).unwrap());
        let appointment = Appointment::new(
            Uuid::new_v4(),
            practitioner_id,
            start_of_slot,
            Duration::minutes(20),
            AppointmentType::Standard,
            Some(Uuid::new_v4()),
        );

        let service = new_availability_service(vec![appointment], Some(wh));

        let result = service.get_available_slots(practitioner_id, date, 15).await;

        assert!(result.is_ok());
        let slots = result.unwrap();
        assert_eq!(slots.len(), 2);
        assert_eq!(slots[0], NaiveTime::from_hms_opt(9, 0, 0).unwrap());
        assert_eq!(slots[1], NaiveTime::from_hms_opt(9, 45, 0).unwrap());
    }
}

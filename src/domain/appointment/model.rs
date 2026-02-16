use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};
use uuid::Uuid;

/// Core appointment entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Appointment {
    pub id: Uuid,
    pub patient_id: Uuid,
    pub practitioner_id: Uuid,

    /// Appointment start date and time
    pub start_time: DateTime<Utc>,

    /// Appointment end date and time
    pub end_time: DateTime<Utc>,

    pub appointment_type: AppointmentType,
    pub status: AppointmentStatus,

    /// Reason for visit
    pub reason: Option<String>,

    /// Internal notes (not visible to patient)
    pub notes: Option<String>,

    /// Is this an emergency appointment?
    pub is_urgent: bool,

    /// SMS reminder sent
    pub reminder_sent: bool,

    /// Appointment confirmed by patient
    pub confirmed: bool,

    /// Cancellation reason (if cancelled)
    pub cancellation_reason: Option<String>,

    /// Audit fields
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
}

impl Appointment {
    pub fn new(
        patient_id: Uuid,
        practitioner_id: Uuid,
        start_time: DateTime<Utc>,
        duration: Duration,
        appointment_type: AppointmentType,
        created_by: Option<Uuid>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            patient_id,
            practitioner_id,
            start_time,
            end_time: start_time + duration,
            appointment_type,
            status: AppointmentStatus::Scheduled,
            reason: None,
            notes: None,
            is_urgent: false,
            reminder_sent: false,
            confirmed: false,
            cancellation_reason: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_by,
            updated_by: None,
        }
    }

    /// Calculate duration in minutes
    pub fn duration_minutes(&self) -> i64 {
        (self.end_time - self.start_time).num_minutes()
    }

    /// Check if appointment is in the past
    pub fn is_past(&self) -> bool {
        self.end_time < Utc::now()
    }

    /// Check if appointment is today
    pub fn is_today(&self) -> bool {
        let today = Utc::now().date_naive();
        self.start_time.date_naive() == today
    }

    /// Mark as arrived
    pub fn mark_arrived(&mut self, user_id: Uuid) {
        self.status = AppointmentStatus::Arrived;
        self.updated_at = Utc::now();
        self.updated_by = Some(user_id);
    }

    /// Mark as in progress
    pub fn mark_in_progress(&mut self, user_id: Uuid) {
        self.status = AppointmentStatus::InProgress;
        self.updated_at = Utc::now();
        self.updated_by = Some(user_id);
    }

    /// Mark as completed
    pub fn mark_completed(&mut self, user_id: Uuid) {
        self.status = AppointmentStatus::Completed;
        self.updated_at = Utc::now();
        self.updated_by = Some(user_id);
    }

    /// Cancel appointment
    pub fn cancel(&mut self, reason: String, user_id: Uuid) {
        self.status = AppointmentStatus::Cancelled;
        self.cancellation_reason = Some(reason);
        self.updated_at = Utc::now();
        self.updated_by = Some(user_id);
    }

    /// Validate if appointment can transition to a new status
    ///
    /// Enforces valid state transitions to prevent illogical status changes.
    /// Terminal states (Completed, NoShow, Cancelled) cannot transition to other states.
    ///
    /// # Valid Transitions
    /// - Scheduled → Confirmed, Cancelled, Rescheduled
    /// - Confirmed → Arrived, Cancelled, Rescheduled
    /// - Arrived → InProgress, NoShow
    /// - InProgress → Completed
    /// - Terminal states (Completed, NoShow, Cancelled) → No transitions allowed
    ///
    /// # Arguments
    /// * `new_status` - The status to transition to
    ///
    /// # Returns
    /// * `Ok(())` - Transition is valid
    /// * `Err(String)` - Transition is invalid with reason
    pub fn can_transition_to(&self, new_status: AppointmentStatus) -> Result<(), String> {
        use AppointmentStatus::*;

        // Same state is always valid (idempotent)
        if self.status == new_status {
            return Ok(());
        }

        let valid = match (self.status, new_status) {
            // From Scheduled
            (Scheduled, Confirmed | Arrived | Cancelled | Rescheduled) => true,

            // From Confirmed
            (Confirmed, Arrived | Cancelled | Rescheduled) => true,

            // From Arrived
            (Arrived, InProgress | NoShow) => true,

            // From InProgress
            (InProgress, Completed) => true,

            // Terminal states cannot transition (data integrity)
            (Completed, _) => false,
            (NoShow, _) => false,
            (Cancelled, _) => false,

            // Rescheduled is also terminal (new appointment should be created)
            (Rescheduled, _) => false,

            // All other transitions are invalid
            _ => false,
        };

        if valid {
            Ok(())
        } else {
            Err(format!(
                "Cannot transition from {} to {}",
                self.status, new_status
            ))
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, Display, EnumString)]
pub enum AppointmentType {
    /// Standard 15-minute consultation
    Standard,

    /// Extended 30-minute consultation
    Long,

    /// Brief 5-10 minute consultation
    Brief,

    /// Initial consultation for new patient (45 minutes)
    NewPatient,

    /// Health assessment (45+ Health Check, 75+ Health Check)
    HealthAssessment,

    /// Chronic disease management appointment
    ChronicDiseaseReview,

    /// Mental health treatment plan
    MentalHealthPlan,

    /// Immunisation appointment
    Immunisation,

    /// Procedure (e.g., minor surgery, wound care)
    Procedure,

    /// Telephone consultation
    Telephone,

    /// Video consultation (telehealth)
    Telehealth,

    /// Home visit
    HomeVisit,

    /// Emergency appointment
    Emergency,
}

impl AppointmentType {
    /// Get default duration in minutes for this appointment type
    pub fn default_duration_minutes(&self) -> i64 {
        match self {
            AppointmentType::Brief => 10,
            AppointmentType::Standard => 15,
            AppointmentType::Long => 30,
            AppointmentType::NewPatient => 45,
            AppointmentType::HealthAssessment => 45,
            AppointmentType::ChronicDiseaseReview => 30,
            AppointmentType::MentalHealthPlan => 45,
            AppointmentType::Immunisation => 15,
            AppointmentType::Procedure => 30,
            AppointmentType::Telephone => 10,
            AppointmentType::Telehealth => 15,
            AppointmentType::HomeVisit => 60,
            AppointmentType::Emergency => 15,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, Display, EnumString)]
pub enum AppointmentStatus {
    /// Appointment scheduled
    Scheduled,

    /// Patient has confirmed attendance
    Confirmed,

    /// Patient has arrived and checked in
    Arrived,

    /// Patient is in the consultation room
    InProgress,

    /// Consultation completed
    Completed,

    /// Patient did not attend (no-show)
    NoShow,

    /// Appointment cancelled
    Cancelled,

    /// Rescheduled to another time
    Rescheduled,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn create_test_appointment(status: AppointmentStatus) -> Appointment {
        let mut appt = Appointment::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Utc::now(),
            Duration::minutes(15),
            AppointmentType::Standard,
            Some(Uuid::new_v4()),
        );
        appt.status = status;
        appt
    }

    #[test]
    fn test_scheduled_can_transition_to_confirmed() {
        let appt = create_test_appointment(AppointmentStatus::Scheduled);
        assert!(appt.can_transition_to(AppointmentStatus::Confirmed).is_ok());
    }

    #[test]
    fn test_scheduled_can_transition_to_cancelled() {
        let appt = create_test_appointment(AppointmentStatus::Scheduled);
        assert!(appt.can_transition_to(AppointmentStatus::Cancelled).is_ok());
    }

    #[test]
    fn test_scheduled_can_transition_to_rescheduled() {
        let appt = create_test_appointment(AppointmentStatus::Scheduled);
        assert!(appt
            .can_transition_to(AppointmentStatus::Rescheduled)
            .is_ok());
    }

    #[test]
    fn test_scheduled_can_transition_to_arrived() {
        let appt = create_test_appointment(AppointmentStatus::Scheduled);
        assert!(appt.can_transition_to(AppointmentStatus::Arrived).is_ok());
    }

    #[test]
    fn test_confirmed_can_transition_to_arrived() {
        let appt = create_test_appointment(AppointmentStatus::Confirmed);
        assert!(appt.can_transition_to(AppointmentStatus::Arrived).is_ok());
    }

    #[test]
    fn test_confirmed_can_transition_to_cancelled() {
        let appt = create_test_appointment(AppointmentStatus::Confirmed);
        assert!(appt.can_transition_to(AppointmentStatus::Cancelled).is_ok());
    }

    #[test]
    fn test_confirmed_cannot_transition_to_completed() {
        let appt = create_test_appointment(AppointmentStatus::Confirmed);
        assert!(appt
            .can_transition_to(AppointmentStatus::Completed)
            .is_err());
    }

    #[test]
    fn test_arrived_can_transition_to_in_progress() {
        let appt = create_test_appointment(AppointmentStatus::Arrived);
        assert!(appt
            .can_transition_to(AppointmentStatus::InProgress)
            .is_ok());
    }

    #[test]
    fn test_arrived_can_transition_to_no_show() {
        let appt = create_test_appointment(AppointmentStatus::Arrived);
        assert!(appt.can_transition_to(AppointmentStatus::NoShow).is_ok());
    }

    #[test]
    fn test_arrived_cannot_transition_to_completed() {
        let appt = create_test_appointment(AppointmentStatus::Arrived);
        assert!(appt
            .can_transition_to(AppointmentStatus::Completed)
            .is_err());
    }

    #[test]
    fn test_in_progress_can_transition_to_completed() {
        let appt = create_test_appointment(AppointmentStatus::InProgress);
        assert!(appt.can_transition_to(AppointmentStatus::Completed).is_ok());
    }

    #[test]
    fn test_in_progress_cannot_transition_to_cancelled() {
        let appt = create_test_appointment(AppointmentStatus::InProgress);
        assert!(appt
            .can_transition_to(AppointmentStatus::Cancelled)
            .is_err());
    }

    #[test]
    fn test_completed_cannot_transition_to_any_status() {
        let appt = create_test_appointment(AppointmentStatus::Completed);
        assert!(appt
            .can_transition_to(AppointmentStatus::Scheduled)
            .is_err());
        assert!(appt
            .can_transition_to(AppointmentStatus::Confirmed)
            .is_err());
        assert!(appt.can_transition_to(AppointmentStatus::Arrived).is_err());
        assert!(appt
            .can_transition_to(AppointmentStatus::InProgress)
            .is_err());
        assert!(appt.can_transition_to(AppointmentStatus::NoShow).is_err());
        assert!(appt
            .can_transition_to(AppointmentStatus::Cancelled)
            .is_err());
    }

    #[test]
    fn test_no_show_cannot_transition_to_any_status() {
        let appt = create_test_appointment(AppointmentStatus::NoShow);
        assert!(appt
            .can_transition_to(AppointmentStatus::Scheduled)
            .is_err());
        assert!(appt.can_transition_to(AppointmentStatus::Arrived).is_err());
        assert!(appt
            .can_transition_to(AppointmentStatus::Completed)
            .is_err());
    }

    #[test]
    fn test_cancelled_cannot_transition_to_any_status() {
        let appt = create_test_appointment(AppointmentStatus::Cancelled);
        assert!(appt
            .can_transition_to(AppointmentStatus::Scheduled)
            .is_err());
        assert!(appt
            .can_transition_to(AppointmentStatus::Confirmed)
            .is_err());
        assert!(appt.can_transition_to(AppointmentStatus::Arrived).is_err());
    }

    #[test]
    fn test_same_status_transition_is_idempotent() {
        let appt = create_test_appointment(AppointmentStatus::Scheduled);
        assert!(appt.can_transition_to(AppointmentStatus::Scheduled).is_ok());

        let appt = create_test_appointment(AppointmentStatus::Completed);
        assert!(appt.can_transition_to(AppointmentStatus::Completed).is_ok());
    }

    #[test]
    fn test_transition_error_message_format() {
        let appt = create_test_appointment(AppointmentStatus::Cancelled);
        let result = appt.can_transition_to(AppointmentStatus::Arrived);
        assert!(result.is_err());
        let err_msg = result.unwrap_err();
        assert!(err_msg.contains("Cannot transition from"));
        assert!(err_msg.contains("Cancelled"));
        assert!(err_msg.contains("Arrived"));
    }
}

/// Waitlist entry for patients waiting for an appointment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaitlistEntry {
    pub id: Uuid,
    pub patient_id: Uuid,
    pub practitioner_id: Option<Uuid>,
    pub appointment_type: AppointmentType,
    pub reason: Option<String>,
    pub priority: WaitlistPriority,
    pub preferred_days: Vec<chrono::Weekday>,
    pub preferred_times: Vec<TimeSlot>,
    pub added_at: DateTime<Utc>,
    pub contacted_at: Option<DateTime<Utc>>,
    pub resolved_at: Option<DateTime<Utc>>,
}

#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Display, EnumString,
)]
pub enum WaitlistPriority {
    Low,
    Normal,
    High,
    Urgent,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Display, EnumString)]
pub enum TimeSlot {
    Morning,   // 8am-12pm
    Afternoon, // 12pm-5pm
    Evening,   // 5pm-8pm
}

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
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
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
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

impl std::fmt::Display for AppointmentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppointmentType::Standard => write!(f, "Standard"),
            AppointmentType::Long => write!(f, "Long"),
            AppointmentType::Brief => write!(f, "Brief"),
            AppointmentType::NewPatient => write!(f, "New Patient"),
            AppointmentType::HealthAssessment => write!(f, "Health Assessment"),
            AppointmentType::ChronicDiseaseReview => write!(f, "Chronic Disease Review"),
            AppointmentType::MentalHealthPlan => write!(f, "Mental Health Plan"),
            AppointmentType::Immunisation => write!(f, "Immunisation"),
            AppointmentType::Procedure => write!(f, "Procedure"),
            AppointmentType::Telephone => write!(f, "Telephone"),
            AppointmentType::Telehealth => write!(f, "Telehealth"),
            AppointmentType::HomeVisit => write!(f, "Home Visit"),
            AppointmentType::Emergency => write!(f, "Emergency"),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
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

impl std::fmt::Display for AppointmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppointmentStatus::Scheduled => write!(f, "Scheduled"),
            AppointmentStatus::Confirmed => write!(f, "Confirmed"),
            AppointmentStatus::Arrived => write!(f, "Arrived"),
            AppointmentStatus::InProgress => write!(f, "In Progress"),
            AppointmentStatus::Completed => write!(f, "Completed"),
            AppointmentStatus::NoShow => write!(f, "No Show"),
            AppointmentStatus::Cancelled => write!(f, "Cancelled"),
            AppointmentStatus::Rescheduled => write!(f, "Rescheduled"),
        }
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum WaitlistPriority {
    Low,
    Normal,
    High,
    Urgent,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TimeSlot {
    Morning,   // 8am-12pm
    Afternoon, // 12pm-5pm
    Evening,   // 5pm-8pm
}

use chrono::{DateTime, NaiveDate, Utc};
use uuid::Uuid;

use opengp_domain::domain::appointment::{AppointmentStatus, NewAppointmentData};

/// Commands dispatched from the UI to the async event loop.
#[derive(Debug)]
pub enum AppCommand {
    RefreshAppointments(NaiveDate),
    CreateAppointment(NewAppointmentData),
    UpdateAppointmentStatus {
        id: Uuid,
        status: AppointmentStatus,
    },
    LoadPractitioners,
    LoadAvailableSlots {
        practitioner_id: Uuid,
        date: NaiveDate,
        duration_minutes: u32,
    },
    CancelAppointment {
        id: Uuid,
        reason: String,
    },
    RescheduleAppointment {
        id: Uuid,
        new_start_time: DateTime<Utc>,
        new_duration_minutes: i64,
        user_id: Uuid,
    },
}

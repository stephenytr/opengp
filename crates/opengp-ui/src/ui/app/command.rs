use chrono::NaiveDate;
use uuid::Uuid;

use opengp_domain::domain::appointment::{AppointmentStatus, NewAppointmentData};

#[derive(Debug)]
pub enum AppCommand {
    RefreshAppointments(NaiveDate),
    CreateAppointment(NewAppointmentData),
    UpdateAppointment {
        id: Uuid,
        data: NewAppointmentData,
        version: i32,
    },
    AppointmentSaveResult(Result<(), String>),
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
}

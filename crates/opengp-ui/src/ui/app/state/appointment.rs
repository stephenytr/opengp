use chrono::NaiveDate;

use crate::ui::app::{App, AppointmentStatusTransition};
use crate::ui::components::appointment::AppointmentState;
use crate::ui::view_models::{PatientListItem, PractitionerViewItem};

impl App {
    pub fn request_refresh_appointments(&mut self, date: NaiveDate) {
        self.pending_appointment_list_refresh = Some(date);
        self.request_refresh_practitioners();
    }

    pub fn request_refresh_practitioners(&mut self) {
        self.pending_practitioners_list_refresh = true;
    }

    pub fn request_load_practitioners(&mut self) {
        self.pending_load_practitioners = true;
    }

    pub fn take_pending_load_practitioners(&mut self) -> bool {
        std::mem::take(&mut self.pending_load_practitioners)
    }

    pub fn take_pending_appointment_save(
        &mut self,
    ) -> Option<opengp_domain::domain::appointment::NewAppointmentData> {
        if !self.authenticated {
            return None;
        }
        self.pending_appointment_save.take()
    }

    pub fn take_pending_appointment_status_transition(
        &mut self,
    ) -> Option<(uuid::Uuid, AppointmentStatusTransition)> {
        self.pending_appointment_status_transition.take()
    }

    pub fn take_pending_reschedule(&mut self) -> Option<crate::ui::app::PendingRescheduleData> {
        self.pending_reschedule.take()
    }

    pub fn appointment_state_mut(&mut self) -> &mut AppointmentState {
        &mut self.appointment_state
    }

    pub fn update_schedule_appointment_status(
        &mut self,
        appointment_id: uuid::Uuid,
        status: opengp_domain::domain::appointment::AppointmentStatus,
    ) {
        if let Some(ref mut schedule) = self.appointment_state.schedule_data {
            for practitioner in &mut schedule.practitioners {
                for appointment in &mut practitioner.appointments {
                    if appointment.id == appointment_id {
                        appointment.status = status;
                        return;
                    }
                }
            }
        }
    }

    pub fn appointment_form_set_patients(&mut self, patients: Vec<PatientListItem>) {
        if let Some(ref mut form) = self.appointment_form {
            form.set_patients(patients);
        }
    }

    pub fn appointment_form_set_practitioners(&mut self, practitioners: Vec<PractitionerViewItem>) {
        if let Some(ref mut form) = self.appointment_form {
            form.set_practitioners(practitioners);
        }
    }

    pub fn take_pending_load_booked_slots(&mut self) -> Option<(uuid::Uuid, NaiveDate, u32)> {
        self.pending_load_booked_slots.take()
    }

    pub fn appointment_form_set_booked_slots(&mut self, booked_slots: Vec<chrono::NaiveTime>) {
        if let Some(ref mut form) = self.appointment_form {
            form.set_booked_slots(booked_slots);
        }
    }
}

use crate::ui::app::{App, AppointmentStatusTransition};
use crate::ui::components::appointment::{AppointmentDetailModalAction, AppointmentFormAction};
use crate::ui::keybinds::Action;
use chrono::Utc;
use crossterm::event::KeyEvent;

impl App {
    pub(crate) fn handle_appointment_form_keys(&mut self, key: KeyEvent) -> Action {
        if let Some(ref mut form) = self.appointment_form {
            if let Some(action) = form.handle_key(key) {
                match action {
                    AppointmentFormAction::FocusChanged | AppointmentFormAction::ValueChanged => {}
                    AppointmentFormAction::Submit => {
                        if let Some(ref mut form) = self.appointment_form {
                            if let Some(data) = form.to_new_appointment_data() {
                                self.pending_appointment_save = Some(data);
                                self.appointment_form = None;
                                self.status_bar.clear_error();
                            } else {
                                self.status_bar.set_error(
                                    "Cannot save: select a patient and practitioner from the picker",
                                );
                            }
                        }
                    }
                    AppointmentFormAction::Cancel => {
                        self.appointment_form = None;
                        self.status_bar.clear_error();
                    }
                    AppointmentFormAction::SaveComplete => {
                        self.appointment_form = None;
                        self.status_bar.clear_error();
                        self.request_refresh_appointments(Utc::now().date_naive());
                    }
                    AppointmentFormAction::OpenTimePicker {
                        practitioner_id,
                        date,
                        duration,
                    } => {
                        let practitioner_id_i64 = practitioner_id.as_u128() as i64;
                        if let Some(ref mut form) = self.appointment_form {
                            form.open_time_picker(practitioner_id_i64, date, duration);
                        }
                        self.pending_load_booked_slots = Some((practitioner_id, date, duration));
                    }
                }
                return Action::Enter;
            }
        }
        Action::Unknown
    }

    pub(crate) fn handle_appointment_detail_modal_keys(&mut self, key: KeyEvent) -> Action {
        if let Some(ref mut modal) = self.appointment_detail_modal {
            if let Some(action) = modal.handle_key(key) {
                match action {
                    AppointmentDetailModalAction::Close => {
                        self.appointment_detail_modal = None;
                    }
                    AppointmentDetailModalAction::ViewClinicalNotes => {
                        let patient_id = modal.patient_id();
                        self.appointment_detail_modal = None;
                        self.clinical_state.clear_patient();
                        self.clinical_state.set_patient(patient_id);
                        self.clinical_state.show_patient_summary();
                        self.tab_bar
                            .select(crate::ui::components::tabs::Tab::Clinical);
                        self.pending_clinical_patient_id = Some(patient_id);
                        self.refresh_status_bar();
                        self.refresh_context();
                    }
                    AppointmentDetailModalAction::MarkArrived => {
                        let appointment_id = modal.appointment_id();
                        self.pending_appointment_status_transition =
                            Some((appointment_id, AppointmentStatusTransition::MarkArrived));
                    }
                    AppointmentDetailModalAction::MarkInProgress => {
                        let appointment_id = modal.appointment_id();
                        self.pending_appointment_status_transition =
                            Some((appointment_id, AppointmentStatusTransition::MarkInProgress));
                    }
                    AppointmentDetailModalAction::MarkCompleted => {
                        let appointment_id = modal.appointment_id();
                        self.pending_appointment_status_transition =
                            Some((appointment_id, AppointmentStatusTransition::MarkCompleted));
                    }
                    AppointmentDetailModalAction::MarkNoShow => {
                        let appointment_id = modal.appointment_id();
                        self.pending_appointment_status_transition =
                            Some((appointment_id, AppointmentStatusTransition::MarkNoShow));
                    }
                }
                return Action::Enter;
            }
        }
        Action::Unknown
    }
}

use crate::ui::app::{App, AppointmentStatusTransition, PendingClinicalSaveData};
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
                                let version = form.form_version();
                                form.set_saving(true);
                                self.pending_appointment_save = Some((data, version));
                            } else {
                                let msg = form
                                    .first_error()
                                    .unwrap_or_else(|| "Check required fields".to_string());
                                self.status_bar.set_error(msg);
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
                        self.request_refresh_consultations(patient_id);
                        self.refresh_status_bar();
                        self.refresh_context();
                    }
                    AppointmentDetailModalAction::MarkStatus(status) => {
                        let appointment_id = modal.appointment_id();
                        self.appointment_detail_modal = None;
                        self.update_schedule_appointment_status(appointment_id, status);
                        self.pending_appointment_status_transition = Some((
                            appointment_id,
                            AppointmentStatusTransition::SetStatus(status),
                        ));
                    }
                    AppointmentDetailModalAction::StartConsultation => {
                        let appointment_id = modal.appointment_id();
                        let patient_id = modal.patient_id();
                        self.appointment_detail_modal = None;
                        self.clinical_state.clear_patient();
                        self.clinical_state.set_patient(patient_id);
                        self.clinical_state.set_active_appointment(appointment_id);
                        self.clinical_state.show_patient_summary();
                        self.pending_clinical_save_data =
                            Some(PendingClinicalSaveData::Consultation {
                                patient_id,
                                practitioner_id: self.current_user_id,
                                appointment_id: Some(appointment_id),
                                reason: None,
                                clinical_notes: None,
                            });
                        self.tab_bar
                            .select(crate::ui::components::tabs::Tab::Clinical);
                        self.pending_clinical_patient_id = Some(patient_id);
                        self.request_refresh_consultations(patient_id);
                        self.refresh_status_bar();
                        self.refresh_context();
                    }
                }
                return Action::Enter;
            }
        }
        Action::Unknown
    }
}

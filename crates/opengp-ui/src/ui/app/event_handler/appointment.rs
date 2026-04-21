use crate::ui::app::{App, AppCommand, AppointmentStatusTransition};
use crate::ui::components::appointment::{AppointmentDetailModalAction, AppointmentFormAction};
use crate::ui::components::tabs::Tab;
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
                                self.status_bar.set_error(Some(msg));
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
                        if let Some(patient_item) = self.patient_list.get_patient_by_id(patient_id) {
                            let _ = self.workspace_manager.open_patient(patient_item.clone());
                            self.tab_bar.select(Tab::PatientSearch);
                            self.previous_tab = Tab::PatientSearch;
                            self.refresh_status_bar();
                            self.refresh_context();
                        }
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
                        let patient_id = modal.patient_id();
                        self.appointment_detail_modal = None;
                        if let Some(patient_item) = self.patient_list.get_patient_by_id(patient_id) {
                            match self.workspace_manager.open_patient(patient_item.clone()) {
                                Ok(_index) => {
                                    // Set active subtab to Clinical
                                    if let Some(workspace) = self.workspace_manager.active_mut() {
                                        workspace.active_subtab = crate::ui::components::SubtabKind::Clinical;
                                    }
                                    
                                    // Switch context to PatientWorkspace (not Tab::PatientSearch)
                                    self.current_context = crate::ui::keybinds::KeyContext::PatientWorkspace;
                                    
                                    // Trigger lazy load of clinical data
                                    let _ = self.command_tx.send(AppCommand::LoadPatientWorkspaceData {
                                        patient_id,
                                        subtab: crate::ui::components::SubtabKind::Clinical,
                                    });
                                    
                                    self.refresh_status_bar();
                                    self.refresh_context();
                                }
                                Err(crate::ui::components::workspace::WorkspaceError::AlreadyAtLimit) => {
                                    let max = self.workspace_manager.max_open;
                                    let error_msg = format!(
                                        "Max open patients reached (max: {}). Close a tab first.",
                                        max
                                    );
                                    self.status_bar.set_error(Some(error_msg));
                                }
                                Err(err) => {
                                    self.status_bar.set_error(Some(err.to_string()));
                                }
                            }
                        }
                    }
                    AppointmentDetailModalAction::RescheduleDate => {
                        // Date picker is opening in modal, keep modal open
                    }
                    AppointmentDetailModalAction::OpenTimePicker {
                        practitioner_id,
                        date,
                        duration,
                    } => {
                        self.pending_load_booked_slots = Some((practitioner_id, date, duration));
                    }
                    AppointmentDetailModalAction::RescheduleTime => {
                        if let Some(ref modal) = self.appointment_detail_modal {
                            if let Some(new_date) = modal.pending_reschedule_date() {
                                if let Some(new_time) = modal.pending_reschedule_time() {
                                    let appointment_id = modal.appointment_id();
                                    let practitioner_id = modal.appointment().practitioner_id;
                                    let duration_minutes =
                                        modal.appointment().duration_minutes() as i64;
                                    self.pending_reschedule =
                                        Some(crate::ui::app::PendingRescheduleData {
                                            appointment_id,
                                            new_date: Some(new_date),
                                            new_time: Some(new_time),
                                            practitioner_id,
                                            duration_minutes,
                                        });
                                    self.appointment_detail_modal = None;
                                }
                            }
                        }
                    }
                }
                return Action::Enter;
            }
        }
        Action::Unknown
    }
}

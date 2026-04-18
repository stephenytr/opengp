mod appointment;
mod global;

#[cfg(test)]
mod workspace_tests;

use crate::ui::app::{App, AppCommand, PendingPatientData};
use crate::ui::components::appointment::{AppointmentForm, AppointmentView};
use crate::ui::components::tabs::Tab;
use crate::ui::keybinds::{Action, KeyContext};
use crate::ui::widgets::FormNavigation;
use crossterm::event::{Event, KeyEvent};

impl App {
    pub fn handle_key_event(&mut self, key: KeyEvent) -> Action {
        if self.server_unavailable_error.is_some() {
            match key.code {
                crossterm::event::KeyCode::Char('r') | crossterm::event::KeyCode::Char('R') => {
                    self.retry_server_unavailable_operation();
                    return Action::Refresh;
                }
                crossterm::event::KeyCode::Esc => {
                    self.clear_server_unavailable_error();
                    return Action::Escape;
                }
                _ => {}
            }
        }

        if self.help_overlay.is_visible() {
            if key.code == crossterm::event::KeyCode::Esc
                || key.code == crossterm::event::KeyCode::F(1)
            {
                self.help_overlay.hide();
                return Action::Escape;
            }
            return Action::Unknown;
        }

        if !self.authenticated {
            if let Some(crate::ui::screens::LoginAction::Submit { username, password }) =
                self.login_screen.handle_key(key)
            {
                self.pending_login_request = Some((username, password));
                return Action::Submit;
            }

            if let Some(action) = self
                .keybinds
                .lookup(key, KeyContext::Global)
                .map(|kb| kb.action.clone())
            {
                match action {
                    Action::OpenHelp => self.help_overlay.toggle(),
                    Action::Quit => {
                        self.should_quit = true;
                    }
                    _ => {}
                }
                return action;
            }

            return Action::Unknown;
        }

        if self.tab_bar.selected() == Tab::Patient
            && self.patient_form.is_none()
            && self.patient_list.is_searching()
        {
            if let Some(action) = self.patient_list.handle_key(key) {
                match action {
                    crate::ui::components::patient::PatientListAction::SelectionChanged => {}
                    crate::ui::components::patient::PatientListAction::OpenPatient(id) => {
                        if let Some(patient_item) = self.patient_list.get_patient_by_id(id) {
                            let _ = self.workspace_manager.open_patient(patient_item.clone());
                            self.tab_bar.select(Tab::Patient);
                        }
                    }
                    crate::ui::components::patient::PatientListAction::FocusSearch => {}
                    crate::ui::components::patient::PatientListAction::SearchChanged => {}
                }
                return Action::Enter;
            }
        }



        if self.appointment_form.is_some() {
            return self.handle_appointment_form_keys(key);
        }

        if self.appointment_detail_modal.is_some() {
            return self.handle_appointment_detail_modal_keys(key);
        }

        if self.patient_form.is_some() {
            if let Some(ref mut form) = self.patient_form {
                if let Some(action) = form.handle_key(key) {
                    match action {
                        crate::ui::components::patient::PatientFormAction::FocusChanged => {}
                        crate::ui::components::patient::PatientFormAction::ValueChanged => {}
                        crate::ui::components::patient::PatientFormAction::Submit => {
                            if let Some(ref mut form) = self.patient_form {
                                // Run full validation on all fields
                                if !form.validate() || form.has_errors() {
                                    // Stay open — errors are visible on fields
                                    form.focus_first_error();
                                } else if form.is_edit_mode() {
                                    if let Some((id, data)) = form.to_update_patient_data() {
                                        self.pending_patient_data =
                                            Some(PendingPatientData::Update { id, data });
                                        self.patient_form = None;
                                        self.current_context = KeyContext::PatientList;
                                    } else {
                                        // Data extraction failed after validation — stay open
                                        form.focus_first_error();
                                    }
                                } else if let Some(data) = form.to_new_patient_data() {
                                    self.pending_patient_data = Some(PendingPatientData::New(data));
                                    self.patient_form = None;
                                    self.current_context = KeyContext::PatientList;
                                } else {
                                    // Data extraction failed after validation — stay open
                                    form.focus_first_error();
                                }
                            }
                        }
                        crate::ui::components::patient::PatientFormAction::Cancel => {
                            self.patient_form = None;
                            self.current_context = KeyContext::PatientList;
                        }
                        crate::ui::components::patient::PatientFormAction::SaveComplete => {
                            self.request_refresh_patients();
                        }
                    }
                    return Action::Enter;
                }
            }
        }

        let action = self
            .keybinds
            .lookup(key, self.current_context)
            .map(|kb| kb.action.clone());

        if let Some(action) = action {
            match action {
                Action::SwitchToPatient => {
                    self.tab_bar.select(Tab::Patient);
                    self.previous_tab = Tab::Patient;
                    self.refresh_status_bar();
                    self.refresh_context();
                }
                Action::SwitchToAppointments => {
                    self.tab_bar.select(Tab::Appointment);
                    let today = chrono::Utc::now().date_naive();
                    self.appointment_state.selected_date = Some(today);
                    // Auto-refresh appointments when switching to Appointment tab
                    if self.previous_tab != Tab::Appointment {
                        self.request_refresh_appointments(today);
                    }
                    self.previous_tab = Tab::Appointment;
                    self.refresh_status_bar();
                    self.refresh_context();
                }

                Action::OpenHelp => {
                    self.help_overlay.toggle();
                }
                Action::Quit => {
                    let is_ctrl_q = key
                        .modifiers
                        .contains(crossterm::event::KeyModifiers::CONTROL);
                    if is_ctrl_q || self.tab_bar.selected() == Tab::Patient {
                        self.should_quit = true;
                    }
                }
                Action::New => {
                    if self.tab_bar.selected() == Tab::Patient && self.patient_form.is_none() {
                        self.patient_form = Some(crate::ui::components::patient::PatientForm::new(
                            self.theme.clone(),
                            &self.patient_config,
                        ));
                        self.current_context = KeyContext::PatientForm;
                    }
                }
                Action::Edit => {
                    if self.tab_bar.selected() == Tab::Patient && self.patient_form.is_none() {
                        if let Some(patient_id) = self.patient_list.selected_patient_id() {
                            self.request_edit_patient(patient_id);
                        }
                    }
                }
                Action::Delete => {}
                Action::Escape => {
                    if self.patient_form.is_some() {
                        self.patient_form = None;
                        self.current_context = KeyContext::PatientList;
                    }
                    if self.appointment_form.is_some() {
                        self.appointment_form = None;
                    }
                    if self.tab_bar.selected() == Tab::Appointment
                        && self.appointment_state.current_view == AppointmentView::Schedule
                        && self.appointment_form.is_none()
                    {
                        self.appointment_state.current_view = AppointmentView::Calendar;
                        self.appointment_state.calendar.focused = true;
                        self.appointment_state.focused = false;
                        self.refresh_context();
                    }
                }
                Action::Save => {}
                Action::Refresh => match self.tab_bar.selected() {
                    Tab::Patient => self.request_refresh_patients(),
                    Tab::Appointment => {
                        let date = self
                            .appointment_state
                            .selected_date
                            .unwrap_or_else(|| chrono::Utc::now().date_naive());
                        self.request_refresh_appointments(date);
                    }
                },
                Action::NavigateDown => {
                    if self.tab_bar.selected() == Tab::Patient && self.patient_form.is_none() {
                        let visible_rows = self.calculate_visible_patient_rows();
                        self.patient_list.move_down_and_scroll(visible_rows);
                    }
                }
                Action::NavigateUp => {
                    if self.tab_bar.selected() == Tab::Patient && self.patient_form.is_none() {
                        let visible_rows = self.calculate_visible_patient_rows();
                        self.patient_list.move_up_and_scroll(visible_rows);
                    }
                }
                Action::PrevDay
                | Action::NextDay
                | Action::Today
                | Action::PrevMonth
                | Action::NextMonth
                | Action::SelectDate => {
                    if self.tab_bar.selected() == Tab::Appointment {
                        return self.handle_appointment_keys(key);
                    }
                }
                Action::PrevPractitioner
                | Action::NextPractitioner
                | Action::PrevTimeSlot
                | Action::NextTimeSlot
                | Action::ScrollViewportUp
                | Action::ScrollViewportDown => {
                    if self.tab_bar.selected() == Tab::Appointment {
                        return self.handle_appointment_keys(key);
                    }
                }
                Action::Enter => {
                    if self.tab_bar.selected() == Tab::Patient {
                        return self.handle_patient_keys(key);
                    }
                    if self.tab_bar.selected() == Tab::Appointment {
                        return self.handle_appointment_keys(key);
                    }
                }
                Action::NewAppointment => {
                    if self.tab_bar.selected() == Tab::Appointment
                        && self.appointment_form.is_none()
                    {
                        self.appointment_form = Some(AppointmentForm::new(
                            self.theme.clone(),
                            self.healthcare_config.clone(),
                        ));
                        self.request_load_practitioners();
                    }
                }

                // Workspace (multi-patient tab) actions
                Action::ClosePatientTab => {
                    let blocked = self
                        .workspace_manager
                        .active()
                        .and_then(|workspace| workspace.clinical.as_ref())
                        .map(|clinical| {
                            clinical.consultations.consultation_form.is_some()
                                || clinical.consultations.active_timer_started_at.is_some()
                        })
                        .unwrap_or(false);

                    if blocked {
                        self.status_bar
                            .set_error("Cannot close: form open or timer active");
                    } else {
                        let _ = self.workspace_manager.close_active();
                        self.refresh_status_bar();
                        self.refresh_context();
                    }
                }

                Action::NextPatientTab => {
                    self.workspace_manager.cycle_next();
                    self.refresh_status_bar();
                    self.refresh_context();
                }

                Action::PrevPatientTab => {
                    self.workspace_manager.cycle_prev();
                    self.refresh_status_bar();
                    self.refresh_context();
                }

                Action::SelectPatientTab(n) => {
                    let _ = self.workspace_manager.select_by_index(n);
                    self.refresh_status_bar();
                    self.refresh_context();
                }

                Action::NextSubtab => {
                    if let Some(workspace) = self.workspace_manager.active_mut() {
                        let current = workspace.active_subtab;
                        let next = match current {
                            crate::ui::components::SubtabKind::Clinical => {
                                crate::ui::components::SubtabKind::Billing
                            }
                            crate::ui::components::SubtabKind::Billing => {
                                crate::ui::components::SubtabKind::Appointments
                            }
                            crate::ui::components::SubtabKind::Appointments => {
                                crate::ui::components::SubtabKind::Clinical
                            }
                            _ => current,
                        };
                        workspace.active_subtab = next;
                        
                        // Lazy load if subtab not yet loaded
                        if !workspace.is_loaded(next) {
                            if let Some(patient_id) = self.workspace_manager.active().map(|w| w.patient_id) {
                                // Send command to load workspace data for this subtab
                                let _ = self.command_tx.send(AppCommand::LoadPatientWorkspaceData {
                                    patient_id,
                                    subtab: next,
                                });
                            }
                        }
                        self.refresh_status_bar();
                    }
                }

                Action::PrevSubtab => {
                    if let Some(workspace) = self.workspace_manager.active_mut() {
                        let current = workspace.active_subtab;
                        let prev = match current {
                            crate::ui::components::SubtabKind::Clinical => {
                                crate::ui::components::SubtabKind::Appointments
                            }
                            crate::ui::components::SubtabKind::Billing => {
                                crate::ui::components::SubtabKind::Clinical
                            }
                            crate::ui::components::SubtabKind::Appointments => {
                                crate::ui::components::SubtabKind::Billing
                            }
                            _ => current,
                        };
                        workspace.active_subtab = prev;
                        
                        // Lazy load if subtab not yet loaded
                        if !workspace.is_loaded(prev) {
                            if let Some(patient_id) = self.workspace_manager.active().map(|w| w.patient_id) {
                                let _ = self.command_tx.send(AppCommand::LoadPatientWorkspaceData {
                                    patient_id,
                                    subtab: prev,
                                });
                            }
                        }
                        self.refresh_status_bar();
                    }
                }

                Action::OpenPatientFromList => {
                    if self.tab_bar.selected() == Tab::Patient && self.patient_form.is_none() {
                        if let Some(patient) = self.patient_list.selected_patient() {
                            match self.workspace_manager.open_patient(patient.clone()) {
                                Ok(index) => {
                                    self.workspace_manager.active_index = Some(index);
                                    self.current_context = KeyContext::PatientWorkspace;
                                    self.refresh_status_bar();
                                    self.refresh_context();
                                }
                                Err(crate::ui::components::workspace::WorkspaceError::AlreadyAtLimit) => {
                                    let max = self.workspace_manager.max_open;
                                    let error_msg = format!(
                                        "Max open patients reached (max: {}). Close a tab first.",
                                        max
                                    );
                                    self.status_bar.set_error(error_msg);
                                }
                                Err(err) => {
                                    self.status_bar.set_error(err.to_string());
                                }
                            }
                        }
                    }
                }

                _ => {}
            }
            return action;
        }

        if let Some(_tab) = self.tab_bar.handle_key(key) {
            self.refresh_status_bar();
            self.refresh_context();
            return Action::Enter;
        }

        if self.tab_bar.selected() == Tab::Patient && self.patient_form.is_none() {
            return self.handle_patient_keys(key);
        }

        if self.tab_bar.selected() == Tab::Appointment {
            return self.handle_appointment_keys(key);
        }

        Action::Unknown
    }

    pub fn handle_event(&mut self, event: Event) {
        match event {
            Event::Key(key) => {
                self.handle_key_event(key);
            }
            Event::Mouse(mouse) => {
                let area = self.terminal_size;
                self.handle_global_mouse_event(mouse, area);
            }
            Event::Resize(w, h) => {
                self.terminal_size = ratatui::layout::Rect::new(0, 0, w, h);
            }
            _ => {}
        }
    }
}

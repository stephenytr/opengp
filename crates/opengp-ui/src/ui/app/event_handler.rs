mod appointment;
mod global;

#[cfg(test)]
mod workspace_tests;

use crate::ui::app::{App, PendingPatientData};
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

        // Route key events to context menu if visible
        if self.is_context_menu_visible() {
            if let Some(ref mut ctx_menu) = self.context_menu_state {
                if let Some(action) = ctx_menu.handle_key(key) {
                    match action {
                        crate::ui::widgets::ContextMenuAction::Selected(app_action) => {
                            self.hide_context_menu();
                            // Handle the AppContextMenuAction
                            match app_action {
                                crate::ui::app::AppContextMenuAction::PatientEdit(id) => {
                                    self.request_edit_patient(id);
                                }
                                crate::ui::app::AppContextMenuAction::PatientDelete(id) => {
                                    // TODO: Implement delete patient
                                }
                                crate::ui::app::AppContextMenuAction::PatientViewHistory(id) => {
                                    // TODO: Implement view history
                                }
                                crate::ui::app::AppContextMenuAction::AppointmentEdit(id) => {
                                    // TODO: Implement edit appointment
                                }
                                crate::ui::app::AppContextMenuAction::AppointmentCancel(id) => {
                                    // TODO: Implement cancel appointment
                                }
                                crate::ui::app::AppContextMenuAction::AppointmentReschedule(id) => {
                                    // TODO: Implement reschedule appointment
                                }
                                crate::ui::app::AppContextMenuAction::ClinicalEdit(id) => {
                                    // TODO: Implement edit clinical record
                                }
                                crate::ui::app::AppContextMenuAction::ClinicalDelete(id) => {
                                    // TODO: Implement delete clinical record
                                }
                                crate::ui::app::AppContextMenuAction::BillingEdit(id) => {
                                    // TODO: Implement edit billing record
                                }
                                crate::ui::app::AppContextMenuAction::BillingViewInvoice(id) => {
                                    // TODO: Implement view invoice
                                }
                            }
                        }
                        crate::ui::widgets::ContextMenuAction::Dismissed => {
                            self.hide_context_menu();
                        }
                        crate::ui::widgets::ContextMenuAction::FocusChanged => {
                            // Just update focus, no state change
                        }
                    }
                    return Action::Enter;
                }
            }
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
                Action::SwitchToSchedule => {
                    self.tab_bar.select(Tab::Schedule);
                    self.previous_tab = Tab::Schedule;
                    self.refresh_status_bar();
                    self.refresh_context();
                }
                Action::SwitchToPatientSearch => {
                    if self.workspace_manager.active().is_some() {
                        self.workspace_manager.active_index = None;
                    }
                    let coming_from_different_tab = self.previous_tab != Tab::PatientSearch;
                    if coming_from_different_tab {
                        self.patient_list.reset_search();
                        self.request_refresh_patients();
                    }
                    self.tab_bar.select(Tab::PatientSearch);
                    self.previous_tab = Tab::PatientSearch;
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
                    if is_ctrl_q || self.tab_bar.selected() == Tab::PatientSearch {
                        self.should_quit = true;
                    }
                }
                Action::New => {
                    if self.tab_bar.selected() == Tab::PatientSearch && self.patient_form.is_none()
                    {
                        self.patient_form = Some(crate::ui::components::patient::PatientForm::new(
                            self.theme.clone(),
                            &self.patient_config,
                        ));
                        self.current_context = KeyContext::PatientForm;
                    }
                }
                Action::Edit => {
                    if self.tab_bar.selected() == Tab::PatientSearch && self.patient_form.is_none()
                    {
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
                    if self.tab_bar.selected() == Tab::Schedule
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
                    Tab::PatientSearch | Tab::PatientWorkspace => self.request_refresh_patients(),
                    Tab::Schedule => {
                        let date = self
                            .appointment_state
                            .selected_date
                            .unwrap_or_else(|| chrono::Utc::now().date_naive());
                        self.request_refresh_appointments(date);
                    }
                },
                Action::NavigateDown => {
                    if (self.tab_bar.selected() == Tab::PatientSearch
                        || self.tab_bar.selected() == Tab::PatientWorkspace)
                        && self.patient_form.is_none()
                    {
                        let visible_rows = self.calculate_visible_patient_rows();
                        self.patient_list.move_down_and_scroll(visible_rows);
                    }
                }
                Action::NavigateUp => {
                    if (self.tab_bar.selected() == Tab::PatientSearch
                        || self.tab_bar.selected() == Tab::PatientWorkspace)
                        && self.patient_form.is_none()
                    {
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
                    if self.tab_bar.selected() == Tab::Schedule {
                        return self.handle_appointment_keys(key);
                    }
                }
                Action::PrevPractitioner
                | Action::NextPractitioner
                | Action::PrevTimeSlot
                | Action::NextTimeSlot
                | Action::ScrollViewportUp
                | Action::ScrollViewportDown => {
                    if self.tab_bar.selected() == Tab::Schedule {
                        return self.handle_appointment_keys(key);
                    }
                }
                Action::Enter => {
                    if self.tab_bar.selected() == Tab::PatientSearch
                        || self.tab_bar.selected() == Tab::PatientWorkspace
                    {
                        return self.handle_patient_keys(key);
                    }
                    if self.tab_bar.selected() == Tab::Schedule {
                        return self.handle_appointment_keys(key);
                    }
                }
                Action::NewAppointment => {
                    if self.tab_bar.selected() == Tab::Schedule && self.appointment_form.is_none() {
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
                            .set_error(Some("Cannot close: form open or timer active".to_string()));
                    } else {
                        let _ = self.workspace_manager.close_active();
                        if self.workspace_manager.active().is_some() {
                            self.tab_bar.select(Tab::PatientWorkspace);
                        } else {
                            self.patient_list.reset_search();
                            self.tab_bar.select(Tab::PatientSearch);
                        }
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
                    if self.workspace_manager.select_by_index(n).is_ok() {
                        self.tab_bar.select(Tab::PatientWorkspace);
                    }
                    self.refresh_status_bar();
                    self.refresh_context();
                }

                Action::NextClinicalMenu => {
                    if let Some(workspace) = self.workspace_manager.active_mut() {
                        workspace.active_clinical_menu = workspace.active_clinical_menu.next();
                        self.sync_clinical_view_to_menu();
                    }
                    self.refresh_status_bar();
                }

                Action::PrevClinicalMenu => {
                    if let Some(workspace) = self.workspace_manager.active_mut() {
                        workspace.active_clinical_menu = workspace.active_clinical_menu.prev();
                        self.sync_clinical_view_to_menu();
                    }
                    self.refresh_status_bar();
                }

                Action::OpenPatientFromList => {
                    if self.tab_bar.selected() == Tab::PatientSearch && self.patient_form.is_none()
                    {
                        if let Some(patient) = self.patient_list.selected_patient().cloned() {
                            match self.open_patient_workspace(patient) {
                                Ok(_) => {}
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

        match self.tab_bar.selected() {
            Tab::PatientSearch => {
                if self.patient_form.is_none() {
                    return self.handle_patient_keys(key);
                }
            }
            Tab::PatientWorkspace => {
                if self.patient_form.is_none() {
                    if let Some(workspace) = self.workspace_manager.active() {
                        match workspace.active_subtab {
                            crate::ui::components::SubtabKind::Clinical => {
                                let action = self.handle_clinical_keys(key);
                                if action != Action::Unknown {
                                    return action;
                                }
                            }
                            crate::ui::components::SubtabKind::Billing => {
                                let action = self.handle_billing_keys(key);
                                if action != Action::Unknown {
                                    return action;
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            Tab::Schedule => {
                return self.handle_appointment_keys(key);
            }
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

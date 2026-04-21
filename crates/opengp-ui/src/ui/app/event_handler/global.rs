use crate::ui::app::App;
use crate::ui::components::appointment::AppointmentView;
use crate::ui::components::clinical_row::{ClinicalMenuKind, ClinicalRow};
use crate::ui::components::status_bar::STATUS_BAR_HEIGHT;
use crate::ui::components::tabs::Tab;
use crossterm::event::MouseEvent;
use ratatui::layout::{Constraint, Direction, Layout, Rect};

impl App {
    pub fn handle_global_mouse_event(&mut self, mouse: MouseEvent, area: Rect) {
        let main_tab_bar_area = Rect::new(area.x, area.y, area.width, 1);
        let patient_tab_bar_area = Rect::new(area.x, area.y + 1, area.width, 1);

        if self.tab_bar.handle_mouse(mouse, main_tab_bar_area).is_some() {
            self.refresh_status_bar();
            self.refresh_context();
            return;
        }

        if !self.workspace_manager.workspaces.is_empty() {
            if self.workspace_manager.handle_patient_tab_mouse(mouse, patient_tab_bar_area).is_some() {
                self.refresh_status_bar();
                self.refresh_context();
                return;
            }

            if let Some(workspace) = self.workspace_manager.active_mut() {
                let clinical_row_area = Rect::new(area.x, area.y + 2, area.width, 1);
                let clinical_items = ClinicalMenuKind::all();
                let active_clinical_idx = workspace.active_clinical_menu.index();
                let mut clinical_row = ClinicalRow::new(
                    clinical_items,
                    active_clinical_idx,
                    workspace.colour,
                    self.theme.clone(),
                );
                if let Some(idx) = clinical_row.handle_mouse(mouse, clinical_row_area) {
                    if let Some(kind) = ClinicalMenuKind::from_index(idx) {
                        workspace.active_clinical_menu = kind;
                        self.refresh_status_bar();
                        return;
                    }
                }
            }
        }

        if let Some(ref mut form) = self.patient_form {
            if let Some(action) = form.handle_mouse(mouse, area) {
                match action {
                    crate::ui::components::patient::PatientFormAction::FocusChanged => {}
                    crate::ui::components::patient::PatientFormAction::ValueChanged => {}
                    crate::ui::components::patient::PatientFormAction::Submit => {}
                    crate::ui::components::patient::PatientFormAction::Cancel => {}
                    crate::ui::components::patient::PatientFormAction::SaveComplete => {
                        self.request_refresh_patients();
                    }
                }
                return;
            }
        }

        let clinical_row_offset = if self.workspace_manager.active().is_some() { 3 } else { 2 };

        if self.tab_bar.selected() == Tab::PatientSearch && self.patient_form.is_none() {
            let content_area = Rect::new(
                area.x,
                area.y + clinical_row_offset,
                area.width,
                area.height.saturating_sub(clinical_row_offset + STATUS_BAR_HEIGHT),
            );
            if let Some(action) = self.patient_list.handle_mouse(mouse, content_area) {
                match action {
                    crate::ui::components::patient::PatientListAction::SelectionChanged => {}
                    crate::ui::components::patient::PatientListAction::OpenPatient(id) => {
                        self.request_edit_patient(id);
                    }
                    crate::ui::components::patient::PatientListAction::FocusSearch => {}
                    crate::ui::components::patient::PatientListAction::SearchChanged => {}
                    crate::ui::components::patient::PatientListAction::ContextMenu { x: _, y: _, patient_id: _ } => {
                    }
                }
            }
        }

        if self.tab_bar.selected() == Tab::Schedule {
            use crate::ui::components::appointment::schedule::ScheduleAction;

            let appointment_content_area = Rect::new(
                area.x,
                area.y + clinical_row_offset,
                area.width,
                area.height.saturating_sub(clinical_row_offset + STATUS_BAR_HEIGHT),
            );

            match self.appointment_state.current_view {
                AppointmentView::Calendar => {
                    self.appointment_state.calendar.focused = true;
                    self.appointment_state.focused = false;
                    if let Some(action) = self
                        .appointment_state
                        .calendar
                        .handle_mouse(mouse, appointment_content_area)
                    {
                        match action {
                            crate::ui::components::appointment::CalendarAction::SelectDate(
                                date,
                            ) => {
                                self.appointment_state.selected_date = Some(date);
                                self.appointment_state.current_view = AppointmentView::Schedule;
                                self.request_refresh_appointments(date);
                                self.refresh_context();
                            }
                            crate::ui::components::appointment::CalendarAction::FocusDate(_) => {}
                            crate::ui::components::appointment::CalendarAction::MonthChanged(_) => {
                            }
                            crate::ui::components::appointment::CalendarAction::GoToToday => {}
                        }
                    }
                }
                AppointmentView::Schedule => {
                    let chunks = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([Constraint::Percentage(25), Constraint::Percentage(75)])
                        .split(appointment_content_area);

                    use crossterm::event::MouseEventKind;
                    if let MouseEventKind::Up(_) | MouseEventKind::Down(_) = mouse.kind {
                        if let Some(action) = self
                            .appointment_state
                            .calendar
                            .handle_mouse(mouse, chunks[0])
                        {
                            self.appointment_state.calendar.focused = true;
                            self.appointment_state.focused = false;
                            match action {
                                crate::ui::components::appointment::CalendarAction::SelectDate(date) => {
                                    self.appointment_state.selected_date = Some(date);
                                    self.request_refresh_appointments(date);
                                }
                                crate::ui::components::appointment::CalendarAction::FocusDate(_) => {}
                                crate::ui::components::appointment::CalendarAction::MonthChanged(_) => {}
                                crate::ui::components::appointment::CalendarAction::GoToToday => {}
                            }
                        }
                    }

                    if let Some(action) = self.appointment_state.handle_mouse(mouse, chunks[1]) {
                        self.appointment_state.focused = true;
                        self.appointment_state.calendar.focused = false;
                        match action {
                            ScheduleAction::SelectPractitioner(id) => {
                                self.appointment_state.selected_practitioner = Some(id);
                            }
                            ScheduleAction::SelectAppointment(id) => {
                                self.appointment_state.selected_appointment = Some(id);
                            }
                            ScheduleAction::NavigateTimeSlot(_) => {}
                            ScheduleAction::NavigatePractitioner(_) => {}
                            ScheduleAction::ToggleColumn => {}
                            ScheduleAction::CreateAtSlot { .. } => {}
                        }
                    }
                }
            }
        }


    }
}

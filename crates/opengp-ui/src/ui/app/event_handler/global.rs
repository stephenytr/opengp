use crate::ui::app::App;
use crate::ui::components::appointment::AppointmentView;
use crate::ui::components::status_bar::STATUS_BAR_HEIGHT;
use crate::ui::components::tabs::Tab;
use crossterm::event::MouseEvent;
use ratatui::layout::{Constraint, Direction, Layout, Rect};

impl App {
    pub fn handle_global_mouse_event(&mut self, mouse: MouseEvent, area: Rect) {
        let tab_bar_area = self.tab_bar.area(area);
        if self.tab_bar.handle_mouse(mouse, tab_bar_area).is_some() {
            self.refresh_status_bar();
            self.refresh_context();
            return;
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

        if self.tab_bar.selected() == Tab::Patient && self.patient_form.is_none() {
            let content_area = Rect::new(
                area.x,
                area.y + 2,
                area.width,
                area.height.saturating_sub(2 + STATUS_BAR_HEIGHT),
            );
            if let Some(action) = self.patient_list.handle_mouse(mouse, content_area) {
                match action {
                    crate::ui::components::patient::PatientListAction::SelectionChanged => {}
                    crate::ui::components::patient::PatientListAction::OpenPatient(id) => {
                        self.request_edit_patient(id);
                    }
                    crate::ui::components::patient::PatientListAction::FocusSearch => {}
                    crate::ui::components::patient::PatientListAction::SearchChanged => {}
                }
            }
        }

        if self.tab_bar.selected() == Tab::Appointment {
            use crate::ui::components::appointment::schedule::ScheduleAction;

            let appointment_content_area = Rect::new(
                area.x,
                area.y + 2,
                area.width,
                area.height.saturating_sub(2 + STATUS_BAR_HEIGHT),
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
                                self.pending_appointment_date = Some(date);
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
                                    self.pending_appointment_date = Some(date);
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

        if self.tab_bar.selected() == Tab::Clinical && !self.clinical_state.is_form_open() {
            use crate::ui::components::clinical::ClinicalView;
            let clinical_area = Rect::new(
                area.x,
                area.y + 2,
                area.width,
                area.height.saturating_sub(2 + STATUS_BAR_HEIGHT),
            );
            match self.clinical_state.view {
                ClinicalView::Consultations => {
                    let _ = self
                        .clinical_state
                        .consultation_list
                        .handle_mouse(mouse, clinical_area);
                }
                ClinicalView::Allergies => {
                    let _ = self
                        .clinical_state
                        .allergy_list
                        .handle_mouse(mouse, clinical_area);
                }
                ClinicalView::MedicalHistory => {
                    let _ = self
                        .clinical_state
                        .medical_history_list
                        .handle_mouse(mouse, clinical_area);
                }
                ClinicalView::VitalSigns => {
                    let _ = self
                        .clinical_state
                        .vitals_list
                        .handle_mouse(mouse, clinical_area);
                }
                ClinicalView::FamilyHistory => {
                    let _ = self
                        .clinical_state
                        .family_history_list
                        .handle_mouse(mouse, clinical_area);
                }
                ClinicalView::PatientSummary
                | ClinicalView::ConsultationSummary
                | ClinicalView::SocialHistory => {}
            }
        }
    }
}

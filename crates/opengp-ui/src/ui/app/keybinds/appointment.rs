use crate::ui::app::App;
use crate::ui::components::appointment::{
    AppointmentForm, AppointmentFormField, AppointmentView, CalendarAction, ScheduleAction,
};
use crate::ui::keybinds::{Action, KeyContext, KeybindRegistry};
use crate::ui::widgets::format_date;
use crossterm::event::KeyEvent;

impl App {
    pub(crate) fn handle_appointment_keys(&mut self, key: KeyEvent) -> Action {
        let registry = KeybindRegistry::global();
        if let Some(keybind) = registry.lookup(key, KeyContext::Schedule) {
            match keybind.action {
                Action::PrevMonth | Action::NextMonth | Action::Today => {
                    if let Some(action) = self.appointment_state.calendar.handle_key(key) {
                        match action {
                            CalendarAction::MonthChanged(date) => {
                                self.appointment_state.selected_date = Some(date);
                                self.refresh_context();
                            }
                            CalendarAction::GoToToday => {
                                self.refresh_context();
                            }
                            _ => {}
                        }
                        return keybind.action.clone();
                    }
                }
                _ => {}
            }
        }

        if self.appointment_state.current_view == AppointmentView::Calendar {
            if let Some(action) = self.appointment_state.calendar.handle_key(key) {
                match action {
                    CalendarAction::SelectDate(date) => {
                        self.appointment_state.selected_date = Some(date);
                        self.appointment_state.current_view = AppointmentView::Schedule;
                        self.appointment_state.focused = true;
                        self.appointment_state.calendar.focused = false;
                        self.pending_appointment_date = Some(date);
                        self.refresh_context();
                    }
                    CalendarAction::FocusDate(_) => {}
                    CalendarAction::MonthChanged(_) => {}
                    CalendarAction::GoToToday => {}
                }
                return Action::Enter;
            }
        }

        if self.appointment_state.current_view == AppointmentView::Schedule {
            if let Some(action) = self.appointment_state.handle_key(key) {
                match action {
                    ScheduleAction::SelectPractitioner(id) => {
                        self.appointment_state.selected_practitioner = Some(id);
                    }
                    ScheduleAction::SelectAppointment(id) => {
                        if let Some(ref schedule_data) = self.appointment_state.schedule_data {
                            for practitioner in &schedule_data.practitioners {
                                if let Some(appointment) =
                                    practitioner.appointments.iter().find(|apt| apt.id == id)
                                {
                                    self.appointment_detail_modal = Some(
                                        crate::ui::components::appointment::AppointmentDetailModal::new(
                                            appointment.clone(),
                                            self.theme.clone(),
                                        ),
                                    );
                                    self.refresh_context();
                                    break;
                                }
                            }
                        }
                        self.appointment_state.selected_appointment = Some(id);
                    }
                    ScheduleAction::NavigateTimeSlot(_delta) => {}
                    ScheduleAction::NavigatePractitioner(_delta) => {}
                    ScheduleAction::ToggleColumn => {}
                    ScheduleAction::CreateAtSlot {
                        practitioner_id,
                        date,
                        time,
                    } => {
                        self.appointment_form = Some(AppointmentForm::new(
                            self.theme.clone(),
                            self.healthcare_config.clone(),
                        ));
                        if let Some(ref mut form) = self.appointment_form {
                            if let Some(ref schedule_data) = self.appointment_state.schedule_data {
                                if let Some(practitioner) = schedule_data
                                    .practitioners
                                    .iter()
                                    .find(|p| p.practitioner_id == practitioner_id)
                                {
                                    form.set_practitioner(
                                        practitioner_id,
                                        practitioner.practitioner_name.clone(),
                                    );
                                }
                            }
                            form.set_value(AppointmentFormField::Date, format_date(date));
                            form.set_value(AppointmentFormField::StartTime, time);
                        }
                        self.request_load_practitioners();
                    }
                }
                return Action::Enter;
            }
        }

        Action::Unknown
    }
}

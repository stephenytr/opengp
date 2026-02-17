use crossterm::event::KeyEvent;

use crate::components::Action;
use crate::ui::keybinds::{KeybindContext, KeybindRegistry};
use crate::ui::App;

pub struct EventDispatcher;

impl EventDispatcher {
    pub fn get_context(app: &App) -> KeybindContext {
        if app.show_patient_form {
            return KeybindContext::PatientForm;
        }

        match app.active_screen {
            crate::ui::app::Screen::Patients => {
                if app.patient_list.is_search_mode() {
                    KeybindContext::PatientListSearch
                } else {
                    KeybindContext::PatientList
                }
            }
            crate::ui::app::Screen::Appointments => KeybindContext::CalendarDayView,
            crate::ui::app::Screen::Clinical => KeybindContext::Global,
            crate::ui::app::Screen::Billing => KeybindContext::Global,
        }
    }

    pub fn dispatch(app: &mut App, key: KeyEvent) {
        let context = Self::get_context(app);

        if let Some(action) = crate::ui::key_dispatcher::KeyDispatcher::dispatch(&context, key) {
            Self::execute_action(app, action);
            return;
        }

        if let Some(action) =
            crate::ui::key_dispatcher::KeyDispatcher::dispatch(&KeybindContext::Global, key)
        {
            Self::execute_action(app, action);
        }
    }

    fn execute_action(app: &mut App, action: Action) {
        match action {
            Action::None => {}
            Action::Tick => {}
            Action::Render => {}
            Action::Quit => {
                app.should_quit = true;
            }
            Action::NavigateToPatients => {
                app.active_screen = crate::ui::app::Screen::Patients;
                app.update_focus();
            }
            Action::NavigateToAppointments => {
                app.active_screen = crate::ui::app::Screen::Appointments;
                app.update_focus();
            }
            Action::NavigateToBilling => {
                app.active_screen = crate::ui::app::Screen::Billing;
                app.update_focus();
            }
            Action::PatientCreate => {
                app.handle_patient_create();
            }
            Action::PatientEdit(id) => {
                app.handle_patient_edit(id);
            }
            Action::PatientFormSubmit => {
                app.handle_patient_form_submit();
            }
            Action::PatientFormCancel => {
                app.handle_patient_form_cancel();
            }
            Action::AppointmentCreate => {}
            Action::AppointmentFormSubmit => {}
            Action::AppointmentFormCancel => {}
            Action::AppointmentMarkCompleted => {}
            Action::AppointmentMarkNoShow => {}
            Action::AppointmentReschedule => {}
            Action::AppointmentBatchMarkArrived => {}
            Action::AppointmentBatchMarkCompleted => {}
        }
    }

    pub fn handle_navigation_key(app: &mut App, key: KeyEvent) {
        use crossterm::event::KeyCode;

        match key.code {
            KeyCode::Tab => {
                app.tabs.next();
                app.active_screen = crate::ui::app::Screen::from_index(app.tabs.selected());
            }
            KeyCode::BackTab => {
                app.tabs.previous();
                app.active_screen = crate::ui::app::Screen::from_index(app.tabs.selected());
            }
            KeyCode::Right => {
                app.tabs.next();
                app.active_screen = crate::ui::app::Screen::from_index(app.tabs.selected());
            }
            KeyCode::Left => {
                app.tabs.previous();
                app.active_screen = crate::ui::app::Screen::from_index(app.tabs.selected());
            }
            KeyCode::Char('1') => {
                app.tabs.select(0);
                app.active_screen = crate::ui::app::Screen::Patients;
            }
            KeyCode::Char('2') => {
                app.tabs.select(1);
                app.active_screen = crate::ui::app::Screen::Appointments;
            }
            KeyCode::Char('3') => {
                app.tabs.select(2);
                app.active_screen = crate::ui::app::Screen::Clinical;
            }
            KeyCode::Char('4') => {
                app.tabs.select(3);
                app.active_screen = crate::ui::app::Screen::Billing;
            }
            KeyCode::Char('q') => {
                app.should_quit = true;
            }
            _ => {
                Self::dispatch(app, key);
                return;
            }
        }
        app.update_focus();
    }
}

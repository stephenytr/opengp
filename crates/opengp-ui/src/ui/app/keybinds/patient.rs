use crate::ui::app::App;
use crate::ui::keybinds::{Action, KeyContext};
use crossterm::event::KeyEvent;

impl App {
    pub(crate) fn handle_patient_keys(&mut self, key: KeyEvent) -> Action {
        if let Some(action) = self.patient_list.handle_key(key) {
            match action {
                crate::ui::components::patient::PatientListAction::SelectionChanged => {
                    let visible_rows = self.calculate_visible_patient_rows();
                    self.patient_list.adjust_scroll(visible_rows);
                }
                crate::ui::components::patient::PatientListAction::OpenPatient(_id) => {
                    todo!("Patient clinical detail workflow moved to workspace subtab in Task 28")
                }
                crate::ui::components::patient::PatientListAction::FocusSearch => {
                    self.current_context = KeyContext::Search;
                }
                crate::ui::components::patient::PatientListAction::SearchChanged => {
                    if !self.patient_list.is_searching() {
                        self.current_context = KeyContext::PatientList;
                    }
                }
            }
            return Action::Enter;
        }
        Action::Unknown
    }
}

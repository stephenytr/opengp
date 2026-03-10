use crate::ui::app::App;
use crate::ui::components::tabs::Tab;
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
                crate::ui::components::patient::PatientListAction::OpenPatient(id) => {
                    self.clinical_state.clear_patient();
                    self.clinical_state.set_patient(id);
                    self.clinical_state.show_patient_summary();
                    self.tab_bar.select(Tab::Clinical);
                    self.pending_clinical_patient_id = Some(id);
                    self.refresh_status_bar();
                    self.refresh_context();
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

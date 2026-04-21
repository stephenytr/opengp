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
                crate::ui::components::patient::PatientListAction::OpenPatient(id) => {
                    if let Some(patient_item) = self.patient_list.get_patient_by_id(id) {
                        match self.workspace_manager.open_patient(patient_item.clone()) {
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
                crate::ui::components::patient::PatientListAction::FocusSearch => {
                    self.current_context = KeyContext::Search;
                }
                crate::ui::components::patient::PatientListAction::SearchChanged => {
                    if !self.patient_list.is_searching() {
                        self.current_context = KeyContext::PatientList;
                    }
                }
                crate::ui::components::patient::PatientListAction::ContextMenu { x: _, y: _, patient_id: _ } => {
                    // Context menu support to be implemented in future
                }
            }
            return Action::Enter;
        }
        Action::Unknown
    }
}

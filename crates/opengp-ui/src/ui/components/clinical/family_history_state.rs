//! Family History State Component
//!
//! Manages family-history-specific state for the clinical UI.
//! Extracted from ClinicalState to handle family history list, form, and detail modal independently.

use crate::ui::theme::Theme;
use opengp_domain::domain::clinical::FamilyHistory;

use crate::ui::components::clinical::{FamilyHistoryDetailModal, FamilyHistoryForm, FamilyHistoryList};

/// Family history state management component
///
/// Encapsulates family-history-specific state:
/// - family_history_list: The list widget state and data
/// - family_history_form: Optional form for creating/editing family history
/// - family_history_detail_modal: Optional detail modal for viewing family history
/// - family_history: Raw family history data
/// - loading: Loading indicator
/// - error: Error message if any
#[derive(Clone)]
pub struct FamilyHistoryState {
    pub family_history_list: FamilyHistoryList,
    pub family_history_form: Option<FamilyHistoryForm>,
    pub family_history_detail_modal: Option<FamilyHistoryDetailModal>,
    pub family_history: Vec<FamilyHistory>,
    pub loading: bool,
    pub error: Option<String>,
    theme: Theme,
}

impl FamilyHistoryState {
    /// Create a new FamilyHistoryState with the given theme.
    pub fn new(theme: Theme) -> Self {
        Self {
            family_history_list: FamilyHistoryList::new(theme.clone()),
            family_history_form: None,
            family_history_detail_modal: None,
            family_history: Vec::new(),
            loading: false,
            error: None,
            theme,
        }
    }

    /// Open the family history form for creating/editing.
    pub fn open_family_history_form(&mut self) {
        self.family_history_form = Some(FamilyHistoryForm::new(self.theme.clone()));
    }

    /// Close the family history form.
    pub fn close_family_history_form(&mut self) {
        self.family_history_form = None;
    }

    /// Open the family history detail modal with the given family history.
    pub fn open_family_history_detail(&mut self, family_history: FamilyHistory) {
        self.family_history_detail_modal =
            Some(FamilyHistoryDetailModal::new(family_history, self.theme.clone()));
    }

    /// Close the family history detail modal.
    pub fn close_family_history_detail(&mut self) {
        self.family_history_detail_modal = None;
    }

    /// Check if the family history form is open.
    pub fn is_form_open(&self) -> bool {
        self.family_history_form.is_some()
    }

    /// Check if the family history detail modal is open.
    pub fn is_detail_modal_open(&self) -> bool {
        self.family_history_detail_modal.is_some()
    }

    /// Set the loading state.
    pub fn set_loading(&mut self, loading: bool) {
        self.loading = loading;
    }

    /// Set the error message.
    pub fn set_error(&mut self, error: Option<String>) {
        self.error = error;
    }

    /// Clear the error message.
    pub fn clear_error(&mut self) {
        self.error = None;
    }

    /// Check if a patient is selected (from parent state context).
    /// This is a helper; actual patient selection is managed by ClinicalState.
    pub fn has_patient(&self) -> bool {
        !self.family_history.is_empty() || self.family_history_form.is_some()
    }

    /// Clear all family history state.
    pub fn clear(&mut self) {
        self.family_history.clear();
        self.family_history_list.move_first();
        self.family_history_form = None;
        self.family_history_detail_modal = None;
        self.loading = false;
        self.error = None;
    }

    /// Get the currently selected family history, if any.
    pub fn get_selected(&self) -> Option<&FamilyHistory> {
        if self.family_history_list.selected_index < self.family_history.len() {
            Some(&self.family_history[self.family_history_list.selected_index])
        } else {
            None
        }
    }

    /// Add a family history entry to the list.
    pub fn add_family_history(&mut self, family_history: FamilyHistory) {
        self.family_history.push(family_history);
    }

    /// Remove a family history entry by ID.
    pub fn remove_family_history(&mut self, id: uuid::Uuid) {
        self.family_history.retain(|fh| fh.id != id);
        if self.family_history_list.selected_index >= self.family_history.len()
            && !self.family_history.is_empty()
        {
            self.family_history_list.selected_index = self.family_history.len() - 1;
        }
    }

    /// Navigate to the next family history entry in the list.
    pub fn next_item(&mut self) {
        self.family_history_list.next();
    }

    /// Navigate to the previous family history entry in the list.
    pub fn prev_item(&mut self) {
        self.family_history_list.prev();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::theme::Theme;
    use opengp_domain::domain::clinical::FamilyHistory;
    use uuid::Uuid;

    fn create_test_theme() -> Theme {
        Theme::default()
    }

    fn create_test_family_history() -> FamilyHistory {
        FamilyHistory {
            id: Uuid::new_v4(),
            patient_id: Uuid::new_v4(),
            relative_relationship: "Mother".to_string(),
            condition: "Diabetes".to_string(),
            age_at_diagnosis: Some(45),
            notes: Some("Type 2 diabetes".to_string()),
            created_at: chrono::Utc::now(),
            created_by: Uuid::new_v4(),
        }
    }

    #[test]
    fn test_family_history_state_construction() {
        let theme = create_test_theme();
        let state = FamilyHistoryState::new(theme);

        assert!(state.family_history.is_empty());
        assert!(state.family_history_form.is_none());
        assert!(state.family_history_detail_modal.is_none());
        assert!(!state.loading);
        assert!(state.error.is_none());
    }

    #[test]
    fn test_add_family_history() {
        let theme = create_test_theme();
        let mut state = FamilyHistoryState::new(theme);

        let fh = create_test_family_history();
        state.add_family_history(fh.clone());

        assert_eq!(state.family_history.len(), 1);
        assert_eq!(state.family_history[0].id, fh.id);
    }

    #[test]
    fn test_remove_family_history() {
        let theme = create_test_theme();
        let mut state = FamilyHistoryState::new(theme);

        let fh1 = create_test_family_history();
        let fh2 = create_test_family_history();

        state.add_family_history(fh1.clone());
        state.add_family_history(fh2.clone());

        assert_eq!(state.family_history.len(), 2);

        state.remove_family_history(fh1.id);

        assert_eq!(state.family_history.len(), 1);
        assert_eq!(state.family_history[0].id, fh2.id);
    }

    #[test]
    fn test_get_selected() {
        let theme = create_test_theme();
        let mut state = FamilyHistoryState::new(theme);

        let fh = create_test_family_history();
        state.add_family_history(fh.clone());

        let selected = state.get_selected();
        assert!(selected.is_some());
        assert_eq!(selected.unwrap().id, fh.id);
    }

    #[test]
    fn test_open_close_form() {
        let theme = create_test_theme();
        let mut state = FamilyHistoryState::new(theme);

        assert!(!state.is_form_open());

        state.open_family_history_form();
        assert!(state.is_form_open());

        state.close_family_history_form();
        assert!(!state.is_form_open());
    }

    #[test]
    fn test_open_close_detail_modal() {
        let theme = create_test_theme();
        let mut state = FamilyHistoryState::new(theme);

        let fh = create_test_family_history();

        assert!(!state.is_detail_modal_open());

        state.open_family_history_detail(fh.clone());
        assert!(state.is_detail_modal_open());

        state.close_family_history_detail();
        assert!(!state.is_detail_modal_open());
    }

    #[test]
    fn test_clear_state() {
        let theme = create_test_theme();
        let mut state = FamilyHistoryState::new(theme);

        let fh = create_test_family_history();
        state.add_family_history(fh);
        state.open_family_history_form();
        state.set_loading(true);
        state.set_error(Some("Test error".to_string()));

        state.clear();

        assert!(state.family_history.is_empty());
        assert!(!state.is_form_open());
        assert!(!state.loading);
        assert!(state.error.is_none());
    }

    #[test]
    fn test_next_prev_navigation() {
        let theme = create_test_theme();
        let mut state = FamilyHistoryState::new(theme);

        let fh1 = create_test_family_history();
        let fh2 = create_test_family_history();

        state.add_family_history(fh1);
        state.add_family_history(fh2);

        // Simulate renderer populating the list entries
        state.family_history_list.entries = state.family_history.clone();

        assert_eq!(state.family_history_list.selected_index, 0);

        state.next_item();
        assert_eq!(state.family_history_list.selected_index, 1);

        state.prev_item();
        assert_eq!(state.family_history_list.selected_index, 0);
    }

    #[test]
    fn test_set_loading() {
        let theme = create_test_theme();
        let mut state = FamilyHistoryState::new(theme);

        assert!(!state.loading);

        state.set_loading(true);
        assert!(state.loading);

        state.set_loading(false);
        assert!(!state.loading);
    }

    #[test]
    fn test_set_error() {
        let theme = create_test_theme();
        let mut state = FamilyHistoryState::new(theme);

        assert!(state.error.is_none());

        state.set_error(Some("Test error".to_string()));
        assert_eq!(state.error, Some("Test error".to_string()));

        state.clear_error();
        assert!(state.error.is_none());
    }

    #[test]
    fn test_has_patient_empty() {
        let state = FamilyHistoryState::new(create_test_theme());
        assert!(!state.has_patient());
    }

    #[test]
    fn test_has_patient_with_data() {
        let mut state = FamilyHistoryState::new(create_test_theme());
        state.add_family_history(create_test_family_history());
        assert!(state.has_patient());
    }

    #[test]
    fn test_has_patient_with_form_open() {
        let mut state = FamilyHistoryState::new(create_test_theme());
        state.open_family_history_form();
        assert!(state.has_patient());
    }
}

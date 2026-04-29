//! Allergy State Component
//!
//! Manages allergy-specific state for the clinical UI.
//! Extracted from ClinicalState to handle allergy list, form, and detail modal independently.

use crate::ui::theme::Theme;
use opengp_config::AllergyConfig;
use opengp_domain::domain::clinical::Allergy;

use crate::ui::components::clinical::{AllergyDetailModal, AllergyForm, AllergyList};

/// Allergy state management component
///
/// Encapsulates allergy-specific state:
/// - allergy_list: The list widget state and data
/// - allergy_form: Optional form for creating/editing allergies
/// - allergy_detail_modal: Optional detail modal for viewing allergies
/// - allergies: Raw allergy data
/// - loading: Loading indicator
/// - error: Error message if any
#[derive(Clone)]
pub struct AllergyState {
    pub allergy_list: AllergyList,
    pub allergy_form: Option<AllergyForm>,
    pub allergy_detail_modal: Option<AllergyDetailModal>,
    pub allergy_config: AllergyConfig,
    pub allergies: Vec<Allergy>,
    pub loading: bool,
    pub error: Option<String>,
    theme: Theme,
}

impl AllergyState {
    /// Create a new AllergyState with the given theme and config.
    pub fn new(theme: Theme, allergy_config: AllergyConfig) -> Self {
        Self {
            allergy_list: AllergyList::new(theme.clone()),
            allergy_form: None,
            allergy_detail_modal: None,
            allergy_config,
            allergies: Vec::new(),
            loading: false,
            error: None,
            theme,
        }
    }

    /// Open the allergy form for creating/editing.
    pub fn open_allergy_form(&mut self) {
        self.allergy_form = Some(AllergyForm::new(self.theme.clone(), &self.allergy_config));
    }

    /// Close the allergy form.
    pub fn close_allergy_form(&mut self) {
        self.allergy_form = None;
    }

    /// Open the allergy detail modal with the given allergy.
    pub fn open_allergy_detail(&mut self, allergy: Allergy) {
        self.allergy_detail_modal = Some(AllergyDetailModal::new(allergy, self.theme.clone()));
    }

    /// Close the allergy detail modal.
    pub fn close_allergy_detail(&mut self) {
        self.allergy_detail_modal = None;
    }

    /// Check if the allergy form is open.
    pub fn is_form_open(&self) -> bool {
        self.allergy_form.is_some()
    }

    /// Check if the allergy detail modal is open.
    pub fn is_detail_modal_open(&self) -> bool {
        self.allergy_detail_modal.is_some()
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
        !self.allergies.is_empty() || self.allergy_form.is_some()
    }

    /// Clear all allergy state.
    pub fn clear(&mut self) {
        self.allergies.clear();
        self.allergy_list.move_first();
        self.allergy_form = None;
        self.allergy_detail_modal = None;
        self.loading = false;
        self.error = None;
    }

    /// Get the currently selected allergy, if any.
    pub fn get_selected(&self) -> Option<&Allergy> {
        if self.allergy_list.selected_index < self.allergies.len() {
            Some(&self.allergies[self.allergy_list.selected_index])
        } else {
            None
        }
    }

    /// Add an allergy to the list.
    pub fn add_allergy(&mut self, allergy: Allergy) {
        self.allergies.push(allergy);
    }

    /// Remove an allergy by ID.
    pub fn remove_allergy(&mut self, id: uuid::Uuid) {
        self.allergies.retain(|a| a.id != id);
        if self.allergy_list.selected_index >= self.allergies.len() && !self.allergies.is_empty() {
            self.allergy_list.selected_index = self.allergies.len() - 1;
        }
    }

    /// Navigate to the next allergy in the list.
    pub fn next_item(&mut self) {
        self.allergy_list.next();
    }

    /// Navigate to the previous allergy in the list.
    pub fn prev_item(&mut self) {
        self.allergy_list.prev();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::theme::Theme;
    use opengp_config::AllergyConfig;
    use opengp_domain::domain::clinical::{AllergyType, Severity};
    use uuid::Uuid;

    fn create_test_theme() -> Theme {
        Theme::default()
    }

    fn create_test_config() -> AllergyConfig {
        AllergyConfig::default()
    }

    fn create_test_allergy() -> Allergy {
        Allergy {
            id: Uuid::new_v4(),
            patient_id: Uuid::new_v4(),
            allergen: "Penicillin".to_string(),
            allergy_type: AllergyType::Drug,
            severity: Severity::Severe,
            reaction: Some("Anaphylaxis".to_string()),
            onset_date: None,
            notes: None,
            is_active: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            created_by: Uuid::new_v4(),
            updated_by: Some(Uuid::new_v4()),
        }
    }

    #[test]
    fn test_allergy_state_construction() {
        let theme = create_test_theme();
        let config = create_test_config();

        let state = AllergyState::new(theme, config);

        assert!(state.allergies.is_empty());
        assert!(state.allergy_form.is_none());
        assert!(state.allergy_detail_modal.is_none());
        assert!(!state.loading);
        assert!(state.error.is_none());
    }

    #[test]
    fn test_add_allergy() {
        let theme = create_test_theme();
        let config = create_test_config();
        let mut state = AllergyState::new(theme, config);

        let allergy = create_test_allergy();
        state.add_allergy(allergy.clone());

        assert_eq!(state.allergies.len(), 1);
        assert_eq!(state.allergies[0].id, allergy.id);
    }

    #[test]
    fn test_remove_allergy() {
        let theme = create_test_theme();
        let config = create_test_config();
        let mut state = AllergyState::new(theme, config);

        let allergy1 = create_test_allergy();
        let allergy2 = create_test_allergy();

        state.add_allergy(allergy1.clone());
        state.add_allergy(allergy2.clone());

        assert_eq!(state.allergies.len(), 2);

        state.remove_allergy(allergy1.id);

        assert_eq!(state.allergies.len(), 1);
        assert_eq!(state.allergies[0].id, allergy2.id);
    }

    #[test]
    fn test_get_selected() {
        let theme = create_test_theme();
        let config = create_test_config();
        let mut state = AllergyState::new(theme, config);

        let allergy = create_test_allergy();
        state.add_allergy(allergy.clone());

        let selected = state.get_selected();
        assert!(selected.is_some());
        assert_eq!(selected.unwrap().id, allergy.id);
    }

    #[test]
    fn test_open_close_form() {
        let theme = create_test_theme();
        let config = create_test_config();
        let mut state = AllergyState::new(theme, config);

        assert!(!state.is_form_open());

        state.open_allergy_form();
        assert!(state.is_form_open());

        state.close_allergy_form();
        assert!(!state.is_form_open());
    }

    #[test]
    fn test_open_close_detail_modal() {
        let theme = create_test_theme();
        let config = create_test_config();
        let mut state = AllergyState::new(theme, config);

        let allergy = create_test_allergy();

        assert!(!state.is_detail_modal_open());

        state.open_allergy_detail(allergy.clone());
        assert!(state.is_detail_modal_open());

        state.close_allergy_detail();
        assert!(!state.is_detail_modal_open());
    }

    #[test]
    fn test_clear_state() {
        let theme = create_test_theme();
        let config = create_test_config();
        let mut state = AllergyState::new(theme, config);

        let allergy = create_test_allergy();
        state.add_allergy(allergy);
        state.open_allergy_form();
        state.set_loading(true);
        state.set_error(Some("Test error".to_string()));

        state.clear();

        assert!(state.allergies.is_empty());
        assert!(!state.is_form_open());
        assert!(!state.loading);
        assert!(state.error.is_none());
    }

    #[test]
    fn test_next_prev_navigation() {
        let theme = create_test_theme();
        let config = create_test_config();
        let mut state = AllergyState::new(theme, config);

        let allergy1 = create_test_allergy();
        let allergy2 = create_test_allergy();

        state.add_allergy(allergy1);
        state.add_allergy(allergy2);

        // Manually sync the list widget with the state data (renderer does this at render time)
        state.allergy_list.allergies = state.allergies.clone();

        assert_eq!(state.allergy_list.selected_index, 0);

        state.next_item();
        assert_eq!(state.allergy_list.selected_index, 1);

        state.prev_item();
        assert_eq!(state.allergy_list.selected_index, 0);
    }

    #[test]
    fn test_has_patient_empty() {
        let state = AllergyState::new(create_test_theme(), create_test_config());
        assert!(!state.has_patient());
    }

    #[test]
    fn test_has_patient_with_data() {
        let mut state = AllergyState::new(create_test_theme(), create_test_config());
        state.add_allergy(create_test_allergy());
        assert!(state.has_patient());
    }

    #[test]
    fn test_has_patient_with_form_open() {
        let mut state = AllergyState::new(create_test_theme(), create_test_config());
        state.open_allergy_form();
        assert!(state.has_patient());
    }
}

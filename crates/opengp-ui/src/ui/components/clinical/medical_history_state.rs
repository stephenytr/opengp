//! Medical History State
//!
//! Extracted state management for medical history operations.
//! Manages medical history list navigation, form lifecycle, and detail modal.

use crate::ui::theme::Theme;
use opengp_config::ClinicalConfig;
use opengp_domain::domain::clinical::MedicalHistory;

use super::{MedicalHistoryDetailModal, MedicalHistoryForm, MedicalHistoryList};

/// Medical History state management.
///
/// Encapsulates:
/// - Medical history list and navigation
/// - Medical history form (create/edit)
/// - Detail modal for read-only viewing
/// - Clinical configuration
#[derive(Clone)]
pub struct MedicalHistoryState {
    /// List widget with navigation state
    pub medical_history_list: MedicalHistoryList,
    /// Form for creating/editing medical history (None when closed)
    pub medical_history_form: Option<MedicalHistoryForm>,
    /// Detail modal for read-only display (None when closed)
    pub medical_history_detail_modal: Option<MedicalHistoryDetailModal>,
    /// Clinical configuration
    pub clinical_config: ClinicalConfig,
    /// All medical history records for current patient
    pub medical_history: Vec<MedicalHistory>,
    /// Loading indicator
    pub loading: bool,
    /// Error message, if any
    pub error: Option<String>,
    /// Theme for rendering components
    theme: Theme,
}

impl MedicalHistoryState {
    /// Create a new MedicalHistoryState.
    pub fn new(theme: Theme, clinical_config: ClinicalConfig) -> Self {
        Self {
            medical_history_list: MedicalHistoryList::new(theme.clone()),
            medical_history_form: None,
            medical_history_detail_modal: None,
            clinical_config,
            medical_history: Vec::new(),
            loading: false,
            error: None,
            theme,
        }
    }

    /// Open the medical history form for creating a new record.
    pub fn open_medical_history_form(&mut self) {
        self.medical_history_form = Some(MedicalHistoryForm::new(
            &self.clinical_config,
            self.theme.clone(),
        ));
    }

    /// Close the medical history form.
    pub fn close_medical_history_form(&mut self) {
        self.medical_history_form = None;
    }

    /// Open the medical history detail modal.
    pub fn open_medical_history_detail(&mut self, medical_history: MedicalHistory) {
        self.medical_history_detail_modal = Some(MedicalHistoryDetailModal::new(
            medical_history,
            self.theme.clone(),
        ));
    }

    /// Close the medical history detail modal.
    pub fn close_medical_history_detail(&mut self) {
        self.medical_history_detail_modal = None;
    }

    /// Set loading state.
    pub fn set_loading(&mut self, loading: bool) {
        self.loading = loading;
    }

    /// Set error state.
    pub fn set_error(&mut self, error: Option<String>) {
        self.error = error;
    }

    /// Clear error state.
    pub fn clear_error(&mut self) {
        self.error = None;
    }

    /// Clear all state (when patient changes or disconnects).
    pub fn clear(&mut self) {
        self.medical_history.clear();
        self.medical_history_list = MedicalHistoryList::new(self.theme.clone());
        self.medical_history_form = None;
        self.medical_history_detail_modal = None;
        self.error = None;
        self.loading = false;
    }

    /// Check if we have an active patient.
    pub fn has_patient(&self) -> bool {
        !self.medical_history.is_empty() || self.medical_history_form.is_some()
    }

    /// Check if the form is open.
    pub fn is_form_open(&self) -> bool {
        self.medical_history_form.is_some()
    }

    /// Move to the next item in the list.
    pub fn next_item(&mut self) {
        self.medical_history_list.selected_index =
            (self.medical_history_list.selected_index + 1)
                .min(self.medical_history.len().saturating_sub(1));
    }

    /// Move to the previous item in the list.
    pub fn prev_item(&mut self) {
        self.medical_history_list.selected_index = self.medical_history_list.selected_index.saturating_sub(1);
    }


}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::theme::Theme;
    use opengp_config::ClinicalConfig;

    fn create_test_state() -> MedicalHistoryState {
        let theme = Theme::default();
        let clinical_config = ClinicalConfig::default();
        MedicalHistoryState::new(theme, clinical_config)
    }

    #[test]
    fn test_medical_history_state_construction() {
        let state = create_test_state();
        assert!(state.medical_history.is_empty());
        assert!(state.medical_history_form.is_none());
        assert!(state.medical_history_detail_modal.is_none());
        assert!(!state.loading);
        assert!(state.error.is_none());
    }

    #[test]
    fn test_open_and_close_medical_history_form() {
        let mut state = create_test_state();
        assert!(state.medical_history_form.is_none());
        assert!(!state.is_form_open());

        state.open_medical_history_form();
        assert!(state.medical_history_form.is_some());
        assert!(state.is_form_open());

        state.close_medical_history_form();
        assert!(state.medical_history_form.is_none());
        assert!(!state.is_form_open());
    }

    #[test]
    fn test_open_and_close_medical_history_detail_modal() {
        let mut state = create_test_state();
        let medical_history = MedicalHistory {
            id: uuid::Uuid::new_v4(),
            patient_id: uuid::Uuid::new_v4(),
            condition: "Hypertension".to_string(),
            diagnosis_date: None,
            status: opengp_domain::domain::clinical::ConditionStatus::Active,
            severity: None,
            notes: None,
            is_active: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            created_by: uuid::Uuid::new_v4(),
            updated_by: None,
        };

        assert!(state.medical_history_detail_modal.is_none());

        state.open_medical_history_detail(medical_history.clone());
        assert!(state.medical_history_detail_modal.is_some());

        state.close_medical_history_detail();
        assert!(state.medical_history_detail_modal.is_none());
    }

    #[test]
    fn test_set_loading() {
        let mut state = create_test_state();
        assert!(!state.loading);

        state.set_loading(true);
        assert!(state.loading);

        state.set_loading(false);
        assert!(!state.loading);
    }

    #[test]
    fn test_clear_state() {
        let mut state = create_test_state();
        state.loading = true;
        state.error = Some("Test error".to_string());
        state.medical_history = vec![MedicalHistory {
            id: uuid::Uuid::new_v4(),
            patient_id: uuid::Uuid::new_v4(),
            condition: "Diabetes".to_string(),
            diagnosis_date: None,
            status: opengp_domain::domain::clinical::ConditionStatus::Chronic,
            severity: None,
            notes: None,
            is_active: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            created_by: uuid::Uuid::new_v4(),
            updated_by: None,
        }];

        state.clear();

        assert!(state.medical_history.is_empty());
        assert!(state.medical_history_form.is_none());
        assert!(state.medical_history_detail_modal.is_none());
        assert!(state.error.is_none());
        assert!(!state.loading);
    }

    #[test]
    fn test_has_patient_with_empty_state() {
        let state = create_test_state();
        assert!(!state.has_patient());
    }

    #[test]
    fn test_list_navigation() {
        let mut state = create_test_state();
        assert_eq!(state.medical_history_list.selected_index, 0);

        // Add test items
        state.medical_history = vec![
            MedicalHistory {
                id: uuid::Uuid::new_v4(),
                patient_id: uuid::Uuid::new_v4(),
                condition: "Hypertension".to_string(),
                diagnosis_date: None,
                status: opengp_domain::domain::clinical::ConditionStatus::Active,
                severity: None,
                notes: None,
                is_active: true,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                created_by: uuid::Uuid::new_v4(),
                updated_by: None,
            },
            MedicalHistory {
                id: uuid::Uuid::new_v4(),
                patient_id: uuid::Uuid::new_v4(),
                condition: "Diabetes".to_string(),
                diagnosis_date: None,
                status: opengp_domain::domain::clinical::ConditionStatus::Chronic,
                severity: None,
                notes: None,
                is_active: true,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                created_by: uuid::Uuid::new_v4(),
                updated_by: None,
            },
        ];

        state.next_item();
        assert_eq!(state.medical_history_list.selected_index, 1);

        state.next_item();
        assert_eq!(state.medical_history_list.selected_index, 1); // clamped at max

        state.prev_item();
        assert_eq!(state.medical_history_list.selected_index, 0); // saturating_sub prevents underflow
    }
}

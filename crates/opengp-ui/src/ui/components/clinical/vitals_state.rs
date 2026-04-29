//! Vital Signs State Component
//!
//! Manages the lifecycle and state of vital signs recording, form interaction,
//! and detail modal display.

use crate::ui::components::clinical::{VitalSignsForm, VitalSignsList, VitalsDetailModal};
use crate::ui::theme::Theme;
use opengp_config::healthcare::HealthcareConfig;
use opengp_domain::domain::clinical::VitalSigns;

/// Manages vital signs state including list, form, and detail modal.
#[derive(Clone)]
pub struct VitalsState {
    /// The list of vital signs records
    pub vitals_list: VitalSignsList,
    /// Active vital signs form (if open)
    pub vitals_form: Option<VitalSignsForm>,
    /// Detail modal for viewing vital signs
    pub vitals_detail_modal: Option<VitalsDetailModal>,
    /// Healthcare configuration (for form validation)
    pub healthcare_config: HealthcareConfig,
    /// Theme for styling
    pub theme: Theme,
    /// All vital signs data for the patient
    pub vital_signs: Vec<VitalSigns>,
    /// Loading state
    pub loading: bool,
    /// Error message if any
    pub error: Option<String>,
}

impl VitalsState {
    /// Create a new VitalsState with default values.
    pub fn new(theme: Theme, healthcare_config: HealthcareConfig) -> Self {
        Self {
            vitals_list: VitalSignsList::new(theme.clone()),
            vitals_form: None,
            vitals_detail_modal: None,
            healthcare_config,
            theme,
            vital_signs: Vec::new(),
            loading: false,
            error: None,
        }
    }

    /// Open the vital signs form.
    pub fn open_vitals_form(&mut self, theme: Theme) {
        self.vitals_form = Some(VitalSignsForm::new(theme, self.healthcare_config.clone()));
    }

    /// Close the vital signs form.
    pub fn close_vitals_form(&mut self) {
        self.vitals_form = None;
    }

    /// Open the detail modal for a vital signs record.
    pub fn open_vitals_detail(&mut self, vitals: VitalSigns, theme: Theme) {
        self.vitals_detail_modal = Some(VitalsDetailModal::new(vitals, theme));
    }

    /// Close the detail modal.
    pub fn close_vitals_detail(&mut self) {
        self.vitals_detail_modal = None;
    }

    /// Set loading state.
    pub fn set_loading(&mut self, loading: bool) {
        self.loading = loading;
    }

    /// Clear all vital signs data (when patient changes).
    pub fn clear(&mut self) {
        self.vital_signs.clear();
        self.vitals_list = VitalSignsList::new(self.theme.clone());
        self.vitals_form = None;
        self.vitals_detail_modal = None;
        self.error = None;
        self.loading = false;
    }

    /// Check if a patient is selected.
    pub fn has_patient(&self) -> bool {
        !self.vital_signs.is_empty() || self.vitals_detail_modal.is_some()
    }

    /// Check if the form is currently open.
    pub fn is_form_open(&self) -> bool {
        self.vitals_form.is_some()
    }

    /// Navigate to the next vital signs item in the list.
    pub fn next_item(&mut self) {
        self.vitals_list.next();
    }

    /// Navigate to the previous vital signs item in the list.
    pub fn prev_item(&mut self) {
        self.vitals_list.prev();
    }

    /// Get the currently selected vital signs if available.
    pub fn get_selected(&self) -> Option<&VitalSigns> {
        if self.vitals_list.selected_index < self.vital_signs.len() {
            Some(&self.vital_signs[self.vitals_list.selected_index])
        } else {
            None
        }
    }

    /// Add a new vital signs reading to the list.
    pub fn add_vitals_reading(&mut self, vitals: VitalSigns) {
        self.vital_signs.push(vitals);
    }

    /// Set all vital signs data.
    pub fn set_vital_signs(&mut self, vital_signs: Vec<VitalSigns>) {
        self.vital_signs = vital_signs;
    }

    /// Set error state.
    pub fn set_error(&mut self, error: Option<String>) {
        self.error = error;
    }

    /// Clear error state.
    pub fn clear_error(&mut self) {
        self.error = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use opengp_config::healthcare::HealthcareConfig;
    use uuid::Uuid;

    fn test_theme() -> Theme {
        Theme::default()
    }

    fn test_healthcare_config() -> HealthcareConfig {
        HealthcareConfig::default()
    }

    #[test]
    fn test_vitals_state_construction() {
        let theme = test_theme();
        let config = test_healthcare_config();
        let state = VitalsState::new(theme, config.clone());

        assert!(!state.loading);
        assert!(state.error.is_none());
        assert!(state.vital_signs.is_empty());
        assert!(state.vitals_form.is_none());
        assert!(state.vitals_detail_modal.is_none());
    }

    #[test]
    fn test_open_close_vitals_form() {
        let theme = test_theme();
        let mut state = VitalsState::new(theme.clone(), test_healthcare_config());

        assert!(!state.is_form_open());

        state.open_vitals_form(theme);
        assert!(state.is_form_open());

        state.close_vitals_form();
        assert!(!state.is_form_open());
    }

    #[test]
    fn test_open_close_vitals_detail() {
        let theme = test_theme();
        let mut state = VitalsState::new(theme.clone(), test_healthcare_config());
        let vitals = VitalSigns {
            id: Uuid::new_v4(),
            patient_id: Uuid::new_v4(),
            consultation_id: None,
            measured_at: chrono::Utc::now(),
            systolic_bp: Some(120),
            diastolic_bp: Some(80),
            heart_rate: Some(72),
            respiratory_rate: Some(16),
            temperature: Some(36.5),
            oxygen_saturation: Some(98),
            height_cm: Some(175),
            weight_kg: Some(75.0),
            bmi: Some(24.5),
            notes: None,
            created_at: chrono::Utc::now(),
            created_by: Uuid::new_v4(),
        };

        assert!(state.vitals_detail_modal.is_none());

        state.open_vitals_detail(vitals.clone(), theme);
        assert!(state.vitals_detail_modal.is_some());

        state.close_vitals_detail();
        assert!(state.vitals_detail_modal.is_none());
    }

    #[test]
    fn test_add_vitals_reading() {
        let mut state = VitalsState::new(test_theme(), test_healthcare_config());
        assert!(state.vital_signs.is_empty());

        let vitals = VitalSigns {
            id: Uuid::new_v4(),
            patient_id: Uuid::new_v4(),
            consultation_id: None,
            measured_at: chrono::Utc::now(),
            systolic_bp: Some(120),
            diastolic_bp: Some(80),
            heart_rate: Some(72),
            respiratory_rate: Some(16),
            temperature: Some(36.5),
            oxygen_saturation: Some(98),
            height_cm: Some(175),
            weight_kg: Some(75.0),
            bmi: Some(24.5),
            notes: None,
            created_at: chrono::Utc::now(),
            created_by: Uuid::new_v4(),
        };

        state.add_vitals_reading(vitals.clone());
        assert_eq!(state.vital_signs.len(), 1);
        assert_eq!(state.vital_signs[0].id, vitals.id);
    }

    #[test]
    fn test_get_selected() {
        let mut state = VitalsState::new(test_theme(), test_healthcare_config());
        assert!(state.get_selected().is_none());

        let vitals = VitalSigns {
            id: Uuid::new_v4(),
            patient_id: Uuid::new_v4(),
            consultation_id: None,
            measured_at: chrono::Utc::now(),
            systolic_bp: Some(120),
            diastolic_bp: Some(80),
            heart_rate: Some(72),
            respiratory_rate: Some(16),
            temperature: Some(36.5),
            oxygen_saturation: Some(98),
            height_cm: Some(175),
            weight_kg: Some(75.0),
            bmi: Some(24.5),
            notes: None,
            created_at: chrono::Utc::now(),
            created_by: Uuid::new_v4(),
        };

        state.add_vitals_reading(vitals.clone());
        let selected = state.get_selected();
        assert!(selected.is_some());
        assert_eq!(selected.unwrap().id, vitals.id);
    }

    #[test]
    fn test_set_loading() {
        let mut state = VitalsState::new(test_theme(), test_healthcare_config());
        assert!(!state.loading);

        state.set_loading(true);
        assert!(state.loading);

        state.set_loading(false);
        assert!(!state.loading);
    }

    #[test]
    fn test_clear() {
        let theme = test_theme();
        let mut state = VitalsState::new(theme.clone(), test_healthcare_config());
        state.set_loading(true);
        state.set_error(Some("Test error".to_string()));

        let vitals = VitalSigns {
            id: Uuid::new_v4(),
            patient_id: Uuid::new_v4(),
            consultation_id: None,
            measured_at: chrono::Utc::now(),
            systolic_bp: Some(120),
            diastolic_bp: Some(80),
            heart_rate: Some(72),
            respiratory_rate: Some(16),
            temperature: Some(36.5),
            oxygen_saturation: Some(98),
            height_cm: Some(175),
            weight_kg: Some(75.0),
            bmi: Some(24.5),
            notes: None,
            created_at: chrono::Utc::now(),
            created_by: Uuid::new_v4(),
        };

        state.add_vitals_reading(vitals);
        state.open_vitals_form(theme);

        state.clear();

        assert!(state.vital_signs.is_empty());
        assert!(!state.loading);
        assert!(state.error.is_none());
        assert!(!state.is_form_open());
    }

    #[test]
    fn test_has_patient() {
        let mut state = VitalsState::new(test_theme(), test_healthcare_config());
        assert!(!state.has_patient());

        let vitals = VitalSigns {
            id: Uuid::new_v4(),
            patient_id: Uuid::new_v4(),
            consultation_id: None,
            measured_at: chrono::Utc::now(),
            systolic_bp: Some(120),
            diastolic_bp: Some(80),
            heart_rate: Some(72),
            respiratory_rate: Some(16),
            temperature: Some(36.5),
            oxygen_saturation: Some(98),
            height_cm: Some(175),
            weight_kg: Some(75.0),
            bmi: Some(24.5),
            notes: None,
            created_at: chrono::Utc::now(),
            created_by: Uuid::new_v4(),
        };

        state.add_vitals_reading(vitals);
        assert!(state.has_patient());
    }

    #[test]
    fn test_navigation_next_prev() {
        let mut state = VitalsState::new(test_theme(), test_healthcare_config());

        for _ in 0..3 {
            let vitals = VitalSigns {
                id: Uuid::new_v4(),
                patient_id: Uuid::new_v4(),
                consultation_id: None,
                measured_at: chrono::Utc::now(),
                systolic_bp: Some(120),
                diastolic_bp: Some(80),
                heart_rate: Some(72),
                respiratory_rate: Some(16),
                temperature: Some(36.5),
                oxygen_saturation: Some(98),
                height_cm: Some(175),
                weight_kg: Some(75.0),
                bmi: Some(24.5),
                notes: None,
                created_at: chrono::Utc::now(),
                created_by: Uuid::new_v4(),
            };
            state.add_vitals_reading(vitals);
        }

        let _initial_index = state.vitals_list.selected_index;
        state.next_item();
        let _next_index = state.vitals_list.selected_index;

        state.prev_item();
    }
}

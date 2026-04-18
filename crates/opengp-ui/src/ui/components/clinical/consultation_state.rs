//! Consultation State Component
//!
//! Manages all consultation-related UI state: list, form, detail modal, timer, pagination.

use crate::ui::components::clinical::{
    ConsultationDetailModal, ConsultationForm, ConsultationList,
};
use crate::ui::theme::Theme;
use chrono::{DateTime, Utc};
use opengp_config::ClinicalConfig;
use opengp_domain::domain::clinical::Consultation;
use uuid::Uuid;

/// Consultation state component
///
/// Encapsulates:
/// - Consultation list with pagination
/// - New/edit form
/// - Detail modal (read-only)
/// - Active timer (started_at timestamp)
/// - Consultation data cache
#[derive(Clone)]
pub struct ConsultationState {
    /// List widget state
    pub consultation_list: ConsultationList,
    /// Open form for creating/editing
    pub consultation_form: Option<ConsultationForm>,
    /// Detail modal for viewing consultation
    pub consultation_detail_modal: Option<ConsultationDetailModal>,
    /// All consultations loaded for current patient
    pub consultations: Vec<Consultation>,
    /// Consultations loading indicator
    pub loading: bool,
    /// Error message if loading/operation failed
    pub error: Option<String>,
    /// When the active timer started (for ongoing consultation)
    pub active_timer_started_at: Option<DateTime<Utc>>,
    /// Appointment ID associated with consultations
    pub active_appointment_id: Option<Uuid>,
    /// Configuration for clinical constraints (e.g. max items, validation)
    pub clinical_config: ClinicalConfig,
    /// Theme for rendering forms/modals
    pub theme: Theme,
}

impl ConsultationState {
    /// Create a new ConsultationState
    pub fn new(theme: Theme, clinical_config: ClinicalConfig) -> Self {
        Self {
            consultation_list: ConsultationList::new(theme.clone()),
            consultation_form: None,
            consultation_detail_modal: None,
            consultations: Vec::new(),
            loading: false,
            error: None,
            active_timer_started_at: None,
            active_appointment_id: None,
            clinical_config,
            theme,
        }
    }

    /// Open the consultation form (for new or edit)
    pub fn open_consultation_form(&mut self) {
        self.consultation_form = Some(ConsultationForm::new(self.theme.clone()));
    }

    /// Close the consultation form without saving
    pub fn close_consultation_form(&mut self) {
        self.consultation_form = None;
    }

    /// Open the consultation detail modal
    pub fn open_consultation_detail(
        &mut self,
        consultation: Consultation,
        patient_name: String,
        practitioner_name: String,
    ) {
        self.consultation_detail_modal = Some(ConsultationDetailModal::new(
            consultation,
            patient_name,
            practitioner_name,
            self.theme.clone(),
        ));
    }

    /// Close the consultation detail modal
    pub fn close_consultation_detail(&mut self) {
        self.consultation_detail_modal = None;
    }

    /// Set the loading state
    pub fn set_loading(&mut self, loading: bool) {
        self.loading = loading;
    }

    /// Clear all state (for switching patient or logout)
    pub fn clear(&mut self) {
        self.consultation_form = None;
        self.consultation_detail_modal = None;
        self.consultations.clear();
        self.active_timer_started_at = None;
        self.active_appointment_id = None;
        self.loading = false;
        self.error = None;
    }

    /// Check if a patient is loaded (has appointment ID)
    pub fn has_patient(&self) -> bool {
        self.active_appointment_id.is_some()
    }

    /// Check if the form is currently open
    pub fn is_form_open(&self) -> bool {
        self.consultation_form.is_some()
    }

    /// Set the timer start time for an ongoing consultation
    pub fn set_active_timer_started_at(&mut self, started_at: DateTime<Utc>) {
        self.active_timer_started_at = Some(started_at);
    }

    /// Clear the timer (consultation ended)
    pub fn clear_active_timer(&mut self) {
        self.active_timer_started_at = None;
    }

    /// Navigate to the next consultation in the list
    pub fn next_item(&mut self) {
        self.consultation_list.next();
    }

    /// Navigate to the previous consultation in the list
    pub fn prev_item(&mut self) {
        self.consultation_list.prev();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consultation_state_creation() {
        let theme = Theme::default();
        let config = ClinicalConfig::default();
        let state = ConsultationState::new(theme.clone(), config.clone());

        assert!(!state.loading);
        assert!(state.error.is_none());
        assert!(state.consultation_form.is_none());
        assert!(state.consultation_detail_modal.is_none());
        assert!(state.consultations.is_empty());
        assert!(state.active_timer_started_at.is_none());
        assert!(state.active_appointment_id.is_none());
    }

    #[test]
    fn test_open_close_form() {
        let theme = Theme::default();
        let config = ClinicalConfig::default();
        let mut state = ConsultationState::new(theme, config);

        assert!(!state.is_form_open());

        state.open_consultation_form();
        assert!(state.is_form_open());

        state.close_consultation_form();
        assert!(!state.is_form_open());
    }

    #[test]
    fn test_set_loading() {
        let theme = Theme::default();
        let config = ClinicalConfig::default();
        let mut state = ConsultationState::new(theme, config);

        assert!(!state.loading);
        state.set_loading(true);
        assert!(state.loading);
        state.set_loading(false);
        assert!(!state.loading);
    }

    #[test]
    fn test_timer_start_stop() {
        let theme = Theme::default();
        let config = ClinicalConfig::default();
        let mut state = ConsultationState::new(theme, config);

        assert!(state.active_timer_started_at.is_none());

        let now = Utc::now();
        state.set_active_timer_started_at(now);
        assert!(state.active_timer_started_at.is_some());
        assert_eq!(state.active_timer_started_at, Some(now));

        state.clear_active_timer();
        assert!(state.active_timer_started_at.is_none());
    }

    #[test]
    fn test_has_patient() {
        let theme = Theme::default();
        let config = ClinicalConfig::default();
        let mut state = ConsultationState::new(theme, config);

        assert!(!state.has_patient());

        state.active_appointment_id = Some(Uuid::new_v4());
        assert!(state.has_patient());

        state.active_appointment_id = None;
        assert!(!state.has_patient());
    }

    #[test]
    fn test_clear_state() {
        let theme = Theme::default();
        let config = ClinicalConfig::default();
        let mut state = ConsultationState::new(theme, config);

        state.active_appointment_id = Some(Uuid::new_v4());
        state.open_consultation_form();
        state.set_loading(true);
        state.error = Some("test error".to_string());

        state.clear();

        assert!(state.consultation_form.is_none());
        assert!(state.consultation_detail_modal.is_none());
        assert!(state.consultations.is_empty());
        assert!(state.active_timer_started_at.is_none());
        assert!(state.active_appointment_id.is_none());
        assert!(!state.loading);
        assert!(state.error.is_none());
    }

    #[test]
    fn test_list_navigation() {
        let theme = Theme::default();
        let config = ClinicalConfig::default();
        let mut state = ConsultationState::new(theme, config);

        // Initially at index 0
        assert_eq!(state.consultation_list.selected_index, 0);

        // Navigate next/prev (list is empty but nav should work)
        state.next_item();
        state.prev_item();

        // Should not panic
    }
}

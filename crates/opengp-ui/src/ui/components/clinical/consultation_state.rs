use crate::ui::components::clinical::{ConsultationForm, ConsultationList};
use crate::ui::theme::Theme;
use chrono::{DateTime, Utc};
use opengp_config::ClinicalConfig;
use opengp_domain::domain::clinical::Consultation;
use uuid::Uuid;

#[derive(Clone)]
pub struct ConsultationState {
    pub consultation_list: ConsultationList,
    pub consultation_form: Option<ConsultationForm>,
    pub consultations: Vec<Consultation>,
    pub loading: bool,
    pub error: Option<String>,
    pub active_timer_started_at: Option<DateTime<Utc>>,
    pub active_appointment_id: Option<Uuid>,
    pub clinical_config: ClinicalConfig,
    pub theme: Theme,
}

impl ConsultationState {
    pub fn new(theme: Theme, clinical_config: ClinicalConfig) -> Self {
        Self {
            consultation_list: ConsultationList::new(theme.clone()),
            consultation_form: None,
            consultations: Vec::new(),
            loading: false,
            error: None,
            active_timer_started_at: None,
            active_appointment_id: None,
            clinical_config,
            theme,
        }
    }

    pub fn open_consultation_form(&mut self) {
        self.consultation_form = Some(ConsultationForm::new(self.theme.clone()));
    }

    pub fn close_consultation_form(&mut self) {
        self.consultation_form = None;
    }

    pub fn set_loading(&mut self, loading: bool) {
        self.loading = loading;
    }

    pub fn clear(&mut self) {
        self.consultation_form = None;
        self.consultations.clear();
        self.active_timer_started_at = None;
        self.active_appointment_id = None;
        self.loading = false;
        self.error = None;
    }

    pub fn has_patient(&self) -> bool {
        self.active_appointment_id.is_some()
    }

    pub fn is_form_open(&self) -> bool {
        self.consultation_form.is_some()
    }

    pub fn set_active_timer_started_at(&mut self, started_at: DateTime<Utc>) {
        self.active_timer_started_at = Some(started_at);
    }

    pub fn clear_active_timer(&mut self) {
        self.active_timer_started_at = None;
    }

    pub fn next_item(&mut self) {
        self.consultation_list.next();
    }

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

        assert_eq!(state.consultation_list.selected_index, 0);
        state.next_item();
        state.prev_item();
    }
}

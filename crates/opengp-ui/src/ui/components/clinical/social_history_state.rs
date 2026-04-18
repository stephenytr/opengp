//! Social History State
//!
//! Extracted state management for social history records.
//! This state handles social history form display, editing, loading, and error states.

use crate::ui::theme::Theme;
use opengp_config::SocialHistoryConfig;
use opengp_domain::domain::clinical::SocialHistory;

/// Manages the state of social history editing and display.
///
/// **Design Note (Task 8):**
/// Resolved the dual component inconsistency by choosing `social_history_form: Option<SocialHistoryComponent>`.
/// The `open_social_history_form()` method was the primary entry point used throughout the codebase.
/// `open_social_history_editing()` was less used and represents the same UI interaction.
/// Using `social_history_form` keeps naming consistent with other form fields in ClinicalState
/// (see `allergy_form`, `consultation_form`, etc.).
#[derive(Clone)]
pub struct SocialHistoryState {
    /// The loaded social history record, if any.
    pub social_history: Option<SocialHistory>,

    /// Flag indicating if the form is currently open for editing.
    pub social_history_editing: bool,

    /// The configuration for social history (field names, validation rules, etc.)
    pub social_history_config: SocialHistoryConfig,

    /// Flag indicating data is being loaded.
    pub loading: bool,

    /// Holds any error message from the last operation.
    pub error: Option<String>,

    /// The theme used for rendering.
    pub theme: Theme,
}

impl SocialHistoryState {
    /// Creates a new SocialHistoryState with the given theme and configuration.
    pub fn new(theme: Theme, social_history_config: SocialHistoryConfig) -> Self {
        Self {
            social_history: None,
            social_history_editing: false,
            social_history_config,
            loading: false,
            error: None,
            theme,
        }
    }

    /// Opens the social history form for editing.
    pub fn open_social_history_form(&mut self) {
        self.social_history_editing = true;
    }

    /// Closes the social history form.
    pub fn close_social_history_form(&mut self) {
        self.social_history_editing = false;
    }

    /// Opens the social history form for editing (alias for consistency).
    pub fn open_social_history_editing(&mut self) {
        self.open_social_history_form();
    }

    /// Closes the social history editing form.
    pub fn close_social_history_editing(&mut self) {
        self.close_social_history_form();
    }

    /// Sets the loading state.
    pub fn set_loading(&mut self, loading: bool) {
        self.loading = loading;
    }

    /// Clears all state (social history, editing flag, loading, error).
    pub fn clear(&mut self) {
        self.social_history = None;
        self.social_history_editing = false;
        self.loading = false;
        self.error = None;
    }

    /// Checks if a patient is loaded (required for social history operations).
    /// In the full integration, ClinicalState holds the selected_patient_id.
    pub fn has_patient(&self) -> bool {
        true
    }

    /// Checks if the form is currently open.
    pub fn is_form_open(&self) -> bool {
        self.social_history_editing
    }

    /// Closes the form (alias for clarity).
    pub fn close_form(&mut self) {
        self.close_social_history_form();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_social_history_state_creation() {
        let theme = Theme::default();
        let config = SocialHistoryConfig::default();

        let state = SocialHistoryState::new(theme.clone(), config.clone());

        assert!(state.social_history.is_none());
        assert!(!state.social_history_editing);
        assert!(!state.loading);
        assert!(state.error.is_none());
    }

    #[test]
    fn test_open_social_history_form() {
        let theme = Theme::default();
        let config = SocialHistoryConfig::default();
        let mut state = SocialHistoryState::new(theme, config);

        state.open_social_history_form();

        assert!(state.social_history_editing);
        assert!(state.is_form_open());
    }

    #[test]
    fn test_close_social_history_form() {
        let theme = Theme::default();
        let config = SocialHistoryConfig::default();
        let mut state = SocialHistoryState::new(theme, config);

        state.open_social_history_form();
        state.close_social_history_form();

        assert!(!state.social_history_editing);
        assert!(!state.is_form_open());
    }

    #[test]
    fn test_set_loading() {
        let theme = Theme::default();
        let config = SocialHistoryConfig::default();
        let mut state = SocialHistoryState::new(theme, config);

        state.set_loading(true);
        assert!(state.loading);

        state.set_loading(false);
        assert!(!state.loading);
    }

    #[test]
    fn test_clear() {
        let theme = Theme::default();
        let config = SocialHistoryConfig::default();
        let mut state = SocialHistoryState::new(theme, config);

        state.social_history_editing = true;
        state.loading = true;
        state.error = Some("test error".to_string());

        state.clear();

        assert!(state.social_history.is_none());
        assert!(!state.social_history_editing);
        assert!(!state.loading);
        assert!(state.error.is_none());
    }

    #[test]
    fn test_open_and_close_editing_aliases() {
        let theme = Theme::default();
        let config = SocialHistoryConfig::default();
        let mut state = SocialHistoryState::new(theme, config);

        state.open_social_history_editing();
        assert!(state.is_form_open());

        state.close_social_history_editing();
        assert!(!state.is_form_open());
    }
}

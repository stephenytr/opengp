//! Social History Form Context Tests
//!
//! These tests verify that Social History integrates with the ClinicalFormView
//! architecture like other clinical forms (Allergy, Vitals, etc.).
//!
//! RED PHASE: These tests should FAIL on the current implementation because
//! Social History uses a separate key handling path instead of ClinicalFormView.

use crate::ui::components::clinical::state::{ClinicalFormView, ClinicalState};
use crate::ui::theme::Theme;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use opengp_config::{
    healthcare::HealthcareConfig, AllergyConfig, ClinicalConfig, SocialHistoryConfig,
};

/// Test that ClinicalState can open Social History through form workflow
/// This verifies the state has proper methods to open Social History through
/// the canonical form context (not separate editing flag).
#[test]
fn clinical_state_opens_social_history_via_form_context() {
    let theme = Theme::dark();
    let mut state = ClinicalState::new(
        theme,
        HealthcareConfig::default(),
        AllergyConfig::default(),
        ClinicalConfig::default(),
        SocialHistoryConfig::default(),
    );

    // The state should have a method to open Social History as a form
    // Using the same pattern as other forms (open_allergy_form, etc.)
    state.open_social_history_form();

    // After opening, form_view should indicate a form is open
    assert!(
        state.is_form_open(),
        "Opening Social History should set form_view to indicate open form"
    );
}

/// Test that ClinicalState can close Social History form via form workflow
#[test]
fn clinical_state_closes_social_history_via_form_context() {
    let theme = Theme::dark();
    let mut state = ClinicalState::new(
        theme,
        HealthcareConfig::default(),
        AllergyConfig::default(),
        ClinicalConfig::default(),
        SocialHistoryConfig::default(),
    );

    // Open then close using form methods
    state.open_social_history_form();
    state.close_social_history_form();

    // Form should be closed
    assert!(!state.is_form_open(), "Form should be closed after closing");
}

/// Test that Social History form state exists in ClinicalState
/// This verifies the form uses persistent state like other clinical forms.
#[test]
fn clinical_state_has_social_history_form_field() {
    let theme = Theme::dark();
    let mut state = ClinicalState::new(
        theme,
        HealthcareConfig::default(),
        AllergyConfig::default(),
        ClinicalConfig::default(),
        SocialHistoryConfig::default(),
    );

    // The state should have a social_history_form field like allergy_form
    state.open_social_history_form();

    // Should be able to access the form via the new field (now part of social_history sub-struct)
    assert!(
        state.social_history.social_history_editing,
        "Should have social_history_editing flag set after opening"
    );
}

/// Test that Social History edit mode uses single lifecycle source
/// This verifies there's one canonical path for entry, not dual flags.
#[test]
fn social_history_single_lifecycle_source() {
    let theme = Theme::dark();
    let mut state = ClinicalState::new(
        theme,
        HealthcareConfig::default(),
        AllergyConfig::default(),
        ClinicalConfig::default(),
        SocialHistoryConfig::default(),
    );

    // Opening via form workflow should set form_view
    state.open_social_history_form();

    // Should be in form view mode - form_view is the source of truth
    assert!(
        state.is_form_view(),
        "Should be in form view after opening via form workflow"
    );
}

/// Test that close_form also closes Social History
/// This verifies the canonical close_form() works for Social History.
#[test]
fn close_form_closes_social_history() {
    let theme = Theme::dark();
    let mut state = ClinicalState::new(
        theme,
        HealthcareConfig::default(),
        AllergyConfig::default(),
        ClinicalConfig::default(),
        SocialHistoryConfig::default(),
    );

    state.open_social_history_form();
    assert!(state.is_form_open());

    // Canonical close_form should close Social History too
    state.close_form();

    assert!(
        !state.is_form_open(),
        "Form should be closed after close_form()"
    );
}

/// Test that ClinicalFormView enum includes SocialHistoryForm variant
/// This test verifies the canonical form view architecture includes Social History.
#[test]
fn clinical_form_view_includes_social_history() {
    let theme = Theme::dark();
    let mut state = ClinicalState::new(
        theme,
        HealthcareConfig::default(),
        AllergyConfig::default(),
        ClinicalConfig::default(),
        SocialHistoryConfig::default(),
    );

    // Open Social History - should set form_view to something other than None
    state.open_social_history_form();

    // The form_view should indicate Social History is active
    // After migration, this should be ClinicalFormView::SocialHistoryForm
    match state.form_view {
        ClinicalFormView::None => {
            panic!("form_view should not be None after opening Social History");
        }
        _ => {
            // Any other variant means it's working through form context
        }
    }
}

use super::*;
use opengp_config::{load_allergy_config, AllergyConfig};

#[test]
fn test_allergy_form_creation() {
    let theme = Theme::dark();
    let config = load_allergy_config().unwrap();
    let form = AllergyForm::new(theme, &config);

    assert_eq!(form.focused_field(), AllergyFormField::Allergen);
    assert!(!form.is_valid);
    assert!(!form.has_errors());
}

#[test]
fn test_allergy_form_validation_required_fields() {
    let theme = Theme::dark();
    let config = load_allergy_config().unwrap();
    let mut form = AllergyForm::new(theme, &config);

    form.validate();
    assert!(!form.is_valid);
    assert!(form.error(AllergyFormField::Allergen).is_some());
    assert!(form.error(AllergyFormField::AllergyType).is_some());
    assert!(form.error(AllergyFormField::Severity).is_some());
}

#[test]
fn test_allergy_form_validation_passes_when_required_filled() {
    let theme = Theme::dark();
    let config = load_allergy_config().unwrap();
    let mut form = AllergyForm::new(theme, &config);

    form.set_value(AllergyFormField::Allergen, "Penicillin".to_string());
    form.set_value(AllergyFormField::AllergyType, "drug".to_string());
    form.set_value(AllergyFormField::Severity, "severe".to_string());

    let valid = form.validate();
    assert!(valid);
    assert!(!form.has_errors());
}

#[test]
fn test_allergy_form_field_navigation() {
    let theme = Theme::dark();
    let config = load_allergy_config().unwrap();
    let mut form = AllergyForm::new(theme, &config);

    assert_eq!(form.focused_field(), AllergyFormField::Allergen);
    form.next_field();
    assert_eq!(form.focused_field(), AllergyFormField::AllergyType);
    form.next_field();
    assert_eq!(form.focused_field(), AllergyFormField::Severity);
    form.prev_field();
    assert_eq!(form.focused_field(), AllergyFormField::AllergyType);
}

#[test]
fn test_allergy_form_onset_date_validation() {
    let theme = Theme::dark();
    let config = AllergyConfig::default();
    let mut form = AllergyForm::new(theme, &config);

    form.set_value(AllergyFormField::OnsetDate, "not-a-date".to_string());
    assert!(form.error(AllergyFormField::OnsetDate).is_some());

    form.set_value(AllergyFormField::OnsetDate, "15/01/2024".to_string());
    assert!(form.error(AllergyFormField::OnsetDate).is_none());
}

#[test]
fn test_allergy_form_all_fields_ordered() {
    let fields = AllergyFormField::all();
    assert_eq!(fields[0], AllergyFormField::Allergen);
    assert_eq!(fields[1], AllergyFormField::AllergyType);
    assert_eq!(fields[2], AllergyFormField::Severity);
    assert_eq!(fields[3], AllergyFormField::Reaction);
    assert_eq!(fields[4], AllergyFormField::OnsetDate);
    assert_eq!(fields[5], AllergyFormField::Notes);
}

#[test]
fn test_allergy_form_textarea_fields_accept_input() {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    let theme = Theme::dark();
    let config = AllergyConfig::default();
    let mut form = AllergyForm::new(theme, &config);

    let key = KeyEvent::new(KeyCode::Char('P'), KeyModifiers::NONE);
    let action = form.handle_key(key);
    assert!(action.is_some());

    let key = KeyEvent::new(KeyCode::Char('e'), KeyModifiers::NONE);
    form.handle_key(key);

    assert!(form.get_value(AllergyFormField::Allergen).contains('P'));
}

#[test]
fn test_allergy_form_get_value_uses_textarea() {
    let theme = Theme::dark();
    let config = AllergyConfig::default();
    let mut form = AllergyForm::new(theme, &config);

    form.set_value(AllergyFormField::Allergen, "Penicillin".to_string());
    form.set_value(AllergyFormField::Reaction, "Rash".to_string());
    form.set_value(AllergyFormField::Notes, "Severe reaction noted".to_string());

    assert_eq!(form.get_value(AllergyFormField::Allergen), "Penicillin");
    assert_eq!(form.get_value(AllergyFormField::Reaction), "Rash");
    assert_eq!(
        form.get_value(AllergyFormField::Notes),
        "Severe reaction noted"
    );
}

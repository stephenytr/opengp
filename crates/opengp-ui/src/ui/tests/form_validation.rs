use crate::ui::components::patient::{FormField, PatientForm};
use crate::ui::widgets::FormNavigation;
use crate::ui::Theme;

#[test]
fn patient_form_required_field_validation() {
    let theme = Theme::dark();
    let mut form = PatientForm::new(theme);

    form.set_value(FormField::DateOfBirth, "01/01/1990".to_string());
    form.set_value(FormField::Gender, "Male".to_string());
    form.validate();

    assert_eq!(
        form.error(FormField::FirstName).map(String::as_str),
        Some("This field is required")
    );
    assert_eq!(
        form.error(FormField::LastName).map(String::as_str),
        Some("This field is required")
    );
}

#[test]
fn patient_form_invalid_email_format_detection() {
    let theme = Theme::dark();
    let mut form = PatientForm::new(theme);

    form.set_value(FormField::Email, "invalid-email".to_string());

    assert_eq!(
        form.error(FormField::Email).map(String::as_str),
        Some("Invalid email format")
    );
}

#[test]
fn patient_form_phone_number_validation() {
    let theme = Theme::dark();
    let mut form = PatientForm::new(theme);

    form.set_value(FormField::PhoneMobile, "12345".to_string());
    assert_eq!(
        form.error(FormField::PhoneMobile).map(String::as_str),
        Some("Invalid phone number")
    );

    form.set_value(FormField::PhoneMobile, "0412 345 678".to_string());
    assert_eq!(form.error(FormField::PhoneMobile), None);
}

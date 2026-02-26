//! Patient Form Component
//!
//! Comprehensive form for creating and editing patients.

use std::collections::HashMap;

use crossterm::event::{KeyEvent, MouseEvent, MouseEventKind};
use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};
use ratatui::style::Style;
use ratatui::widgets::{Block, Borders, Widget};
use uuid::Uuid;

use crate::domain::patient::{Address, EmergencyContact, NewPatientData, Patient};
use crate::ui::input::to_ratatui_key;
use crate::ui::layout::LABEL_WIDTH;
use crate::ui::theme::Theme;
use crate::ui::view_models::PatientFormData;
use crate::ui::widgets::{
    format_date, parse_date, DropdownAction, DropdownOption, DropdownWidget, HeightMode,
    ScrollableFormState, TextareaState, TextareaWidget,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FormMode {
    #[default]
    Create,
    Edit(Uuid),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FormField {
    Title,
    FirstName,
    MiddleName,
    LastName,
    PreferredName,
    DateOfBirth,
    Gender,
    AddressLine1,
    AddressLine2,
    Suburb,
    State,
    Postcode,
    Country,
    PhoneHome,
    PhoneMobile,
    Email,
    MedicareNumber,
    MedicareIrn,
    MedicareExpiry,
    Ihi,
    EmergencyName,
    EmergencyPhone,
    EmergencyRelationship,
    ConcessionType,
    ConcessionNumber,
    PreferredLanguage,
    InterpreterRequired,
    AtsiStatus,
}

impl FormField {
    pub fn all() -> Vec<FormField> {
        vec![
            FormField::Title,
            FormField::FirstName,
            FormField::MiddleName,
            FormField::LastName,
            FormField::PreferredName,
            FormField::DateOfBirth,
            FormField::Gender,
            FormField::AddressLine1,
            FormField::AddressLine2,
            FormField::Suburb,
            FormField::State,
            FormField::Postcode,
            FormField::Country,
            FormField::PhoneHome,
            FormField::PhoneMobile,
            FormField::Email,
            FormField::MedicareNumber,
            FormField::MedicareIrn,
            FormField::MedicareExpiry,
            FormField::Ihi,
            FormField::EmergencyName,
            FormField::EmergencyPhone,
            FormField::EmergencyRelationship,
            FormField::ConcessionType,
            FormField::ConcessionNumber,
            FormField::PreferredLanguage,
            FormField::InterpreterRequired,
            FormField::AtsiStatus,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            FormField::Title => "Title",
            FormField::FirstName => "First Name *",
            FormField::MiddleName => "Middle Name",
            FormField::LastName => "Last Name *",
            FormField::PreferredName => "Preferred Name",
            FormField::DateOfBirth => "Date of Birth * (dd/mm/yyyy)",
            FormField::Gender => "Gender *",
            FormField::AddressLine1 => "Address Line 1",
            FormField::AddressLine2 => "Address Line 2",
            FormField::Suburb => "Suburb",
            FormField::State => "State",
            FormField::Postcode => "Postcode",
            FormField::Country => "Country",
            FormField::PhoneHome => "Phone (Home)",
            FormField::PhoneMobile => "Phone (Mobile)",
            FormField::Email => "Email",
            FormField::MedicareNumber => "Medicare Number",
            FormField::MedicareIrn => "Medicare IRN",
            FormField::MedicareExpiry => "Medicare Expiry (dd/mm/yyyy)",
            FormField::Ihi => "IHI",
            FormField::EmergencyName => "Emergency Contact Name",
            FormField::EmergencyPhone => "Emergency Contact Phone",
            FormField::EmergencyRelationship => "Emergency Contact Relationship",
            FormField::ConcessionType => "Concession Type",
            FormField::ConcessionNumber => "Concession Number",
            FormField::PreferredLanguage => "Preferred Language",
            FormField::InterpreterRequired => "Interpreter Required",
            FormField::AtsiStatus => "ATSI Status",
        }
    }

    pub fn is_required(&self) -> bool {
        matches!(
            self,
            FormField::FirstName | FormField::LastName | FormField::DateOfBirth | FormField::Gender
        )
    }

    pub fn is_dropdown(&self) -> bool {
        matches!(
            self,
            FormField::Gender
                | FormField::ConcessionType
                | FormField::InterpreterRequired
                | FormField::AtsiStatus
        )
    }
}

pub struct PatientForm {
    mode: FormMode,
    data: PatientFormData,
    errors: HashMap<FormField, String>,
    focused_field: FormField,
    saving: bool,
    theme: Theme,
    scroll: ScrollableFormState,
    gender_dropdown: DropdownWidget,
    concession_type_dropdown: DropdownWidget,
    atsi_status_dropdown: DropdownWidget,
    interpreter_required_dropdown: DropdownWidget,
    title: TextareaState,
    first_name: TextareaState,
    middle_name: TextareaState,
    last_name: TextareaState,
    preferred_name: TextareaState,
    date_of_birth: TextareaState,
    address_line1: TextareaState,
    address_line2: TextareaState,
    suburb: TextareaState,
    state: TextareaState,
    postcode: TextareaState,
    country: TextareaState,
    phone_home: TextareaState,
    phone_mobile: TextareaState,
    email: TextareaState,
    medicare_number: TextareaState,
    medicare_irn: TextareaState,
    medicare_expiry: TextareaState,
    ihi: TextareaState,
    emergency_name: TextareaState,
    emergency_phone: TextareaState,
    emergency_relationship: TextareaState,
    concession_number: TextareaState,
    preferred_language: TextareaState,
}

impl Clone for PatientForm {
    fn clone(&self) -> Self {
        Self {
            mode: self.mode,
            data: self.data.clone(),
            errors: self.errors.clone(),
            focused_field: self.focused_field,
            saving: self.saving,
            theme: self.theme.clone(),
            scroll: self.scroll.clone(),
            gender_dropdown: self.gender_dropdown.clone(),
            concession_type_dropdown: self.concession_type_dropdown.clone(),
            atsi_status_dropdown: self.atsi_status_dropdown.clone(),
            interpreter_required_dropdown: self.interpreter_required_dropdown.clone(),
            title: self.title.clone(),
            first_name: self.first_name.clone(),
            middle_name: self.middle_name.clone(),
            last_name: self.last_name.clone(),
            preferred_name: self.preferred_name.clone(),
            date_of_birth: self.date_of_birth.clone(),
            address_line1: self.address_line1.clone(),
            address_line2: self.address_line2.clone(),
            suburb: self.suburb.clone(),
            state: self.state.clone(),
            postcode: self.postcode.clone(),
            country: self.country.clone(),
            phone_home: self.phone_home.clone(),
            phone_mobile: self.phone_mobile.clone(),
            email: self.email.clone(),
            medicare_number: self.medicare_number.clone(),
            medicare_irn: self.medicare_irn.clone(),
            medicare_expiry: self.medicare_expiry.clone(),
            ihi: self.ihi.clone(),
            emergency_name: self.emergency_name.clone(),
            emergency_phone: self.emergency_phone.clone(),
            emergency_relationship: self.emergency_relationship.clone(),
            concession_number: self.concession_number.clone(),
            preferred_language: self.preferred_language.clone(),
        }
    }
}

fn single_line(label: &'static str) -> TextareaState {
    TextareaState::new(label).with_height_mode(HeightMode::SingleLine)
}

impl PatientForm {
    pub fn new(theme: Theme) -> Self {
        let gender_options = vec![
            DropdownOption::new("Male", "Male"),
            DropdownOption::new("Female", "Female"),
            DropdownOption::new("Other", "Other"),
            DropdownOption::new("PreferNotToSay", "Prefer not to say"),
        ];
        let gender_dropdown = DropdownWidget::new("Gender", gender_options, theme.clone());

        let concession_options = vec![
            DropdownOption::new("DVA", "DVA"),
            DropdownOption::new("Pensioner", "Pensioner"),
            DropdownOption::new("HealthcareCard", "Healthcare Card"),
            DropdownOption::new("SafetyNetCard", "Safety Net Card"),
        ];
        let concession_type_dropdown =
            DropdownWidget::new("Concession Type", concession_options, theme.clone());

        let atsi_options = vec![
            DropdownOption::new(
                "AboriginalNotTorresStrait",
                "Aboriginal (not Torres Strait)",
            ),
            DropdownOption::new(
                "TorresStraitNotAboriginal",
                "Torres Strait (not Aboriginal)",
            ),
            DropdownOption::new(
                "BothAboriginalAndTorresStrait",
                "Both Aboriginal and Torres Strait",
            ),
            DropdownOption::new(
                "NeitherAboriginalNorTorresStrait",
                "Neither Aboriginal nor Torres Strait",
            ),
            DropdownOption::new("NotStated", "Not stated"),
        ];
        let atsi_status_dropdown = DropdownWidget::new("ATSI Status", atsi_options, theme.clone());

        let interpreter_options = vec![
            DropdownOption::new("Yes", "Yes"),
            DropdownOption::new("No", "No"),
        ];
        let interpreter_required_dropdown =
            DropdownWidget::new("Interpreter Required", interpreter_options, theme.clone());

        Self {
            mode: FormMode::Create,
            data: PatientFormData::empty(),
            errors: HashMap::new(),
            focused_field: FormField::FirstName,
            saving: false,
            theme,
            scroll: ScrollableFormState::new(),
            gender_dropdown,
            concession_type_dropdown,
            atsi_status_dropdown,
            interpreter_required_dropdown,
            title: single_line("Title"),
            first_name: single_line("First Name"),
            middle_name: single_line("Middle Name"),
            last_name: single_line("Last Name"),
            preferred_name: single_line("Preferred Name"),
            date_of_birth: single_line("Date of Birth"),
            address_line1: single_line("Address Line 1"),
            address_line2: single_line("Address Line 2"),
            suburb: single_line("Suburb"),
            state: single_line("State"),
            postcode: single_line("Postcode"),
            country: single_line("Country"),
            phone_home: single_line("Phone (Home)"),
            phone_mobile: single_line("Phone (Mobile)"),
            email: single_line("Email"),
            medicare_number: single_line("Medicare Number").max_length(10),
            medicare_irn: single_line("Medicare IRN").max_length(1),
            medicare_expiry: single_line("Medicare Expiry"),
            ihi: single_line("IHI"),
            emergency_name: single_line("Emergency Contact Name"),
            emergency_phone: single_line("Emergency Contact Phone"),
            emergency_relationship: single_line("Emergency Contact Relationship"),
            concession_number: single_line("Concession Number"),
            preferred_language: single_line("Preferred Language"),
        }
    }

    pub fn from_patient(patient: Patient, theme: Theme) -> Self {
        let gender = patient.gender;
        let concession_type = patient.concession_type;
        let atsi_status = patient.aboriginal_torres_strait_islander;
        let interpreter_required = patient.interpreter_required;

        let mut form = Self::new(theme);
        form.mode = FormMode::Edit(patient.id);

        if let Some(ref t) = patient.title {
            form.title = single_line("Title").with_value(t.clone());
        }
        form.first_name = single_line("First Name").with_value(patient.first_name.clone());
        if let Some(ref mn) = patient.middle_name {
            form.middle_name = single_line("Middle Name").with_value(mn.clone());
        }
        form.last_name = single_line("Last Name").with_value(patient.last_name.clone());
        if let Some(ref pn) = patient.preferred_name {
            form.preferred_name = single_line("Preferred Name").with_value(pn.clone());
        }
        form.date_of_birth =
            single_line("Date of Birth").with_value(format_date(patient.date_of_birth));
        if let Some(ref l1) = patient.address.line1 {
            form.address_line1 = single_line("Address Line 1").with_value(l1.clone());
        }
        if let Some(ref l2) = patient.address.line2 {
            form.address_line2 = single_line("Address Line 2").with_value(l2.clone());
        }
        if let Some(ref s) = patient.address.suburb {
            form.suburb = single_line("Suburb").with_value(s.clone());
        }
        if let Some(ref st) = patient.address.state {
            form.state = single_line("State").with_value(st.clone());
        }
        if let Some(ref pc) = patient.address.postcode {
            form.postcode = single_line("Postcode").with_value(pc.clone());
        }
        form.country = single_line("Country").with_value(patient.address.country.clone());
        if let Some(ref ph) = patient.phone_home {
            form.phone_home = single_line("Phone (Home)").with_value(ph.clone());
        }
        if let Some(ref pm) = patient.phone_mobile {
            form.phone_mobile = single_line("Phone (Mobile)").with_value(pm.clone());
        }
        if let Some(ref em) = patient.email {
            form.email = single_line("Email").with_value(em.clone());
        }
        if let Some(ref mn) = patient.medicare_number {
            form.medicare_number = single_line("Medicare Number")
                .max_length(10)
                .with_value(mn.clone());
        }
        if let Some(irn) = patient.medicare_irn {
            form.medicare_irn = single_line("Medicare IRN")
                .max_length(1)
                .with_value(irn.to_string());
        }
        if let Some(exp) = patient.medicare_expiry {
            form.medicare_expiry = single_line("Medicare Expiry").with_value(format_date(exp));
        }
        if let Some(ref ihi) = patient.ihi {
            form.ihi = single_line("IHI").with_value(ihi.clone());
        }
        if let Some(ref ec) = patient.emergency_contact {
            form.emergency_name = single_line("Emergency Contact Name").with_value(ec.name.clone());
            form.emergency_phone =
                single_line("Emergency Contact Phone").with_value(ec.phone.clone());
            form.emergency_relationship =
                single_line("Emergency Contact Relationship").with_value(ec.relationship.clone());
        }
        if let Some(ref cn) = patient.concession_number {
            form.concession_number = single_line("Concession Number").with_value(cn.clone());
        }
        form.preferred_language =
            single_line("Preferred Language").with_value(patient.preferred_language.clone());

        form.data = PatientFormData::from(patient);

        form.gender_dropdown.set_value(&gender.to_string());
        if let Some(concession) = concession_type {
            form.concession_type_dropdown
                .set_value(&concession.to_string());
        }
        if let Some(atsi) = atsi_status {
            form.atsi_status_dropdown.set_value(&atsi.to_string());
        }
        form.interpreter_required_dropdown
            .set_value(if interpreter_required { "Yes" } else { "No" });

        form
    }

    pub fn is_edit_mode(&self) -> bool {
        matches!(self.mode, FormMode::Edit(_))
    }

    pub fn patient_id(&self) -> Option<Uuid> {
        match self.mode {
            FormMode::Edit(id) => Some(id),
            FormMode::Create => None,
        }
    }

    pub fn get_value(&self, field: FormField) -> String {
        match field {
            FormField::Title => self.title.value(),
            FormField::FirstName => self.first_name.value(),
            FormField::MiddleName => self.middle_name.value(),
            FormField::LastName => self.last_name.value(),
            FormField::PreferredName => self.preferred_name.value(),
            FormField::DateOfBirth => self.date_of_birth.value(),
            FormField::Gender => self
                .gender_dropdown
                .selected_value()
                .unwrap_or("")
                .to_string(),
            FormField::AddressLine1 => self.address_line1.value(),
            FormField::AddressLine2 => self.address_line2.value(),
            FormField::Suburb => self.suburb.value(),
            FormField::State => self.state.value(),
            FormField::Postcode => self.postcode.value(),
            FormField::Country => self.country.value(),
            FormField::PhoneHome => self.phone_home.value(),
            FormField::PhoneMobile => self.phone_mobile.value(),
            FormField::Email => self.email.value(),
            FormField::MedicareNumber => self.medicare_number.value(),
            FormField::MedicareIrn => self.medicare_irn.value(),
            FormField::MedicareExpiry => self.medicare_expiry.value(),
            FormField::Ihi => self.ihi.value(),
            FormField::EmergencyName => self.emergency_name.value(),
            FormField::EmergencyPhone => self.emergency_phone.value(),
            FormField::EmergencyRelationship => self.emergency_relationship.value(),
            FormField::ConcessionType => self
                .concession_type_dropdown
                .selected_value()
                .unwrap_or("")
                .to_string(),
            FormField::ConcessionNumber => self.concession_number.value(),
            FormField::PreferredLanguage => self.preferred_language.value(),
            FormField::InterpreterRequired => self
                .interpreter_required_dropdown
                .selected_value()
                .unwrap_or("No")
                .to_string(),
            FormField::AtsiStatus => self
                .atsi_status_dropdown
                .selected_value()
                .unwrap_or("")
                .to_string(),
        }
    }

    pub fn set_value(&mut self, field: FormField, value: String) {
        match field {
            FormField::Title => {
                self.title = single_line("Title").with_value(value);
            }
            FormField::FirstName => {
                self.first_name = single_line("First Name").with_value(value);
            }
            FormField::MiddleName => {
                self.middle_name = single_line("Middle Name").with_value(value);
            }
            FormField::LastName => {
                self.last_name = single_line("Last Name").with_value(value);
            }
            FormField::PreferredName => {
                self.preferred_name = single_line("Preferred Name").with_value(value);
            }
            FormField::DateOfBirth => {
                self.date_of_birth = single_line("Date of Birth").with_value(value);
            }
            FormField::Gender => {
                self.gender_dropdown.set_value(&value);
                if let Ok(gender) = value.parse() {
                    self.data.gender = gender;
                }
            }
            FormField::AddressLine1 => {
                self.address_line1 = single_line("Address Line 1").with_value(value);
            }
            FormField::AddressLine2 => {
                self.address_line2 = single_line("Address Line 2").with_value(value);
            }
            FormField::Suburb => {
                self.suburb = single_line("Suburb").with_value(value);
            }
            FormField::State => {
                self.state = single_line("State").with_value(value);
            }
            FormField::Postcode => {
                self.postcode = single_line("Postcode").with_value(value);
            }
            FormField::Country => {
                self.country = single_line("Country").with_value(value);
            }
            FormField::PhoneHome => {
                self.phone_home = single_line("Phone (Home)").with_value(value);
            }
            FormField::PhoneMobile => {
                self.phone_mobile = single_line("Phone (Mobile)").with_value(value);
            }
            FormField::Email => {
                self.email = single_line("Email").with_value(value);
            }
            FormField::MedicareNumber => {
                self.medicare_number = single_line("Medicare Number")
                    .max_length(10)
                    .with_value(value);
            }
            FormField::MedicareIrn => {
                self.medicare_irn = single_line("Medicare IRN").max_length(1).with_value(value);
            }
            FormField::MedicareExpiry => {
                self.medicare_expiry = single_line("Medicare Expiry").with_value(value);
            }
            FormField::Ihi => {
                self.ihi = single_line("IHI").with_value(value);
            }
            FormField::EmergencyName => {
                self.emergency_name = single_line("Emergency Contact Name").with_value(value);
            }
            FormField::EmergencyPhone => {
                self.emergency_phone = single_line("Emergency Contact Phone").with_value(value);
            }
            FormField::EmergencyRelationship => {
                self.emergency_relationship =
                    single_line("Emergency Contact Relationship").with_value(value);
            }
            FormField::ConcessionType => {
                self.concession_type_dropdown.set_value(&value);
                self.data.concession_type = value.parse().ok();
            }
            FormField::ConcessionNumber => {
                self.concession_number = single_line("Concession Number").with_value(value);
            }
            FormField::PreferredLanguage => {
                self.preferred_language = single_line("Preferred Language").with_value(value);
            }
            FormField::InterpreterRequired => {
                self.interpreter_required_dropdown.set_value(&value);
                self.data.interpreter_required = value == "Yes";
            }
            FormField::AtsiStatus => {
                self.atsi_status_dropdown.set_value(&value);
                self.data.aboriginal_torres_strait_islander = value.parse().ok();
            }
        }
        self.validate_field(&field);
    }

    fn focused_textarea_mut(&mut self) -> Option<&mut TextareaState> {
        match self.focused_field {
            FormField::Title => Some(&mut self.title),
            FormField::FirstName => Some(&mut self.first_name),
            FormField::MiddleName => Some(&mut self.middle_name),
            FormField::LastName => Some(&mut self.last_name),
            FormField::PreferredName => Some(&mut self.preferred_name),
            FormField::DateOfBirth => Some(&mut self.date_of_birth),
            FormField::AddressLine1 => Some(&mut self.address_line1),
            FormField::AddressLine2 => Some(&mut self.address_line2),
            FormField::Suburb => Some(&mut self.suburb),
            FormField::State => Some(&mut self.state),
            FormField::Postcode => Some(&mut self.postcode),
            FormField::Country => Some(&mut self.country),
            FormField::PhoneHome => Some(&mut self.phone_home),
            FormField::PhoneMobile => Some(&mut self.phone_mobile),
            FormField::Email => Some(&mut self.email),
            FormField::MedicareNumber => Some(&mut self.medicare_number),
            FormField::MedicareIrn => Some(&mut self.medicare_irn),
            FormField::MedicareExpiry => Some(&mut self.medicare_expiry),
            FormField::Ihi => Some(&mut self.ihi),
            FormField::EmergencyName => Some(&mut self.emergency_name),
            FormField::EmergencyPhone => Some(&mut self.emergency_phone),
            FormField::EmergencyRelationship => Some(&mut self.emergency_relationship),
            FormField::ConcessionNumber => Some(&mut self.concession_number),
            FormField::PreferredLanguage => Some(&mut self.preferred_language),
            FormField::Gender
            | FormField::ConcessionType
            | FormField::InterpreterRequired
            | FormField::AtsiStatus => None,
        }
    }

    fn textarea_for(&self, field: FormField) -> Option<&TextareaState> {
        match field {
            FormField::Title => Some(&self.title),
            FormField::FirstName => Some(&self.first_name),
            FormField::MiddleName => Some(&self.middle_name),
            FormField::LastName => Some(&self.last_name),
            FormField::PreferredName => Some(&self.preferred_name),
            FormField::DateOfBirth => Some(&self.date_of_birth),
            FormField::AddressLine1 => Some(&self.address_line1),
            FormField::AddressLine2 => Some(&self.address_line2),
            FormField::Suburb => Some(&self.suburb),
            FormField::State => Some(&self.state),
            FormField::Postcode => Some(&self.postcode),
            FormField::Country => Some(&self.country),
            FormField::PhoneHome => Some(&self.phone_home),
            FormField::PhoneMobile => Some(&self.phone_mobile),
            FormField::Email => Some(&self.email),
            FormField::MedicareNumber => Some(&self.medicare_number),
            FormField::MedicareIrn => Some(&self.medicare_irn),
            FormField::MedicareExpiry => Some(&self.medicare_expiry),
            FormField::Ihi => Some(&self.ihi),
            FormField::EmergencyName => Some(&self.emergency_name),
            FormField::EmergencyPhone => Some(&self.emergency_phone),
            FormField::EmergencyRelationship => Some(&self.emergency_relationship),
            FormField::ConcessionNumber => Some(&self.concession_number),
            FormField::PreferredLanguage => Some(&self.preferred_language),
            FormField::Gender
            | FormField::ConcessionType
            | FormField::InterpreterRequired
            | FormField::AtsiStatus => None,
        }
    }

    pub fn focused_field(&self) -> FormField {
        self.focused_field
    }

    pub fn set_focus(&mut self, field: FormField) {
        self.focused_field = field;
    }

    pub fn next_field(&mut self) {
        let fields = FormField::all();
        if let Some(current_idx) = fields.iter().position(|f| *f == self.focused_field) {
            let next_idx = (current_idx + 1) % fields.len();
            self.focused_field = fields[next_idx];
        }
    }

    pub fn prev_field(&mut self) {
        let fields = FormField::all();
        if let Some(current_idx) = fields.iter().position(|f| *f == self.focused_field) {
            let prev_idx = if current_idx == 0 {
                fields.len() - 1
            } else {
                current_idx - 1
            };
            self.focused_field = fields[prev_idx];
        }
    }

    fn get_field_position(&self, field: FormField) -> (u16, u16) {
        let fields = FormField::all();
        let mut y: u16 = 0;

        for f in fields {
            if f == field {
                return (y, self.get_field_height(f));
            }
            y += self.get_field_height(f) + 1;
        }

        (0, 0)
    }

    fn get_field_height(&self, field: FormField) -> u16 {
        match field {
            FormField::Gender
            | FormField::ConcessionType
            | FormField::AtsiStatus
            | FormField::InterpreterRequired => 4,
            _ => {
                if let Some(textarea) = self.textarea_for(field) {
                    textarea.height()
                } else {
                    1
                }
            }
        }
    }

    pub fn is_saving(&self) -> bool {
        self.saving
    }

    pub fn set_saving(&mut self, saving: bool) {
        self.saving = saving;
    }

    fn validate_field(&mut self, field: &FormField) {
        self.errors.remove(field);

        let value = self.get_value(*field);

        match field {
            FormField::FirstName | FormField::LastName => {
                if value.trim().is_empty() {
                    self.errors
                        .insert(*field, "This field is required".to_string());
                } else if value.len() > 100 {
                    self.errors
                        .insert(*field, "Maximum 100 characters".to_string());
                }
            }
            FormField::DateOfBirth => {
                if value.is_empty() {
                    self.errors
                        .insert(*field, "This field is required".to_string());
                } else if parse_date(&value).is_none() {
                    self.errors
                        .insert(*field, "Use dd/mm/yyyy format".to_string());
                }
            }
            FormField::Gender => {
                if value.is_empty() {
                    self.errors
                        .insert(*field, "This field is required".to_string());
                }
            }
            FormField::MedicareNumber => {
                if !value.is_empty() && value.len() != 10 {
                    self.errors
                        .insert(*field, "Medicare number must be 10 digits".to_string());
                } else if !value.chars().all(|c| c.is_ascii_digit()) {
                    self.errors.insert(
                        *field,
                        "Medicare number must contain only digits".to_string(),
                    );
                }
            }
            FormField::Email => {
                if !value.is_empty() && !value.contains('@') {
                    self.errors
                        .insert(*field, "Invalid email format".to_string());
                }
            }
            FormField::PhoneHome | FormField::PhoneMobile => {
                if !value.is_empty() {
                    let cleaned: String = value
                        .chars()
                        .filter(|c| {
                            c.is_ascii_digit() || *c == ' ' || *c == '-' || *c == '(' || *c == ')'
                        })
                        .collect();
                    if cleaned.len() < 8 {
                        self.errors
                            .insert(*field, "Invalid phone number".to_string());
                    }
                }
            }
            _ => {}
        }

        let error_msg = self.errors.get(field).cloned();
        match field {
            FormField::Title => self.title.set_error(error_msg),
            FormField::FirstName => self.first_name.set_error(error_msg),
            FormField::MiddleName => self.middle_name.set_error(error_msg),
            FormField::LastName => self.last_name.set_error(error_msg),
            FormField::PreferredName => self.preferred_name.set_error(error_msg),
            FormField::DateOfBirth => self.date_of_birth.set_error(error_msg),
            FormField::AddressLine1 => self.address_line1.set_error(error_msg),
            FormField::AddressLine2 => self.address_line2.set_error(error_msg),
            FormField::Suburb => self.suburb.set_error(error_msg),
            FormField::State => self.state.set_error(error_msg),
            FormField::Postcode => self.postcode.set_error(error_msg),
            FormField::Country => self.country.set_error(error_msg),
            FormField::PhoneHome => self.phone_home.set_error(error_msg),
            FormField::PhoneMobile => self.phone_mobile.set_error(error_msg),
            FormField::Email => self.email.set_error(error_msg),
            FormField::MedicareNumber => self.medicare_number.set_error(error_msg),
            FormField::MedicareIrn => self.medicare_irn.set_error(error_msg),
            FormField::MedicareExpiry => self.medicare_expiry.set_error(error_msg),
            FormField::Ihi => self.ihi.set_error(error_msg),
            FormField::EmergencyName => self.emergency_name.set_error(error_msg),
            FormField::EmergencyPhone => self.emergency_phone.set_error(error_msg),
            FormField::EmergencyRelationship => self.emergency_relationship.set_error(error_msg),
            FormField::ConcessionNumber => self.concession_number.set_error(error_msg),
            FormField::PreferredLanguage => self.preferred_language.set_error(error_msg),
            FormField::Gender
            | FormField::ConcessionType
            | FormField::InterpreterRequired
            | FormField::AtsiStatus => {}
        }
    }

    pub fn validate(&mut self) -> bool {
        self.errors.clear();

        for field in FormField::all() {
            self.validate_field(&field);
        }

        self.errors.is_empty()
    }

    pub fn error(&self, field: FormField) -> Option<&String> {
        self.errors.get(&field)
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn to_new_patient_data(&mut self) -> Option<NewPatientData> {
        if !self.validate() {
            return None;
        }

        let dob = parse_date(&self.get_value(FormField::DateOfBirth))?;
        let gender = self.get_value(FormField::Gender).parse().ok()?;

        let address = Address {
            line1: self.get_value(FormField::AddressLine1).empty_to_none(),
            line2: self.get_value(FormField::AddressLine2).empty_to_none(),
            suburb: self.get_value(FormField::Suburb).empty_to_none(),
            state: self.get_value(FormField::State).empty_to_none(),
            postcode: self.get_value(FormField::Postcode).empty_to_none(),
            country: or_default(self.get_value(FormField::Country), "Australia"),
        };

        let emergency_contact = if !self.get_value(FormField::EmergencyName).is_empty() {
            Some(EmergencyContact {
                name: self.get_value(FormField::EmergencyName),
                phone: self.get_value(FormField::EmergencyPhone),
                relationship: self.get_value(FormField::EmergencyRelationship),
            })
        } else {
            None
        };

        let concession_type = self.get_value(FormField::ConcessionType).parse().ok();
        let atsi_status = self.get_value(FormField::AtsiStatus).parse().ok();

        Some(NewPatientData {
            ihi: self.get_value(FormField::Ihi).empty_to_none(),
            medicare_number: self.get_value(FormField::MedicareNumber).empty_to_none(),
            medicare_irn: self.get_value(FormField::MedicareIrn).parse().ok(),
            medicare_expiry: parse_date(&self.get_value(FormField::MedicareExpiry)),
            title: self.get_value(FormField::Title).empty_to_none(),
            first_name: self.get_value(FormField::FirstName),
            middle_name: self.get_value(FormField::MiddleName).empty_to_none(),
            last_name: self.get_value(FormField::LastName),
            preferred_name: self.get_value(FormField::PreferredName).empty_to_none(),
            date_of_birth: dob,
            gender,
            address,
            phone_home: self.get_value(FormField::PhoneHome).empty_to_none(),
            phone_mobile: self.get_value(FormField::PhoneMobile).empty_to_none(),
            email: self.get_value(FormField::Email).empty_to_none(),
            emergency_contact,
            concession_type,
            concession_number: self.get_value(FormField::ConcessionNumber).empty_to_none(),
            preferred_language: Some(self.get_value(FormField::PreferredLanguage)),
            interpreter_required: Some(self.get_value(FormField::InterpreterRequired) == "Yes"),
            aboriginal_torres_strait_islander: atsi_status,
        })
    }

    pub fn to_update_patient_data(
        &mut self,
    ) -> Option<(Uuid, crate::domain::patient::UpdatePatientData)> {
        let patient_id = self.patient_id()?;

        if !self.validate() {
            return None;
        }

        let dob = parse_date(&self.get_value(FormField::DateOfBirth));
        let gender = self.get_value(FormField::Gender).parse().ok();

        let address = Address {
            line1: self.get_value(FormField::AddressLine1).empty_to_none(),
            line2: self.get_value(FormField::AddressLine2).empty_to_none(),
            suburb: self.get_value(FormField::Suburb).empty_to_none(),
            state: self.get_value(FormField::State).empty_to_none(),
            postcode: self.get_value(FormField::Postcode).empty_to_none(),
            country: or_default(self.get_value(FormField::Country), "Australia"),
        };

        let emergency_contact = if !self.get_value(FormField::EmergencyName).is_empty() {
            Some(EmergencyContact {
                name: self.get_value(FormField::EmergencyName),
                phone: self.get_value(FormField::EmergencyPhone),
                relationship: self.get_value(FormField::EmergencyRelationship),
            })
        } else {
            None
        };

        let concession_type = self.get_value(FormField::ConcessionType).parse().ok();
        let atsi_status = self.get_value(FormField::AtsiStatus).parse().ok();

        let data = crate::domain::patient::UpdatePatientData {
            ihi: self.get_value(FormField::Ihi).empty_to_none(),
            medicare_number: self.get_value(FormField::MedicareNumber).empty_to_none(),
            medicare_irn: self.get_value(FormField::MedicareIrn).parse().ok(),
            medicare_expiry: parse_date(&self.get_value(FormField::MedicareExpiry)),
            title: self.get_value(FormField::Title).empty_to_none(),
            first_name: Some(self.get_value(FormField::FirstName)),
            middle_name: self.get_value(FormField::MiddleName).empty_to_none(),
            last_name: Some(self.get_value(FormField::LastName)),
            preferred_name: self.get_value(FormField::PreferredName).empty_to_none(),
            date_of_birth: dob,
            gender,
            address: Some(address),
            phone_home: self.get_value(FormField::PhoneHome).empty_to_none(),
            phone_mobile: self.get_value(FormField::PhoneMobile).empty_to_none(),
            email: self.get_value(FormField::Email).empty_to_none(),
            emergency_contact,
            concession_type,
            concession_number: self.get_value(FormField::ConcessionNumber).empty_to_none(),
            preferred_language: Some(self.get_value(FormField::PreferredLanguage)),
            interpreter_required: Some(self.get_value(FormField::InterpreterRequired) == "Yes"),
            aboriginal_torres_strait_islander: atsi_status,
        };

        Some((patient_id, data))
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Option<PatientFormAction> {
        use crossterm::event::{KeyCode, KeyEventKind};

        // Ignore non-press key events (e.g., Release events from terminals with keyboard enhancement)
        if key.kind != KeyEventKind::Press {
            return None;
        }

        if self.saving {
            return None;
        }

        // Ctrl+Enter submits the form from any field.
        if key
            .modifiers
            .contains(crossterm::event::KeyModifiers::CONTROL)
            && key.code == KeyCode::Enter
        {
            self.validate();
            return Some(PatientFormAction::Submit);
        }

        if let Some(dropdown_action) = self.handle_dropdown_key(key) {
            return dropdown_action;
        }

        if !self.focused_field.is_dropdown() {
            let ratatui_key = to_ratatui_key(key);
            if let Some(textarea) = self.focused_textarea_mut() {
                let consumed = textarea.handle_key(ratatui_key);
                if consumed {
                    let field = self.focused_field;
                    self.validate_field(&field);
                    return Some(PatientFormAction::ValueChanged);
                }
            }
        }

        match key.code {
            KeyCode::Tab => {
                if key
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::SHIFT)
                {
                    self.prev_field();
                } else {
                    self.next_field();
                }
                Some(PatientFormAction::FocusChanged)
            }
            KeyCode::BackTab => {
                self.prev_field();
                Some(PatientFormAction::FocusChanged)
            }
            KeyCode::Up => {
                self.prev_field();
                Some(PatientFormAction::FocusChanged)
            }
            KeyCode::Down => {
                self.next_field();
                Some(PatientFormAction::FocusChanged)
            }
            KeyCode::PageUp => {
                self.scroll.scroll_up();
                Some(PatientFormAction::ValueChanged)
            }
            KeyCode::PageDown => {
                self.scroll.scroll_down();
                Some(PatientFormAction::ValueChanged)
            }
            KeyCode::Enter => {
                self.validate();
                Some(PatientFormAction::Submit)
            }
            KeyCode::Esc => Some(PatientFormAction::Cancel),
            _ => None,
        }
    }

    fn handle_dropdown_key(&mut self, key: KeyEvent) -> Option<Option<PatientFormAction>> {
        let dropdown: Option<(&mut DropdownWidget, FormField)> = match self.focused_field {
            FormField::Gender => Some((&mut self.gender_dropdown, FormField::Gender)),
            FormField::ConcessionType => Some((
                &mut self.concession_type_dropdown,
                FormField::ConcessionType,
            )),
            FormField::AtsiStatus => Some((&mut self.atsi_status_dropdown, FormField::AtsiStatus)),
            FormField::InterpreterRequired => Some((
                &mut self.interpreter_required_dropdown,
                FormField::InterpreterRequired,
            )),
            _ => None,
        };

        if let Some((dropdown, field)) = dropdown {
            if let Some(action) = dropdown.handle_key(key) {
                match action {
                    DropdownAction::Selected(_) | DropdownAction::Closed => {
                        let value = dropdown.selected_value().map(|v| v.to_string());
                        if let Some(v) = value {
                            self.set_value(field, v);
                        }
                        return Some(Some(PatientFormAction::ValueChanged));
                    }
                    DropdownAction::Opened | DropdownAction::FocusChanged => {
                        return Some(Some(PatientFormAction::ValueChanged));
                    }
                }
            }
            return Some(None);
        }

        None
    }

    pub fn handle_mouse(&mut self, mouse: MouseEvent, area: Rect) -> Option<PatientFormAction> {
        if mouse.kind != MouseEventKind::Up(crossterm::event::MouseButton::Left) {
            return None;
        }

        if !area.contains(Position::new(mouse.column, mouse.row)) {
            return None;
        }

        let click_pos = Position::new(mouse.column, mouse.row);

        let inner = Rect::new(area.x + 1, area.y + 1, area.width - 2, area.height - 2);
        if !inner.contains(click_pos) {
            return None;
        }

        let fields: Vec<FormField> = FormField::all();
        let mut y = inner.y + 1;
        let max_y = inner.y + inner.height - 2;

        for field in &fields {
            if y > max_y {
                break;
            }

            let field_height: u16 = 3;
            let field_area = Rect::new(inner.x + 1, y, inner.width - 2, field_height);

            if field_area.contains(click_pos) {
                if *field != self.focused_field {
                    self.focused_field = *field;
                    return Some(PatientFormAction::FocusChanged);
                }
                return None;
            }

            y += field_height + 1;
        }

        None
    }
}

trait EmptyToNone {
    fn empty_to_none(self) -> Option<String>;
}

impl EmptyToNone for String {
    fn empty_to_none(self) -> Option<String> {
        if self.is_empty() {
            None
        } else {
            Some(self)
        }
    }
}

fn or_default(s: String, default: &str) -> String {
    if s.is_empty() {
        default.to_string()
    } else {
        s
    }
}

#[derive(Debug, Clone)]
pub enum PatientFormAction {
    FocusChanged,
    ValueChanged,
    Submit,
    Cancel,
    SaveComplete,
}

impl Widget for PatientForm {
    fn render(mut self, area: Rect, buf: &mut Buffer) {
        if area.is_empty() {
            return;
        }

        let block = Block::default()
            .title(if self.is_edit_mode() {
                " Edit Patient "
            } else {
                " New Patient "
            })
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.colors.border));

        block.clone().render(area, buf);

        let inner = block.inner(area);
        if inner.is_empty() {
            return;
        }

        let label_width = LABEL_WIDTH;
        let field_start = inner.x + label_width + 2;

        let fields: Vec<FormField> = FormField::all();

        // Calculate total content height first
        let mut total_height: u16 = 0;
        for field in &fields {
            total_height += self.get_field_height(*field) + 1;
        }
        self.scroll.set_total_height(total_height);
        self.scroll.clamp_offset(inner.height.saturating_sub(2));

        // Scroll to focused field if needed
        let (focused_y, focused_height) = self.get_field_position(self.focused_field);
        self.scroll
            .scroll_to_field(focused_y, focused_height, inner.height.saturating_sub(2));

        let mut y: i32 = (inner.y as i32) + 1 - (self.scroll.scroll_offset as i32);
        let max_y = inner.y as i32 + inner.height as i32 - 2;

        for field in fields {
            let field_height = self.get_field_height(field) as i32;

            // Skip fields outside viewport
            if y + field_height <= inner.y as i32 || y >= max_y {
                y += field_height + 1;
                continue;
            }

            let is_focused = field == self.focused_field;

            match field {
                FormField::Gender => {
                    let dropdown = self.gender_dropdown.clone();
                    if y >= inner.y as i32 && y < max_y {
                        let dropdown_area = Rect::new(
                            field_start,
                            y as u16,
                            inner.width.saturating_sub(label_width + 4),
                            3,
                        );
                        dropdown.focused(is_focused).render(dropdown_area, buf);
                    }
                    y += 4;
                }
                FormField::ConcessionType => {
                    let dropdown = self.concession_type_dropdown.clone();
                    if y >= inner.y as i32 && y < max_y {
                        let dropdown_area = Rect::new(
                            field_start,
                            y as u16,
                            inner.width.saturating_sub(label_width + 4),
                            3,
                        );
                        dropdown.focused(is_focused).render(dropdown_area, buf);
                    }
                    y += 4;
                }
                FormField::AtsiStatus => {
                    let dropdown = self.atsi_status_dropdown.clone();
                    if y >= inner.y as i32 && y < max_y {
                        let dropdown_area = Rect::new(
                            field_start,
                            y as u16,
                            inner.width.saturating_sub(label_width + 4),
                            3,
                        );
                        dropdown.focused(is_focused).render(dropdown_area, buf);
                    }
                    y += 4;
                }
                FormField::InterpreterRequired => {
                    let dropdown = self.interpreter_required_dropdown.clone();
                    if y >= inner.y as i32 && y < max_y {
                        let dropdown_area = Rect::new(
                            field_start,
                            y as u16,
                            inner.width.saturating_sub(label_width + 4),
                            3,
                        );
                        dropdown.focused(is_focused).render(dropdown_area, buf);
                    }
                    y += 4;
                }
                _ => {
                    if let Some(textarea) = self.textarea_for(field) {
                        let textarea_height = textarea.height() as i32;
                        if y >= inner.y as i32 && y < max_y {
                            let textarea_area = Rect::new(
                                inner.x + 1,
                                y as u16,
                                inner.width - 2,
                                textarea_height as u16,
                            );
                            TextareaWidget::new(textarea, self.theme.clone())
                                .focused(is_focused)
                                .render(textarea_area, buf);
                        }
                        y += textarea_height + 1;
                    }
                }
            }
        }

        // Render scrollbar
        self.scroll.render_scrollbar(inner, buf);

        let help_y = inner.y + inner.height - 1;
        buf.set_string(
            inner.x + 1,
            help_y,
            "Tab: Next | Ctrl+Enter: Submit | Esc: Cancel",
            Style::default().fg(self.theme.colors.disabled),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_form_creation() {
        let theme = Theme::dark();
        let form = PatientForm::new(theme);

        assert!(!form.is_edit_mode());
        assert_eq!(form.focused_field(), FormField::FirstName);
        assert!(!form.has_errors());
    }

    #[test]
    fn test_form_validation_required() {
        let theme = Theme::dark();
        let mut form = PatientForm::new(theme);

        form.validate();
        assert!(form.has_errors());
        assert!(form.error(FormField::FirstName).is_some());
        assert!(form.error(FormField::LastName).is_some());
    }

    #[test]
    fn test_form_validation_email() {
        let theme = Theme::dark();
        let mut form = PatientForm::new(theme);

        form.set_value(FormField::Email, "invalid".to_string());
        form.validate();
        assert!(form.error(FormField::Email).is_some());

        form.set_value(FormField::Email, "test@example.com".to_string());
        form.validate();
        assert!(form.error(FormField::Email).is_none());
    }

    #[test]
    fn test_text_fields_use_textarea_state() {
        let theme = Theme::dark();
        let mut form = PatientForm::new(theme);

        form.set_value(FormField::FirstName, "Alice".to_string());
        assert_eq!(form.get_value(FormField::FirstName), "Alice");
        assert_eq!(form.first_name.value(), "Alice");
    }

    #[test]
    fn test_handle_key_char_updates_textarea() {
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

        let theme = Theme::dark();
        let mut form = PatientForm::new(theme);
        form.focused_field = FormField::FirstName;

        let key = KeyEvent::new(KeyCode::Char('J'), KeyModifiers::NONE);
        let action = form.handle_key(key);
        assert!(action.is_some());
        assert_eq!(form.get_value(FormField::FirstName), "J");
    }

    #[test]
    fn test_handle_key_tab_navigates_fields() {
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

        let theme = Theme::dark();
        let mut form = PatientForm::new(theme);
        assert_eq!(form.focused_field(), FormField::FirstName);

        let key = KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE);
        form.handle_key(key);
        assert_eq!(form.focused_field(), FormField::MiddleName);
    }

    #[test]
    fn test_single_line_height_mode() {
        let theme = Theme::dark();
        let form = PatientForm::new(theme);

        assert_eq!(form.first_name.height_mode, HeightMode::SingleLine);
        assert_eq!(form.last_name.height_mode, HeightMode::SingleLine);
        assert_eq!(form.email.height_mode, HeightMode::SingleLine);
        assert_eq!(form.medicare_number.height_mode, HeightMode::SingleLine);
    }

    #[test]
    fn test_error_synced_to_textarea_state() {
        let theme = Theme::dark();
        let mut form = PatientForm::new(theme);

        form.validate();
        assert!(form.first_name.error.is_some());
        assert!(form.last_name.error.is_some());
    }
}

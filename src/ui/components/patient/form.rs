//! Patient Form Component
//!
//! Comprehensive form for creating and editing patients.

use std::collections::HashMap;

use chrono::NaiveDate;
use crossterm::event::{KeyEvent, MouseEvent, MouseEventKind};
use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Borders, Widget};
use uuid::Uuid;

use crate::domain::patient::{Address, EmergencyContact, NewPatientData, Patient};
use crate::ui::layout::LABEL_WIDTH;
use crate::ui::theme::Theme;
use crate::ui::view_models::PatientFormData;
use crate::ui::widgets::{DropdownAction, DropdownOption, DropdownWidget};

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
            FormField::DateOfBirth => "Date of Birth *",
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
            FormField::MedicareExpiry => "Medicare Expiry",
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
}

pub struct PatientForm {
    mode: FormMode,
    data: PatientFormData,
    errors: HashMap<FormField, String>,
    focused_field: FormField,
    saving: bool,
    theme: Theme,
    gender_dropdown: DropdownWidget,
    concession_type_dropdown: DropdownWidget,
    atsi_status_dropdown: DropdownWidget,
    interpreter_required_dropdown: DropdownWidget,
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
            gender_dropdown: self.gender_dropdown.clone(),
            concession_type_dropdown: self.concession_type_dropdown.clone(),
            atsi_status_dropdown: self.atsi_status_dropdown.clone(),
            interpreter_required_dropdown: self.interpreter_required_dropdown.clone(),
        }
    }
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
            gender_dropdown,
            concession_type_dropdown,
            atsi_status_dropdown,
            interpreter_required_dropdown,
        }
    }

    pub fn from_patient(patient: Patient, theme: Theme) -> Self {
        let gender = patient.gender;
        let concession_type = patient.concession_type;
        let atsi_status = patient.aboriginal_torres_strait_islander;
        let interpreter_required = patient.interpreter_required;

        let mut form = Self::new(theme);
        form.mode = FormMode::Edit(patient.id);
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

    pub fn set_value(&mut self, field: FormField, value: String) {
        match field {
            FormField::Title => self.data.title = if value.is_empty() { None } else { Some(value) },
            FormField::FirstName => self.data.first_name = value,
            FormField::MiddleName => {
                self.data.middle_name = if value.is_empty() { None } else { Some(value) }
            }
            FormField::LastName => self.data.last_name = value,
            FormField::PreferredName => {
                self.data.preferred_name = if value.is_empty() { None } else { Some(value) }
            }
            FormField::DateOfBirth => {
                if let Ok(dob) = NaiveDate::parse_from_str(&value, "%Y-%m-%d") {
                    self.data.date_of_birth = dob;
                }
            }
            FormField::Gender => {
                self.gender_dropdown.set_value(&value);
                if let Ok(gender) = value.parse() {
                    self.data.gender = gender;
                }
            }
            FormField::AddressLine1 => {
                self.data.address_line1 = if value.is_empty() { None } else { Some(value) }
            }
            FormField::AddressLine2 => {
                self.data.address_line2 = if value.is_empty() { None } else { Some(value) }
            }
            FormField::Suburb => {
                self.data.suburb = if value.is_empty() { None } else { Some(value) }
            }
            FormField::State => self.data.state = if value.is_empty() { None } else { Some(value) },
            FormField::Postcode => {
                self.data.postcode = if value.is_empty() { None } else { Some(value) }
            }
            FormField::Country => {
                self.data.country = if value.is_empty() { None } else { Some(value) }
            }
            FormField::PhoneHome => {
                self.data.phone_home = if value.is_empty() { None } else { Some(value) }
            }
            FormField::PhoneMobile => {
                self.data.phone_mobile = if value.is_empty() { None } else { Some(value) }
            }
            FormField::Email => self.data.email = if value.is_empty() { None } else { Some(value) },
            FormField::MedicareNumber => {
                self.data.medicare_number = if value.is_empty() { None } else { Some(value) }
            }
            FormField::MedicareIrn => self.data.medicare_irn = value.parse().ok(),
            FormField::MedicareExpiry => {
                self.data.medicare_expiry = NaiveDate::parse_from_str(&value, "%Y-%m-%d").ok();
            }
            FormField::Ihi => self.data.ihi = if value.is_empty() { None } else { Some(value) },
            FormField::EmergencyName => {
                self.data.emergency_contact_name = if value.is_empty() { None } else { Some(value) }
            }
            FormField::EmergencyPhone => {
                self.data.emergency_contact_phone =
                    if value.is_empty() { None } else { Some(value) }
            }
            FormField::EmergencyRelationship => {
                self.data.emergency_contact_relationship =
                    if value.is_empty() { None } else { Some(value) }
            }
            FormField::ConcessionType => {
                self.concession_type_dropdown.set_value(&value);
                self.data.concession_type = value.parse().ok();
            }
            FormField::ConcessionNumber => {
                self.data.concession_number = if value.is_empty() { None } else { Some(value) }
            }
            FormField::PreferredLanguage => {
                self.data.preferred_language = if value.is_empty() { None } else { Some(value) }
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

    pub fn get_value(&self, field: FormField) -> String {
        match field {
            FormField::Title => self.data.title.clone().unwrap_or_default(),
            FormField::FirstName => self.data.first_name.clone(),
            FormField::MiddleName => self.data.middle_name.clone().unwrap_or_default(),
            FormField::LastName => self.data.last_name.clone(),
            FormField::PreferredName => self.data.preferred_name.clone().unwrap_or_default(),
            FormField::DateOfBirth => self.data.date_of_birth.format("%Y-%m-%d").to_string(),
            FormField::Gender => self
                .gender_dropdown
                .selected_value()
                .unwrap_or("")
                .to_string(),
            FormField::AddressLine1 => self.data.address_line1.clone().unwrap_or_default(),
            FormField::AddressLine2 => self.data.address_line2.clone().unwrap_or_default(),
            FormField::Suburb => self.data.suburb.clone().unwrap_or_default(),
            FormField::State => self.data.state.clone().unwrap_or_default(),
            FormField::Postcode => self.data.postcode.clone().unwrap_or_default(),
            FormField::Country => self.data.country.clone().unwrap_or_default(),
            FormField::PhoneHome => self.data.phone_home.clone().unwrap_or_default(),
            FormField::PhoneMobile => self.data.phone_mobile.clone().unwrap_or_default(),
            FormField::Email => self.data.email.clone().unwrap_or_default(),
            FormField::MedicareNumber => self.data.medicare_number.clone().unwrap_or_default(),
            FormField::MedicareIrn => self
                .data
                .medicare_irn
                .map(|n| n.to_string())
                .unwrap_or_default(),
            FormField::MedicareExpiry => self
                .data
                .medicare_expiry
                .map(|d| d.format("%Y-%m-%d").to_string())
                .unwrap_or_default(),
            FormField::Ihi => self.data.ihi.clone().unwrap_or_default(),
            FormField::EmergencyName => {
                self.data.emergency_contact_name.clone().unwrap_or_default()
            }
            FormField::EmergencyPhone => self
                .data
                .emergency_contact_phone
                .clone()
                .unwrap_or_default(),
            FormField::EmergencyRelationship => self
                .data
                .emergency_contact_relationship
                .clone()
                .unwrap_or_default(),
            FormField::ConcessionType => self
                .concession_type_dropdown
                .selected_value()
                .unwrap_or("")
                .to_string(),
            FormField::ConcessionNumber => self.data.concession_number.clone().unwrap_or_default(),
            FormField::PreferredLanguage => {
                self.data.preferred_language.clone().unwrap_or_default()
            }
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
                } else if NaiveDate::parse_from_str(&value, "%Y-%m-%d").is_err() {
                    self.errors
                        .insert(*field, "Use YYYY-MM-DD format".to_string());
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

        let dob =
            NaiveDate::parse_from_str(&self.get_value(FormField::DateOfBirth), "%Y-%m-%d").ok()?;
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
            medicare_expiry: NaiveDate::parse_from_str(
                &self.get_value(FormField::MedicareExpiry),
                "%Y-%m-%d",
            )
            .ok(),
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

        let dob =
            NaiveDate::parse_from_str(&self.get_value(FormField::DateOfBirth), "%Y-%m-%d").ok();
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
            medicare_expiry: NaiveDate::parse_from_str(
                &self.get_value(FormField::MedicareExpiry),
                "%Y-%m-%d",
            )
            .ok(),
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
        use crossterm::event::KeyCode;

        if self.saving {
            return None;
        }

        if let Some(dropdown_action) = self.handle_dropdown_key(key) {
            return dropdown_action;
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
            KeyCode::Up | KeyCode::Down => {
                self.handle_field_navigation(key.code);
                Some(PatientFormAction::FocusChanged)
            }
            KeyCode::Enter => {
                self.validate();
                Some(PatientFormAction::Submit)
            }
            KeyCode::Esc => Some(PatientFormAction::Cancel),
            KeyCode::Char(c) => {
                let mut value = self.get_value(self.focused_field);
                value.push(c);
                self.set_value(self.focused_field, value);
                Some(PatientFormAction::ValueChanged)
            }
            KeyCode::Backspace => {
                let mut value = self.get_value(self.focused_field);
                value.pop();
                self.set_value(self.focused_field, value);
                Some(PatientFormAction::ValueChanged)
            }
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

    fn handle_field_navigation(&mut self, code: crossterm::event::KeyCode) {
        match code {
            crossterm::event::KeyCode::Up => {
                self.prev_field();
            }
            crossterm::event::KeyCode::Down => {
                self.next_field();
            }
            _ => {}
        }
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

        let label_width = LABEL_WIDTH;
        let field_start = inner.x + label_width + 2;

        let fields: Vec<FormField> = FormField::all();
        let mut y = inner.y + 1;
        let max_y = inner.y + inner.height - 2;

        for field in &fields {
            if y > max_y {
                break;
            }

            let field_height = if self.error(*field).is_some() { 2 } else { 1 };
            let field_area = Rect::new(
                field_start,
                y,
                inner.width.saturating_sub(label_width + 4),
                field_height,
            );

            if field_area.contains(click_pos) {
                if *field != self.focused_field {
                    self.focused_field = *field;
                    return Some(PatientFormAction::FocusChanged);
                }
                return None;
            }

            y += 2;
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
    fn render(self, area: Rect, buf: &mut Buffer) {
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

        let mut y = inner.y + 1;
        let max_y = inner.y + inner.height - 2;

        for field in fields {
            if y > max_y {
                break;
            }

            let is_focused = field == self.focused_field;
            let has_error = self.error(field).is_some();

            let label_style = if is_focused {
                Style::default()
                    .fg(self.theme.colors.primary)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(self.theme.colors.foreground)
            };

            buf.set_string(inner.x + 1, y, field.label(), label_style);

            if is_focused {
                buf.set_string(
                    field_start - 1,
                    y,
                    ">",
                    Style::default().fg(self.theme.colors.primary),
                );
            }

            match field {
                FormField::Gender => {
                    let mut dropdown = self.gender_dropdown.clone();
                    let dropdown_area = Rect::new(
                        field_start,
                        y,
                        inner.width.saturating_sub(label_width + 4),
                        3,
                    );
                    dropdown.render(dropdown_area, buf);
                }
                FormField::ConcessionType => {
                    let mut dropdown = self.concession_type_dropdown.clone();
                    let dropdown_area = Rect::new(
                        field_start,
                        y,
                        inner.width.saturating_sub(label_width + 4),
                        3,
                    );
                    dropdown.render(dropdown_area, buf);
                }
                FormField::AtsiStatus => {
                    let mut dropdown = self.atsi_status_dropdown.clone();
                    let dropdown_area = Rect::new(
                        field_start,
                        y,
                        inner.width.saturating_sub(label_width + 4),
                        3,
                    );
                    dropdown.render(dropdown_area, buf);
                }
                FormField::InterpreterRequired => {
                    let mut dropdown = self.interpreter_required_dropdown.clone();
                    let dropdown_area = Rect::new(
                        field_start,
                        y,
                        inner.width.saturating_sub(label_width + 4),
                        3,
                    );
                    dropdown.render(dropdown_area, buf);
                }
                _ => {
                    let value = self.get_value(field);
                    let value_style = if has_error {
                        Style::default().fg(self.theme.colors.error)
                    } else {
                        Style::default().fg(self.theme.colors.foreground)
                    };

                    let max_value_width = inner.width.saturating_sub(label_width + 4);
                    let display_value = if value.len() > max_value_width as usize {
                        &value[value.len() - max_value_width as usize..]
                    } else {
                        &value
                    };

                    buf.set_string(field_start, y, display_value, value_style);

                    if let Some(error_msg) = self.error(field) {
                        let error_style = Style::default().fg(self.theme.colors.error);
                        buf.set_string(field_start, y + 1, format!("  {}", error_msg), error_style);
                        y += 1;
                    }
                }
            }

            y += 2;
        }

        let help_y = inner.y + inner.height - 1;
        buf.set_string(
            inner.x + 1,
            help_y,
            "Tab: Next | Enter: Submit | Esc: Cancel",
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
}

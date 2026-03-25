//! Patient Form Component
//!
//! Comprehensive form for creating and editing patients.

use std::collections::HashMap;

use crossterm::event::{KeyCode, KeyEvent, MouseEvent, MouseEventKind};
use opengp_config::forms::{
    FieldDefinition, FieldType as ConfigFieldType, FormConfig, ValidationRules,
};
use opengp_domain::domain::patient::{Address, EmergencyContact, NewPatientData, Patient};
use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};
use ratatui::style::Style;
use ratatui::widgets::{Block, Borders, Widget};
use uuid::Uuid;

use crate::ui::input::to_ratatui_key;
use crate::ui::layout::LABEL_WIDTH;
use crate::ui::theme::Theme;
use crate::ui::view_models::PatientFormData;
use crate::ui::widgets::{
    format_date, parse_date, DatePickerAction, DatePickerPopup, DropdownAction, DropdownOption,
    DropdownWidget, FormFieldMeta, FormNavigation, FormValidator, HeightMode, ScrollableFormState,
    TextareaState, TextareaWidget,
};

const FIELD_TITLE: &str = "title";
const FIELD_FIRST_NAME: &str = "first_name";
const FIELD_MIDDLE_NAME: &str = "middle_name";
const FIELD_LAST_NAME: &str = "last_name";
const FIELD_PREFERRED_NAME: &str = "preferred_name";
const FIELD_DATE_OF_BIRTH: &str = "date_of_birth";
const FIELD_GENDER: &str = "gender";
const FIELD_ADDRESS_LINE1: &str = "address_line1";
const FIELD_ADDRESS_LINE2: &str = "address_line2";
const FIELD_SUBURB: &str = "suburb";
const FIELD_STATE: &str = "state";
const FIELD_POSTCODE: &str = "postcode";
const FIELD_COUNTRY: &str = "country";
const FIELD_PHONE_HOME: &str = "phone_home";
const FIELD_PHONE_MOBILE: &str = "phone_mobile";
const FIELD_EMAIL: &str = "email";
const FIELD_MEDICARE_NUMBER: &str = "medicare_number";
const FIELD_MEDICARE_IRN: &str = "medicare_irn";
const FIELD_MEDICARE_EXPIRY: &str = "medicare_expiry";
const FIELD_IHI: &str = "ihi";
const FIELD_EMERGENCY_NAME: &str = "emergency_name";
const FIELD_EMERGENCY_PHONE: &str = "emergency_phone";
const FIELD_EMERGENCY_RELATIONSHIP: &str = "emergency_relationship";
const FIELD_CONCESSION_TYPE: &str = "concession_type";
const FIELD_CONCESSION_NUMBER: &str = "concession_number";
const FIELD_PREFERRED_LANGUAGE: &str = "preferred_language";
const FIELD_INTERPRETER_REQUIRED: &str = "interpreter_required";
const FIELD_ATSI_STATUS: &str = "atsi_status";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FormMode {
    #[default]
    Create,
    Edit(Uuid),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, strum::EnumIter, strum::IntoStaticStr)]
pub enum FormField {
    #[strum(to_string = "Title")]
    Title,
    #[strum(to_string = "First Name *")]
    FirstName,
    #[strum(to_string = "Middle Name")]
    MiddleName,
    #[strum(to_string = "Last Name *")]
    LastName,
    #[strum(to_string = "Preferred Name")]
    PreferredName,
    #[strum(to_string = "Date of Birth * (dd/mm/yyyy)")]
    DateOfBirth,
    #[strum(to_string = "Gender *")]
    Gender,
    #[strum(to_string = "Address Line 1")]
    AddressLine1,
    #[strum(to_string = "Address Line 2")]
    AddressLine2,
    #[strum(to_string = "Suburb")]
    Suburb,
    #[strum(to_string = "State")]
    State,
    #[strum(to_string = "Postcode")]
    Postcode,
    #[strum(to_string = "Country")]
    Country,
    #[strum(to_string = "Phone (Home)")]
    PhoneHome,
    #[strum(to_string = "Phone (Mobile)")]
    PhoneMobile,
    #[strum(to_string = "Email")]
    Email,
    #[strum(to_string = "Medicare Number")]
    MedicareNumber,
    #[strum(to_string = "Medicare IRN")]
    MedicareIrn,
    #[strum(to_string = "Medicare Expiry (dd/mm/yyyy)")]
    MedicareExpiry,
    #[strum(to_string = "IHI")]
    Ihi,
    #[strum(to_string = "Emergency Contact Name")]
    EmergencyName,
    #[strum(to_string = "Emergency Contact Phone")]
    EmergencyPhone,
    #[strum(to_string = "Emergency Contact Relationship")]
    EmergencyRelationship,
    #[strum(to_string = "Concession Type")]
    ConcessionType,
    #[strum(to_string = "Concession Number")]
    ConcessionNumber,
    #[strum(to_string = "Preferred Language")]
    PreferredLanguage,
    #[strum(to_string = "Interpreter Required")]
    InterpreterRequired,
    #[strum(to_string = "ATSI Status")]
    AtsiStatus,
}

impl FormField {
    pub fn all() -> Vec<FormField> {
        use strum::IntoEnumIterator;
        FormField::iter().collect()
    }

    pub fn label(&self) -> &'static str {
        (*self).into()
    }

    pub fn id(&self) -> &'static str {
        match self {
            FormField::Title => FIELD_TITLE,
            FormField::FirstName => FIELD_FIRST_NAME,
            FormField::MiddleName => FIELD_MIDDLE_NAME,
            FormField::LastName => FIELD_LAST_NAME,
            FormField::PreferredName => FIELD_PREFERRED_NAME,
            FormField::DateOfBirth => FIELD_DATE_OF_BIRTH,
            FormField::Gender => FIELD_GENDER,
            FormField::AddressLine1 => FIELD_ADDRESS_LINE1,
            FormField::AddressLine2 => FIELD_ADDRESS_LINE2,
            FormField::Suburb => FIELD_SUBURB,
            FormField::State => FIELD_STATE,
            FormField::Postcode => FIELD_POSTCODE,
            FormField::Country => FIELD_COUNTRY,
            FormField::PhoneHome => FIELD_PHONE_HOME,
            FormField::PhoneMobile => FIELD_PHONE_MOBILE,
            FormField::Email => FIELD_EMAIL,
            FormField::MedicareNumber => FIELD_MEDICARE_NUMBER,
            FormField::MedicareIrn => FIELD_MEDICARE_IRN,
            FormField::MedicareExpiry => FIELD_MEDICARE_EXPIRY,
            FormField::Ihi => FIELD_IHI,
            FormField::EmergencyName => FIELD_EMERGENCY_NAME,
            FormField::EmergencyPhone => FIELD_EMERGENCY_PHONE,
            FormField::EmergencyRelationship => FIELD_EMERGENCY_RELATIONSHIP,
            FormField::ConcessionType => FIELD_CONCESSION_TYPE,
            FormField::ConcessionNumber => FIELD_CONCESSION_NUMBER,
            FormField::PreferredLanguage => FIELD_PREFERRED_LANGUAGE,
            FormField::InterpreterRequired => FIELD_INTERPRETER_REQUIRED,
            FormField::AtsiStatus => FIELD_ATSI_STATUS,
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            FIELD_TITLE => Some(FormField::Title),
            FIELD_FIRST_NAME => Some(FormField::FirstName),
            FIELD_MIDDLE_NAME => Some(FormField::MiddleName),
            FIELD_LAST_NAME => Some(FormField::LastName),
            FIELD_PREFERRED_NAME => Some(FormField::PreferredName),
            FIELD_DATE_OF_BIRTH => Some(FormField::DateOfBirth),
            FIELD_GENDER => Some(FormField::Gender),
            FIELD_ADDRESS_LINE1 => Some(FormField::AddressLine1),
            FIELD_ADDRESS_LINE2 => Some(FormField::AddressLine2),
            FIELD_SUBURB => Some(FormField::Suburb),
            FIELD_STATE => Some(FormField::State),
            FIELD_POSTCODE => Some(FormField::Postcode),
            FIELD_COUNTRY => Some(FormField::Country),
            FIELD_PHONE_HOME => Some(FormField::PhoneHome),
            FIELD_PHONE_MOBILE => Some(FormField::PhoneMobile),
            FIELD_EMAIL => Some(FormField::Email),
            FIELD_MEDICARE_NUMBER => Some(FormField::MedicareNumber),
            FIELD_MEDICARE_IRN => Some(FormField::MedicareIrn),
            FIELD_MEDICARE_EXPIRY => Some(FormField::MedicareExpiry),
            FIELD_IHI => Some(FormField::Ihi),
            FIELD_EMERGENCY_NAME => Some(FormField::EmergencyName),
            FIELD_EMERGENCY_PHONE => Some(FormField::EmergencyPhone),
            FIELD_EMERGENCY_RELATIONSHIP => Some(FormField::EmergencyRelationship),
            FIELD_CONCESSION_TYPE => Some(FormField::ConcessionType),
            FIELD_CONCESSION_NUMBER => Some(FormField::ConcessionNumber),
            FIELD_PREFERRED_LANGUAGE => Some(FormField::PreferredLanguage),
            FIELD_INTERPRETER_REQUIRED => Some(FormField::InterpreterRequired),
            FIELD_ATSI_STATUS => Some(FormField::AtsiStatus),
            _ => None,
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
    errors: HashMap<String, String>,
    focused_field: String,
    field_ids: Vec<String>,
    field_configs: HashMap<String, FieldDefinition>,
    saving: bool,
    theme: Theme,
    scroll: ScrollableFormState,
    textareas: HashMap<String, TextareaState>,
    dropdowns: HashMap<String, DropdownWidget>,
    validator: FormValidator,
    date_picker: DatePickerPopup,
}

impl Clone for PatientForm {
    fn clone(&self) -> Self {
        Self {
            mode: self.mode,
            data: self.data.clone(),
            errors: self.errors.clone(),
            focused_field: self.focused_field.clone(),
            field_ids: self.field_ids.clone(),
            field_configs: self.field_configs.clone(),
            saving: self.saving,
            theme: self.theme.clone(),
            scroll: self.scroll.clone(),
            textareas: self.textareas.clone(),
            dropdowns: self.dropdowns.clone(),
            validator: build_validator(&self.field_configs),
            date_picker: self.date_picker.clone(),
        }
    }
}

impl FormFieldMeta for FormField {
    fn label(&self) -> &'static str {
        FormField::label(self)
    }

    fn is_required(&self) -> bool {
        FormField::is_required(self)
    }
}

impl FormNavigation for PatientForm {
    type FormField = FormField;

    fn get_error(&self, field: Self::FormField) -> Option<&str> {
        self.errors.get(field.id()).map(|s| s.as_str())
    }

    fn set_error(&mut self, field: Self::FormField, error: Option<String>) {
        self.set_error_by_id(field.id(), error);
    }

    fn validate(&mut self) -> bool {
        <Self as crate::ui::widgets::DynamicForm>::validate(self)
    }

    fn current_field(&self) -> Self::FormField {
        FormField::from_id(&self.focused_field).unwrap_or(FormField::FirstName)
    }

    fn fields(&self) -> Vec<Self::FormField> {
        self.field_ids
            .iter()
            .filter_map(|field_id| FormField::from_id(field_id))
            .collect()
    }

    fn set_current_field(&mut self, field: Self::FormField) {
        self.focused_field = field.id().to_string();
    }
}

impl crate::ui::widgets::DynamicFormMeta for PatientForm {
    fn label(&self, field_id: &str) -> String {
        self.field_configs
            .get(field_id)
            .map(|field| field.label.clone())
            .unwrap_or_else(|| field_id.to_string())
    }

    fn is_required(&self, field_id: &str) -> bool {
        self.field_configs
            .get(field_id)
            .map(|field| field.required)
            .unwrap_or(false)
    }

    fn field_type(&self, field_id: &str) -> crate::ui::widgets::FieldType {
        match self
            .field_configs
            .get(field_id)
            .map(|field| &field.field_type)
        {
            Some(ConfigFieldType::Date) => crate::ui::widgets::FieldType::Date,
            Some(ConfigFieldType::Select) => crate::ui::widgets::FieldType::Select(vec![]),
            _ => crate::ui::widgets::FieldType::Text,
        }
    }
}

impl crate::ui::widgets::DynamicForm for PatientForm {
    fn field_ids(&self) -> &[String] {
        &self.field_ids
    }

    fn current_field(&self) -> &str {
        &self.focused_field
    }

    fn set_current_field(&mut self, field_id: &str) {
        if self.field_ids.iter().any(|id| id == field_id) {
            self.focused_field = field_id.to_string();
        }
    }

    fn get_value(&self, field_id: &str) -> String {
        self.get_value_by_id(field_id)
    }

    fn set_value(&mut self, field_id: &str, value: String) {
        self.set_value_by_id(field_id, value)
    }

    fn validate(&mut self) -> bool {
        self.errors.clear();
        for field_id in self.field_ids.clone() {
            self.validate_field_by_id(&field_id);
        }
        self.errors.is_empty()
    }

    fn get_error(&self, field_id: &str) -> Option<&str> {
        self.errors.get(field_id).map(|s| s.as_str())
    }

    fn set_error(&mut self, field_id: &str, error: Option<String>) {
        self.set_error_by_id(field_id, error);
    }
}

impl PatientForm {
    pub fn new(theme: Theme) -> Self {
        let field_definitions = load_patient_field_definitions();
        let field_ids: Vec<String> = field_definitions
            .iter()
            .filter(|field| field.visible && field.navigable)
            .map(|field| field.id.clone())
            .collect();
        let field_configs: HashMap<String, FieldDefinition> = field_definitions
            .into_iter()
            .map(|field| (field.id.clone(), field))
            .collect();

        let mut textareas = HashMap::new();
        let mut dropdowns = HashMap::new();

        for field_id in &field_ids {
            if let Some(field) = field_configs.get(field_id) {
                match field.field_type {
                    ConfigFieldType::Select => {
                        let options = field
                            .options
                            .iter()
                            .map(|option| {
                                DropdownOption::new(option.value.as_str(), option.label.as_str())
                            })
                            .collect();
                        dropdowns.insert(
                            field.id.clone(),
                            DropdownWidget::new(field.label.as_str(), options, theme.clone()),
                        );
                    }
                    _ => {
                        textareas.insert(field.id.clone(), make_textarea_state(field, None));
                    }
                }
            }
        }

        let mut form = Self {
            mode: FormMode::Create,
            data: PatientFormData::empty(),
            errors: HashMap::new(),
            focused_field: if field_ids.iter().any(|id| id == FIELD_FIRST_NAME) {
                FIELD_FIRST_NAME.to_string()
            } else {
                field_ids.first().cloned().unwrap_or_default()
            },
            field_ids,
            field_configs,
            saving: false,
            theme,
            scroll: ScrollableFormState::new(),
            textareas,
            dropdowns,
            validator: FormValidator::new(&HashMap::new()),
            date_picker: DatePickerPopup::new(),
        };

        form.validator = build_validator(&form.field_configs);
        form
    }

    pub fn from_patient(patient: Patient, theme: Theme) -> Self {
        let gender = patient.gender;
        let concession_type = patient.concession_type;
        let atsi_status = patient.aboriginal_torres_strait_islander;
        let interpreter_required = patient.interpreter_required;

        let mut form = Self::new(theme);
        form.mode = FormMode::Edit(patient.id);

        if let Some(ref title) = patient.title {
            form.set_value(FormField::Title, title.clone());
        }
        form.set_value(FormField::FirstName, patient.first_name.clone());
        if let Some(ref middle_name) = patient.middle_name {
            form.set_value(FormField::MiddleName, middle_name.clone());
        }
        form.set_value(FormField::LastName, patient.last_name.clone());
        if let Some(ref preferred_name) = patient.preferred_name {
            form.set_value(FormField::PreferredName, preferred_name.clone());
        }
        form.set_value(FormField::DateOfBirth, format_date(patient.date_of_birth));
        if let Some(ref line1) = patient.address.line1 {
            form.set_value(FormField::AddressLine1, line1.clone());
        }
        if let Some(ref line2) = patient.address.line2 {
            form.set_value(FormField::AddressLine2, line2.clone());
        }
        if let Some(ref suburb) = patient.address.suburb {
            form.set_value(FormField::Suburb, suburb.clone());
        }
        if let Some(ref state) = patient.address.state {
            form.set_value(FormField::State, state.clone());
        }
        if let Some(ref postcode) = patient.address.postcode {
            form.set_value(FormField::Postcode, postcode.clone());
        }
        form.set_value(FormField::Country, patient.address.country.clone());
        if let Some(ref phone_home) = patient.phone_home {
            form.set_value(FormField::PhoneHome, phone_home.clone());
        }
        if let Some(ref phone_mobile) = patient.phone_mobile {
            form.set_value(FormField::PhoneMobile, phone_mobile.clone());
        }
        if let Some(ref email) = patient.email {
            form.set_value(FormField::Email, email.clone());
        }
        if let Some(ref medicare_number) = patient.medicare_number {
            form.set_value(FormField::MedicareNumber, medicare_number.clone());
        }
        if let Some(medicare_irn) = patient.medicare_irn {
            form.set_value(FormField::MedicareIrn, medicare_irn.to_string());
        }
        if let Some(medicare_expiry) = patient.medicare_expiry {
            form.set_value(FormField::MedicareExpiry, format_date(medicare_expiry));
        }
        if let Some(ref ihi) = patient.ihi {
            form.set_value(FormField::Ihi, ihi.clone());
        }
        if let Some(ref emergency_contact) = patient.emergency_contact {
            form.set_value(FormField::EmergencyName, emergency_contact.name.clone());
            form.set_value(FormField::EmergencyPhone, emergency_contact.phone.clone());
            form.set_value(
                FormField::EmergencyRelationship,
                emergency_contact.relationship.clone(),
            );
        }
        if let Some(ref concession_number) = patient.concession_number {
            form.set_value(FormField::ConcessionNumber, concession_number.clone());
        }
        form.set_value(
            FormField::PreferredLanguage,
            patient.preferred_language.clone(),
        );

        form.data = PatientFormData::from(patient);

        form.set_value(FormField::Gender, gender.to_string());
        if let Some(concession) = concession_type {
            form.set_value(FormField::ConcessionType, concession.to_string());
        }
        if let Some(atsi) = atsi_status {
            form.set_value(FormField::AtsiStatus, atsi.to_string());
        }
        form.set_value(
            FormField::InterpreterRequired,
            if interpreter_required {
                "Yes".to_string()
            } else {
                "No".to_string()
            },
        );

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
        self.get_value_by_id(field.id())
    }

    pub fn set_value(&mut self, field: FormField, value: String) {
        self.set_value_by_id(field.id(), value);
    }

    fn get_value_by_id(&self, field_id: &str) -> String {
        if let Some(textarea) = self.textareas.get(field_id) {
            return textarea.value();
        }

        if let Some(dropdown) = self.dropdowns.get(field_id) {
            return dropdown.selected_value().unwrap_or("").to_string();
        }

        String::new()
    }

    fn set_value_by_id(&mut self, field_id: &str, value: String) {
        if let Some(textarea) = self.textareas.get_mut(field_id) {
            let label = textarea.label.clone();
            let height_mode = textarea.height_mode.clone();
            let max_length = textarea.max_length;
            let focused = textarea.focused;

            let mut updated = TextareaState::new(label)
                .with_height_mode(height_mode)
                .with_value(value.clone())
                .focused(focused);
            if let Some(limit) = max_length {
                updated = updated.max_length(limit);
            }
            *textarea = updated;
        } else if let Some(dropdown) = self.dropdowns.get_mut(field_id) {
            dropdown.set_value(&value);
        }

        self.sync_data_for_field(field_id, &value);
        self.validate_field_by_id(field_id);
    }

    fn sync_data_for_field(&mut self, field_id: &str, value: &str) {
        match field_id {
            FIELD_GENDER => {
                if let Ok(gender) = value.parse() {
                    self.data.gender = gender;
                }
            }
            FIELD_CONCESSION_TYPE => {
                self.data.concession_type = value.parse().ok();
            }
            FIELD_INTERPRETER_REQUIRED => {
                self.data.interpreter_required = value == "Yes";
            }
            FIELD_ATSI_STATUS => {
                self.data.aboriginal_torres_strait_islander = value.parse().ok();
            }
            _ => {}
        }
    }

    fn focused_textarea_mut(&mut self) -> Option<&mut TextareaState> {
        self.textareas.get_mut(&self.focused_field)
    }

    fn textarea_for(&self, field_id: &str) -> Option<&TextareaState> {
        self.textareas.get(field_id)
    }

    pub fn focused_field(&self) -> FormField {
        FormField::from_id(&self.focused_field).unwrap_or(FormField::FirstName)
    }

    pub fn set_focus(&mut self, field: FormField) {
        self.focused_field = field.id().to_string();
    }

    fn get_field_position(&self, field_id: &str) -> (u16, u16) {
        let mut y: u16 = 0;

        for id in &self.field_ids {
            if id == field_id {
                return (y, self.get_field_height(id));
            }
            y += self.get_field_height(id) + 1;
        }

        (0, 0)
    }

    fn get_field_height(&self, field_id: &str) -> u16 {
        if self.dropdowns.contains_key(field_id) {
            4
        } else if let Some(textarea) = self.textarea_for(field_id) {
            textarea.height()
        } else {
            1
        }
    }

    pub fn is_saving(&self) -> bool {
        self.saving
    }

    pub fn set_saving(&mut self, saving: bool) {
        self.saving = saving;
    }

    fn validate_field_by_id(&mut self, field_id: &str) {
        self.errors.remove(field_id);

        let value = self.get_value_by_id(field_id);
        let mut errors = self.validator.validate(field_id, &value);

        if field_id == FIELD_MEDICARE_NUMBER && !value.is_empty() {
            if value.len() != 10 {
                errors = vec!["Medicare number must be 10 digits".to_string()];
            } else if !value.chars().all(|c| c.is_ascii_digit()) {
                errors = vec!["Medicare number must contain only digits".to_string()];
            }
        }

        if matches!(field_id, FIELD_DATE_OF_BIRTH | FIELD_MEDICARE_EXPIRY)
            && !value.trim().is_empty()
            && parse_date(&value).is_none()
        {
            errors = vec!["Use dd/mm/yyyy format".to_string()];
        }

        let error_msg = errors.into_iter().next();
        self.set_error_by_id(field_id, error_msg.clone());
        if let Some(textarea) = self.textareas.get_mut(field_id) {
            textarea.set_error(error_msg);
        }
    }

    fn set_error_by_id(&mut self, field_id: &str, error: Option<String>) {
        match error {
            Some(msg) => {
                self.errors.insert(field_id.to_string(), msg);
            }
            None => {
                self.errors.remove(field_id);
            }
        }
    }

    pub fn error(&self, field: FormField) -> Option<&String> {
        self.errors.get(field.id())
    }

    pub fn to_new_patient_data(&mut self) -> Option<NewPatientData> {
        if !FormNavigation::validate(self) {
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
    ) -> Option<(Uuid, opengp_domain::domain::patient::UpdatePatientData)> {
        let patient_id = self.patient_id()?;

        if !FormNavigation::validate(self) {
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

        let data = opengp_domain::domain::patient::UpdatePatientData {
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
        use crossterm::event::{KeyEventKind, KeyModifiers};

        if key.kind != KeyEventKind::Press {
            return None;
        }

        if self.saving {
            return None;
        }

        if key.modifiers.contains(KeyModifiers::CONTROL) && matches!(key.code, KeyCode::Char('s')) {
            FormNavigation::validate(self);
            return Some(PatientFormAction::Submit);
        }

        if self.date_picker.is_visible() {
            if let Some(action) = self.date_picker.handle_key(key) {
                match action {
                    DatePickerAction::Selected(date) => {
                        self.set_value_by_id(FIELD_DATE_OF_BIRTH, format_date(date));
                        return Some(PatientFormAction::ValueChanged);
                    }
                    DatePickerAction::Dismissed => {
                        return Some(PatientFormAction::FocusChanged);
                    }
                }
            }
            return Some(PatientFormAction::FocusChanged);
        }

        if self.focused_field == FIELD_DATE_OF_BIRTH
            && matches!(key.code, KeyCode::Enter | KeyCode::Char(' '))
        {
            let current_value = parse_date(&self.get_value_by_id(FIELD_DATE_OF_BIRTH));
            self.date_picker.open(current_value);
            return Some(PatientFormAction::FocusChanged);
        }

        if let Some(dropdown_action) = self.handle_dropdown_key(key) {
            return dropdown_action;
        }

        if !self.dropdowns.contains_key(&self.focused_field) {
            let ratatui_key = to_ratatui_key(key);
            if let Some(textarea) = self.focused_textarea_mut() {
                let consumed = textarea.handle_key(ratatui_key);
                if consumed {
                    let field = self.focused_field.clone();
                    self.validate_field_by_id(&field);
                    return Some(PatientFormAction::ValueChanged);
                }
            }
        }

        match key.code {
            KeyCode::Tab => {
                if key.modifiers.contains(KeyModifiers::SHIFT) {
                    FormNavigation::prev_field(self);
                } else {
                    FormNavigation::next_field(self);
                }
                Some(PatientFormAction::FocusChanged)
            }
            KeyCode::BackTab => {
                FormNavigation::prev_field(self);
                Some(PatientFormAction::FocusChanged)
            }
            KeyCode::Up => {
                FormNavigation::prev_field(self);
                Some(PatientFormAction::FocusChanged)
            }
            KeyCode::Down => {
                FormNavigation::next_field(self);
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
            KeyCode::Enter => None,
            KeyCode::Esc => Some(PatientFormAction::Cancel),
            _ => None,
        }
    }

    fn handle_dropdown_key(&mut self, key: KeyEvent) -> Option<Option<PatientFormAction>> {
        let field_id = self.focused_field.clone();
        if !self.dropdowns.contains_key(&field_id) {
            return None;
        }

        let mut selected_value: Option<String> = None;
        let action = {
            let dropdown = self.dropdowns.get_mut(&field_id)?;
            dropdown.handle_key(key)
        };

        if let Some(action) = action {
            match key.code {
                KeyCode::Tab | KeyCode::BackTab | KeyCode::Esc => {
                    return None;
                }
                _ => match action {
                    DropdownAction::Selected(_) | DropdownAction::Closed => {
                        selected_value = self
                            .dropdowns
                            .get(&field_id)
                            .and_then(|dropdown| dropdown.selected_value().map(|v| v.to_string()));
                    }
                    DropdownAction::Opened | DropdownAction::FocusChanged => {
                        return Some(Some(PatientFormAction::ValueChanged));
                    }
                },
            }
        } else {
            match key.code {
                KeyCode::Tab | KeyCode::BackTab | KeyCode::Esc => return None,
                _ => return Some(None),
            }
        }

        if let Some(value) = selected_value {
            self.set_value_by_id(&field_id, value);
        }

        Some(Some(PatientFormAction::ValueChanged))
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

        let mut y = inner.y + 1;
        let max_y = inner.y + inner.height - 2;

        for field_id in &self.field_ids {
            if y > max_y {
                break;
            }

            let field_height = self.get_field_height(field_id);
            let field_area = Rect::new(inner.x + 1, y, inner.width - 2, field_height);

            if field_area.contains(click_pos) {
                if *field_id != self.focused_field {
                    self.focused_field = field_id.clone();
                    return Some(PatientFormAction::FocusChanged);
                }
                return None;
            }

            y += field_height + 1;
        }

        None
    }
}

fn load_patient_field_definitions() -> Vec<FieldDefinition> {
    if let Ok(config) = FormConfig::load() {
        if let Some(form) = config.forms.get("patient") {
            return form.fields.clone();
        }
    }

    fallback_patient_field_definitions()
}

fn fallback_patient_field_definitions() -> Vec<FieldDefinition> {
    FormField::all()
        .into_iter()
        .map(|field| {
            let mut definition = FieldDefinition {
                id: field.id().to_string(),
                label: field.label().to_string(),
                required: field.is_required(),
                field_type: if field.is_dropdown() {
                    ConfigFieldType::Select
                } else {
                    ConfigFieldType::Text
                },
                ..FieldDefinition::default()
            };

            definition.validation = match field {
                FormField::FirstName | FormField::LastName => ValidationRules {
                    max_length: Some(100),
                    required: true,
                    ..ValidationRules::default()
                },
                FormField::Email => ValidationRules {
                    email: true,
                    ..ValidationRules::default()
                },
                FormField::PhoneHome | FormField::PhoneMobile | FormField::EmergencyPhone => {
                    ValidationRules {
                        phone: true,
                        ..ValidationRules::default()
                    }
                }
                FormField::DateOfBirth => ValidationRules {
                    required: true,
                    date_format: Some("dd/mm/yyyy".to_string()),
                    ..ValidationRules::default()
                },
                FormField::MedicareNumber => ValidationRules {
                    max_length: Some(10),
                    ..ValidationRules::default()
                },
                FormField::MedicareIrn => ValidationRules {
                    max_length: Some(1),
                    ..ValidationRules::default()
                },
                FormField::MedicareExpiry => ValidationRules {
                    date_format: Some("dd/mm/yyyy".to_string()),
                    ..ValidationRules::default()
                },
                _ => ValidationRules::default(),
            };

            if field.is_dropdown() {
                definition.options = match field {
                    FormField::Gender => vec![
                        opengp_config::forms::SelectOption {
                            value: "Male".to_string(),
                            label: "Male".to_string(),
                        },
                        opengp_config::forms::SelectOption {
                            value: "Female".to_string(),
                            label: "Female".to_string(),
                        },
                        opengp_config::forms::SelectOption {
                            value: "Other".to_string(),
                            label: "Other".to_string(),
                        },
                        opengp_config::forms::SelectOption {
                            value: "PreferNotToSay".to_string(),
                            label: "Prefer not to say".to_string(),
                        },
                    ],
                    FormField::ConcessionType => vec![
                        opengp_config::forms::SelectOption {
                            value: "DVA".to_string(),
                            label: "DVA".to_string(),
                        },
                        opengp_config::forms::SelectOption {
                            value: "Pensioner".to_string(),
                            label: "Pensioner".to_string(),
                        },
                        opengp_config::forms::SelectOption {
                            value: "HealthcareCard".to_string(),
                            label: "Healthcare Card".to_string(),
                        },
                        opengp_config::forms::SelectOption {
                            value: "SafetyNetCard".to_string(),
                            label: "Safety Net Card".to_string(),
                        },
                    ],
                    FormField::InterpreterRequired => vec![
                        opengp_config::forms::SelectOption {
                            value: "Yes".to_string(),
                            label: "Yes".to_string(),
                        },
                        opengp_config::forms::SelectOption {
                            value: "No".to_string(),
                            label: "No".to_string(),
                        },
                    ],
                    FormField::AtsiStatus => vec![
                        opengp_config::forms::SelectOption {
                            value: "AboriginalNotTorresStrait".to_string(),
                            label: "Aboriginal (not Torres Strait)".to_string(),
                        },
                        opengp_config::forms::SelectOption {
                            value: "TorresStraitNotAboriginal".to_string(),
                            label: "Torres Strait (not Aboriginal)".to_string(),
                        },
                        opengp_config::forms::SelectOption {
                            value: "BothAboriginalAndTorresStrait".to_string(),
                            label: "Both Aboriginal and Torres Strait".to_string(),
                        },
                        opengp_config::forms::SelectOption {
                            value: "NeitherAboriginalNorTorresStrait".to_string(),
                            label: "Neither Aboriginal nor Torres Strait".to_string(),
                        },
                        opengp_config::forms::SelectOption {
                            value: "NotStated".to_string(),
                            label: "Not stated".to_string(),
                        },
                    ],
                    _ => vec![],
                };
            }

            definition
        })
        .collect()
}

fn make_textarea_state(field: &FieldDefinition, value: Option<String>) -> TextareaState {
    let mut state =
        TextareaState::new(field.label.clone()).with_height_mode(HeightMode::SingleLine);
    if let Some(max_length) = field.validation.max_length {
        state = state.max_length(max_length);
    }
    if let Some(value) = value {
        state = state.with_value(value);
    }
    state
}

fn build_validator(field_configs: &HashMap<String, FieldDefinition>) -> FormValidator {
    let rules: HashMap<String, ValidationRules> = field_configs
        .iter()
        .map(|(field_id, field)| {
            let mut validation = field.validation.clone();
            if field.required {
                validation.required = true;
            }
            if matches!(field.field_type, ConfigFieldType::Date) {
                validation.date_format = None;
            }
            (field_id.clone(), validation)
        })
        .collect();

    FormValidator::new(&rules)
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

        let fields = self.field_ids.clone();

        let mut total_height: u16 = 0;
        for field_id in &fields {
            total_height += self.get_field_height(field_id) + 1;
        }
        self.scroll.set_total_height(total_height);
        self.scroll.clamp_offset(inner.height.saturating_sub(2));

        let (focused_y, focused_height) = self.get_field_position(&self.focused_field);
        self.scroll
            .scroll_to_field(focused_y, focused_height, inner.height.saturating_sub(2));

        let mut y: i32 = (inner.y as i32) + 1 - (self.scroll.scroll_offset as i32);
        let max_y = inner.y as i32 + inner.height as i32 - 2;
        let mut open_dropdown: Option<(DropdownWidget, Rect)> = None;

        for field_id in fields {
            let field_height = self.get_field_height(&field_id) as i32;

            if y + field_height <= inner.y as i32 || y >= max_y {
                y += field_height + 1;
                continue;
            }

            let is_focused = field_id == self.focused_field;

            if let Some(dropdown) = self.dropdowns.get(&field_id).cloned() {
                if y >= inner.y as i32 && y < max_y {
                    let dropdown_area = Rect::new(
                        field_start,
                        y as u16,
                        inner.width.saturating_sub(label_width + 4),
                        3,
                    );
                    if dropdown.is_open() {
                        open_dropdown = Some((dropdown.clone(), dropdown_area));
                    }
                    dropdown.focused(is_focused).render(dropdown_area, buf);
                }
                y += 4;
                continue;
            }

            if let Some(textarea) = self.textareas.get(&field_id) {
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

        if let Some((dropdown, dropdown_area)) = open_dropdown {
            dropdown.render(dropdown_area, buf);
        }

        self.scroll.render_scrollbar(inner, buf);

        let help_y = inner.y + inner.height - 1;
        buf.set_string(
            inner.x + 1,
            help_y,
            "Tab: Next | Ctrl+S: Submit | Esc: Cancel",
            Style::default().fg(self.theme.colors.disabled),
        );

        if self.date_picker.is_visible() {
            self.date_picker.render(area, buf);
        }
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

        FormNavigation::validate(&mut form);
        assert!(form.has_errors());
        assert!(form.error(FormField::FirstName).is_some());
        assert!(form.error(FormField::LastName).is_some());
    }

    #[test]
    fn test_form_validation_email() {
        let theme = Theme::dark();
        let mut form = PatientForm::new(theme);

        form.set_value(FormField::Email, "invalid".to_string());
        FormNavigation::validate(&mut form);
        assert!(form.error(FormField::Email).is_some());

        form.set_value(FormField::Email, "test@example.com".to_string());
        FormNavigation::validate(&mut form);
        assert!(form.error(FormField::Email).is_none());
    }

    #[test]
    fn test_text_fields_use_textarea_state() {
        let theme = Theme::dark();
        let mut form = PatientForm::new(theme);

        form.set_value(FormField::FirstName, "Alice".to_string());
        assert_eq!(form.get_value(FormField::FirstName), "Alice");
        assert_eq!(
            form.textareas
                .get(FIELD_FIRST_NAME)
                .expect("first_name textarea should exist")
                .value(),
            "Alice"
        );
    }

    #[test]
    fn test_dynamic_form_string_access() {
        let theme = Theme::dark();
        let mut form = PatientForm::new(theme);

        <PatientForm as crate::ui::widgets::DynamicForm>::set_value(
            &mut form,
            FIELD_FIRST_NAME,
            "John".to_string(),
        );

        let by_string =
            <PatientForm as crate::ui::widgets::DynamicForm>::get_value(&form, FIELD_FIRST_NAME);
        let by_enum = form.get_value(FormField::FirstName);
        assert_eq!(by_string, by_enum);
    }

    #[test]
    fn test_handle_key_char_updates_textarea() {
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

        let theme = Theme::dark();
        let mut form = PatientForm::new(theme);
        form.focused_field = FIELD_FIRST_NAME.to_string();

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

        assert_eq!(
            form.textareas
                .get(FIELD_FIRST_NAME)
                .expect("first_name textarea should exist")
                .height_mode,
            HeightMode::SingleLine
        );
        assert_eq!(
            form.textareas
                .get(FIELD_LAST_NAME)
                .expect("last_name textarea should exist")
                .height_mode,
            HeightMode::SingleLine
        );
        assert_eq!(
            form.textareas
                .get(FIELD_EMAIL)
                .expect("email textarea should exist")
                .height_mode,
            HeightMode::SingleLine
        );
        assert_eq!(
            form.textareas
                .get(FIELD_MEDICARE_NUMBER)
                .expect("medicare_number textarea should exist")
                .height_mode,
            HeightMode::SingleLine
        );
    }

    #[test]
    fn test_error_synced_to_textarea_state() {
        let theme = Theme::dark();
        let mut form = PatientForm::new(theme);

        FormNavigation::validate(&mut form);
        assert!(form
            .textareas
            .get(FIELD_FIRST_NAME)
            .expect("first_name textarea should exist")
            .error
            .is_some());
        assert!(form
            .textareas
            .get(FIELD_LAST_NAME)
            .expect("last_name textarea should exist")
            .error
            .is_some());
    }

    #[test]
    fn test_to_new_patient_data_valid() {
        let theme = Theme::dark();
        let mut form = PatientForm::new(theme);

        form.set_value(FormField::FirstName, "Alice".to_string());
        form.set_value(FormField::LastName, "Smith".to_string());
        form.set_value(FormField::DateOfBirth, "15/05/1990".to_string());
        form.set_value(FormField::Gender, "Female".to_string());
        form.set_value(FormField::Email, "alice@test.com".to_string());
        form.set_value(FormField::PhoneMobile, "0412345678".to_string());
        form.set_value(FormField::PreferredLanguage, "English".to_string());

        let result = form.to_new_patient_data();
        assert!(result.is_some());
        let data = result.expect("result should be present");
        assert_eq!(data.first_name, "Alice");
        assert_eq!(data.last_name, "Smith");
    }

    #[test]
    fn test_to_new_patient_data_invalid_returns_none() {
        let theme = Theme::dark();
        let mut form = PatientForm::new(theme);

        form.set_value(FormField::FirstName, "Alice".to_string());
        form.set_value(FormField::LastName, "".to_string());

        let result = form.to_new_patient_data();
        assert!(result.is_none());
    }
}

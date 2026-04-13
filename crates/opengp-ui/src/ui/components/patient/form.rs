//! Patient Form Component
//!
//! Comprehensive form for creating and editing patients.

use std::cmp::max;
use std::collections::HashMap;

use crossterm::event::{KeyCode, KeyEvent, MouseEvent, MouseEventKind};
use opengp_config::{
    forms::{FieldDefinition, FieldType as ConfigFieldType, FormConfig, ValidationRules},
    PatientConfig,
};
use opengp_domain::domain::patient::{
    Address, EmergencyContact, Ihi, MedicareNumber, NewPatientData, Patient, PhoneNumber,
};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Position, Rect};
use ratatui::style::Style;
use ratatui::widgets::{Block, Borders, Paragraph, Widget};
use uuid::Uuid;

use crate::ui::input::to_ratatui_key;
use crate::ui::layout::LABEL_WIDTH;
use crate::ui::shared::FormAction;
use crate::ui::theme::Theme;
use crate::ui::view_models::PatientFormData;
use crate::ui::widgets::{
    format_date, impl_form_field_wrapper, parse_date, DatePickerAction, DatePickerPopup,
    DropdownAction, DropdownOption, DropdownWidget, FormField as FormFieldTrait, FormFieldMeta,
    FormNavigation, FormState, FormValidator, HeightMode, TextareaState, TextareaWidget,
};

pub use crate::ui::shared::FormMode;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, strum::EnumIter, strum::IntoStaticStr)]
pub enum PatientFormField {
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

impl_form_field_wrapper!(PatientFormField, FieldDefinition);

impl FormFieldTrait for PatientFormField {
    fn all() -> Vec<Self> {
        use strum::IntoEnumIterator;
        Self::iter().collect()
    }

    fn label(&self) -> &'static str {
        (*self).into()
    }

    fn id(&self) -> &'static str {
        match self {
            PatientFormField::Title => FIELD_TITLE,
            PatientFormField::FirstName => FIELD_FIRST_NAME,
            PatientFormField::MiddleName => FIELD_MIDDLE_NAME,
            PatientFormField::LastName => FIELD_LAST_NAME,
            PatientFormField::PreferredName => FIELD_PREFERRED_NAME,
            PatientFormField::DateOfBirth => FIELD_DATE_OF_BIRTH,
            PatientFormField::Gender => FIELD_GENDER,
            PatientFormField::AddressLine1 => FIELD_ADDRESS_LINE1,
            PatientFormField::AddressLine2 => FIELD_ADDRESS_LINE2,
            PatientFormField::Suburb => FIELD_SUBURB,
            PatientFormField::State => FIELD_STATE,
            PatientFormField::Postcode => FIELD_POSTCODE,
            PatientFormField::Country => FIELD_COUNTRY,
            PatientFormField::PhoneHome => FIELD_PHONE_HOME,
            PatientFormField::PhoneMobile => FIELD_PHONE_MOBILE,
            PatientFormField::Email => FIELD_EMAIL,
            PatientFormField::MedicareNumber => FIELD_MEDICARE_NUMBER,
            PatientFormField::MedicareIrn => FIELD_MEDICARE_IRN,
            PatientFormField::MedicareExpiry => FIELD_MEDICARE_EXPIRY,
            PatientFormField::Ihi => FIELD_IHI,
            PatientFormField::EmergencyName => FIELD_EMERGENCY_NAME,
            PatientFormField::EmergencyPhone => FIELD_EMERGENCY_PHONE,
            PatientFormField::EmergencyRelationship => FIELD_EMERGENCY_RELATIONSHIP,
            PatientFormField::ConcessionType => FIELD_CONCESSION_TYPE,
            PatientFormField::ConcessionNumber => FIELD_CONCESSION_NUMBER,
            PatientFormField::PreferredLanguage => FIELD_PREFERRED_LANGUAGE,
            PatientFormField::InterpreterRequired => FIELD_INTERPRETER_REQUIRED,
            PatientFormField::AtsiStatus => FIELD_ATSI_STATUS,
        }
    }

    fn from_id(id: &str) -> Option<Self> {
        match id {
            FIELD_TITLE => Some(PatientFormField::Title),
            FIELD_FIRST_NAME => Some(PatientFormField::FirstName),
            FIELD_MIDDLE_NAME => Some(PatientFormField::MiddleName),
            FIELD_LAST_NAME => Some(PatientFormField::LastName),
            FIELD_PREFERRED_NAME => Some(PatientFormField::PreferredName),
            FIELD_DATE_OF_BIRTH => Some(PatientFormField::DateOfBirth),
            FIELD_GENDER => Some(PatientFormField::Gender),
            FIELD_ADDRESS_LINE1 => Some(PatientFormField::AddressLine1),
            FIELD_ADDRESS_LINE2 => Some(PatientFormField::AddressLine2),
            FIELD_SUBURB => Some(PatientFormField::Suburb),
            FIELD_STATE => Some(PatientFormField::State),
            FIELD_POSTCODE => Some(PatientFormField::Postcode),
            FIELD_COUNTRY => Some(PatientFormField::Country),
            FIELD_PHONE_HOME => Some(PatientFormField::PhoneHome),
            FIELD_PHONE_MOBILE => Some(PatientFormField::PhoneMobile),
            FIELD_EMAIL => Some(PatientFormField::Email),
            FIELD_MEDICARE_NUMBER => Some(PatientFormField::MedicareNumber),
            FIELD_MEDICARE_IRN => Some(PatientFormField::MedicareIrn),
            FIELD_MEDICARE_EXPIRY => Some(PatientFormField::MedicareExpiry),
            FIELD_IHI => Some(PatientFormField::Ihi),
            FIELD_EMERGENCY_NAME => Some(PatientFormField::EmergencyName),
            FIELD_EMERGENCY_PHONE => Some(PatientFormField::EmergencyPhone),
            FIELD_EMERGENCY_RELATIONSHIP => Some(PatientFormField::EmergencyRelationship),
            FIELD_CONCESSION_TYPE => Some(PatientFormField::ConcessionType),
            FIELD_CONCESSION_NUMBER => Some(PatientFormField::ConcessionNumber),
            FIELD_PREFERRED_LANGUAGE => Some(PatientFormField::PreferredLanguage),
            FIELD_INTERPRETER_REQUIRED => Some(PatientFormField::InterpreterRequired),
            FIELD_ATSI_STATUS => Some(PatientFormField::AtsiStatus),
            _ => None,
        }
    }

    fn is_required(&self) -> bool {
        matches!(
            self,
            PatientFormField::FirstName
                | PatientFormField::LastName
                | PatientFormField::DateOfBirth
                | PatientFormField::Gender
        )
    }

    fn is_textarea(&self) -> bool {
        false
    }

    fn is_dropdown(&self) -> bool {
        matches!(
            self,
            PatientFormField::Title
                | PatientFormField::Gender
                | PatientFormField::State
                | PatientFormField::ConcessionType
                | PatientFormField::InterpreterRequired
                | PatientFormField::PreferredLanguage
                | PatientFormField::AtsiStatus
        )
    }
}

pub struct PatientForm {
    mode: FormMode,
    data: PatientFormData,
    pub focused_field: String,
    is_valid: bool,
    field_ids: Vec<String>,
    field_configs: HashMap<String, FieldDefinition>,
    saving: bool,
    form_state: FormState<PatientFormField>,
    validator: FormValidator,
    date_picker: DatePickerPopup,
}

impl Clone for PatientForm {
    fn clone(&self) -> Self {
        Self {
            mode: self.mode,
            data: self.data.clone(),
            focused_field: self.focused_field.clone(),
            is_valid: self.is_valid,
            field_ids: self.field_ids.clone(),
            field_configs: self.field_configs.clone(),
            saving: self.saving,
            form_state: self.form_state.clone(),
            validator: build_validator(&self.field_configs),
            date_picker: self.date_picker.clone(),
        }
    }
}

impl FormFieldMeta for PatientFormField {
    fn label(&self) -> &'static str {
        PatientFormField::label(self)
    }

    fn is_required(&self) -> bool {
        PatientFormField::is_required(self)
    }
}

impl FormNavigation for PatientForm {
    type FormField = PatientFormField;

    fn get_error(&self, field: Self::FormField) -> Option<&str> {
        self.form_state.errors.get(field.id()).map(|s| s.as_str())
    }

    fn set_error(&mut self, field: Self::FormField, error: Option<String>) {
        self.set_error_by_id(field.id(), error);
    }

    fn validate(&mut self) -> bool {
        <Self as crate::ui::widgets::DynamicForm>::validate(self)
    }

    fn current_field(&self) -> Self::FormField {
        PatientFormField::from_id(&self.focused_field).unwrap_or(PatientFormField::FirstName)
    }

    fn fields(&self) -> Vec<Self::FormField> {
        self.field_ids
            .iter()
            .filter_map(|field_id| PatientFormField::from_id(field_id))
            .collect()
    }

    fn set_current_field(&mut self, field: Self::FormField) {
        self.focused_field = field.id().to_string();
        self.form_state.focused_field = field;
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
            if let Some(field) = PatientFormField::from_id(field_id) {
                self.form_state.focused_field = field;
            }
        }
    }

    fn get_value(&self, field_id: &str) -> String {
        self.get_value_by_id(field_id)
    }

    fn set_value(&mut self, field_id: &str, value: String) {
        self.set_value_by_id(field_id, value)
    }

    fn validate(&mut self) -> bool {
        self.form_state.errors.clear();
        for field_id in self.field_ids.clone() {
            self.validate_field_by_id(&field_id);
        }
        self.is_valid = self.form_state.errors.is_empty();
        self.is_valid
    }

    fn get_error(&self, field_id: &str) -> Option<&str> {
        self.form_state.errors.get(field_id).map(|s| s.as_str())
    }

    fn set_error(&mut self, field_id: &str, error: Option<String>) {
        self.set_error_by_id(field_id, error);
    }
}

impl PatientForm {
    pub fn new(theme: Theme, patient_config: &opengp_config::PatientConfig) -> Self {
        let field_definitions = load_patient_field_definitions(patient_config);
        let field_ids: Vec<String> = field_definitions
            .iter()
            .filter(|field| field.visible && field.navigable)
            .map(|field| field.id.clone())
            .collect();
        let field_configs: HashMap<String, FieldDefinition> = field_definitions
            .into_iter()
            .map(|field| (field.id.clone(), field))
            .collect();

        let initial_focused_field = if field_ids.iter().any(|id| id == FIELD_FIRST_NAME) {
            PatientFormField::FirstName
        } else {
            field_ids
                .first()
                .and_then(|id| PatientFormField::from_id(id))
                .unwrap_or(PatientFormField::FirstName)
        };

        let mut form_state = FormState::new(theme.clone(), initial_focused_field);

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
                        form_state.dropdowns.insert(
                            field.id.clone(),
                            DropdownWidget::new(field.label.as_str(), options, theme.clone()),
                        );
                    }
                    _ => {
                        form_state
                            .textareas
                            .insert(field.id.clone(), make_textarea_state(field, None));
                    }
                }
            }
        }

        let mut form = Self {
            mode: FormMode::Create,
            data: PatientFormData::empty(),
            focused_field: initial_focused_field.id().to_string(),
            is_valid: false,
            field_ids,
            field_configs,
            saving: false,
            form_state,
            validator: FormValidator::new(&HashMap::new()),
            date_picker: DatePickerPopup::new(theme),
        };

        form.validator = build_validator(&form.field_configs);
        form
    }

    pub fn from_patient(
        patient: Patient,
        theme: Theme,
        patient_config: &opengp_config::PatientConfig,
    ) -> Self {
        let gender = patient.gender;
        let concession_type = patient.concession_type;
        let atsi_status = patient.aboriginal_torres_strait_islander;
        let interpreter_required = patient.interpreter_required;

        let mut form = Self::new(theme, patient_config);
        form.mode = FormMode::Edit(patient.id);
        form.form_state.mode = FormMode::Edit(patient.id);

        if let Some(ref title) = patient.title {
            form.set_value(PatientFormField::Title, title.clone());
        }
        form.set_value(PatientFormField::FirstName, patient.first_name.clone());
        if let Some(ref middle_name) = patient.middle_name {
            form.set_value(PatientFormField::MiddleName, middle_name.clone());
        }
        form.set_value(PatientFormField::LastName, patient.last_name.clone());
        if let Some(ref preferred_name) = patient.preferred_name {
            form.set_value(PatientFormField::PreferredName, preferred_name.clone());
        }
        form.set_value(
            PatientFormField::DateOfBirth,
            format_date(patient.date_of_birth),
        );
        if let Some(ref line1) = patient.address.line1 {
            form.set_value(PatientFormField::AddressLine1, line1.clone());
        }
        if let Some(ref line2) = patient.address.line2 {
            form.set_value(PatientFormField::AddressLine2, line2.clone());
        }
        if let Some(ref suburb) = patient.address.suburb {
            form.set_value(PatientFormField::Suburb, suburb.clone());
        }
        if let Some(ref state) = patient.address.state {
            form.set_value(PatientFormField::State, state.clone());
        }
        if let Some(ref postcode) = patient.address.postcode {
            form.set_value(PatientFormField::Postcode, postcode.clone());
        }
        form.set_value(PatientFormField::Country, patient.address.country.clone());
        if let Some(ref phone_home) = patient.phone_home {
            form.set_value(PatientFormField::PhoneHome, phone_home.to_string());
        }
        if let Some(ref phone_mobile) = patient.phone_mobile {
            form.set_value(PatientFormField::PhoneMobile, phone_mobile.to_string());
        }
        if let Some(ref email) = patient.email {
            form.set_value(PatientFormField::Email, email.clone());
        }
        if let Some(ref medicare_number) = patient.medicare_number {
            form.set_value(
                PatientFormField::MedicareNumber,
                medicare_number.to_string(),
            );
        }
        if let Some(medicare_irn) = patient.medicare_irn {
            form.set_value(PatientFormField::MedicareIrn, medicare_irn.to_string());
        }
        if let Some(medicare_expiry) = patient.medicare_expiry {
            form.set_value(
                PatientFormField::MedicareExpiry,
                format_date(medicare_expiry),
            );
        }
        if let Some(ref ihi) = patient.ihi {
            form.set_value(PatientFormField::Ihi, ihi.to_string());
        }
        if let Some(ref emergency_contact) = patient.emergency_contact {
            form.set_value(
                PatientFormField::EmergencyName,
                emergency_contact.name.clone(),
            );
            form.set_value(
                PatientFormField::EmergencyPhone,
                emergency_contact.phone.clone(),
            );
            form.set_value(
                PatientFormField::EmergencyRelationship,
                emergency_contact.relationship.clone(),
            );
        }
        if let Some(ref concession_number) = patient.concession_number {
            form.set_value(
                PatientFormField::ConcessionNumber,
                concession_number.clone(),
            );
        }
        form.set_value(
            PatientFormField::PreferredLanguage,
            patient.preferred_language.clone(),
        );

        form.data = PatientFormData::from(patient);

        form.set_value(PatientFormField::Gender, gender.to_string());
        if let Some(concession) = concession_type {
            form.set_value(PatientFormField::ConcessionType, concession.to_string());
        }
        if let Some(atsi) = atsi_status {
            form.set_value(PatientFormField::AtsiStatus, atsi.to_string());
        }
        form.set_value(
            PatientFormField::InterpreterRequired,
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

    pub fn get_value(&self, field: PatientFormField) -> String {
        self.get_value_by_id(field.id())
    }

    pub fn set_value(&mut self, field: PatientFormField, value: String) {
        self.set_value_by_id(field.id(), value);
    }

    fn get_value_by_id(&self, field_id: &str) -> String {
        self.form_state.get_value_by_id(field_id)
    }

    fn set_value_by_id(&mut self, field_id: &str, value: String) {
        self.form_state.set_value_by_id(field_id, value.clone());

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
        self.form_state.textareas.get_mut(&self.focused_field)
    }

    fn textarea_for(&self, field_id: &str) -> Option<&TextareaState> {
        self.form_state.textareas.get(field_id)
    }

    pub fn focused_field(&self) -> PatientFormField {
        PatientFormField::from_id(&self.focused_field).unwrap_or(PatientFormField::FirstName)
    }

    pub fn set_focus(&mut self, field: PatientFormField) {
        self.focused_field = field.id().to_string();
        self.form_state.focused_field = field;
    }

    fn get_field_position(&self, field_id: &str) -> (u16, u16) {
        let mut y: u16 = 0;

        for id in &self.field_ids {
            if id == field_id {
                return (y, self.get_field_height(id));
            }
            y += self.get_field_height(id);
        }

        (0, 0)
    }

    fn get_field_height(&self, field_id: &str) -> u16 {
        if self.form_state.dropdowns.contains_key(field_id) {
            3
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

    pub fn focus_first_error(&mut self) {
        if let Some(field_id) = self
            .field_ids
            .iter()
            .find(|id| self.form_state.errors.contains_key(*id))
        {
            self.focused_field = field_id.clone();
            if let Some(field) = PatientFormField::from_id(field_id) {
                self.form_state.focused_field = field;
            }
        }
    }

    fn validate_field_by_id(&mut self, field_id: &str) {
        self.form_state.errors.remove(field_id);

        if let Some(field) = PatientFormField::from_id(field_id) {
            let _ = self.form_state.validate_field(field);
        }

        let value = self.get_value_by_id(field_id);
        let mut errors = self.validator.validate(field_id, &value);

        if field_id == FIELD_MEDICARE_NUMBER
            && !value.trim().is_empty()
            && MedicareNumber::new_strict(value.clone()).is_err()
        {
            errors = vec!["Medicare number must be 10 digits".to_string()];
        }

        if field_id == FIELD_IHI
            && !value.trim().is_empty()
            && Ihi::new_strict(value.clone()).is_err()
        {
            errors = vec!["IHI must be 16 digits".to_string()];
        }

        if matches!(field_id, FIELD_PHONE_HOME | FIELD_PHONE_MOBILE)
            && !value.trim().is_empty()
            && PhoneNumber::new_strict(value.clone()).is_err()
        {
            errors = vec!["Enter a valid Australian phone number".to_string()];
        }

        if matches!(field_id, FIELD_DATE_OF_BIRTH | FIELD_MEDICARE_EXPIRY)
            && !value.trim().is_empty()
            && parse_date(&value).is_none()
        {
            errors = vec!["Use dd/mm/yyyy format".to_string()];
        }

        let error_msg = errors.into_iter().next();
        self.set_error_by_id(field_id, error_msg.clone());
        if let Some(textarea) = self.form_state.textareas.get_mut(field_id) {
            textarea.set_error(error_msg);
        }

        self.is_valid = self.form_state.errors.is_empty();
    }

    fn set_error_by_id(&mut self, field_id: &str, error: Option<String>) {
        match error {
            Some(msg) => {
                self.form_state.errors.insert(field_id.to_string(), msg);
            }
            None => {
                self.form_state.errors.remove(field_id);
            }
        }
    }

    pub fn error(&self, field: PatientFormField) -> Option<&String> {
        self.form_state.errors.get(field.id())
    }

    pub fn to_new_patient_data(&mut self) -> Option<NewPatientData> {
        if !FormNavigation::validate(self) {
            return None;
        }

        let dob = parse_date(&self.get_value(PatientFormField::DateOfBirth))?;
        let gender = self.get_value(PatientFormField::Gender).parse().ok()?;

        let address = Address {
            line1: self
                .get_value(PatientFormField::AddressLine1)
                .empty_to_none(),
            line2: self
                .get_value(PatientFormField::AddressLine2)
                .empty_to_none(),
            suburb: self.get_value(PatientFormField::Suburb).empty_to_none(),
            state: self.get_value(PatientFormField::State).empty_to_none(),
            postcode: self.get_value(PatientFormField::Postcode).empty_to_none(),
            country: or_default(self.get_value(PatientFormField::Country), "Australia"),
        };

        let emergency_contact = if !self.get_value(PatientFormField::EmergencyName).is_empty() {
            Some(EmergencyContact {
                name: self.get_value(PatientFormField::EmergencyName),
                phone: self.get_value(PatientFormField::EmergencyPhone),
                relationship: self.get_value(PatientFormField::EmergencyRelationship),
            })
        } else {
            None
        };

        let concession_type = self
            .get_value(PatientFormField::ConcessionType)
            .parse()
            .ok();
        let atsi_status = self.get_value(PatientFormField::AtsiStatus).parse().ok();
        let ihi = self.get_value(PatientFormField::Ihi);
        let ihi = if ihi.trim().is_empty() {
            None
        } else {
            Some(Ihi::new_strict(ihi).ok()?)
        };
        let medicare_number = self.get_value(PatientFormField::MedicareNumber);
        let medicare_number = if medicare_number.trim().is_empty() {
            None
        } else {
            Some(MedicareNumber::new_strict(medicare_number).ok()?)
        };
        let phone_home = self.get_value(PatientFormField::PhoneHome);
        let phone_home = if phone_home.trim().is_empty() {
            None
        } else {
            Some(PhoneNumber::new_strict(phone_home).ok()?)
        };
        let phone_mobile = self.get_value(PatientFormField::PhoneMobile);
        let phone_mobile = if phone_mobile.trim().is_empty() {
            None
        } else {
            Some(PhoneNumber::new_strict(phone_mobile).ok()?)
        };

        Some(NewPatientData {
            ihi,
            medicare_number,
            medicare_irn: self.get_value(PatientFormField::MedicareIrn).parse().ok(),
            medicare_expiry: parse_date(&self.get_value(PatientFormField::MedicareExpiry)),
            title: self.get_value(PatientFormField::Title).empty_to_none(),
            first_name: self.get_value(PatientFormField::FirstName),
            middle_name: self.get_value(PatientFormField::MiddleName).empty_to_none(),
            last_name: self.get_value(PatientFormField::LastName),
            preferred_name: self
                .get_value(PatientFormField::PreferredName)
                .empty_to_none(),
            date_of_birth: dob,
            gender,
            address,
            phone_home,
            phone_mobile,
            email: self.get_value(PatientFormField::Email).empty_to_none(),
            emergency_contact,
            concession_type,
            concession_number: self
                .get_value(PatientFormField::ConcessionNumber)
                .empty_to_none(),
            preferred_language: Some(self.get_value(PatientFormField::PreferredLanguage)),
            interpreter_required: Some(
                self.get_value(PatientFormField::InterpreterRequired) == "Yes",
            ),
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

        let dob = parse_date(&self.get_value(PatientFormField::DateOfBirth));
        let gender = self.get_value(PatientFormField::Gender).parse().ok();

        let address = Address {
            line1: self
                .get_value(PatientFormField::AddressLine1)
                .empty_to_none(),
            line2: self
                .get_value(PatientFormField::AddressLine2)
                .empty_to_none(),
            suburb: self.get_value(PatientFormField::Suburb).empty_to_none(),
            state: self.get_value(PatientFormField::State).empty_to_none(),
            postcode: self.get_value(PatientFormField::Postcode).empty_to_none(),
            country: or_default(self.get_value(PatientFormField::Country), "Australia"),
        };

        let emergency_contact = if !self.get_value(PatientFormField::EmergencyName).is_empty() {
            Some(EmergencyContact {
                name: self.get_value(PatientFormField::EmergencyName),
                phone: self.get_value(PatientFormField::EmergencyPhone),
                relationship: self.get_value(PatientFormField::EmergencyRelationship),
            })
        } else {
            None
        };

        let concession_type = self
            .get_value(PatientFormField::ConcessionType)
            .parse()
            .ok();
        let atsi_status = self.get_value(PatientFormField::AtsiStatus).parse().ok();
        let ihi = self.get_value(PatientFormField::Ihi);
        let ihi = if ihi.trim().is_empty() {
            None
        } else {
            Some(Ihi::new_strict(ihi).ok()?)
        };
        let medicare_number = self.get_value(PatientFormField::MedicareNumber);
        let medicare_number = if medicare_number.trim().is_empty() {
            None
        } else {
            Some(MedicareNumber::new_strict(medicare_number).ok()?)
        };
        let phone_home = self.get_value(PatientFormField::PhoneHome);
        let phone_home = if phone_home.trim().is_empty() {
            None
        } else {
            Some(PhoneNumber::new_strict(phone_home).ok()?)
        };
        let phone_mobile = self.get_value(PatientFormField::PhoneMobile);
        let phone_mobile = if phone_mobile.trim().is_empty() {
            None
        } else {
            Some(PhoneNumber::new_strict(phone_mobile).ok()?)
        };

        let data = opengp_domain::domain::patient::UpdatePatientData {
            ihi,
            medicare_number,
            medicare_irn: self.get_value(PatientFormField::MedicareIrn).parse().ok(),
            medicare_expiry: parse_date(&self.get_value(PatientFormField::MedicareExpiry)),
            title: self.get_value(PatientFormField::Title).empty_to_none(),
            first_name: Some(self.get_value(PatientFormField::FirstName)),
            middle_name: self.get_value(PatientFormField::MiddleName).empty_to_none(),
            last_name: Some(self.get_value(PatientFormField::LastName)),
            preferred_name: self
                .get_value(PatientFormField::PreferredName)
                .empty_to_none(),
            date_of_birth: dob,
            gender,
            address: Some(address),
            phone_home,
            phone_mobile,
            email: self.get_value(PatientFormField::Email).empty_to_none(),
            emergency_contact,
            concession_type,
            concession_number: self
                .get_value(PatientFormField::ConcessionNumber)
                .empty_to_none(),
            preferred_language: Some(self.get_value(PatientFormField::PreferredLanguage)),
            interpreter_required: Some(
                self.get_value(PatientFormField::InterpreterRequired) == "Yes",
            ),
            aboriginal_torres_strait_islander: atsi_status,
        };

        Some((patient_id, data))
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Option<PatientFormAction> {
        use crossterm::event::KeyEventKind;

        if key.kind != KeyEventKind::Press {
            return None;
        }

        self.sync_focus_to_state();

        if self.saving {
            return None;
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

        if !self.form_state.dropdowns.contains_key(&self.focused_field) {
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
            KeyCode::PageUp => {
                self.form_state.scroll.scroll_up();
                Some(PatientFormAction::ValueChanged)
            }
            KeyCode::PageDown => {
                self.form_state.scroll.scroll_down();
                Some(PatientFormAction::ValueChanged)
            }
            KeyCode::Enter => None,
            _ => {
                if let Some(action) = self.form_state.handle_navigation_key(key) {
                    if matches!(action, FormAction::Submit) {
                        let _ = FormNavigation::validate(self);
                    }
                    self.sync_focus_from_state();
                    return Some(action.into());
                }
                None
            }
        }
    }

    fn handle_dropdown_key(&mut self, key: KeyEvent) -> Option<Option<PatientFormAction>> {
        let field_id = self.focused_field.clone();
        if !self.form_state.dropdowns.contains_key(&field_id) {
            return None;
        }

        let action = {
            let dropdown = self.form_state.dropdowns.get_mut(&field_id)?;
            dropdown.handle_key(key)
        };

        let selected_value = if let Some(action) = action {
            match key.code {
                KeyCode::Tab | KeyCode::BackTab | KeyCode::Esc => {
                    return None;
                }
                _ => match action {
                    DropdownAction::Selected(_) | DropdownAction::Closed => self
                        .form_state
                        .dropdowns
                        .get(&field_id)
                        .and_then(|dropdown| dropdown.selected_value().map(|v| v.to_string())),
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
        };

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
                    self.sync_focus_to_state();
                    return Some(PatientFormAction::FocusChanged);
                }
                return None;
            }

            y += field_height;
        }

        None
    }

    fn sync_focus_to_state(&mut self) {
        if let Some(field) = PatientFormField::from_id(&self.focused_field) {
            self.form_state.focused_field = field;
        }
    }

    fn sync_focus_from_state(&mut self) {
        self.focused_field = self.form_state.focused_field.id().to_string();
    }
}

fn load_patient_field_definitions(
    patient_config: &opengp_config::PatientConfig,
) -> Vec<FieldDefinition> {
    if let Ok(config) = FormConfig::load() {
        if let Some(form) = config.forms.get("patient") {
            return form.fields.clone();
        }
    }

    fallback_patient_field_definitions(patient_config)
}

fn fallback_patient_field_definitions(
    patient_config: &opengp_config::PatientConfig,
) -> Vec<FieldDefinition> {
    PatientFormField::all()
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
                PatientFormField::FirstName | PatientFormField::LastName => ValidationRules {
                    max_length: Some(100),
                    required: true,
                    ..ValidationRules::default()
                },
                PatientFormField::Email => ValidationRules {
                    email: true,
                    ..ValidationRules::default()
                },
                PatientFormField::PhoneHome
                | PatientFormField::PhoneMobile
                | PatientFormField::EmergencyPhone => ValidationRules {
                    phone: true,
                    ..ValidationRules::default()
                },
                PatientFormField::DateOfBirth => ValidationRules {
                    required: true,
                    date_format: Some("dd/mm/yyyy".to_string()),
                    ..ValidationRules::default()
                },
                PatientFormField::MedicareNumber => ValidationRules {
                    max_length: Some(10),
                    ..ValidationRules::default()
                },
                PatientFormField::MedicareIrn => ValidationRules {
                    max_length: Some(1),
                    ..ValidationRules::default()
                },
                PatientFormField::MedicareExpiry => ValidationRules {
                    date_format: Some("dd/mm/yyyy".to_string()),
                    ..ValidationRules::default()
                },
                _ => ValidationRules::default(),
            };

            if field.is_dropdown() {
                definition.options = match field {
                    PatientFormField::Gender => patient_config
                        .gender
                        .iter()
                        .filter(|(_, opt)| opt.enabled)
                        .map(|(value, opt)| opengp_config::forms::SelectOption {
                            value: value.clone(),
                            label: opt.label.clone(),
                        })
                        .collect(),
                    PatientFormField::ConcessionType => patient_config
                        .concession_type
                        .iter()
                        .filter(|(_, opt)| opt.enabled)
                        .map(|(value, opt)| opengp_config::forms::SelectOption {
                            value: value.clone(),
                            label: opt.label.clone(),
                        })
                        .collect(),
                    PatientFormField::InterpreterRequired => vec![
                        opengp_config::forms::SelectOption {
                            value: "Yes".to_string(),
                            label: "Yes".to_string(),
                        },
                        opengp_config::forms::SelectOption {
                            value: "No".to_string(),
                            label: "No".to_string(),
                        },
                    ],
                    PatientFormField::AtsiStatus => patient_config
                        .atsi_status
                        .iter()
                        .filter(|(_, opt)| opt.enabled)
                        .map(|(value, opt)| opengp_config::forms::SelectOption {
                            value: value.clone(),
                            label: opt.label.clone(),
                        })
                        .collect(),
                    _ => vec![],
                };
            }

            definition.column = match field {
                PatientFormField::AddressLine1
                | PatientFormField::AddressLine2
                | PatientFormField::Suburb
                | PatientFormField::State
                | PatientFormField::Postcode
                | PatientFormField::Country
                | PatientFormField::EmergencyName
                | PatientFormField::EmergencyPhone
                | PatientFormField::EmergencyRelationship
                | PatientFormField::ConcessionType
                | PatientFormField::ConcessionNumber
                | PatientFormField::PreferredLanguage
                | PatientFormField::InterpreterRequired
                | PatientFormField::AtsiStatus => 1,
                _ => 0,
            };
            definition.width_percent = 50;

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

impl From<FormAction> for PatientFormAction {
    fn from(value: FormAction) -> Self {
        match value {
            FormAction::FocusChanged => PatientFormAction::FocusChanged,
            FormAction::ValueChanged => PatientFormAction::ValueChanged,
            FormAction::Submit => PatientFormAction::Submit,
            FormAction::Cancel => PatientFormAction::Cancel,
        }
    }
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
            .border_style(Style::default().fg(self.form_state.theme.colors.border));

        block.clone().render(area, buf);

        let inner = block.inner(area);
        if inner.is_empty() {
            return;
        }

        // Detect multi-column mode: check if any field has column > 0
        let is_multi_column = self.field_configs.values().any(|f| f.column > 0);

        // Check terminal width for multi-column rendering
        if is_multi_column && inner.width < 80 {
            let msg = Paragraph::new("Terminal too small for multi-column");
            msg.render(inner, buf);
            return;
        }

        let fields = self.field_ids.clone();
        let mut open_dropdown: Option<(DropdownWidget, Rect)> = None;

        if is_multi_column {
            // Multi-column render mode (no scroll)
            let [left_col, right_col] =
                Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .areas(inner);

            let mut left_y: i32 = (left_col.y as i32) + 1;
            let mut right_y: i32 = (right_col.y as i32) + 1;
            let max_y = (inner.y as i32) + (inner.height as i32) - 2;

            for field_id in fields {
                let field_config = self.field_configs.get(&field_id);
                let column = field_config.map(|fc| fc.column).unwrap_or(0);
                let width_percent = field_config.map(|fc| fc.width_percent).unwrap_or(100);
                let field_height = self.get_field_height(&field_id) as i32;
                let is_focused = field_id == self.focused_field;

                if width_percent == 100 {
                    // Full-width field
                    let y = max(left_y, right_y);
                    if y < max_y {
                        if let Some(dropdown) = self.form_state.dropdowns.get(&field_id).cloned() {
                            let dropdown_area =
                                Rect::new(inner.x + 1, y as u16, inner.width - 2, 3);
                            if dropdown.is_open() {
                                open_dropdown = Some((dropdown.clone(), dropdown_area));
                            }
                            dropdown.focused(is_focused).render(dropdown_area, buf);
                        } else if let Some(textarea) = self.form_state.textareas.get(&field_id) {
                            let textarea_area = Rect::new(
                                inner.x + 1,
                                y as u16,
                                inner.width - 2,
                                textarea.height(),
                            );
                            TextareaWidget::new(textarea, self.form_state.theme.clone())
                                .focused(is_focused)
                                .render(textarea_area, buf);
                        }
                    }
                    // Advance both columns
                    left_y += field_height;
                    right_y += field_height;
                } else if column == 0 {
                    // Left column
                    if left_y < max_y {
                        if let Some(dropdown) = self.form_state.dropdowns.get(&field_id).cloned() {
                            let dropdown_area = Rect::new(
                                left_col.x + 1,
                                left_y as u16,
                                left_col.width.saturating_sub(2),
                                3,
                            );
                            if dropdown.is_open() {
                                open_dropdown = Some((dropdown.clone(), dropdown_area));
                            }
                            dropdown.focused(is_focused).render(dropdown_area, buf);
                        } else if let Some(textarea) = self.form_state.textareas.get(&field_id) {
                            let textarea_area = Rect::new(
                                left_col.x + 1,
                                left_y as u16,
                                left_col.width.saturating_sub(2),
                                textarea.height(),
                            );
                            TextareaWidget::new(textarea, self.form_state.theme.clone())
                                .focused(is_focused)
                                .render(textarea_area, buf);
                        }
                    }
                    left_y += field_height;
                } else if column == 1 {
                    // Right column
                    if right_y < max_y {
                        if let Some(dropdown) = self.form_state.dropdowns.get(&field_id).cloned() {
                            let dropdown_area = Rect::new(
                                right_col.x + 1,
                                right_y as u16,
                                right_col.width.saturating_sub(2),
                                3,
                            );
                            if dropdown.is_open() {
                                open_dropdown = Some((dropdown.clone(), dropdown_area));
                            }
                            dropdown.focused(is_focused).render(dropdown_area, buf);
                        } else if let Some(textarea) = self.form_state.textareas.get(&field_id) {
                            let textarea_area = Rect::new(
                                right_col.x + 1,
                                right_y as u16,
                                right_col.width.saturating_sub(2),
                                textarea.height(),
                            );
                            TextareaWidget::new(textarea, self.form_state.theme.clone())
                                .focused(is_focused)
                                .render(textarea_area, buf);
                        }
                    }
                    right_y += field_height;
                }
            }
        } else {
            // Single-column render with scroll
            let mut total_height: u16 = 0;
            for field_id in &fields {
                total_height += self.get_field_height(field_id);
            }
            self.form_state.scroll.set_total_height(total_height);
            self.form_state
                .scroll
                .clamp_offset(inner.height.saturating_sub(2));

            let (focused_y, focused_height) = self.get_field_position(&self.focused_field);
            self.form_state.scroll.scroll_to_field(
                focused_y,
                focused_height,
                inner.height.saturating_sub(2),
            );

            let mut y: i32 = (inner.y as i32) + 1 - (self.form_state.scroll.scroll_offset as i32);
            let max_y = inner.y as i32 + inner.height as i32 - 2;

            for field_id in fields {
                let field_height = self.get_field_height(&field_id) as i32;

                if y + field_height <= inner.y as i32 || y >= max_y {
                    y += field_height;
                    continue;
                }

                let is_focused = field_id == self.focused_field;

                if let Some(dropdown) = self.form_state.dropdowns.get(&field_id).cloned() {
                    if y >= inner.y as i32 && y < max_y {
                        let dropdown_area = Rect::new(inner.x + 1, y as u16, inner.width - 2, 3);
                        if dropdown.is_open() {
                            open_dropdown = Some((dropdown.clone(), dropdown_area));
                        }
                        dropdown.focused(is_focused).render(dropdown_area, buf);
                    }
                    y += 3;
                    continue;
                }

                if let Some(textarea) = self.form_state.textareas.get(&field_id) {
                    let textarea_height = textarea.height() as i32;
                    if y >= inner.y as i32 && y < max_y {
                        let textarea_area = Rect::new(
                            inner.x + 1,
                            y as u16,
                            inner.width - 2,
                            textarea_height as u16,
                        );
                        TextareaWidget::new(textarea, self.form_state.theme.clone())
                            .focused(is_focused)
                            .render(textarea_area, buf);
                    }
                    y += textarea_height;
                }
            }

            self.form_state
                .scroll
                .render_scrollbar(inner, buf, &self.form_state.theme);
        }

        if let Some((dropdown, dropdown_area)) = open_dropdown {
            dropdown.render(dropdown_area, buf);
        }

        let help_y = inner.y + inner.height - 1;
        buf.set_string(
            inner.x + 1,
            help_y,
            "Tab: Next | Ctrl+S: Submit | Esc: Cancel",
            Style::default().fg(self.form_state.theme.colors.disabled),
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
        let form = PatientForm::new(theme, &PatientConfig::default());

        assert!(!form.is_edit_mode());
        assert_eq!(form.focused_field(), PatientFormField::FirstName);
        assert!(!form.has_errors());
    }

    #[test]
    fn test_form_validation_required() {
        let theme = Theme::dark();
        let mut form = PatientForm::new(theme, &PatientConfig::default());

        FormNavigation::validate(&mut form);
        assert!(form.has_errors());
        assert!(form.error(PatientFormField::FirstName).is_some());
        assert!(form.error(PatientFormField::LastName).is_some());
    }

    #[test]
    fn test_form_validation_email() {
        let theme = Theme::dark();
        let mut form = PatientForm::new(theme, &PatientConfig::default());

        form.set_value(PatientFormField::Email, "invalid".to_string());
        FormNavigation::validate(&mut form);
        assert!(form.error(PatientFormField::Email).is_some());

        form.set_value(PatientFormField::Email, "test@example.com".to_string());
        FormNavigation::validate(&mut form);
        assert!(form.error(PatientFormField::Email).is_none());
    }

    #[test]
    fn test_text_fields_use_textarea_state() {
        let theme = Theme::dark();
        let mut form = PatientForm::new(theme, &PatientConfig::default());

        form.set_value(PatientFormField::FirstName, "Alice".to_string());
        assert_eq!(form.get_value(PatientFormField::FirstName), "Alice");
        assert_eq!(
            form.form_state
                .textareas
                .get(FIELD_FIRST_NAME)
                .expect("first_name textarea should exist")
                .value(),
            "Alice"
        );
    }

    #[test]
    fn test_dynamic_form_string_access() {
        let theme = Theme::dark();
        let mut form = PatientForm::new(theme, &PatientConfig::default());

        <PatientForm as crate::ui::widgets::DynamicForm>::set_value(
            &mut form,
            FIELD_FIRST_NAME,
            "John".to_string(),
        );

        let by_string =
            <PatientForm as crate::ui::widgets::DynamicForm>::get_value(&form, FIELD_FIRST_NAME);
        let by_enum = form.get_value(PatientFormField::FirstName);
        assert_eq!(by_string, by_enum);
    }

    #[test]
    fn test_handle_key_char_updates_textarea() {
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

        let theme = Theme::dark();
        let mut form = PatientForm::new(theme, &PatientConfig::default());
        form.focused_field = FIELD_FIRST_NAME.to_string();

        let key = KeyEvent::new(KeyCode::Char('J'), KeyModifiers::NONE);
        let action = form.handle_key(key);
        assert!(action.is_some());
        assert_eq!(form.get_value(PatientFormField::FirstName), "J");
    }

    #[test]
    fn test_handle_key_tab_navigates_fields() {
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

        let theme = Theme::dark();
        let mut form = PatientForm::new(theme, &PatientConfig::default());
        assert_eq!(form.focused_field(), PatientFormField::FirstName);

        let key = KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE);
        form.handle_key(key);
        assert_eq!(form.focused_field(), PatientFormField::MiddleName);
    }

    #[test]
    fn test_single_line_height_mode() {
        let theme = Theme::dark();
        let form = PatientForm::new(theme, &PatientConfig::default());

        assert_eq!(
            form.form_state
                .textareas
                .get(FIELD_FIRST_NAME)
                .expect("first_name textarea should exist")
                .height_mode,
            HeightMode::SingleLine
        );
        assert_eq!(
            form.form_state
                .textareas
                .get(FIELD_LAST_NAME)
                .expect("last_name textarea should exist")
                .height_mode,
            HeightMode::SingleLine
        );
        assert_eq!(
            form.form_state
                .textareas
                .get(FIELD_EMAIL)
                .expect("email textarea should exist")
                .height_mode,
            HeightMode::SingleLine
        );
        assert_eq!(
            form.form_state
                .textareas
                .get(FIELD_MEDICARE_NUMBER)
                .expect("medicare_number textarea should exist")
                .height_mode,
            HeightMode::SingleLine
        );
    }

    #[test]
    fn test_error_synced_to_textarea_state() {
        let theme = Theme::dark();
        let mut form = PatientForm::new(theme, &PatientConfig::default());

        FormNavigation::validate(&mut form);
        assert!(form
            .form_state
            .textareas
            .get(FIELD_FIRST_NAME)
            .expect("first_name textarea should exist")
            .error
            .is_some());
        assert!(form
            .form_state
            .textareas
            .get(FIELD_LAST_NAME)
            .expect("last_name textarea should exist")
            .error
            .is_some());
    }

    #[test]
    fn test_to_new_patient_data_valid() {
        let theme = Theme::dark();
        let mut form = PatientForm::new(theme, &PatientConfig::default());

        form.set_value(PatientFormField::FirstName, "Alice".to_string());
        form.set_value(PatientFormField::LastName, "Smith".to_string());
        form.set_value(PatientFormField::DateOfBirth, "15/05/1990".to_string());
        form.set_value(PatientFormField::Gender, "Female".to_string());
        form.set_value(PatientFormField::Email, "alice@test.com".to_string());
        form.set_value(PatientFormField::PhoneMobile, "0412345678".to_string());
        form.set_value(PatientFormField::PreferredLanguage, "English".to_string());

        let result = form.to_new_patient_data();
        assert!(result.is_some());
        let data = result.expect("result should be present");
        assert_eq!(data.first_name, "Alice");
        assert_eq!(data.last_name, "Smith");
    }

    #[test]
    fn test_to_new_patient_data_invalid_returns_none() {
        let theme = Theme::dark();
        let mut form = PatientForm::new(theme, &PatientConfig::default());

        form.set_value(PatientFormField::FirstName, "Alice".to_string());
        form.set_value(PatientFormField::LastName, "".to_string());

        let result = form.to_new_patient_data();
        assert!(result.is_none());
    }

    #[test]
    fn test_submit_keeps_form_open_on_error() {
        let theme = Theme::dark();
        let mut form = PatientForm::new(theme, &PatientConfig::default());

        form.validate();
        assert!(form.has_errors());

        form.focus_first_error();
        assert_eq!(form.focused_field(), PatientFormField::FirstName);
    }
}

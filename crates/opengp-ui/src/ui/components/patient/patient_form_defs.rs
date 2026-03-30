use std::collections::HashMap;

use opengp_config::forms::{
    FieldDefinition, FieldType as ConfigFieldType, FormConfig, ValidationRules,
};

use crate::ui::widgets::{
    DropdownOption, FormField as SharedFormFieldTrait, FormFieldMeta, FormValidator, HeightMode,
    TextareaState,
};

pub(super) const FIELD_TITLE: &str = "title";
pub(super) const FIELD_FIRST_NAME: &str = "first_name";
pub(super) const FIELD_MIDDLE_NAME: &str = "middle_name";
pub(super) const FIELD_LAST_NAME: &str = "last_name";
pub(super) const FIELD_PREFERRED_NAME: &str = "preferred_name";
pub(super) const FIELD_DATE_OF_BIRTH: &str = "date_of_birth";
pub(super) const FIELD_GENDER: &str = "gender";
pub(super) const FIELD_ADDRESS_LINE1: &str = "address_line1";
pub(super) const FIELD_ADDRESS_LINE2: &str = "address_line2";
pub(super) const FIELD_SUBURB: &str = "suburb";
pub(super) const FIELD_STATE: &str = "state";
pub(super) const FIELD_POSTCODE: &str = "postcode";
pub(super) const FIELD_COUNTRY: &str = "country";
pub(super) const FIELD_PHONE_HOME: &str = "phone_home";
pub(super) const FIELD_PHONE_MOBILE: &str = "phone_mobile";
pub(super) const FIELD_EMAIL: &str = "email";
pub(super) const FIELD_MEDICARE_NUMBER: &str = "medicare_number";
pub(super) const FIELD_MEDICARE_IRN: &str = "medicare_irn";
pub(super) const FIELD_MEDICARE_EXPIRY: &str = "medicare_expiry";
pub(super) const FIELD_IHI: &str = "ihi";
pub(super) const FIELD_EMERGENCY_NAME: &str = "emergency_name";
pub(super) const FIELD_EMERGENCY_PHONE: &str = "emergency_phone";
pub(super) const FIELD_EMERGENCY_RELATIONSHIP: &str = "emergency_relationship";
pub(super) const FIELD_CONCESSION_TYPE: &str = "concession_type";
pub(super) const FIELD_CONCESSION_NUMBER: &str = "concession_number";
pub(super) const FIELD_PREFERRED_LANGUAGE: &str = "preferred_language";
pub(super) const FIELD_INTERPRETER_REQUIRED: &str = "interpreter_required";
pub(super) const FIELD_ATSI_STATUS: &str = "atsi_status";

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

impl PatientFormField {
    pub fn all() -> Vec<Self> {
        use strum::IntoEnumIterator;
        Self::iter().collect()
    }

    pub fn label(&self) -> &'static str {
        (*self).into()
    }

    pub fn id(&self) -> &'static str {
        match self {
            Self::Title => FIELD_TITLE,
            Self::FirstName => FIELD_FIRST_NAME,
            Self::MiddleName => FIELD_MIDDLE_NAME,
            Self::LastName => FIELD_LAST_NAME,
            Self::PreferredName => FIELD_PREFERRED_NAME,
            Self::DateOfBirth => FIELD_DATE_OF_BIRTH,
            Self::Gender => FIELD_GENDER,
            Self::AddressLine1 => FIELD_ADDRESS_LINE1,
            Self::AddressLine2 => FIELD_ADDRESS_LINE2,
            Self::Suburb => FIELD_SUBURB,
            Self::State => FIELD_STATE,
            Self::Postcode => FIELD_POSTCODE,
            Self::Country => FIELD_COUNTRY,
            Self::PhoneHome => FIELD_PHONE_HOME,
            Self::PhoneMobile => FIELD_PHONE_MOBILE,
            Self::Email => FIELD_EMAIL,
            Self::MedicareNumber => FIELD_MEDICARE_NUMBER,
            Self::MedicareIrn => FIELD_MEDICARE_IRN,
            Self::MedicareExpiry => FIELD_MEDICARE_EXPIRY,
            Self::Ihi => FIELD_IHI,
            Self::EmergencyName => FIELD_EMERGENCY_NAME,
            Self::EmergencyPhone => FIELD_EMERGENCY_PHONE,
            Self::EmergencyRelationship => FIELD_EMERGENCY_RELATIONSHIP,
            Self::ConcessionType => FIELD_CONCESSION_TYPE,
            Self::ConcessionNumber => FIELD_CONCESSION_NUMBER,
            Self::PreferredLanguage => FIELD_PREFERRED_LANGUAGE,
            Self::InterpreterRequired => FIELD_INTERPRETER_REQUIRED,
            Self::AtsiStatus => FIELD_ATSI_STATUS,
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            FIELD_TITLE => Some(Self::Title),
            FIELD_FIRST_NAME => Some(Self::FirstName),
            FIELD_MIDDLE_NAME => Some(Self::MiddleName),
            FIELD_LAST_NAME => Some(Self::LastName),
            FIELD_PREFERRED_NAME => Some(Self::PreferredName),
            FIELD_DATE_OF_BIRTH => Some(Self::DateOfBirth),
            FIELD_GENDER => Some(Self::Gender),
            FIELD_ADDRESS_LINE1 => Some(Self::AddressLine1),
            FIELD_ADDRESS_LINE2 => Some(Self::AddressLine2),
            FIELD_SUBURB => Some(Self::Suburb),
            FIELD_STATE => Some(Self::State),
            FIELD_POSTCODE => Some(Self::Postcode),
            FIELD_COUNTRY => Some(Self::Country),
            FIELD_PHONE_HOME => Some(Self::PhoneHome),
            FIELD_PHONE_MOBILE => Some(Self::PhoneMobile),
            FIELD_EMAIL => Some(Self::Email),
            FIELD_MEDICARE_NUMBER => Some(Self::MedicareNumber),
            FIELD_MEDICARE_IRN => Some(Self::MedicareIrn),
            FIELD_MEDICARE_EXPIRY => Some(Self::MedicareExpiry),
            FIELD_IHI => Some(Self::Ihi),
            FIELD_EMERGENCY_NAME => Some(Self::EmergencyName),
            FIELD_EMERGENCY_PHONE => Some(Self::EmergencyPhone),
            FIELD_EMERGENCY_RELATIONSHIP => Some(Self::EmergencyRelationship),
            FIELD_CONCESSION_TYPE => Some(Self::ConcessionType),
            FIELD_CONCESSION_NUMBER => Some(Self::ConcessionNumber),
            FIELD_PREFERRED_LANGUAGE => Some(Self::PreferredLanguage),
            FIELD_INTERPRETER_REQUIRED => Some(Self::InterpreterRequired),
            FIELD_ATSI_STATUS => Some(Self::AtsiStatus),
            _ => None,
        }
    }

    pub fn is_required(&self) -> bool {
        matches!(
            self,
            Self::FirstName | Self::LastName | Self::DateOfBirth | Self::Gender
        )
    }

    pub fn is_dropdown(&self) -> bool {
        matches!(
            self,
            Self::Gender | Self::ConcessionType | Self::InterpreterRequired | Self::AtsiStatus
        )
    }

    pub fn is_textarea(&self) -> bool {
        !self.is_dropdown()
    }
}

impl FormFieldMeta for PatientFormField {
    fn label(&self) -> &'static str {
        Self::label(self)
    }

    fn is_required(&self) -> bool {
        Self::is_required(self)
    }
}

impl SharedFormFieldTrait for PatientFormField {
    fn all() -> Vec<Self> {
        Self::all()
    }

    fn label(&self) -> &'static str {
        Self::label(self)
    }

    fn id(&self) -> &'static str {
        Self::id(self)
    }

    fn from_id(id: &str) -> Option<Self> {
        Self::from_id(id)
    }

    fn is_required(&self) -> bool {
        Self::is_required(self)
    }

    fn is_textarea(&self) -> bool {
        Self::is_textarea(self)
    }

    fn is_dropdown(&self) -> bool {
        Self::is_dropdown(self)
    }
}

pub(super) fn load_patient_field_definitions() -> Vec<FieldDefinition> {
    if let Ok(config) = FormConfig::load() {
        if let Some(form) = config.forms.get("patient") {
            return form.fields.clone();
        }
    }
    fallback_patient_field_definitions()
}

fn fallback_patient_field_definitions() -> Vec<FieldDefinition> {
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
                    PatientFormField::Gender => options(&[
                        ("Male", "Male"),
                        ("Female", "Female"),
                        ("Other", "Other"),
                        ("PreferNotToSay", "Prefer not to say"),
                    ]),
                    PatientFormField::ConcessionType => options(&[
                        ("DVA", "DVA"),
                        ("Pensioner", "Pensioner"),
                        ("HealthcareCard", "Healthcare Card"),
                        ("SafetyNetCard", "Safety Net Card"),
                    ]),
                    PatientFormField::InterpreterRequired => {
                        options(&[("Yes", "Yes"), ("No", "No")])
                    }
                    PatientFormField::AtsiStatus => options(&[
                        (
                            "AboriginalNotTorresStrait",
                            "Aboriginal (not Torres Strait)",
                        ),
                        (
                            "TorresStraitNotAboriginal",
                            "Torres Strait (not Aboriginal)",
                        ),
                        (
                            "BothAboriginalAndTorresStrait",
                            "Both Aboriginal and Torres Strait",
                        ),
                        (
                            "NeitherAboriginalNorTorresStrait",
                            "Neither Aboriginal nor Torres Strait",
                        ),
                        ("NotStated", "Not stated"),
                    ]),
                    _ => vec![],
                };
            }

            definition
        })
        .collect()
}

fn options(items: &[(&str, &str)]) -> Vec<opengp_config::forms::SelectOption> {
    items
        .iter()
        .map(|(value, label)| opengp_config::forms::SelectOption {
            value: (*value).to_string(),
            label: (*label).to_string(),
        })
        .collect()
}

pub(super) fn make_textarea_state(field: &FieldDefinition, value: Option<String>) -> TextareaState {
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

pub(super) fn build_validator(field_configs: &HashMap<String, FieldDefinition>) -> FormValidator {
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

pub(super) fn dropdown_options(field: &FieldDefinition) -> Vec<DropdownOption> {
    field
        .options
        .iter()
        .map(|option| DropdownOption::new(option.value.as_str(), option.label.as_str()))
        .collect()
}

//! Form configuration types for TOML-based form definitions
//!
//! Provides Serde-derivable types for loading and managing form configurations
//! from TOML files. All types support deserialization with sensible defaults.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

/// Top-level form configuration container
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FormConfig {
    /// Map of form name to form definition
    pub forms: HashMap<String, FormDefinition>,
}

impl FormConfig {
    pub fn load() -> Result<Self, FormConfigError> {
        let defaults = include_str!("forms.toml");
        let mut config: FormConfig = toml::from_str(defaults).map_err(FormConfigError::Parse)?;

        if let Ok(path) = std::env::var("FORMS_CONFIG_PATH") {
            let content = std::fs::read_to_string(&path).map_err(FormConfigError::Io)?;
            let overrides: PartialFormConfig =
                toml::from_str(&content).map_err(FormConfigError::Parse)?;
            config.deep_merge(overrides);
        }

        config.validate()?;
        Ok(config)
    }

    fn deep_merge(&mut self, overrides: PartialFormConfig) {
        for (form_name, form_override) in overrides.forms {
            if let Some(existing_form) = self.forms.get_mut(&form_name) {
                existing_form.merge(form_override);
            } else {
                self.forms
                    .insert(form_name, form_override.into_form_definition());
            }
        }
    }

    fn validate(&self) -> Result<(), FormConfigError> {
        for (form_name, form) in &self.forms {
            for field in &form.fields {
                if field.id.trim().is_empty() {
                    return Err(FormConfigError::Validation(format!(
                        "form '{form_name}' contains a field with empty id"
                    )));
                }

                if field.required && !field.visible {
                    return Err(FormConfigError::Validation(format!(
                        "form '{form_name}' field '{}' is required but hidden",
                        field.id
                    )));
                }

                if field.field_type == FieldType::Select {
                    if field.options.is_empty() {
                        return Err(FormConfigError::Validation(format!(
                            "form '{form_name}' field '{}' is a dropdown with no options",
                            field.id
                        )));
                    }

                    if field
                        .options
                        .iter()
                        .any(|option| option.value.trim().is_empty())
                    {
                        return Err(FormConfigError::Validation(format!(
                            "form '{form_name}' field '{}' has dropdown options with empty values",
                            field.id
                        )));
                    }
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
pub enum FormConfigError {
    Parse(toml::de::Error),
    Validation(String),
    Io(std::io::Error),
}

impl fmt::Display for FormConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse(err) => write!(f, "Failed to parse forms configuration: {err}"),
            Self::Validation(message) => {
                write!(f, "Invalid forms configuration: {message}")
            }
            Self::Io(err) => write!(f, "Failed to read forms configuration: {err}"),
        }
    }
}

impl Error for FormConfigError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Parse(err) => Some(err),
            Self::Validation(_) => None,
            Self::Io(err) => Some(err),
        }
    }
}

#[derive(Debug, Deserialize, Default)]
struct PartialFormConfig {
    #[serde(default)]
    forms: HashMap<String, PartialFormDefinition>,
}

#[derive(Debug, Deserialize, Default)]
struct PartialFormDefinition {
    title: Option<String>,
    #[serde(default)]
    fields: Vec<PartialFieldDefinition>,
    rules: Option<Vec<FormRule>>,
}

impl PartialFormDefinition {
    fn into_form_definition(self) -> FormDefinition {
        FormDefinition {
            title: self.title.unwrap_or_default(),
            fields: self
                .fields
                .into_iter()
                .map(PartialFieldDefinition::into_field_definition)
                .collect(),
            rules: self.rules.unwrap_or_default(),
        }
    }
}

impl FormDefinition {
    fn merge(&mut self, form_override: PartialFormDefinition) {
        if let Some(title) = form_override.title {
            self.title = title;
        }

        if let Some(rules) = form_override.rules {
            self.rules = rules;
        }

        for field_override in form_override.fields {
            if let Some(existing_field) = self
                .fields
                .iter_mut()
                .find(|existing| existing.id == field_override.id)
            {
                existing_field.merge(field_override);
            } else {
                self.fields.push(field_override.into_field_definition());
            }
        }
    }
}

#[derive(Debug, Deserialize)]
struct PartialFieldDefinition {
    id: String,
    label: Option<String>,
    #[serde(rename = "type")]
    field_type: Option<FieldType>,
    required: Option<bool>,
    visible: Option<bool>,
    navigable: Option<bool>,
    options: Option<Vec<SelectOption>>,
    validation: Option<ValidationRules>,
    placeholder: Option<String>,
}

impl PartialFieldDefinition {
    fn into_field_definition(self) -> FieldDefinition {
        let mut field = FieldDefinition {
            id: self.id,
            ..FieldDefinition::default()
        };

        if let Some(label) = self.label {
            field.label = label;
        }
        if let Some(field_type) = self.field_type {
            field.field_type = field_type;
        }
        if let Some(required) = self.required {
            field.required = required;
        }
        if let Some(visible) = self.visible {
            field.visible = visible;
        }
        if let Some(navigable) = self.navigable {
            field.navigable = navigable;
        }
        if let Some(options) = self.options {
            field.options = options;
        }
        if let Some(validation) = self.validation {
            field.validation = validation;
        }
        if let Some(placeholder) = self.placeholder {
            field.placeholder = Some(placeholder);
        }

        field
    }
}

impl FieldDefinition {
    fn merge(&mut self, field_override: PartialFieldDefinition) {
        if let Some(label) = field_override.label {
            self.label = label;
        }
        if let Some(field_type) = field_override.field_type {
            self.field_type = field_type;
        }
        if let Some(required) = field_override.required {
            self.required = required;
        }
        if let Some(visible) = field_override.visible {
            self.visible = visible;
        }
        if let Some(navigable) = field_override.navigable {
            self.navigable = navigable;
        }
        if let Some(options) = field_override.options {
            self.options = options;
        }
        if let Some(validation) = field_override.validation {
            self.validation = validation;
        }
        if let Some(placeholder) = field_override.placeholder {
            self.placeholder = Some(placeholder);
        }
    }
}

/// Definition of a single form
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FormDefinition {
    /// Display title of the form
    pub title: String,
    /// List of fields in the form
    pub fields: Vec<FieldDefinition>,
    /// Optional validation rules for the form
    #[serde(default)]
    pub rules: Vec<FormRule>,
}

/// Definition of a single form field
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldDefinition {
    /// Unique identifier for the field
    pub id: String,
    /// Display label for the field
    pub label: String,
    /// Type of field (text, date, select, etc.)
    #[serde(rename = "type")]
    pub field_type: FieldType,
    /// Whether the field is required
    #[serde(default)]
    pub required: bool,
    /// Whether the field is visible
    #[serde(default = "default_true")]
    pub visible: bool,
    /// Whether the field is navigable
    #[serde(default = "default_true")]
    pub navigable: bool,
    /// Options for select fields
    #[serde(default)]
    pub options: Vec<SelectOption>,
    /// Validation rules for the field
    #[serde(default)]
    pub validation: ValidationRules,
    /// Placeholder text for the field
    pub placeholder: Option<String>,
}

impl Default for FieldDefinition {
    fn default() -> Self {
        Self {
            id: String::new(),
            label: String::new(),
            field_type: FieldType::Text,
            required: false,
            visible: true,
            navigable: true,
            options: Vec::new(),
            validation: ValidationRules::default(),
            placeholder: None,
        }
    }
}

/// Type of form field
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FieldType {
    /// Text input field
    #[default]
    Text,
    /// Date input field
    Date,
    /// Select dropdown field
    Select,
    /// Numeric input field
    Numeric,
    /// Textarea field
    Textarea,
}

/// Option for select fields
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SelectOption {
    /// Value of the option
    pub value: String,
    /// Display label of the option
    pub label: String,
}

/// Validation rules for a field
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ValidationRules {
    /// Maximum length for text fields
    pub max_length: Option<usize>,
    /// Minimum length for text fields
    pub min_length: Option<usize>,
    /// Whether the field is required
    #[serde(default)]
    pub required: bool,
    /// Whether the field must be a valid email
    #[serde(default)]
    pub email: bool,
    /// Whether the field must be a valid phone number
    #[serde(default)]
    pub phone: bool,
    /// Numeric range validation
    pub numeric_range: Option<NumericRange>,
    /// Regular expression pattern for validation
    pub regex: Option<String>,
    /// Expected date format
    pub date_format: Option<String>,
}

/// Numeric range validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NumericRange {
    /// Minimum value
    pub min: f64,
    /// Maximum value
    pub max: f64,
}

impl Default for NumericRange {
    fn default() -> Self {
        Self {
            min: 0.0,
            max: 100.0,
        }
    }
}

/// Form-level validation rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormRule {
    /// Type of rule
    #[serde(rename = "type")]
    pub rule_type: FormRuleType,
    /// Fields involved in the rule
    pub fields: Vec<String>,
    /// Error message to display
    pub message: String,
}

impl Default for FormRule {
    fn default() -> Self {
        Self {
            rule_type: FormRuleType::AnyNotEmpty,
            fields: Vec::new(),
            message: String::new(),
        }
    }
}

/// Type of form-level validation rule
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FormRuleType {
    /// At least one of the specified fields must not be empty
    #[default]
    AnyNotEmpty,
}

/// Helper function for default_true serde attribute
fn default_true() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn test_form_config_default() {
        let config = FormConfig::default();
        assert!(config.forms.is_empty());
    }

    #[test]
    fn test_field_definition_default() {
        let field = FieldDefinition::default();
        assert_eq!(field.id, "");
        assert_eq!(field.label, "");
        assert_eq!(field.field_type, FieldType::Text);
        assert!(!field.required);
        assert!(field.visible);
        assert!(field.navigable);
    }

    #[test]
    fn test_field_type_default() {
        assert_eq!(FieldType::default(), FieldType::Text);
    }

    #[test]
    fn test_select_option_default() {
        let option = SelectOption::default();
        assert_eq!(option.value, "");
        assert_eq!(option.label, "");
    }

    #[test]
    fn test_validation_rules_default() {
        let rules = ValidationRules::default();
        assert!(rules.max_length.is_none());
        assert!(rules.min_length.is_none());
        assert!(!rules.required);
        assert!(!rules.email);
        assert!(!rules.phone);
    }

    #[test]
    fn test_numeric_range_default() {
        let range = NumericRange::default();
        assert_eq!(range.min, 0.0);
        assert_eq!(range.max, 100.0);
    }

    #[test]
    fn test_form_rule_default() {
        let rule = FormRule::default();
        assert_eq!(rule.rule_type, FormRuleType::AnyNotEmpty);
        assert!(rule.fields.is_empty());
    }

    #[test]
    fn test_form_rule_type_default() {
        assert_eq!(FormRuleType::default(), FormRuleType::AnyNotEmpty);
    }

    #[test]
    fn test_field_type_serialization() {
        let field_type = FieldType::Text;
        let json = serde_json::to_string(&field_type).unwrap();
        assert_eq!(json, "\"text\"");

        let field_type = FieldType::Textarea;
        let json = serde_json::to_string(&field_type).unwrap();
        assert_eq!(json, "\"textarea\"");
    }

    #[test]
    fn test_field_type_deserialization() {
        let json = "\"text\"";
        let field_type: FieldType = serde_json::from_str(json).unwrap();
        assert_eq!(field_type, FieldType::Text);

        let json = "\"numeric\"";
        let field_type: FieldType = serde_json::from_str(json).unwrap();
        assert_eq!(field_type, FieldType::Numeric);
    }

    #[test]
    fn test_form_rule_type_serialization() {
        let rule_type = FormRuleType::AnyNotEmpty;
        let json = serde_json::to_string(&rule_type).unwrap();
        assert_eq!(json, "\"any_not_empty\"");
    }

    #[test]
    fn test_form_rule_type_deserialization() {
        let json = "\"any_not_empty\"";
        let rule_type: FormRuleType = serde_json::from_str(json).unwrap();
        assert_eq!(rule_type, FormRuleType::AnyNotEmpty);
    }

    #[test]
    fn test_form_definition_default() {
        let form = FormDefinition::default();
        assert_eq!(form.title, "");
        assert!(form.fields.is_empty());
        assert!(form.rules.is_empty());
    }

    #[test]
    fn test_toml_deserialization_simple() {
        let toml_str = r#"
[forms.patient]
title = "Patient Registration"

[[forms.patient.fields]]
id = "first_name"
label = "First Name"
type = "text"
required = true
visible = true
navigable = true

[forms.patient.fields.validation]
max_length = 100
"#;

        let config: FormConfig = toml::from_str(toml_str).expect("should deserialize");
        assert!(config.forms.contains_key("patient"));

        let patient_form = &config.forms["patient"];
        assert_eq!(patient_form.title, "Patient Registration");
        assert_eq!(patient_form.fields.len(), 1);

        let first_field = &patient_form.fields[0];
        assert_eq!(first_field.id, "first_name");
        assert_eq!(first_field.label, "First Name");
        assert_eq!(first_field.field_type, FieldType::Text);
        assert!(first_field.required);
        assert!(first_field.visible);
        assert!(first_field.navigable);
        assert_eq!(first_field.validation.max_length, Some(100));
    }

    #[test]
    fn test_toml_deserialization_with_rules() {
        let toml_str = r#"
[forms.contact]
title = "Contact Form"

[[forms.contact.fields]]
id = "email"
label = "Email"
type = "text"

[[forms.contact.rules]]
type = "any_not_empty"
fields = ["email", "phone"]
message = "Either email or phone is required"
"#;

        let config: FormConfig = toml::from_str(toml_str).expect("should deserialize");
        let contact_form = &config.forms["contact"];
        assert_eq!(contact_form.rules.len(), 1);

        let rule = &contact_form.rules[0];
        assert_eq!(rule.rule_type, FormRuleType::AnyNotEmpty);
        assert_eq!(rule.fields, vec!["email", "phone"]);
        assert_eq!(rule.message, "Either email or phone is required");
    }

    #[test]
    fn test_toml_deserialization_with_select_options() {
        let toml_str = r#"
[forms.gender]
title = "Gender Selection"

[[forms.gender.fields]]
id = "gender"
label = "Gender"
type = "select"

[[forms.gender.fields.options]]
value = "M"
label = "Male"

[[forms.gender.fields.options]]
value = "F"
label = "Female"

[[forms.gender.fields.options]]
value = "O"
label = "Other"
"#;

        let config: FormConfig = toml::from_str(toml_str).expect("should deserialize");
        let gender_form = &config.forms["gender"];
        let gender_field = &gender_form.fields[0];

        assert_eq!(gender_field.field_type, FieldType::Select);
        assert_eq!(gender_field.options.len(), 3);
        assert_eq!(gender_field.options[0].value, "M");
        assert_eq!(gender_field.options[0].label, "Male");
    }

    #[test]
    fn test_toml_deserialization_with_numeric_range() {
        let toml_str = r#"
[forms.age]
title = "Age Form"

[[forms.age.fields]]
id = "age"
label = "Age"
type = "numeric"

[forms.age.fields.validation]
numeric_range = { min = 0.0, max = 150.0 }
"#;

        let config: FormConfig = toml::from_str(toml_str).expect("should deserialize");
        let age_form = &config.forms["age"];
        let age_field = &age_form.fields[0];

        assert_eq!(age_field.field_type, FieldType::Numeric);
        let range = age_field.validation.numeric_range.as_ref().unwrap();
        assert_eq!(range.min, 0.0);
        assert_eq!(range.max, 150.0);
    }

    #[test]
    fn test_toml_deserialization_defaults() {
        let toml_str = r#"
[forms.minimal]
title = "Minimal Form"

[[forms.minimal.fields]]
id = "field1"
label = "Field 1"
type = "text"
"#;

        let config: FormConfig = toml::from_str(toml_str).expect("should deserialize");
        let form = &config.forms["minimal"];
        let field = &form.fields[0];

        assert!(!field.required);
        assert!(field.visible);
        assert!(field.navigable);
        assert!(field.options.is_empty());
        assert!(field.placeholder.is_none());
    }

    #[test]
    fn test_load_embedded_defaults() {
        temp_env::with_vars([("FORMS_CONFIG_PATH", None::<&str>)], || {
            let config = FormConfig::load().expect("embedded forms should load");
            assert!(config.forms.contains_key("patient"));
            assert!(config.forms.contains_key("appointment"));

            assert_eq!(config.forms.len(), 8);
            assert!(config.forms.contains_key("vitals"));
            assert!(config.forms.contains_key("consultation"));
            assert!(config.forms.contains_key("allergy"));
            assert!(config.forms.contains_key("medical_history"));
            assert!(config.forms.contains_key("family_history"));
            assert!(config.forms.contains_key("social_history"));

            let patient_form = config
                .forms
                .get("patient")
                .expect("patient form should exist");
            assert!(patient_form.fields.len() >= 28);

            let first_name = patient_form
                .fields
                .iter()
                .find(|field| field.id == "first_name")
                .expect("first_name field should exist");
            assert!(first_name.required);
            assert_eq!(first_name.field_type, FieldType::Text);

            let gender = patient_form
                .fields
                .iter()
                .find(|field| field.id == "gender")
                .expect("gender field should exist");
            assert_eq!(gender.field_type, FieldType::Select);
            assert!(!gender.options.is_empty());

            let vitals_form = config
                .forms
                .get("vitals")
                .expect("vitals form should exist");
            assert!(vitals_form
                .rules
                .iter()
                .any(|rule| rule.rule_type == FormRuleType::AnyNotEmpty));
        });
    }

    #[test]
    fn test_load_external_override_deep_merge() {
        temp_env::with_vars([("FORMS_CONFIG_PATH", None::<&str>)], || {
            let override_toml = r#"
[forms.patient]

[[forms.patient.fields]]
id = "first_name"
label = "Given Name"
navigable = false

[[forms.patient.fields]]
id = "nickname"
label = "Nickname"
type = "text"
visible = true

[forms.custom]
title = "Custom Form"

[[forms.custom.fields]]
id = "custom_field"
label = "Custom Field"
type = "text"
required = true
visible = true
"#;

            let path = write_temp_forms_override(override_toml);

            temp_env::with_vars(
                [("FORMS_CONFIG_PATH", Some(path.to_string_lossy().as_ref()))],
                || {
                    let config = FormConfig::load().expect("forms with override should load");
                    let patient = config
                        .forms
                        .get("patient")
                        .expect("patient form should exist");

                    let first_name = patient
                        .fields
                        .iter()
                        .find(|field| field.id == "first_name")
                        .expect("first_name field should exist");
                    assert_eq!(first_name.label, "Given Name");
                    assert!(!first_name.navigable);

                    let last_name = patient
                        .fields
                        .iter()
                        .find(|field| field.id == "last_name")
                        .expect("last_name field should remain from defaults");
                    assert_eq!(last_name.label, "Last Name *");

                    let nickname = patient
                        .fields
                        .iter()
                        .find(|field| field.id == "nickname")
                        .expect("new field should be added");
                    assert_eq!(nickname.label, "Nickname");

                    let custom = config
                        .forms
                        .get("custom")
                        .expect("new form from override should be added");
                    assert_eq!(custom.title, "Custom Form");
                    assert_eq!(custom.fields.len(), 1);
                    assert_eq!(custom.fields[0].id, "custom_field");
                },
            );

            let _ = fs::remove_file(path);
        });
    }

    #[test]
    fn test_load_validation_error_for_required_hidden_field() {
        temp_env::with_vars([("FORMS_CONFIG_PATH", None::<&str>)], || {
            let override_toml = r#"
[forms.patient]

[[forms.patient.fields]]
id = "first_name"
required = true
visible = false
"#;
            let path = write_temp_forms_override(override_toml);

            temp_env::with_vars(
                [("FORMS_CONFIG_PATH", Some(path.to_string_lossy().as_ref()))],
                || {
                    let err = FormConfig::load().expect_err("required hidden field should fail");
                    assert!(matches!(err, FormConfigError::Validation(_)));
                    assert!(err.to_string().contains("required but hidden"));
                },
            );

            let _ = fs::remove_file(path);
        });
    }

    #[test]
    fn test_load_validation_error_for_empty_dropdown_options() {
        temp_env::with_vars([("FORMS_CONFIG_PATH", None::<&str>)], || {
            let override_toml = r#"
[forms.new_select]
title = "Select Test"

[[forms.new_select.fields]]
id = "choice"
label = "Choice"
type = "select"
required = true
visible = true
options = []
"#;
            let path = write_temp_forms_override(override_toml);

            temp_env::with_vars(
                [("FORMS_CONFIG_PATH", Some(path.to_string_lossy().as_ref()))],
                || {
                    let err = FormConfig::load().expect_err("empty dropdown should fail");
                    assert!(matches!(err, FormConfigError::Validation(_)));
                    assert!(err.to_string().contains("dropdown with no options"));
                },
            );

            let _ = fs::remove_file(path);
        });
    }

    #[test]
    fn test_load_validation_error_for_empty_field_id() {
        temp_env::with_vars([("FORMS_CONFIG_PATH", None::<&str>)], || {
            let override_toml = r#"
[forms.invalid]
title = "Invalid"

[[forms.invalid.fields]]
id = ""
label = "No Id"
type = "text"
visible = true
"#;
            let path = write_temp_forms_override(override_toml);

            temp_env::with_vars(
                [("FORMS_CONFIG_PATH", Some(path.to_string_lossy().as_ref()))],
                || {
                    let err = FormConfig::load().expect_err("empty id should fail");
                    assert!(matches!(err, FormConfigError::Validation(_)));
                    assert!(err.to_string().contains("empty id"));
                },
            );

            let _ = fs::remove_file(path);
        });
    }

    #[test]
    fn test_load_external_malformed_toml_returns_parse_error() {
        temp_env::with_vars([("FORMS_CONFIG_PATH", None::<&str>)], || {
            let malformed_toml = "[forms.invalid\n title = \"broken\"";
            let path = write_temp_forms_override(malformed_toml);

            temp_env::with_vars(
                [("FORMS_CONFIG_PATH", Some(path.to_string_lossy().as_ref()))],
                || {
                    let err = FormConfig::load().expect_err("malformed toml should fail");
                    assert!(matches!(err, FormConfigError::Parse(_)));
                },
            );

            let _ = fs::remove_file(path);
        });
    }

    fn write_temp_forms_override(content: &str) -> PathBuf {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be after unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("opengp-forms-override-{now}.toml"));
        fs::write(&path, content).expect("should write temporary forms override");
        path
    }
}

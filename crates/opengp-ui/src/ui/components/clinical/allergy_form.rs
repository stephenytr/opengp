//! Allergy Form Component
//!
//! Form for creating or editing a patient allergy.

use std::collections::HashMap;

use chrono::NaiveDate;
use opengp_config::forms::ValidationRules;
use opengp_config::AllergyConfig;
use opengp_domain::domain::clinical::{Allergy, AllergyType, Severity};
use uuid::Uuid;
use rat_focus::{FocusFlag, HasFocus, FocusBuilder};

use crate::ui::shared::{FormAction, FormMode};
use crate::ui::theme::Theme;
use crate::ui::widgets::{
    format_date, impl_form_field_wrapper, parse_date, DatePickerPopup, DropdownOption,
    DropdownWidget, FormField, FormFieldMeta, FormNavigation,
    FormState, FormValidator, HeightMode, TextareaState,
};

const FIELD_ALLERGEN: &str = "allergen";
const FIELD_ALLERGY_TYPE: &str = "allergy_type";
const FIELD_SEVERITY: &str = "severity";
const FIELD_REACTION: &str = "reaction";
const FIELD_ONSET_DATE: &str = "onset_date";
const FIELD_NOTES: &str = "notes";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, strum::EnumIter, strum::IntoStaticStr)]
pub enum AllergyFormField {
    #[strum(to_string = "Allergen *")]
    Allergen,
    #[strum(to_string = "Allergy Type *")]
    AllergyType,
    #[strum(to_string = "Severity *")]
    Severity,
    #[strum(to_string = "Reaction")]
    Reaction,
    #[strum(to_string = "Onset Date (dd/mm/yyyy)")]
    OnsetDate,
    #[strum(to_string = "Notes")]
    Notes,
}

impl_form_field_wrapper!(AllergyFormField, opengp_config::forms::FieldDefinition);

impl FormField for AllergyFormField {
    fn all() -> Vec<Self> {
        use strum::IntoEnumIterator;
        Self::iter().collect()
    }

    fn label(&self) -> &'static str {
        (*self).into()
    }

    fn id(&self) -> &'static str {
        match self {
            AllergyFormField::Allergen => FIELD_ALLERGEN,
            AllergyFormField::AllergyType => FIELD_ALLERGY_TYPE,
            AllergyFormField::Severity => FIELD_SEVERITY,
            AllergyFormField::Reaction => FIELD_REACTION,
            AllergyFormField::OnsetDate => FIELD_ONSET_DATE,
            AllergyFormField::Notes => FIELD_NOTES,
        }
    }

    fn from_id(id: &str) -> Option<Self> {
        match id {
            FIELD_ALLERGEN => Some(AllergyFormField::Allergen),
            FIELD_ALLERGY_TYPE => Some(AllergyFormField::AllergyType),
            FIELD_SEVERITY => Some(AllergyFormField::Severity),
            FIELD_REACTION => Some(AllergyFormField::Reaction),
            FIELD_ONSET_DATE => Some(AllergyFormField::OnsetDate),
            FIELD_NOTES => Some(AllergyFormField::Notes),
            _ => None,
        }
    }

    fn is_required(&self) -> bool {
        matches!(
            self,
            AllergyFormField::Allergen | AllergyFormField::AllergyType | AllergyFormField::Severity
        )
    }

    fn is_textarea(&self) -> bool {
        matches!(
            self,
            AllergyFormField::Allergen | AllergyFormField::Reaction | AllergyFormField::Notes
        )
    }

    fn is_dropdown(&self) -> bool {
        matches!(
            self,
            AllergyFormField::AllergyType | AllergyFormField::Severity
        )
    }
}

pub type AllergyFormAction = FormAction;

mod interaction;
mod render;
#[cfg(test)]
mod tests;

pub struct AllergyForm {
    form_state: FormState<AllergyFormField>,
    field_ids: Vec<String>,
    allergy_type: Option<AllergyType>,
    severity: Option<Severity>,
    onset_date: Option<NaiveDate>,
    is_valid: bool,
    validator: FormValidator,
    date_picker: DatePickerPopup,
    pub focus: FocusFlag,
}

impl Clone for AllergyForm {
    fn clone(&self) -> Self {
        Self {
            form_state: self.form_state.clone(),
            field_ids: self.field_ids.clone(),
            allergy_type: self.allergy_type,
            severity: self.severity,
            onset_date: self.onset_date,
            is_valid: self.is_valid,
            validator: build_validator(),
            date_picker: self.date_picker.clone(),
            focus: self.focus.clone(),
        }
    }
}

impl AllergyForm {
    pub fn new(theme: Theme, allergy_config: &AllergyConfig) -> Self {
        // Build allergy type options from config, filtering to enabled only
        let allergy_type_options: Vec<DropdownOption> = allergy_config
            .allergy_types
            .iter()
            .filter(|(_, option)| option.enabled)
            .map(|(key, option)| DropdownOption::new(key, &option.label))
            .collect();

        let severity_options: Vec<DropdownOption> = allergy_config
            .severities
            .iter()
            .filter(|(_, option)| option.enabled)
            .map(|(key, option)| DropdownOption::new(key, &option.label))
            .collect();

        // Build severity options from config, filtering to enabled only
        let severity_options: Vec<DropdownOption> = allergy_config
            .severities
            .iter()
            .filter(|(_, option)| option.enabled)
            .map(|(key, option)| DropdownOption::new(key, &option.label))
            .collect();

        let form_state = FormState::new(theme.clone(), AllergyFormField::Allergen);

        let field_ids = AllergyFormField::all()
            .into_iter()
            .map(|field| field.id().to_string())
            .collect();

        let mut form = Self {
            form_state,
            field_ids,
            allergy_type: None,
            severity: None,
            onset_date: None,
            is_valid: false,
            validator: FormValidator::new(&HashMap::new()),
            date_picker: DatePickerPopup::new(theme.clone()),
            focus: FocusFlag::default(),
        };

        form.form_state.textareas.insert(
            FIELD_ALLERGEN.to_string(),
            TextareaState::new("Allergen *").with_height_mode(HeightMode::SingleLine),
        );
        form.form_state.textareas.insert(
            FIELD_REACTION.to_string(),
            TextareaState::new("Reaction").with_height_mode(HeightMode::SingleLine),
        );
        form.form_state.textareas.insert(
            FIELD_NOTES.to_string(),
            TextareaState::new("Notes").with_height_mode(HeightMode::FixedLines(3)),
        );

        form.form_state.dropdowns.insert(
            FIELD_ALLERGY_TYPE.to_string(),
            DropdownWidget::new("Allergy Type *", allergy_type_options, theme.clone()),
        );
        form.form_state.dropdowns.insert(
            FIELD_SEVERITY.to_string(),
            DropdownWidget::new("Severity *", severity_options, theme),
        );

        form.validator = build_validator();
        form
    }

    pub fn from_allergy(allergy: Allergy, theme: Theme, allergy_config: &AllergyConfig) -> Self {
        let mut form = Self::new(theme, allergy_config);
        form.form_state.mode = FormMode::Edit(allergy.id);

        form.set_value(AllergyFormField::Allergen, allergy.allergen);
        form.set_value(
            AllergyFormField::AllergyType,
            allergy.allergy_type.to_string(),
        );
        form.set_value(AllergyFormField::Severity, allergy.severity.to_string());

        if let Some(reaction) = allergy.reaction {
            form.set_value(AllergyFormField::Reaction, reaction);
        }

        form.onset_date = allergy.onset_date;

        if let Some(notes) = allergy.notes {
            form.set_value(AllergyFormField::Notes, notes);
        }

        form
    }

    pub fn is_edit_mode(&self) -> bool {
        matches!(self.form_state.mode, FormMode::Edit(_))
    }

    pub fn allergy_id(&self) -> Option<Uuid> {
        match self.form_state.mode {
            FormMode::Edit(id) => Some(id),
            FormMode::Create => None,
        }
    }

    pub fn focused_field(&self) -> AllergyFormField {
        self.form_state.focused_field()
    }

    pub fn get_value(&self, field: AllergyFormField) -> String {
        self.get_value_by_id(field.id())
    }

    pub fn set_value(&mut self, field: AllergyFormField, value: String) {
        self.set_value_by_id(field.id(), value)
    }

    fn get_value_by_id(&self, field_id: &str) -> String {
        if field_id == FIELD_ONSET_DATE {
            return self.onset_date.map(format_date).unwrap_or_default();
        }

        self.form_state.get_value_by_id(field_id)
    }

    fn set_value_by_id(&mut self, field_id: &str, value: String) {
        if field_id == FIELD_ONSET_DATE {
            let parsed = if value.trim().is_empty() {
                None
            } else {
                parse_date(&value)
            };
            self.onset_date = parsed;
            if !value.trim().is_empty() && parsed.is_none() {
                self.set_error_by_id(FIELD_ONSET_DATE, Some("Use dd/mm/yyyy format".to_string()));
                self.is_valid = false;
                return;
            }
        } else {
            self.form_state.set_value_by_id(field_id, value.clone());
        }

        self.sync_domain_enum_fields(field_id, &value);
        self.validate_field_by_id(field_id);
    }

    pub fn to_allergy(&self, patient_id: uuid::Uuid, created_by: uuid::Uuid) -> Allergy {
        Allergy {
            id: uuid::Uuid::new_v4(),
            patient_id,
            allergen: self.get_value(AllergyFormField::Allergen),
            allergy_type: self.allergy_type.unwrap_or(AllergyType::Other),
            severity: self.severity.unwrap_or(Severity::Moderate),
            reaction: Some(self.get_value(AllergyFormField::Reaction)).filter(|s| !s.is_empty()),
            onset_date: self.onset_date,
            notes: Some(self.get_value(AllergyFormField::Notes)).filter(|s| !s.is_empty()),
            is_active: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            created_by,
            updated_by: None,
        }
    }
}

impl FormFieldMeta for AllergyFormField {
    fn label(&self) -> &'static str {
        AllergyFormField::label(self)
    }

    fn is_required(&self) -> bool {
        AllergyFormField::is_required(self)
    }
}

impl FormNavigation for AllergyForm {
    type FormField = AllergyFormField;

    fn get_error(&self, field: Self::FormField) -> Option<&str> {
        self.form_state.errors.get(field.id()).map(|s| s.as_str())
    }

    fn set_error(&mut self, field: Self::FormField, error: Option<String>) {
        match error {
            Some(msg) => {
                self.form_state.errors.insert(field.id().to_string(), msg);
            }
            None => {
                self.form_state.errors.remove(field.id());
            }
        }
    }

    fn validate(&mut self) -> bool {
        self.form_state.errors.clear();

        for field in AllergyFormField::all() {
            self.validate_field_by_id(field.id());
        }

        self.is_valid = self.form_state.errors.is_empty();
        self.is_valid
    }

    fn current_field(&self) -> Self::FormField {
        self.focused_field()
    }

    fn fields(&self) -> Vec<Self::FormField> {
        AllergyFormField::all()
    }

    fn set_current_field(&mut self, field: Self::FormField) {
        self.form_state.focused_field = field;
    }
}

impl HasFocus for AllergyForm {
    fn build(&self, builder: &mut FocusBuilder) {
        builder.leaf_widget(self);
    }

    fn focus(&self) -> FocusFlag {
        self.focus.clone()
    }

    fn area(&self) -> ratatui::layout::Rect {
        ratatui::layout::Rect::default()
    }
}

fn build_validator() -> FormValidator {
    let mut rules: HashMap<String, ValidationRules> = HashMap::new();
    rules.insert(
        FIELD_ALLERGEN.to_string(),
        ValidationRules {
            required: true,
            ..ValidationRules::default()
        },
    );
    rules.insert(
        FIELD_ALLERGY_TYPE.to_string(),
        ValidationRules {
            required: true,
            ..ValidationRules::default()
        },
    );
    rules.insert(
        FIELD_SEVERITY.to_string(),
        ValidationRules {
            required: true,
            ..ValidationRules::default()
        },
    );
    rules.insert(
        FIELD_ONSET_DATE.to_string(),
        ValidationRules {
            date_format: Some("dd/mm/yyyy".to_string()),
            ..ValidationRules::default()
        },
    );

    FormValidator::new(&rules)
}

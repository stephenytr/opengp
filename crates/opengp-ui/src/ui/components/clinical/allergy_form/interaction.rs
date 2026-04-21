use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};

use super::{
    AllergyForm, AllergyFormAction, AllergyFormField, AllergyType, FIELD_ALLERGEN,
    FIELD_ALLERGY_TYPE, FIELD_ONSET_DATE, FIELD_SEVERITY,
};
use crate::ui::input::to_ratatui_key;
use crate::ui::shared::FormAction;
use crate::ui::widgets::{
    format_date, parse_date, DatePickerAction, DropdownAction, DropdownWidget, DynamicForm,
    TextareaState,
};

impl AllergyForm {
    pub(super) fn sync_domain_enum_fields(&mut self, field_id: &str, value: &str) {
        match field_id {
            FIELD_ALLERGY_TYPE => self.allergy_type = value.parse::<AllergyType>().ok(),
            FIELD_SEVERITY => self.severity = value.parse().ok(),
            _ => {}
        }
    }

    pub(super) fn validate_field_by_id(&mut self, field_id: &str) {
        self.form_state.errors.remove(field_id);

        if let Some(field) = AllergyFormField::from_id(field_id) {
            let _ = self.form_state.validate_field(field);
        }

        let value = self.get_value_by_id(field_id);
        let mut errors = self.validator.validate(field_id, &value);

        if field_id == FIELD_ALLERGEN
            && errors.iter().any(|error| error == "This field is required")
        {
            errors = vec!["Allergen is required".to_string()];
        }

        if field_id == FIELD_ALLERGY_TYPE
            && errors.iter().any(|error| error == "This field is required")
        {
            errors = vec!["Allergy type is required".to_string()];
        }

        if field_id == FIELD_SEVERITY
            && errors.iter().any(|error| error == "This field is required")
        {
            errors = vec!["Severity is required".to_string()];
        }

        if field_id == FIELD_ONSET_DATE && !value.trim().is_empty() && parse_date(&value).is_none()
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

    pub(super) fn set_error_by_id(&mut self, field_id: &str, error: Option<String>) {
        match error {
            Some(msg) => {
                self.form_state.errors.insert(field_id.to_string(), msg);
            }
            None => {
                self.form_state.errors.remove(field_id);
            }
        }
    }

    fn focused_textarea_mut(&mut self) -> Option<&mut TextareaState> {
        self.form_state
            .textareas
            .get_mut(self.form_state.focused_field.id())
    }

    pub(super) fn textarea_for(&self, field: AllergyFormField) -> Option<&TextareaState> {
        self.form_state.textareas.get(field.id())
    }

    pub(super) fn dropdown_for(&self, field: AllergyFormField) -> Option<&DropdownWidget> {
        self.form_state.dropdowns.get(field.id())
    }

    fn handle_dropdown_key(&mut self, key: KeyEvent) -> Option<Option<AllergyFormAction>> {
        let field_id = self.form_state.focused_field.id().to_string();
        if !self.form_state.dropdowns.contains_key(&field_id) {
            return None;
        }

        let action = {
            let dropdown = self.form_state.dropdowns.get_mut(&field_id)?;
            dropdown.handle_key(key)
        };

        if let Some(action) = action {
            match key.code {
                KeyCode::Tab | KeyCode::BackTab | KeyCode::Esc => return None,
                _ => match action {
                    DropdownAction::Selected(_) | DropdownAction::Closed => {
                        let selected_value = self
                            .form_state
                            .dropdowns
                            .get(&field_id)
                            .and_then(|dropdown| dropdown.selected_value().map(|v| v.to_string()));
                        if let Some(value) = selected_value {
                            self.set_value_by_id(&field_id, value);
                        }
                    }
                    DropdownAction::Opened | DropdownAction::FocusChanged | DropdownAction::ContextMenu { .. } => {
                        return Some(Some(AllergyFormAction::ValueChanged));
                    }
                },
            }
        } else {
            match key.code {
                KeyCode::Tab | KeyCode::BackTab | KeyCode::Esc => return None,
                _ => return Some(None),
            }
        }

        Some(Some(AllergyFormAction::ValueChanged))
    }

    pub(super) fn error(&self, field: AllergyFormField) -> Option<&String> {
        self.form_state.errors.get(field.id())
    }

    pub fn validate(&mut self) -> bool {
        <Self as DynamicForm>::validate(self)
    }

    pub fn next_field(&mut self) {
        self.form_state.next_field();
    }

    pub fn prev_field(&mut self) {
        self.form_state.prev_field();
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Option<AllergyFormAction> {
        if key.kind != KeyEventKind::Press {
            return None;
        }

        if self.date_picker.is_visible() {
            if let Some(action) = self.date_picker.handle_key(key) {
                match action {
                    DatePickerAction::Selected(date) => {
                        self.set_value_by_id(FIELD_ONSET_DATE, format_date(date));
                        return Some(AllergyFormAction::ValueChanged);
                    }
                    DatePickerAction::Dismissed => {
                        return Some(AllergyFormAction::FocusChanged);
                    }
                }
            }
            return Some(AllergyFormAction::FocusChanged);
        }

        if self.form_state.focused_field == AllergyFormField::OnsetDate
            && matches!(key.code, KeyCode::Enter | KeyCode::Char(' '))
        {
            self.date_picker.open(self.onset_date);
            return Some(AllergyFormAction::FocusChanged);
        }

        if let Some(dropdown_action) = self.handle_dropdown_key(key) {
            return dropdown_action;
        }

        if !self
            .form_state
            .dropdowns
            .contains_key(self.form_state.focused_field.id())
        {
            let ratatui_key = to_ratatui_key(key);
            if let Some(textarea) = self.focused_textarea_mut() {
                let consumed = textarea.handle_key(ratatui_key);
                if consumed {
                    let field_id = self.form_state.focused_field.id().to_string();
                    self.validate_field_by_id(&field_id);
                    return Some(AllergyFormAction::ValueChanged);
                }
            }
        }

        match key.code {
            KeyCode::PageUp => {
                self.form_state.scroll.scroll_up();
                Some(AllergyFormAction::FocusChanged)
            }
            KeyCode::PageDown => {
                self.form_state.scroll.scroll_down();
                Some(AllergyFormAction::FocusChanged)
            }
            KeyCode::Enter => None,
            _ => {
                if let Some(action) = self.form_state.handle_navigation_key(key) {
                    if matches!(action, FormAction::Submit) {
                        let _ = self.validate();
                    }
                    return Some(action);
                }
                None
            }
        }
    }
}

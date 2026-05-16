use std::collections::HashMap;

use crossterm::event::{Event, KeyEvent, KeyModifiers};
use opengp_config::forms::ValidationRules;
use opengp_domain::domain::clinical::FamilyHistory;
use rat_event::ct_event;
use rat_focus::{FocusBuilder, FocusFlag, HasFocus};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::{Block, Borders, Widget};
use uuid::Uuid;

use crate::ui::input::to_ratatui_key;
use crate::ui::theme::Theme;
use crate::ui::widgets::{
    FormFieldMeta, FormNavigation, FormValidator, HeightMode, TextareaState, TextareaWidget,
};

const FIELD_RELATIONSHIP: &str = "relationship";
const FIELD_CONDITION: &str = "condition";
const FIELD_AGE_AT_DIAGNOSIS: &str = "age_at_diagnosis";
const FIELD_NOTES: &str = "notes";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, strum::EnumIter, strum::IntoStaticStr)]
pub enum FamilyHistoryFormField {
    #[strum(to_string = "Relationship *")]
    Relationship,
    #[strum(to_string = "Condition *")]
    Condition,
    #[strum(to_string = "Age at Diagnosis")]
    AgeAtDiagnosis,
    #[strum(to_string = "Notes")]
    Notes,
}

impl FamilyHistoryFormField {
    pub fn all() -> Vec<FamilyHistoryFormField> {
        use strum::IntoEnumIterator;
        FamilyHistoryFormField::iter().collect()
    }

    pub fn label(&self) -> &'static str {
        (*self).into()
    }

    pub fn is_required(&self) -> bool {
        matches!(
            self,
            FamilyHistoryFormField::Relationship | FamilyHistoryFormField::Condition
        )
    }

    pub fn is_textarea(&self) -> bool {
        true
    }

    pub fn is_dropdown(&self) -> bool {
        false
    }

    pub fn id(&self) -> &'static str {
        match self {
            FamilyHistoryFormField::Relationship => FIELD_RELATIONSHIP,
            FamilyHistoryFormField::Condition => FIELD_CONDITION,
            FamilyHistoryFormField::AgeAtDiagnosis => FIELD_AGE_AT_DIAGNOSIS,
            FamilyHistoryFormField::Notes => FIELD_NOTES,
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            FIELD_RELATIONSHIP => Some(FamilyHistoryFormField::Relationship),
            FIELD_CONDITION => Some(FamilyHistoryFormField::Condition),
            FIELD_AGE_AT_DIAGNOSIS => Some(FamilyHistoryFormField::AgeAtDiagnosis),
            FIELD_NOTES => Some(FamilyHistoryFormField::Notes),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum FamilyHistoryFormAction {
    FocusChanged,
    ValueChanged,
    Submit,
    Cancel,
}

pub struct FamilyHistoryForm {
    form_state: crate::ui::widgets::FormState<FamilyHistoryFormField>,
    field_ids: Vec<String>,
    pub is_valid: bool,
    validator: FormValidator,
    pub focus: FocusFlag,
}

impl Clone for FamilyHistoryForm {
    fn clone(&self) -> Self {
        Self {
            form_state: self.form_state.clone(),
            field_ids: self.field_ids.clone(),
            is_valid: self.is_valid,
            validator: build_validator(),
            focus: self.focus.clone(),
        }
    }
}

impl crate::ui::widgets::FormField for FamilyHistoryFormField {
    fn all() -> Vec<Self> {
        FamilyHistoryFormField::all()
    }

    fn label(&self) -> &'static str {
        FamilyHistoryFormField::label(self)
    }

    fn id(&self) -> &'static str {
        FamilyHistoryFormField::id(self)
    }

    fn from_id(id: &str) -> Option<Self> {
        FamilyHistoryFormField::from_id(id)
    }

    fn is_required(&self) -> bool {
        FamilyHistoryFormField::is_required(self)
    }

    fn is_textarea(&self) -> bool {
        FamilyHistoryFormField::is_textarea(self)
    }

    fn is_dropdown(&self) -> bool {
        FamilyHistoryFormField::is_dropdown(self)
    }
}

impl FormFieldMeta for FamilyHistoryFormField {
    fn label(&self) -> &'static str {
        FamilyHistoryFormField::label(self)
    }

    fn is_required(&self) -> bool {
        FamilyHistoryFormField::is_required(self)
    }
}

impl FormNavigation for FamilyHistoryForm {
    type FormField = FamilyHistoryFormField;

    fn get_error(&self, field: Self::FormField) -> Option<&str> {
        self.form_state.errors.get(field.id()).map(|s| s.as_str())
    }

    fn set_error(&mut self, field: Self::FormField, error: Option<String>) {
        self.set_error_by_id(field.id(), error);
    }

    fn validate(&mut self) -> bool {
        self.form_state.errors.clear();

        for field in self.form_state.field_order.clone() {
            self.validate_field_by_id(field.id());
        }

        self.is_valid = self.form_state.errors.is_empty();
        self.is_valid
    }

    fn current_field(&self) -> Self::FormField {
        self.form_state.focused_field
    }

    fn fields(&self) -> Vec<Self::FormField> {
        self.form_state.field_order.clone()
    }

    fn set_current_field(&mut self, field: Self::FormField) {
        self.form_state.focused_field = field;
    }
}

impl HasFocus for FamilyHistoryForm {
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

impl FamilyHistoryForm {
    pub fn new(theme: Theme) -> Self {
        let fields = FamilyHistoryFormField::all();
        let field_ids = fields
            .iter()
            .map(|field| field.id().to_string())
            .collect::<Vec<_>>();

        let mut form_state =
            crate::ui::widgets::FormState::new(theme, FamilyHistoryFormField::Relationship);
        for field in fields {
            form_state
                .textareas
                .insert(field.id().to_string(), make_textarea_state(field, None));
        }

        Self {
            form_state,
            field_ids,
            is_valid: false,
            validator: build_validator(),
            focus: FocusFlag::default(),
        }
    }

    pub fn from_family_history(family_history: FamilyHistory, theme: Theme) -> Self {
        let mut form = Self::new(theme);
        form.form_state.mode = crate::shared::FormMode::Edit(family_history.id);

        form.set_value(
            FamilyHistoryFormField::Relationship,
            family_history.relative_relationship,
        );

        form.set_value(FamilyHistoryFormField::Condition, family_history.condition);

        if let Some(age) = family_history.age_at_diagnosis {
            form.set_value(FamilyHistoryFormField::AgeAtDiagnosis, age.to_string());
        }

        if let Some(notes) = family_history.notes {
            form.set_value(FamilyHistoryFormField::Notes, notes);
        }

        form
    }

    pub fn is_edit_mode(&self) -> bool {
        matches!(self.form_state.mode, crate::shared::FormMode::Edit(_))
    }

    pub fn family_history_id(&self) -> Option<Uuid> {
        match self.form_state.mode {
            crate::shared::FormMode::Edit(id) => Some(id),
            crate::shared::FormMode::Create => None,
        }
    }

    pub fn focused_field(&self) -> FamilyHistoryFormField {
        self.form_state.focused_field
    }

    pub fn get_value(&self, field: FamilyHistoryFormField) -> String {
        self.get_value_by_id(field.id())
    }

    pub fn set_value(&mut self, field: FamilyHistoryFormField, value: String) {
        self.set_value_by_id(field.id(), value);
    }

    fn get_value_by_id(&self, field_id: &str) -> String {
        self.form_state.get_value_by_id(field_id)
    }

    fn set_value_by_id(&mut self, field_id: &str, value: String) {
        self.form_state.set_value_by_id(field_id, value);

        self.validate_field_by_id(field_id);
    }

    fn focused_textarea_mut(&mut self) -> Option<&mut TextareaState> {
        let focused_field_id = self.form_state.focused_field.id();
        self.form_state.textareas.get_mut(focused_field_id)
    }

    fn textarea_for(&self, field_id: &str) -> Option<&TextareaState> {
        self.form_state.textareas.get(field_id)
    }

    fn validate_field_by_id(&mut self, field_id: &str) {
        self.form_state.errors.remove(field_id);

        let value = self.get_value_by_id(field_id);
        let mut errors = self.validator.validate(field_id, &value);

        if field_id == FIELD_AGE_AT_DIAGNOSIS && !value.is_empty() && value.parse::<u8>().is_err() {
            errors = vec!["Age must be a number (0-255)".to_string()];
        }

        let error_msg = errors.into_iter().next();
        self.set_error_by_id(field_id, error_msg.clone());
        if let Some(textarea) = self.form_state.textareas.get_mut(field_id) {
            textarea.set_error(error_msg);
        }
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

    pub fn error(&self, field: FamilyHistoryFormField) -> Option<&String> {
        self.form_state.errors.get(field.id())
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Option<FamilyHistoryFormAction> {
        use crossterm::event::KeyEventKind;

        if key.kind != KeyEventKind::Press {
            return None;
        }

        let event = Event::Key(key);
        if key.modifiers.contains(KeyModifiers::CONTROL)
            && matches!(&event, ct_event!(key press CONTROL-'s'))
        {
            FormNavigation::validate(self);
            return Some(FamilyHistoryFormAction::Submit);
        }

        let event = Event::Key(key);
        match &event {
            ct_event!(keycode press Tab) => {
                if key.modifiers.contains(KeyModifiers::SHIFT) {
                    FormNavigation::prev_field(self);
                } else {
                    FormNavigation::next_field(self);
                }
                return Some(FamilyHistoryFormAction::FocusChanged);
            }
            ct_event!(keycode press BackTab) => {
                FormNavigation::prev_field(self);
                return Some(FamilyHistoryFormAction::FocusChanged);
            }
            ct_event!(keycode press PageUp) => {
                self.form_state.scroll.scroll_up();
                return Some(FamilyHistoryFormAction::FocusChanged);
            }
            ct_event!(keycode press PageDown) => {
                self.form_state.scroll.scroll_down();
                return Some(FamilyHistoryFormAction::FocusChanged);
            }
            ct_event!(keycode press Esc) => return Some(FamilyHistoryFormAction::Cancel),
            _ => {}
        }

        let ratatui_key = to_ratatui_key(key);
        if let Some(textarea) = self.focused_textarea_mut() {
            let consumed = textarea.handle_key(ratatui_key);
            if consumed {
                let field_id = self.form_state.focused_field.id().to_string();
                self.validate_field_by_id(&field_id);
                return Some(FamilyHistoryFormAction::ValueChanged);
            }
        }

        None
    }

    pub fn to_family_history(
        &self,
        patient_id: uuid::Uuid,
        created_by: uuid::Uuid,
    ) -> FamilyHistory {
        FamilyHistory {
            id: self.family_history_id().unwrap_or_else(uuid::Uuid::new_v4),
            patient_id,
            relative_relationship: self.get_value(FamilyHistoryFormField::Relationship),
            condition: self.get_value(FamilyHistoryFormField::Condition),
            age_at_diagnosis: self
                .get_value(FamilyHistoryFormField::AgeAtDiagnosis)
                .parse()
                .ok(),
            notes: Some(self.get_value(FamilyHistoryFormField::Notes)).filter(|s| !s.is_empty()),
            created_at: chrono::Utc::now(),
            created_by,
        }
    }
}

fn make_textarea_state(field: FamilyHistoryFormField, value: Option<String>) -> TextareaState {
    let mut state = match field {
        FamilyHistoryFormField::Relationship => {
            TextareaState::new("Relationship *").with_height_mode(HeightMode::SingleLine)
        }
        FamilyHistoryFormField::Condition => {
            TextareaState::new("Condition *").with_height_mode(HeightMode::SingleLine)
        }
        FamilyHistoryFormField::AgeAtDiagnosis => {
            TextareaState::new("Age at Diagnosis").with_height_mode(HeightMode::SingleLine)
        }
        FamilyHistoryFormField::Notes => {
            TextareaState::new("Notes").with_height_mode(HeightMode::FixedLines(3))
        }
    };

    if let Some(value) = value {
        state = state.with_value(value);
    }

    state
}

fn build_validator() -> FormValidator {
    let mut rules = HashMap::new();
    rules.insert(
        FIELD_RELATIONSHIP.to_string(),
        ValidationRules {
            required: true,
            ..ValidationRules::default()
        },
    );
    rules.insert(
        FIELD_CONDITION.to_string(),
        ValidationRules {
            required: true,
            ..ValidationRules::default()
        },
    );
    rules.insert(
        FIELD_AGE_AT_DIAGNOSIS.to_string(),
        ValidationRules::default(),
    );
    rules.insert(FIELD_NOTES.to_string(), ValidationRules::default());

    FormValidator::new(&rules)
}

impl Widget for FamilyHistoryForm {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.is_empty() {
            return;
        }

        let title = if self.is_edit_mode() {
            " Edit Family History "
        } else {
            " New Family History "
        };

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.form_state.theme.colors.border));

        block.clone().render(area, buf);

        let inner = block.inner(area);
        if inner.is_empty() {
            return;
        }

        let fields = self.field_ids.clone();

        let mut y = inner.y + 1;
        let max_y = inner.y + inner.height - 2;

        for field_id in fields {
            if y > max_y {
                break;
            }

            let Some(textarea) = self.textarea_for(&field_id) else {
                continue;
            };

            let field_height = textarea.height();
            let field_area = Rect::new(inner.x + 1, y, inner.width - 2, field_height);
            let is_focused = field_id == self.form_state.focused_field.id();

            TextareaWidget::new(textarea, self.form_state.theme.clone())
                .focused(is_focused)
                .render(field_area, buf);

            y += field_height;

            if let Some(error_msg) = self.form_state.errors.get(&field_id) {
                if y <= max_y {
                    let error_style = Style::default().fg(self.form_state.theme.colors.error);
                    buf.set_string(inner.x + 2, y, error_msg.as_str(), error_style);
                    y += 1;
                }
            }
        }

        self.form_state
            .scroll
            .render_scrollbar(inner, buf, &self.form_state.theme);

        let help_y = inner.y + inner.height - 1;
        buf.set_string(
            inner.x + 1,
            help_y,
            "Tab: Next | Ctrl+S: Submit | Esc: Cancel",
            Style::default().fg(self.form_state.theme.colors.disabled),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_family_history_form_creation() {
        let theme = Theme::dark();
        let form = FamilyHistoryForm::new(theme);

        assert_eq!(form.focused_field(), FamilyHistoryFormField::Relationship);
        assert!(!form.is_valid);
        assert!(!form.has_errors());
    }

    #[test]
    fn test_family_history_form_validation_required_fields() {
        let theme = Theme::dark();
        let mut form = FamilyHistoryForm::new(theme);

        form.validate();
        assert!(!form.is_valid);
        assert!(form.error(FamilyHistoryFormField::Relationship).is_some());
        assert!(form.error(FamilyHistoryFormField::Condition).is_some());
    }

    #[test]
    fn test_family_history_form_validation_passes_when_required_filled() {
        let theme = Theme::dark();
        let mut form = FamilyHistoryForm::new(theme);

        form.set_value(FamilyHistoryFormField::Relationship, "Father".to_string());
        form.set_value(
            FamilyHistoryFormField::Condition,
            "Diabetes Type 2".to_string(),
        );

        let valid = form.validate();
        assert!(valid);
        assert!(!form.has_errors());
    }

    #[test]
    fn test_family_history_form_field_navigation() {
        let theme = Theme::dark();
        let mut form = FamilyHistoryForm::new(theme);

        assert_eq!(form.focused_field(), FamilyHistoryFormField::Relationship);
        form.next_field();
        assert_eq!(form.focused_field(), FamilyHistoryFormField::Condition);
        form.next_field();
        assert_eq!(form.focused_field(), FamilyHistoryFormField::AgeAtDiagnosis);
        form.prev_field();
        assert_eq!(form.focused_field(), FamilyHistoryFormField::Condition);
    }

    #[test]
    fn test_family_history_form_age_validation() {
        let theme = Theme::dark();
        let mut form = FamilyHistoryForm::new(theme);

        form.set_value(
            FamilyHistoryFormField::AgeAtDiagnosis,
            "not-a-number".to_string(),
        );
        assert!(form.error(FamilyHistoryFormField::AgeAtDiagnosis).is_some());

        form.set_value(FamilyHistoryFormField::AgeAtDiagnosis, "65".to_string());
        assert!(form.error(FamilyHistoryFormField::AgeAtDiagnosis).is_none());
    }

    #[test]
    fn test_family_history_form_all_fields_ordered() {
        let fields = FamilyHistoryFormField::all();
        assert_eq!(fields[0], FamilyHistoryFormField::Relationship);
        assert_eq!(fields[1], FamilyHistoryFormField::Condition);
        assert_eq!(fields[2], FamilyHistoryFormField::AgeAtDiagnosis);
        assert_eq!(fields[3], FamilyHistoryFormField::Notes);
    }

    #[test]
    fn test_family_history_form_is_required() {
        assert!(FamilyHistoryFormField::Relationship.is_required());
        assert!(FamilyHistoryFormField::Condition.is_required());
        assert!(!FamilyHistoryFormField::AgeAtDiagnosis.is_required());
        assert!(!FamilyHistoryFormField::Notes.is_required());
    }

    #[test]
    fn test_family_history_form_get_set_value() {
        let theme = Theme::dark();
        let mut form = FamilyHistoryForm::new(theme);

        form.set_value(FamilyHistoryFormField::Relationship, "Mother".to_string());
        assert_eq!(
            form.get_value(FamilyHistoryFormField::Relationship),
            "Mother"
        );

        form.set_value(FamilyHistoryFormField::Notes, "Some notes".to_string());
        assert_eq!(form.get_value(FamilyHistoryFormField::Notes), "Some notes");
    }

    #[test]
    fn test_family_history_form_tab_navigates_fields() {
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

        let theme = Theme::dark();
        let mut form = FamilyHistoryForm::new(theme);

        assert_eq!(form.focused_field(), FamilyHistoryFormField::Relationship);

        let key = KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE);
        let action = form.handle_key(key);
        assert!(matches!(
            action,
            Some(FamilyHistoryFormAction::FocusChanged)
        ));
        assert_eq!(form.focused_field(), FamilyHistoryFormField::Condition);
    }

    #[test]
    fn test_family_history_form_esc_cancels() {
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

        let theme = Theme::dark();
        let mut form = FamilyHistoryForm::new(theme);

        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        let action = form.handle_key(key);
        assert!(matches!(action, Some(FamilyHistoryFormAction::Cancel)));
    }

    #[test]
    fn test_family_history_form_ctrl_s_submits() {
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

        let theme = Theme::dark();
        let mut form = FamilyHistoryForm::new(theme);

        let key = KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL);
        let action = form.handle_key(key);
        assert!(matches!(action, Some(FamilyHistoryFormAction::Submit)));
    }

    #[test]
    fn test_dynamic_form_string_access() {
        let theme = Theme::dark();
        let mut form = FamilyHistoryForm::new(theme);

        form.set_value_by_id(FIELD_RELATIONSHIP, "Sibling".to_string());

        let by_string = form.get_value_by_id(FIELD_RELATIONSHIP);
        let by_enum = form.get_value(FamilyHistoryFormField::Relationship);
        assert_eq!(by_string, by_enum);
    }

    #[test]
    fn ctrl_tab_does_not_cancel() {
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

        let theme = Theme::dark();
        let mut form = FamilyHistoryForm::new(theme);

        let key = KeyEvent::new(KeyCode::Tab, KeyModifiers::CONTROL);
        let action = form.handle_key(key);
        assert!(!matches!(action, Some(FamilyHistoryFormAction::Cancel)));
    }
}

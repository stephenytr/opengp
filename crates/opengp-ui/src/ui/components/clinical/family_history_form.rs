//! Family History Form Component
//!
//! Form for creating new patient family history entries.

use std::collections::HashMap;

use crossterm::event::{KeyEvent, KeyModifiers};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::{Block, Borders, Widget};

use crate::ui::input::to_ratatui_key;
use crate::ui::theme::Theme;
use crate::ui::widgets::{
    FormNavigation, HeightMode, ScrollableFormState, TextareaState, TextareaWidget,
};
use opengp_domain::domain::clinical::FamilyHistory;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FamilyHistoryFormField {
    Relationship,
    Condition,
    AgeAtDiagnosis,
    Notes,
}

impl FamilyHistoryFormField {
    pub fn all() -> Vec<FamilyHistoryFormField> {
        vec![
            FamilyHistoryFormField::Relationship,
            FamilyHistoryFormField::Condition,
            FamilyHistoryFormField::AgeAtDiagnosis,
            FamilyHistoryFormField::Notes,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            FamilyHistoryFormField::Relationship => "Relationship *",
            FamilyHistoryFormField::Condition => "Condition *",
            FamilyHistoryFormField::AgeAtDiagnosis => "Age at Diagnosis",
            FamilyHistoryFormField::Notes => "Notes",
        }
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
}

#[derive(Debug, Clone)]
pub enum FamilyHistoryFormAction {
    FocusChanged,
    ValueChanged,
    Submit,
    Cancel,
}

pub struct FamilyHistoryForm {
    pub relationship: TextareaState,
    pub condition: TextareaState,
    pub age_at_diagnosis: TextareaState,
    pub notes: TextareaState,
    pub focused_field: FamilyHistoryFormField,
    pub is_valid: bool,
    errors: HashMap<FamilyHistoryFormField, String>,
    theme: Theme,
    scroll: ScrollableFormState,
}

impl Clone for FamilyHistoryForm {
    fn clone(&self) -> Self {
        Self {
            relationship: self.relationship.clone(),
            condition: self.condition.clone(),
            age_at_diagnosis: self.age_at_diagnosis.clone(),
            notes: self.notes.clone(),
            focused_field: self.focused_field,
            is_valid: self.is_valid,
            errors: self.errors.clone(),
            theme: self.theme.clone(),
            scroll: self.scroll.clone(),
        }
    }
}

impl FormNavigation for FamilyHistoryForm {
    type FormField = FamilyHistoryFormField;

    fn validate(&mut self) -> bool {
        self.errors.clear();

        for field in FamilyHistoryFormField::all() {
            self.validate_field(&field);
        }

        self.is_valid = self.errors.is_empty();
        self.is_valid
    }

    fn current_field(&self) -> Self::FormField {
        self.focused_field
    }

    fn fields(&self) -> &[Self::FormField] {
        &[]
    }

    fn set_current_field(&mut self, field: Self::FormField) {
        self.focused_field = field;
    }

    fn next_field(&mut self) {
        let fields = FamilyHistoryFormField::all();
        if let Some(current_idx) = fields.iter().position(|f| *f == self.focused_field) {
            let next_idx = (current_idx + 1) % fields.len();
            self.focused_field = fields[next_idx];
        }
    }

    fn prev_field(&mut self) {
        let fields = FamilyHistoryFormField::all();
        if let Some(current_idx) = fields.iter().position(|f| *f == self.focused_field) {
            let prev_idx = if current_idx == 0 {
                fields.len() - 1
            } else {
                current_idx - 1
            };
            self.focused_field = fields[prev_idx];
        }
    }
}

impl FamilyHistoryForm {
    pub fn new(theme: Theme) -> Self {
        Self {
            relationship: TextareaState::new("Relationship *")
                .with_height_mode(HeightMode::SingleLine),
            condition: TextareaState::new("Condition *").with_height_mode(HeightMode::SingleLine),
            age_at_diagnosis: TextareaState::new("Age at Diagnosis")
                .with_height_mode(HeightMode::SingleLine),
            notes: TextareaState::new("Notes").with_height_mode(HeightMode::FixedLines(3)),
            focused_field: FamilyHistoryFormField::Relationship,
            is_valid: false,
            errors: HashMap::new(),
            theme,
            scroll: ScrollableFormState::new(),
        }
    }

    pub fn focused_field(&self) -> FamilyHistoryFormField {
        self.focused_field
    }

    pub fn get_value(&self, field: FamilyHistoryFormField) -> String {
        match field {
            FamilyHistoryFormField::Relationship => self.relationship.value(),
            FamilyHistoryFormField::Condition => self.condition.value(),
            FamilyHistoryFormField::AgeAtDiagnosis => self.age_at_diagnosis.value(),
            FamilyHistoryFormField::Notes => self.notes.value(),
        }
    }

    pub fn set_value(&mut self, field: FamilyHistoryFormField, value: String) {
        match field {
            FamilyHistoryFormField::Relationship => {
                self.relationship = TextareaState::new("Relationship *")
                    .with_height_mode(HeightMode::SingleLine)
                    .with_value(value);
            }
            FamilyHistoryFormField::Condition => {
                self.condition = TextareaState::new("Condition *")
                    .with_height_mode(HeightMode::SingleLine)
                    .with_value(value);
            }
            FamilyHistoryFormField::AgeAtDiagnosis => {
                self.age_at_diagnosis = TextareaState::new("Age at Diagnosis")
                    .with_height_mode(HeightMode::SingleLine)
                    .with_value(value);
            }
            FamilyHistoryFormField::Notes => {
                self.notes = TextareaState::new("Notes")
                    .with_height_mode(HeightMode::FixedLines(3))
                    .with_value(value);
            }
        }
        self.validate_field(&field);
    }

    fn focused_textarea_mut(&mut self) -> &mut TextareaState {
        match self.focused_field {
            FamilyHistoryFormField::Relationship => &mut self.relationship,
            FamilyHistoryFormField::Condition => &mut self.condition,
            FamilyHistoryFormField::AgeAtDiagnosis => &mut self.age_at_diagnosis,
            FamilyHistoryFormField::Notes => &mut self.notes,
        }
    }

    fn validate_field(&mut self, field: &FamilyHistoryFormField) {
        self.errors.remove(field);

        let value = self.get_value(*field);

        match field {
            FamilyHistoryFormField::Relationship => {
                if value.trim().is_empty() {
                    self.errors
                        .insert(*field, "Relationship is required".to_string());
                }
            }
            FamilyHistoryFormField::Condition => {
                if value.trim().is_empty() {
                    self.errors
                        .insert(*field, "Condition is required".to_string());
                }
            }
            FamilyHistoryFormField::AgeAtDiagnosis => {
                if !value.is_empty() && value.parse::<u8>().is_err() {
                    self.errors
                        .insert(*field, "Age must be a number (0-255)".to_string());
                }
            }
            _ => {}
        }
    }

    pub fn validate(&mut self) -> bool {
        self.errors.clear();

        for field in FamilyHistoryFormField::all() {
            self.validate_field(&field);
        }

        self.is_valid = self.errors.is_empty();
        self.is_valid
    }

    pub fn error(&self, field: FamilyHistoryFormField) -> Option<&String> {
        self.errors.get(&field)
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Option<FamilyHistoryFormAction> {
        use crossterm::event::{KeyCode, KeyEventKind};

        if key.kind != KeyEventKind::Press {
            return None;
        }

        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Enter {
            self.validate();
            return Some(FamilyHistoryFormAction::Submit);
        }

        // Ctrl+Tab exits the form from any field.
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Tab {
            return Some(FamilyHistoryFormAction::Cancel);
        }

        match key.code {
            KeyCode::Tab => {
                if key.modifiers.contains(KeyModifiers::SHIFT) {
                    self.prev_field();
                } else {
                    self.next_field();
                }
                return Some(FamilyHistoryFormAction::FocusChanged);
            }
            KeyCode::BackTab => {
                self.prev_field();
                return Some(FamilyHistoryFormAction::FocusChanged);
            }
            KeyCode::PageUp => {
                self.scroll.scroll_up();
                return Some(FamilyHistoryFormAction::FocusChanged);
            }
            KeyCode::PageDown => {
                self.scroll.scroll_down();
                return Some(FamilyHistoryFormAction::FocusChanged);
            }
            KeyCode::Esc => return Some(FamilyHistoryFormAction::Cancel),
            _ => {}
        }

        let ratatui_key = to_ratatui_key(key);
        let consumed = self.focused_textarea_mut().handle_key(ratatui_key);
        if consumed {
            let field = self.focused_field;
            self.validate_field(&field);
            return Some(FamilyHistoryFormAction::ValueChanged);
        }

        None
    }

    pub fn to_family_history(
        &self,
        patient_id: uuid::Uuid,
        created_by: uuid::Uuid,
    ) -> FamilyHistory {
        FamilyHistory {
            id: uuid::Uuid::new_v4(),
            patient_id,
            relative_relationship: self.relationship.value(),
            condition: self.condition.value(),
            age_at_diagnosis: self.age_at_diagnosis.value().parse().ok(),
            notes: Some(self.notes.value()).filter(|s| !s.is_empty()),
            created_at: chrono::Utc::now(),
            created_by,
        }
    }
}

impl Widget for FamilyHistoryForm {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.is_empty() {
            return;
        }

        let block = Block::default()
            .title(" New Family History ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.colors.border));

        block.clone().render(area, buf);

        let inner = block.inner(area);
        if inner.is_empty() {
            return;
        }

        let fields = FamilyHistoryFormField::all();

        let mut y = inner.y + 1;
        let max_y = inner.y + inner.height - 2;

        for field in fields {
            if y > max_y {
                break;
            }

            let textarea = match field {
                FamilyHistoryFormField::Relationship => &self.relationship,
                FamilyHistoryFormField::Condition => &self.condition,
                FamilyHistoryFormField::AgeAtDiagnosis => &self.age_at_diagnosis,
                FamilyHistoryFormField::Notes => &self.notes,
            };

            let field_height = textarea.height();
            let field_area = Rect::new(inner.x + 1, y, inner.width - 2, field_height);
            let is_focused = field == self.focused_field;

            TextareaWidget::new(textarea, self.theme.clone())
                .focused(is_focused)
                .render(field_area, buf);

            y += field_height;

            if let Some(error_msg) = self.error(field) {
                if y <= max_y {
                    let error_style = Style::default().fg(self.theme.colors.error);
                    buf.set_string(inner.x + 2, y, error_msg.as_str(), error_style);
                    y += 1;
                }
            }
        }

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
    fn test_family_history_form_ctrl_enter_submits() {
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

        let theme = Theme::dark();
        let mut form = FamilyHistoryForm::new(theme);

        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::CONTROL);
        let action = form.handle_key(key);
        assert!(matches!(action, Some(FamilyHistoryFormAction::Submit)));
    }
}

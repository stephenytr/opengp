//! Family History Form Component
//!
//! Form for creating new patient family history entries.

use std::collections::HashMap;

use crossterm::event::{KeyEvent, KeyModifiers};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Borders, Widget};

use crate::domain::clinical::FamilyHistory;
use crate::ui::layout::LABEL_WIDTH;
use crate::ui::theme::Theme;

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
}

#[derive(Debug, Clone)]
pub enum FamilyHistoryFormAction {
    FocusChanged,
    ValueChanged,
    Submit,
    Cancel,
}

pub struct FamilyHistoryForm {
    pub relationship: String,
    pub condition: String,
    pub age_at_diagnosis: String,
    pub notes: String,
    pub focused_field: FamilyHistoryFormField,
    pub is_valid: bool,
    errors: HashMap<FamilyHistoryFormField, String>,
    theme: Theme,
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
        }
    }
}

impl FamilyHistoryForm {
    pub fn new(theme: Theme) -> Self {
        Self {
            relationship: String::new(),
            condition: String::new(),
            age_at_diagnosis: String::new(),
            notes: String::new(),
            focused_field: FamilyHistoryFormField::Relationship,
            is_valid: false,
            errors: HashMap::new(),
            theme,
        }
    }

    pub fn focused_field(&self) -> FamilyHistoryFormField {
        self.focused_field
    }

    pub fn next_field(&mut self) {
        let fields = FamilyHistoryFormField::all();
        if let Some(current_idx) = fields.iter().position(|f| *f == self.focused_field) {
            let next_idx = (current_idx + 1) % fields.len();
            self.focused_field = fields[next_idx];
        }
    }

    pub fn prev_field(&mut self) {
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

    pub fn get_value(&self, field: FamilyHistoryFormField) -> String {
        match field {
            FamilyHistoryFormField::Relationship => self.relationship.clone(),
            FamilyHistoryFormField::Condition => self.condition.clone(),
            FamilyHistoryFormField::AgeAtDiagnosis => self.age_at_diagnosis.clone(),
            FamilyHistoryFormField::Notes => self.notes.clone(),
        }
    }

    pub fn set_value(&mut self, field: FamilyHistoryFormField, value: String) {
        match field {
            FamilyHistoryFormField::Relationship => self.relationship = value,
            FamilyHistoryFormField::Condition => self.condition = value,
            FamilyHistoryFormField::AgeAtDiagnosis => self.age_at_diagnosis = value,
            FamilyHistoryFormField::Notes => self.notes = value,
        }
        self.validate_field(&field);
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
        use crossterm::event::KeyCode;

        match key.code {
            KeyCode::Tab => {
                if key.modifiers.contains(KeyModifiers::SHIFT) {
                    self.prev_field();
                } else {
                    self.next_field();
                }
                Some(FamilyHistoryFormAction::FocusChanged)
            }
            KeyCode::Up => {
                self.prev_field();
                Some(FamilyHistoryFormAction::FocusChanged)
            }
            KeyCode::Down => {
                self.next_field();
                Some(FamilyHistoryFormAction::FocusChanged)
            }
            KeyCode::Enter => {
                self.validate();
                Some(FamilyHistoryFormAction::Submit)
            }
            KeyCode::Esc => Some(FamilyHistoryFormAction::Cancel),
            KeyCode::Char(c) => {
                let mut value = self.get_value(self.focused_field);
                value.push(c);
                self.set_value(self.focused_field, value);
                Some(FamilyHistoryFormAction::ValueChanged)
            }
            KeyCode::Backspace => {
                let mut value = self.get_value(self.focused_field);
                value.pop();
                self.set_value(self.focused_field, value);
                Some(FamilyHistoryFormAction::ValueChanged)
            }
            _ => None,
        }
    }

    pub fn to_family_history(
        &self,
        patient_id: uuid::Uuid,
        created_by: uuid::Uuid,
    ) -> FamilyHistory {
        FamilyHistory {
            id: uuid::Uuid::new_v4(),
            patient_id,
            relative_relationship: self.relationship.clone(),
            condition: self.condition.clone(),
            age_at_diagnosis: self.age_at_diagnosis.parse().ok(),
            notes: Some(self.notes.clone()).filter(|s| !s.is_empty()),
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

        let label_width = LABEL_WIDTH;
        let field_start = inner.x + label_width + 2;

        let fields = FamilyHistoryFormField::all();

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
}

//! Consultation Form Component
//!
//! Form for creating and editing patient consultations (SOAP notes).

use std::collections::HashMap;

use crossterm::event::{KeyEvent, KeyModifiers};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Borders, Widget};

use crate::domain::clinical::Consultation;
use crate::ui::layout::LABEL_WIDTH;
use crate::ui::theme::Theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConsultationFormField {
    Reason,
    Subjective,
    Objective,
    Assessment,
    Plan,
}

impl ConsultationFormField {
    pub fn all() -> Vec<ConsultationFormField> {
        vec![
            ConsultationFormField::Reason,
            ConsultationFormField::Subjective,
            ConsultationFormField::Objective,
            ConsultationFormField::Assessment,
            ConsultationFormField::Plan,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            ConsultationFormField::Reason => "Reason",
            ConsultationFormField::Subjective => "Subjective",
            ConsultationFormField::Objective => "Objective",
            ConsultationFormField::Assessment => "Assessment",
            ConsultationFormField::Plan => "Plan",
        }
    }

    pub fn is_required(&self) -> bool {
        false
    }
}

#[derive(Debug, Clone)]
pub enum ConsultationFormAction {
    FocusChanged,
    ValueChanged,
    Submit,
    Cancel,
}

pub struct ConsultationForm {
    pub reason: String,
    pub subjective: String,
    pub objective: String,
    pub assessment: String,
    pub plan: String,
    pub focused_field: ConsultationFormField,
    pub is_valid: bool,
    pub is_edit_mode: bool,
    pub consultation_id: Option<uuid::Uuid>,
    errors: HashMap<ConsultationFormField, String>,
    theme: Theme,
}

impl Clone for ConsultationForm {
    fn clone(&self) -> Self {
        Self {
            reason: self.reason.clone(),
            subjective: self.subjective.clone(),
            objective: self.objective.clone(),
            assessment: self.assessment.clone(),
            plan: self.plan.clone(),
            focused_field: self.focused_field,
            is_valid: self.is_valid,
            is_edit_mode: self.is_edit_mode,
            consultation_id: self.consultation_id,
            errors: self.errors.clone(),
            theme: self.theme.clone(),
        }
    }
}

impl ConsultationForm {
    pub fn new(theme: Theme) -> Self {
        Self {
            reason: String::new(),
            subjective: String::new(),
            objective: String::new(),
            assessment: String::new(),
            plan: String::new(),
            focused_field: ConsultationFormField::Reason,
            is_valid: true,
            is_edit_mode: false,
            consultation_id: None,
            errors: HashMap::new(),
            theme: theme.clone(),
        }
    }

    pub fn from_consultation(theme: Theme, consultation: &Consultation) -> Self {
        Self {
            reason: consultation.reason.clone().unwrap_or_default(),
            subjective: consultation
                .soap_notes
                .subjective
                .clone()
                .unwrap_or_default(),
            objective: consultation
                .soap_notes
                .objective
                .clone()
                .unwrap_or_default(),
            assessment: consultation
                .soap_notes
                .assessment
                .clone()
                .unwrap_or_default(),
            plan: consultation.soap_notes.plan.clone().unwrap_or_default(),
            focused_field: ConsultationFormField::Reason,
            is_valid: true,
            is_edit_mode: true,
            consultation_id: Some(consultation.id),
            errors: HashMap::new(),
            theme,
        }
    }

    pub fn focused_field(&self) -> ConsultationFormField {
        self.focused_field
    }

    pub fn next_field(&mut self) {
        let fields = ConsultationFormField::all();
        if let Some(current_idx) = fields.iter().position(|f| *f == self.focused_field) {
            let next_idx = (current_idx + 1) % fields.len();
            self.focused_field = fields[next_idx];
        }
    }

    pub fn prev_field(&mut self) {
        let fields = ConsultationFormField::all();
        if let Some(current_idx) = fields.iter().position(|f| *f == self.focused_field) {
            let prev_idx = if current_idx == 0 {
                fields.len() - 1
            } else {
                current_idx - 1
            };
            self.focused_field = fields[prev_idx];
        }
    }

    pub fn get_value(&self, field: ConsultationFormField) -> String {
        match field {
            ConsultationFormField::Reason => self.reason.clone(),
            ConsultationFormField::Subjective => self.subjective.clone(),
            ConsultationFormField::Objective => self.objective.clone(),
            ConsultationFormField::Assessment => self.assessment.clone(),
            ConsultationFormField::Plan => self.plan.clone(),
        }
    }

    pub fn set_value(&mut self, field: ConsultationFormField, value: String) {
        match field {
            ConsultationFormField::Reason => self.reason = value,
            ConsultationFormField::Subjective => self.subjective = value,
            ConsultationFormField::Objective => self.objective = value,
            ConsultationFormField::Assessment => self.assessment = value,
            ConsultationFormField::Plan => self.plan = value,
        }
        self.validate_field(&field);
    }

    fn validate_field(&mut self, field: &ConsultationFormField) {
        self.errors.remove(field);
    }

    pub fn validate(&mut self) -> bool {
        self.errors.clear();
        self.is_valid = self.errors.is_empty();
        self.is_valid
    }

    pub fn error(&self, field: ConsultationFormField) -> Option<&String> {
        self.errors.get(&field)
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Option<ConsultationFormAction> {
        use crossterm::event::KeyCode;

        match key.code {
            KeyCode::Tab => {
                if key.modifiers.contains(KeyModifiers::SHIFT) {
                    self.prev_field();
                } else {
                    self.next_field();
                }
                Some(ConsultationFormAction::FocusChanged)
            }
            KeyCode::Up => {
                self.prev_field();
                Some(ConsultationFormAction::FocusChanged)
            }
            KeyCode::Down => {
                self.next_field();
                Some(ConsultationFormAction::FocusChanged)
            }
            KeyCode::Enter => {
                self.validate();
                Some(ConsultationFormAction::Submit)
            }
            KeyCode::Esc => Some(ConsultationFormAction::Cancel),
            KeyCode::Char(c) => {
                let mut value = self.get_value(self.focused_field);
                value.push(c);
                self.set_value(self.focused_field, value);
                Some(ConsultationFormAction::ValueChanged)
            }
            KeyCode::Backspace => {
                let mut value = self.get_value(self.focused_field);
                value.pop();
                self.set_value(self.focused_field, value);
                Some(ConsultationFormAction::ValueChanged)
            }
            _ => None,
        }
    }

    pub fn to_consultation(
        &self,
        patient_id: uuid::Uuid,
        practitioner_id: uuid::Uuid,
        created_by: uuid::Uuid,
    ) -> Consultation {
        Consultation {
            id: self.consultation_id.unwrap_or_else(uuid::Uuid::new_v4),
            patient_id,
            practitioner_id,
            appointment_id: None,
            consultation_date: chrono::Utc::now(),
            reason: Some(self.reason.clone()).filter(|s| !s.is_empty()),
            soap_notes: crate::domain::clinical::SOAPNotes {
                subjective: Some(self.subjective.clone()).filter(|s| !s.is_empty()),
                objective: Some(self.objective.clone()).filter(|s| !s.is_empty()),
                assessment: Some(self.assessment.clone()).filter(|s| !s.is_empty()),
                plan: Some(self.plan.clone()).filter(|s| !s.is_empty()),
            },
            is_signed: false,
            signed_at: None,
            signed_by: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            created_by,
            updated_by: None,
        }
    }
}

impl Widget for ConsultationForm {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.is_empty() {
            return;
        }

        let title = if self.is_edit_mode {
            " Edit Consultation "
        } else {
            " New Consultation "
        };

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.colors.border));

        block.clone().render(area, buf);

        let inner = block.inner(area);
        if inner.is_empty() {
            return;
        }

        let label_width = LABEL_WIDTH;
        let field_start = inner.x + label_width + 2;

        let fields = ConsultationFormField::all();

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

            let max_value_width = inner.width.saturating_sub(label_width + 4);

            let value = self.get_value(field);
            let value_style = if has_error {
                Style::default().fg(self.theme.colors.error)
            } else {
                Style::default().fg(self.theme.colors.foreground)
            };

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
    fn test_consultation_form_creation() {
        let theme = Theme::dark();
        let form = ConsultationForm::new(theme);

        assert_eq!(form.focused_field(), ConsultationFormField::Reason);
        assert!(form.is_valid);
        assert!(!form.has_errors());
    }

    #[test]
    fn test_consultation_form_validation() {
        let theme = Theme::dark();
        let mut form = ConsultationForm::new(theme);

        let valid = form.validate();
        assert!(valid);
        assert!(!form.has_errors());
    }

    #[test]
    fn test_consultation_form_field_navigation() {
        let theme = Theme::dark();
        let mut form = ConsultationForm::new(theme);

        assert_eq!(form.focused_field(), ConsultationFormField::Reason);
        form.next_field();
        assert_eq!(form.focused_field(), ConsultationFormField::Subjective);
        form.next_field();
        assert_eq!(form.focused_field(), ConsultationFormField::Objective);
        form.prev_field();
        assert_eq!(form.focused_field(), ConsultationFormField::Subjective);
    }

    #[test]
    fn test_consultation_form_all_fields_ordered() {
        let fields = ConsultationFormField::all();
        assert_eq!(fields[0], ConsultationFormField::Reason);
        assert_eq!(fields[1], ConsultationFormField::Subjective);
        assert_eq!(fields[2], ConsultationFormField::Objective);
        assert_eq!(fields[3], ConsultationFormField::Assessment);
        assert_eq!(fields[4], ConsultationFormField::Plan);
    }
}

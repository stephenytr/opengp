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
use crate::ui::widgets::{HeightMode, TextareaState, TextareaWidget};

type RatatuiKeyEvent = ratatui::crossterm::event::KeyEvent;
type RatatuiKeyCode = ratatui::crossterm::event::KeyCode;
type RatatuiKeyModifiers = ratatui::crossterm::event::KeyModifiers;
type RatatuiKeyEventKind = ratatui::crossterm::event::KeyEventKind;
type RatatuiKeyEventState = ratatui::crossterm::event::KeyEventState;

fn to_ratatui_key(key: KeyEvent) -> RatatuiKeyEvent {
    use crossterm::event::KeyCode;

    let code = match key.code {
        KeyCode::Backspace => RatatuiKeyCode::Backspace,
        KeyCode::Enter => RatatuiKeyCode::Enter,
        KeyCode::Left => RatatuiKeyCode::Left,
        KeyCode::Right => RatatuiKeyCode::Right,
        KeyCode::Up => RatatuiKeyCode::Up,
        KeyCode::Down => RatatuiKeyCode::Down,
        KeyCode::Home => RatatuiKeyCode::Home,
        KeyCode::End => RatatuiKeyCode::End,
        KeyCode::PageUp => RatatuiKeyCode::PageUp,
        KeyCode::PageDown => RatatuiKeyCode::PageDown,
        KeyCode::Tab => RatatuiKeyCode::Tab,
        KeyCode::BackTab => RatatuiKeyCode::BackTab,
        KeyCode::Delete => RatatuiKeyCode::Delete,
        KeyCode::Insert => RatatuiKeyCode::Insert,
        KeyCode::F(n) => RatatuiKeyCode::F(n),
        KeyCode::Char(c) => RatatuiKeyCode::Char(c),
        KeyCode::Null => RatatuiKeyCode::Null,
        KeyCode::Esc => RatatuiKeyCode::Esc,
        KeyCode::CapsLock => RatatuiKeyCode::CapsLock,
        KeyCode::ScrollLock => RatatuiKeyCode::ScrollLock,
        KeyCode::NumLock => RatatuiKeyCode::NumLock,
        KeyCode::PrintScreen => RatatuiKeyCode::PrintScreen,
        KeyCode::Pause => RatatuiKeyCode::Pause,
        KeyCode::Menu => RatatuiKeyCode::Menu,
        KeyCode::KeypadBegin => RatatuiKeyCode::KeypadBegin,
        _ => RatatuiKeyCode::Null,
    };

    let modifiers = RatatuiKeyModifiers::from_bits_truncate(key.modifiers.bits());

    let kind = match key.kind {
        crossterm::event::KeyEventKind::Press => RatatuiKeyEventKind::Press,
        crossterm::event::KeyEventKind::Repeat => RatatuiKeyEventKind::Repeat,
        crossterm::event::KeyEventKind::Release => RatatuiKeyEventKind::Release,
    };

    let state = RatatuiKeyEventState::from_bits_truncate(key.state.bits());

    RatatuiKeyEvent {
        code,
        modifiers,
        kind,
        state,
    }
}

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

    /// Returns true if this field uses TextareaWidget (multi-line SOAP fields).
    pub fn is_textarea(&self) -> bool {
        matches!(
            self,
            ConsultationFormField::Subjective
                | ConsultationFormField::Objective
                | ConsultationFormField::Assessment
                | ConsultationFormField::Plan
        )
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
    pub subjective: TextareaState,
    pub objective: TextareaState,
    pub assessment: TextareaState,
    pub plan: TextareaState,
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

fn soap_textarea(label: &'static str) -> TextareaState {
    TextareaState::new(label).with_height_mode(HeightMode::AutoGrow { min: 3, max: 8 })
}

impl ConsultationForm {
    pub fn new(theme: Theme) -> Self {
        Self {
            reason: String::new(),
            subjective: soap_textarea("Subjective"),
            objective: soap_textarea("Objective"),
            assessment: soap_textarea("Assessment"),
            plan: soap_textarea("Plan"),
            focused_field: ConsultationFormField::Reason,
            is_valid: true,
            is_edit_mode: false,
            consultation_id: None,
            errors: HashMap::new(),
            theme: theme.clone(),
        }
    }

    pub fn from_consultation(theme: Theme, consultation: &Consultation) -> Self {
        let subjective = soap_textarea("Subjective").with_value(
            consultation
                .soap_notes
                .subjective
                .clone()
                .unwrap_or_default(),
        );
        let objective = soap_textarea("Objective").with_value(
            consultation
                .soap_notes
                .objective
                .clone()
                .unwrap_or_default(),
        );
        let assessment = soap_textarea("Assessment").with_value(
            consultation
                .soap_notes
                .assessment
                .clone()
                .unwrap_or_default(),
        );
        let plan = soap_textarea("Plan")
            .with_value(consultation.soap_notes.plan.clone().unwrap_or_default());

        Self {
            reason: consultation.reason.clone().unwrap_or_default(),
            subjective,
            objective,
            assessment,
            plan,
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
            ConsultationFormField::Subjective => self.subjective.value(),
            ConsultationFormField::Objective => self.objective.value(),
            ConsultationFormField::Assessment => self.assessment.value(),
            ConsultationFormField::Plan => self.plan.value(),
        }
    }

    pub fn set_value(&mut self, field: ConsultationFormField, value: String) {
        match field {
            ConsultationFormField::Reason => self.reason = value,
            ConsultationFormField::Subjective => {
                self.subjective = soap_textarea("Subjective").with_value(value);
            }
            ConsultationFormField::Objective => {
                self.objective = soap_textarea("Objective").with_value(value);
            }
            ConsultationFormField::Assessment => {
                self.assessment = soap_textarea("Assessment").with_value(value);
            }
            ConsultationFormField::Plan => {
                self.plan = soap_textarea("Plan").with_value(value);
            }
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
        use crossterm::event::{KeyCode, KeyEventKind};

        if key.kind != KeyEventKind::Press {
            return None;
        }

        // Ctrl+Enter submits the form from any field.
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Enter {
            self.validate();
            return Some(ConsultationFormAction::Submit);
        }

        // For textarea SOAP fields, delegate to TextareaState.
        if self.focused_field.is_textarea() {
            let ratatui_key = to_ratatui_key(key);
            let consumed = match self.focused_field {
                ConsultationFormField::Subjective => self.subjective.handle_key(ratatui_key),
                ConsultationFormField::Objective => self.objective.handle_key(ratatui_key),
                ConsultationFormField::Assessment => self.assessment.handle_key(ratatui_key),
                ConsultationFormField::Plan => self.plan.handle_key(ratatui_key),
                _ => false,
            };
            if consumed {
                return Some(ConsultationFormAction::ValueChanged);
            }
        }

        match key.code {
            KeyCode::Tab => {
                if key.modifiers.contains(KeyModifiers::SHIFT) {
                    self.prev_field();
                } else {
                    self.next_field();
                }
                Some(ConsultationFormAction::FocusChanged)
            }
            KeyCode::BackTab => {
                self.prev_field();
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
                self.next_field();
                Some(ConsultationFormAction::FocusChanged)
            }
            KeyCode::Esc => Some(ConsultationFormAction::Cancel),
            KeyCode::Char(c) => {
                self.reason.push(c);
                Some(ConsultationFormAction::ValueChanged)
            }
            KeyCode::Backspace => {
                self.reason.pop();
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
                subjective: Some(self.subjective.value()).filter(|s| !s.is_empty()),
                objective: Some(self.objective.value()).filter(|s| !s.is_empty()),
                assessment: Some(self.assessment.value()).filter(|s| !s.is_empty()),
                plan: Some(self.plan.value()).filter(|s| !s.is_empty()),
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

            // Textarea fields (SOAP notes) use TextareaWidget.
            if field.is_textarea() {
                let textarea = match field {
                    ConsultationFormField::Subjective => &self.subjective,
                    ConsultationFormField::Objective => &self.objective,
                    ConsultationFormField::Assessment => &self.assessment,
                    ConsultationFormField::Plan => &self.plan,
                    _ => unreachable!(),
                };
                let field_height = textarea.height();
                let field_area = Rect::new(inner.x + 1, y, inner.width - 2, field_height);
                TextareaWidget::new(textarea, self.theme.clone())
                    .focused(is_focused)
                    .render(field_area, buf);
                y += field_height;
                continue;
            }

            // Single-line Reason field.
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
            "Tab: Next | Ctrl+Enter: Submit | Esc: Cancel",
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

    #[test]
    fn test_soap_fields_are_textarea() {
        assert!(!ConsultationFormField::Reason.is_textarea());
        assert!(ConsultationFormField::Subjective.is_textarea());
        assert!(ConsultationFormField::Objective.is_textarea());
        assert!(ConsultationFormField::Assessment.is_textarea());
        assert!(ConsultationFormField::Plan.is_textarea());
    }

    #[test]
    fn test_soap_field_get_set_value() {
        let theme = Theme::dark();
        let mut form = ConsultationForm::new(theme);

        form.set_value(
            ConsultationFormField::Subjective,
            "Patient reports headache".to_string(),
        );
        assert_eq!(
            form.get_value(ConsultationFormField::Subjective),
            "Patient reports headache"
        );
    }
}

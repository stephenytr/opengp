//! Consultation Form Component
//!
//! Form for creating and editing patient consultations (SOAP notes).

use std::collections::HashMap;

use crossterm::event::{KeyEvent, KeyModifiers};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Borders, Widget};

use crate::ui::input::to_ratatui_key;
use crate::ui::layout::LABEL_WIDTH;
use crate::ui::theme::Theme;
use crate::ui::widgets::{
    FormFieldMeta, FormNavigation, HeightMode, ScrollableFormState, TextareaState, TextareaWidget,
};
use opengp_domain::domain::clinical::Consultation;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConsultationFormField {
    Reason,
    ClinicalNotes,
}

impl ConsultationFormField {
    pub fn all() -> Vec<ConsultationFormField> {
        vec![
            ConsultationFormField::Reason,
            ConsultationFormField::ClinicalNotes,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            ConsultationFormField::Reason => "Reason",
            ConsultationFormField::ClinicalNotes => "Clinical Notes",
        }
    }

    pub fn is_required(&self) -> bool {
        true
    }

    pub fn is_textarea(&self) -> bool {
        true
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
    pub reason: TextareaState,
    pub clinical_notes: TextareaState,
    pub focused_field: ConsultationFormField,
    pub is_valid: bool,
    pub is_edit_mode: bool,
    pub consultation_id: Option<uuid::Uuid>,
    errors: HashMap<ConsultationFormField, String>,
    theme: Theme,
    scroll: ScrollableFormState,
}

impl Clone for ConsultationForm {
    fn clone(&self) -> Self {
        Self {
            reason: self.reason.clone(),
            clinical_notes: self.clinical_notes.clone(),
            focused_field: self.focused_field,
            is_valid: self.is_valid,
            is_edit_mode: self.is_edit_mode,
            consultation_id: self.consultation_id,
            errors: self.errors.clone(),
            theme: self.theme.clone(),
            scroll: self.scroll.clone(),
        }
    }
}

fn reason_textarea() -> TextareaState {
    TextareaState::new("Reason").with_height_mode(HeightMode::AutoGrow { min: 2, max: 4 })
}

fn clinical_textarea() -> TextareaState {
    TextareaState::new("Clinical Notes").with_height_mode(HeightMode::AutoGrow { min: 10, max: 20 })
}

impl ConsultationForm {
    pub fn new(theme: Theme) -> Self {
        Self {
            reason: reason_textarea(),
            clinical_notes: clinical_textarea(),
            focused_field: ConsultationFormField::Reason,
            is_valid: true,
            is_edit_mode: false,
            consultation_id: None,
            errors: HashMap::new(),
            theme: theme.clone(),
            scroll: ScrollableFormState::new(),
        }
    }

    pub fn from_consultation(theme: Theme, consultation: &Consultation) -> Self {
        let reason = reason_textarea().with_value(consultation.reason.clone().unwrap_or_default());
        let clinical_notes =
            clinical_textarea().with_value(consultation.clinical_notes.clone().unwrap_or_default());

        Self {
            reason,
            clinical_notes,
            focused_field: ConsultationFormField::Reason,
            is_valid: true,
            is_edit_mode: true,
            consultation_id: Some(consultation.id),
            errors: HashMap::new(),
            theme,
            scroll: ScrollableFormState::new(),
        }
    }

    pub fn focused_field(&self) -> ConsultationFormField {
        self.focused_field
    }

    pub fn get_value(&self, field: ConsultationFormField) -> String {
        match field {
            ConsultationFormField::Reason => self.reason.value(),
            ConsultationFormField::ClinicalNotes => self.clinical_notes.value(),
        }
    }

    pub fn set_value(&mut self, field: ConsultationFormField, value: String) {
        match field {
            ConsultationFormField::Reason => {
                self.reason = reason_textarea().with_value(value);
            }
            ConsultationFormField::ClinicalNotes => {
                self.clinical_notes = clinical_textarea().with_value(value);
            }
        }
        self.validate_field(&field);
    }

    fn validate_field(&mut self, field: &ConsultationFormField) {
        self.errors.remove(field);

        let value = self.get_value(*field);

        match field {
            ConsultationFormField::Reason => {
                if value.trim().is_empty() {
                    self.errors.insert(*field, "Reason is required".to_string());
                }
            }
            ConsultationFormField::ClinicalNotes => {
                if value.trim().is_empty() {
                    self.errors
                        .insert(*field, "Clinical notes are required".to_string());
                }
            }
        }
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

        // Ctrl+S submits the form from any field
        if key.modifiers.contains(KeyModifiers::CONTROL) && matches!(key.code, KeyCode::Char('s')) {
            self.validate();
            return Some(ConsultationFormAction::Submit);
        }

        if self.focused_field.is_textarea() {
            let ratatui_key = to_ratatui_key(key);
            let consumed = match self.focused_field {
                ConsultationFormField::Reason => self.reason.handle_key(ratatui_key),
                ConsultationFormField::ClinicalNotes => self.clinical_notes.handle_key(ratatui_key),
            };
            if consumed {
                return Some(ConsultationFormAction::ValueChanged);
            }
        }

        match key.code {
            KeyCode::Tab => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    return Some(ConsultationFormAction::Cancel);
                }
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
            KeyCode::PageUp => {
                self.scroll.scroll_up();
                Some(ConsultationFormAction::FocusChanged)
            }
            KeyCode::PageDown => {
                self.scroll.scroll_down();
                Some(ConsultationFormAction::FocusChanged)
            }
            KeyCode::Enter => None,
            KeyCode::Esc => Some(ConsultationFormAction::Cancel),
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
            reason: Some(self.reason.value()).filter(|s| !s.is_empty()),
            clinical_notes: Some(self.clinical_notes.value()).filter(|s| !s.is_empty()),
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

impl FormFieldMeta for ConsultationFormField {
    fn label(&self) -> &'static str {
        ConsultationFormField::label(self)
    }

    fn is_required(&self) -> bool {
        ConsultationFormField::is_required(self)
    }
}

impl FormNavigation for ConsultationForm {
    type FormField = ConsultationFormField;

    fn get_error(&self, field: Self::FormField) -> Option<&str> {
        self.errors.get(&field).map(|s| s.as_str())
    }

    fn set_error(&mut self, field: Self::FormField, error: Option<String>) {
        match error {
            Some(msg) => {
                self.errors.insert(field, msg);
            }
            None => {
                self.errors.remove(&field);
            }
        }
    }

    fn validate(&mut self) -> bool {
        self.errors.clear();
        self.is_valid = self.errors.is_empty();
        self.is_valid
    }

    fn current_field(&self) -> Self::FormField {
        self.focused_field
    }

    fn fields(&self) -> Vec<Self::FormField> {
        ConsultationFormField::all()
    }

    fn set_current_field(&mut self, field: Self::FormField) {
        self.focused_field = field;
    }
}

impl Widget for ConsultationForm {
    fn render(mut self, area: Rect, buf: &mut Buffer) {
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

        let mut total_height: u16 = 0;
        for _field in &fields {
            total_height += 4;
        }
        self.scroll.set_total_height(total_height);
        self.scroll.clamp_offset(inner.height.saturating_sub(2));

        let mut y: i32 = (inner.y as i32) + 1 - (self.scroll.scroll_offset as i32);
        let max_y = inner.y as i32 + inner.height as i32 - 2;

        for field in fields {
            if y + 2 <= inner.y as i32 || y >= max_y {
                y += 4;
                continue;
            }

            let is_focused = field == self.focused_field;

            if field.is_textarea() {
                let textarea = match field {
                    ConsultationFormField::Reason => &self.reason,
                    ConsultationFormField::ClinicalNotes => &self.clinical_notes,
                };
                let field_height = textarea.height();
                if y >= inner.y as i32 && y < max_y {
                    let field_area =
                        Rect::new(inner.x + 1, y as u16, inner.width - 2, field_height);
                    TextareaWidget::new(textarea, self.theme.clone())
                        .focused(is_focused)
                        .render(field_area, buf);
                }
                y += field_height as i32;
                continue;
            }

            let has_error = self.error(field).is_some();

            if y >= inner.y as i32 && y < max_y {
                let label_style = if is_focused {
                    Style::default()
                        .fg(self.theme.colors.primary)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(self.theme.colors.foreground)
                };

                buf.set_string(inner.x + 1, y as u16, field.label(), label_style);

                if is_focused {
                    buf.set_string(
                        field_start - 1,
                        y as u16,
                        ">",
                        Style::default().fg(self.theme.colors.primary),
                    );
                }
            }

            let max_value_width = inner.width.saturating_sub(label_width + 4);

            if y >= inner.y as i32 && y < max_y {
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

                buf.set_string(field_start, y as u16, display_value, value_style);
            }

            if y >= inner.y as i32 && y < max_y {
                if let Some(error_msg) = self.error(field) {
                    let error_style = Style::default().fg(self.theme.colors.error);
                    buf.set_string(
                        field_start,
                        (y as u16) + 1,
                        format!("  {}", error_msg),
                        error_style,
                    );
                }
            }

            y += 2;
        }

        self.scroll.render_scrollbar(inner, buf);

        let help_y = inner.y + inner.height - 1;
        buf.set_string(
            inner.x + 1,
            help_y,
            "Tab: Next | Ctrl+S: Submit | Esc: Cancel",
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
        assert_eq!(form.focused_field(), ConsultationFormField::ClinicalNotes);
        form.next_field();
        assert_eq!(form.focused_field(), ConsultationFormField::Reason);
        form.prev_field();
        assert_eq!(form.focused_field(), ConsultationFormField::ClinicalNotes);
    }

    #[test]
    fn test_consultation_form_all_fields_ordered() {
        let fields = ConsultationFormField::all();
        assert_eq!(fields[0], ConsultationFormField::Reason);
        assert_eq!(fields[1], ConsultationFormField::ClinicalNotes);
    }

    #[test]
    fn test_clinical_notes_is_textarea() {
        assert!(ConsultationFormField::Reason.is_textarea());
        assert!(ConsultationFormField::ClinicalNotes.is_textarea());
    }

    #[test]
    fn test_clinical_notes_field_get_set_value() {
        let theme = Theme::dark();
        let mut form = ConsultationForm::new(theme);

        form.set_value(
            ConsultationFormField::ClinicalNotes,
            "Patient reports headache".to_string(),
        );
        assert_eq!(
            form.get_value(ConsultationFormField::ClinicalNotes),
            "Patient reports headache"
        );
    }

    #[test]
    fn test_consultation_form_ctrl_s_submits() {
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

        let theme = Theme::dark();
        let mut form = ConsultationForm::new(theme);

        let key = KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL);
        let action = form.handle_key(key);
        assert!(matches!(action, Some(ConsultationFormAction::Submit)));
    }
}

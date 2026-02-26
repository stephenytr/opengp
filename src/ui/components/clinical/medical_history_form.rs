//! Medical History Form Component
//!
//! Form for creating a new medical history entry for a patient.

use std::collections::HashMap;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Borders, Widget};

use crate::domain::clinical::{ConditionStatus, MedicalHistory, Severity};
use crate::ui::layout::LABEL_WIDTH;
use crate::ui::theme::Theme;
use crate::ui::widgets::{
    parse_date, DropdownAction, DropdownOption, DropdownWidget, HeightMode, TextareaState,
    TextareaWidget,
};

type RatatuiKeyEvent = ratatui::crossterm::event::KeyEvent;
type RatatuiKeyCode = ratatui::crossterm::event::KeyCode;
type RatatuiKeyModifiers = ratatui::crossterm::event::KeyModifiers;
type RatatuiKeyEventKind = ratatui::crossterm::event::KeyEventKind;
type RatatuiKeyEventState = ratatui::crossterm::event::KeyEventState;

fn to_ratatui_key(key: KeyEvent) -> RatatuiKeyEvent {
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
pub enum MedicalHistoryFormField {
    Condition,
    DiagnosisDate,
    Status,
    Severity,
    Notes,
}

impl MedicalHistoryFormField {
    pub fn all() -> Vec<MedicalHistoryFormField> {
        vec![
            MedicalHistoryFormField::Condition,
            MedicalHistoryFormField::DiagnosisDate,
            MedicalHistoryFormField::Status,
            MedicalHistoryFormField::Severity,
            MedicalHistoryFormField::Notes,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            MedicalHistoryFormField::Condition => "Condition *",
            MedicalHistoryFormField::DiagnosisDate => "Diagnosis Date (dd/mm/yyyy)",
            MedicalHistoryFormField::Status => "Status *",
            MedicalHistoryFormField::Severity => "Severity",
            MedicalHistoryFormField::Notes => "Notes",
        }
    }

    pub fn is_required(&self) -> bool {
        matches!(
            self,
            MedicalHistoryFormField::Condition | MedicalHistoryFormField::Status
        )
    }
}

pub struct MedicalHistoryForm {
    pub condition: TextareaState,
    pub diagnosis_date: TextareaState,
    pub status_dropdown: DropdownWidget,
    pub severity_dropdown: DropdownWidget,
    pub notes: TextareaState,
    pub focused_field: MedicalHistoryFormField,
    errors: HashMap<MedicalHistoryFormField, String>,
    theme: Theme,
}

impl Clone for MedicalHistoryForm {
    fn clone(&self) -> Self {
        Self {
            condition: self.condition.clone(),
            diagnosis_date: self.diagnosis_date.clone(),
            status_dropdown: self.status_dropdown.clone(),
            severity_dropdown: self.severity_dropdown.clone(),
            notes: self.notes.clone(),
            focused_field: self.focused_field,
            errors: self.errors.clone(),
            theme: self.theme.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum MedicalHistoryFormAction {
    FocusChanged,
    ValueChanged,
    Submit,
    Cancel,
}

impl MedicalHistoryForm {
    pub fn new(theme: Theme) -> Self {
        let status_options = vec![
            DropdownOption::new("Active", "Active"),
            DropdownOption::new("Resolved", "Resolved"),
            DropdownOption::new("Chronic", "Chronic"),
            DropdownOption::new("Recurring", "Recurring"),
            DropdownOption::new("InRemission", "InRemission"),
        ];
        let severity_options = vec![
            DropdownOption::new("Mild", "Mild"),
            DropdownOption::new("Moderate", "Moderate"),
            DropdownOption::new("Severe", "Severe"),
        ];

        Self {
            condition: TextareaState::new("Condition").with_height_mode(HeightMode::SingleLine),
            diagnosis_date: TextareaState::new("DiagnosisDate")
                .with_height_mode(HeightMode::SingleLine),
            status_dropdown: DropdownWidget::new("Status *", status_options, theme.clone()),
            severity_dropdown: DropdownWidget::new("Severity", severity_options, theme.clone()),
            notes: TextareaState::new("Notes").with_height_mode(HeightMode::FixedLines(4)),
            focused_field: MedicalHistoryFormField::Condition,
            errors: HashMap::new(),
            theme,
        }
    }

    pub fn focused_field(&self) -> MedicalHistoryFormField {
        self.focused_field
    }

    pub fn set_focus(&mut self, field: MedicalHistoryFormField) {
        self.focused_field = field;
    }

    pub fn next_field(&mut self) {
        let fields = MedicalHistoryFormField::all();
        if let Some(current_idx) = fields.iter().position(|f| *f == self.focused_field) {
            let next_idx = (current_idx + 1) % fields.len();
            self.focused_field = fields[next_idx];
        }
    }

    pub fn prev_field(&mut self) {
        let fields = MedicalHistoryFormField::all();
        if let Some(current_idx) = fields.iter().position(|f| *f == self.focused_field) {
            let prev_idx = if current_idx == 0 {
                fields.len() - 1
            } else {
                current_idx - 1
            };
            self.focused_field = fields[prev_idx];
        }
    }

    pub fn get_value(&self, field: MedicalHistoryFormField) -> String {
        match field {
            MedicalHistoryFormField::Condition => self.condition.value(),
            MedicalHistoryFormField::DiagnosisDate => self.diagnosis_date.value(),
            MedicalHistoryFormField::Status => self
                .status_dropdown
                .selected_value()
                .map(String::from)
                .unwrap_or_default(),
            MedicalHistoryFormField::Severity => self
                .severity_dropdown
                .selected_value()
                .map(String::from)
                .unwrap_or_default(),
            MedicalHistoryFormField::Notes => self.notes.value(),
        }
    }

    pub fn set_value(&mut self, field: MedicalHistoryFormField, value: String) {
        match field {
            MedicalHistoryFormField::Condition => {
                self.condition = TextareaState::new("Condition")
                    .with_height_mode(HeightMode::SingleLine)
                    .with_value(value)
            }
            MedicalHistoryFormField::DiagnosisDate => {
                self.diagnosis_date = TextareaState::new("DiagnosisDate")
                    .with_height_mode(HeightMode::SingleLine)
                    .with_value(value)
            }
            MedicalHistoryFormField::Status => self.status_dropdown.set_value(&value),
            MedicalHistoryFormField::Severity => self.severity_dropdown.set_value(&value),
            MedicalHistoryFormField::Notes => {
                self.notes = TextareaState::new("Notes")
                    .with_height_mode(HeightMode::FixedLines(4))
                    .with_value(value)
            }
        }
        self.validate_field(&field);
    }

    fn validate_field(&mut self, field: &MedicalHistoryFormField) {
        self.errors.remove(field);

        let value = self.get_value(*field);

        match field {
            MedicalHistoryFormField::Condition => {
                if value.trim().is_empty() {
                    self.errors
                        .insert(*field, "Condition is required".to_string());
                }
            }
            MedicalHistoryFormField::Status => {
                if value.trim().is_empty() {
                    self.errors.insert(*field, "Status is required".to_string());
                } else if value.parse::<ConditionStatus>().is_err() {
                    self.errors.insert(
                        *field,
                        "Valid values: Active, Resolved, Chronic, Recurring, InRemission"
                            .to_string(),
                    );
                }
            }
            MedicalHistoryFormField::DiagnosisDate => {
                if !value.is_empty() && parse_date(&value).is_none() {
                    self.errors
                        .insert(*field, "Use dd/mm/yyyy format".to_string());
                }
            }
            MedicalHistoryFormField::Severity => {
                if !value.is_empty() && value.parse::<Severity>().is_err() {
                    self.errors
                        .insert(*field, "Valid values: Mild, Moderate, Severe".to_string());
                }
            }
            MedicalHistoryFormField::Notes => {}
        }
    }

    pub fn validate(&mut self) -> bool {
        self.errors.clear();

        for field in MedicalHistoryFormField::all() {
            self.validate_field(&field);
        }

        self.errors.is_empty()
    }

    pub fn error(&self, field: MedicalHistoryFormField) -> Option<&String> {
        self.errors.get(&field)
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Option<MedicalHistoryFormAction> {
        use crossterm::event::KeyEventKind;

        if key.kind != KeyEventKind::Press {
            return None;
        }

        // Delegate to dropdown when focused on Status or Severity
        match self.focused_field {
            MedicalHistoryFormField::Status => {
                if let Some(action) = self.status_dropdown.handle_key(key) {
                    match action {
                        DropdownAction::Selected(_) => {
                            let field = self.focused_field;
                            self.validate_field(&field);
                            return Some(MedicalHistoryFormAction::ValueChanged);
                        }
                        DropdownAction::Opened
                        | DropdownAction::Closed
                        | DropdownAction::FocusChanged => {
                            return Some(MedicalHistoryFormAction::FocusChanged);
                        }
                    }
                }
                return None;
            }
            MedicalHistoryFormField::Severity => {
                if let Some(action) = self.severity_dropdown.handle_key(key) {
                    match action {
                        DropdownAction::Selected(_) => {
                            let field = self.focused_field;
                            self.validate_field(&field);
                            return Some(MedicalHistoryFormAction::ValueChanged);
                        }
                        DropdownAction::Opened
                        | DropdownAction::Closed
                        | DropdownAction::FocusChanged => {
                            return Some(MedicalHistoryFormAction::FocusChanged);
                        }
                    }
                }
                return None;
            }
            _ => {}
        }

        // For text fields (Condition, DiagnosisDate, Notes), delegate to TextareaState.
        match self.focused_field {
            MedicalHistoryFormField::Condition => {
                let ratatui_key = to_ratatui_key(key);
                let consumed = self.condition.handle_key(ratatui_key);
                if consumed {
                    return Some(MedicalHistoryFormAction::ValueChanged);
                }
            }
            MedicalHistoryFormField::DiagnosisDate => {
                let ratatui_key = to_ratatui_key(key);
                let consumed = self.diagnosis_date.handle_key(ratatui_key);
                if consumed {
                    return Some(MedicalHistoryFormAction::ValueChanged);
                }
            }
            MedicalHistoryFormField::Notes => {
                // Ctrl+Enter submits the form from any field.
                if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Enter {
                    self.validate();
                    return Some(MedicalHistoryFormAction::Submit);
                }
                let ratatui_key = to_ratatui_key(key);
                let consumed = self.notes.handle_key(ratatui_key);
                if consumed {
                    return Some(MedicalHistoryFormAction::ValueChanged);
                }
            }
            _ => {}
        }

        match key.code {
            KeyCode::Tab => {
                if key
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::SHIFT)
                {
                    self.prev_field();
                } else {
                    self.next_field();
                }
                Some(MedicalHistoryFormAction::FocusChanged)
            }
            KeyCode::BackTab => {
                self.prev_field();
                Some(MedicalHistoryFormAction::FocusChanged)
            }
            KeyCode::Up => {
                self.prev_field();
                Some(MedicalHistoryFormAction::FocusChanged)
            }
            KeyCode::Down => {
                self.next_field();
                Some(MedicalHistoryFormAction::FocusChanged)
            }
            KeyCode::Enter => {
                self.validate();
                Some(MedicalHistoryFormAction::Submit)
            }
            KeyCode::Esc => Some(MedicalHistoryFormAction::Cancel),
            _ => None,
        }
    }

    pub fn to_medical_history(
        &mut self,
        patient_id: uuid::Uuid,
        created_by: uuid::Uuid,
    ) -> Option<MedicalHistory> {
        if !self.validate() {
            return None;
        }

        let status = self
            .get_value(MedicalHistoryFormField::Status)
            .parse::<ConditionStatus>()
            .ok()?;
        let severity = self
            .get_value(MedicalHistoryFormField::Severity)
            .parse::<Severity>()
            .ok();
        let diagnosis_date = parse_date(&self.diagnosis_date.value());

        Some(MedicalHistory {
            id: uuid::Uuid::new_v4(),
            patient_id,
            condition: self.condition.value(),
            diagnosis_date,
            status,
            severity,
            notes: Some(self.notes.value()).filter(|s: &String| !s.is_empty()),
            is_active: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            created_by,
            updated_by: None,
        })
    }
}

impl Widget for MedicalHistoryForm {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.is_empty() {
            return;
        }

        let block = Block::default()
            .title(" New Medical History ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.colors.border));

        block.clone().render(area, buf);

        let inner = block.inner(area);
        if inner.is_empty() {
            return;
        }

        let label_width = LABEL_WIDTH;
        let field_start = inner.x + label_width + 2;

        let fields = MedicalHistoryFormField::all();

        let mut y = inner.y + 1;
        let max_y = inner.y + inner.height - 2;

        let mut open_dropdown: Option<(DropdownWidget, Rect)> = None;

        for field in fields {
            if y > max_y {
                break;
            }

            let is_focused = field == self.focused_field;

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

            match field {
                MedicalHistoryFormField::Status => {
                    let dropdown = self.status_dropdown.clone();
                    let dropdown_width = inner.width.saturating_sub(label_width + 4);
                    let dropdown_area = Rect::new(field_start, y, dropdown_width, 3);
                    if dropdown.is_open() {
                        open_dropdown = Some((dropdown.clone(), dropdown_area));
                    }
                    dropdown.render(dropdown_area, buf);
                    y += 2;
                }
                MedicalHistoryFormField::Severity => {
                    let dropdown = self.severity_dropdown.clone();
                    let dropdown_width = inner.width.saturating_sub(label_width + 4);
                    let dropdown_area = Rect::new(field_start, y, dropdown_width, 3);
                    if dropdown.is_open() {
                        open_dropdown = Some((dropdown.clone(), dropdown_area));
                    }
                    dropdown.render(dropdown_area, buf);
                    y += 2;
                }
                _ => {
                    let textarea_state: &TextareaState;
                    let height: u16;
                    match field {
                        MedicalHistoryFormField::Condition => {
                            textarea_state = &self.condition;
                            height = 3;
                        }
                        MedicalHistoryFormField::DiagnosisDate => {
                            textarea_state = &self.diagnosis_date;
                            height = 3;
                        }
                        MedicalHistoryFormField::Notes => {
                            textarea_state = &self.notes;
                            height = 6;
                        }
                        _ => continue,
                    };

                    let textarea_width = inner.width.saturating_sub(label_width + 4);
                    let textarea_area = Rect::new(field_start, y, textarea_width, height);

                    TextareaWidget::new(textarea_state, self.theme.clone())
                        .focused(is_focused)
                        .render(textarea_area, buf);

                    if let Some(error_msg) = self.error(field) {
                        let error_style = Style::default().fg(self.theme.colors.error);
                        buf.set_string(
                            field_start,
                            y + height,
                            format!("  {}", error_msg),
                            error_style,
                        );
                        y += 1;
                    }

                    y += height;
                }
            }
        }

        if let Some((dropdown, dropdown_area)) = open_dropdown {
            dropdown.render(dropdown_area, buf);
        }

        let help_y = inner.y + inner.height - 1;
        buf.set_string(
            inner.x + 1,
            help_y,
            "Tab: Next | Shift+Tab: Prev | Ctrl+Enter: Submit | Enter in notes: Newline | Esc: Cancel",
            Style::default().fg(self.theme.colors.disabled),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_form_creation() {
        let theme = Theme::dark();
        let form = MedicalHistoryForm::new(theme);

        assert_eq!(form.focused_field(), MedicalHistoryFormField::Condition);
        assert!(!form.has_errors());
    }

    #[test]
    fn test_form_validation_required_fields() {
        let theme = Theme::dark();
        let mut form = MedicalHistoryForm::new(theme);

        form.validate();
        assert!(form.has_errors());
        assert!(form.error(MedicalHistoryFormField::Condition).is_some());
        assert!(form.error(MedicalHistoryFormField::Status).is_some());
    }

    #[test]
    fn test_form_validation_passes_with_required_fields() {
        let theme = Theme::dark();
        let mut form = MedicalHistoryForm::new(theme);

        form.set_value(
            MedicalHistoryFormField::Condition,
            "Hypertension".to_string(),
        );
        form.set_value(MedicalHistoryFormField::Status, "Active".to_string());

        assert!(form.validate());
        assert!(!form.has_errors());
    }

    #[test]
    fn test_field_navigation() {
        let theme = Theme::dark();
        let mut form = MedicalHistoryForm::new(theme);

        assert_eq!(form.focused_field(), MedicalHistoryFormField::Condition);
        form.next_field();
        assert_eq!(form.focused_field(), MedicalHistoryFormField::DiagnosisDate);
        form.prev_field();
        assert_eq!(form.focused_field(), MedicalHistoryFormField::Condition);
    }

    #[test]
    fn test_all_fields_ordered() {
        let fields = MedicalHistoryFormField::all();
        assert_eq!(fields.len(), 5);
        assert_eq!(fields[0], MedicalHistoryFormField::Condition);
        assert_eq!(fields[1], MedicalHistoryFormField::DiagnosisDate);
        assert_eq!(fields[2], MedicalHistoryFormField::Status);
        assert_eq!(fields[3], MedicalHistoryFormField::Severity);
        assert_eq!(fields[4], MedicalHistoryFormField::Notes);
    }

    #[test]
    fn test_invalid_status_shows_error() {
        let theme = Theme::dark();
        let mut form = MedicalHistoryForm::new(theme);

        form.set_value(MedicalHistoryFormField::Status, "NotAStatus".to_string());
        assert!(form.error(MedicalHistoryFormField::Status).is_some());
    }

    #[test]
    fn test_invalid_date_shows_error() {
        let theme = Theme::dark();
        let mut form = MedicalHistoryForm::new(theme);

        form.set_value(
            MedicalHistoryFormField::DiagnosisDate,
            "not-a-date".to_string(),
        );
        assert!(form.error(MedicalHistoryFormField::DiagnosisDate).is_some());
    }
}

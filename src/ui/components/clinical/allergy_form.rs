//! Allergy Form Component
//!
//! Form for creating new patient allergies.

use std::collections::HashMap;

use crossterm::event::{KeyEvent, KeyModifiers};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Borders, Widget};

use crate::domain::clinical::{Allergy, AllergyType, Severity};
use crate::ui::layout::LABEL_WIDTH;
use crate::ui::theme::Theme;
use crate::ui::widgets::{
    parse_date, DropdownOption, DropdownWidget, HeightMode, TextareaState, TextareaWidget,
};

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
pub enum AllergyFormField {
    Allergen,
    AllergyType,
    Severity,
    Reaction,
    OnsetDate,
    Notes,
}

impl AllergyFormField {
    pub fn all() -> Vec<AllergyFormField> {
        vec![
            AllergyFormField::Allergen,
            AllergyFormField::AllergyType,
            AllergyFormField::Severity,
            AllergyFormField::Reaction,
            AllergyFormField::OnsetDate,
            AllergyFormField::Notes,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            AllergyFormField::Allergen => "Allergen *",
            AllergyFormField::AllergyType => "Allergy Type *",
            AllergyFormField::Severity => "Severity *",
            AllergyFormField::Reaction => "Reaction",
            AllergyFormField::OnsetDate => "Onset Date (dd/mm/yyyy)",
            AllergyFormField::Notes => "Notes",
        }
    }

    pub fn is_required(&self) -> bool {
        matches!(
            self,
            AllergyFormField::Allergen | AllergyFormField::AllergyType | AllergyFormField::Severity
        )
    }

    /// Returns true if this field uses TextareaWidget.
    pub fn is_textarea(&self) -> bool {
        matches!(
            self,
            AllergyFormField::Allergen | AllergyFormField::Reaction | AllergyFormField::Notes
        )
    }
}

#[derive(Debug, Clone)]
pub enum AllergyFormAction {
    FocusChanged,
    ValueChanged,
    Submit,
    Cancel,
}

pub struct AllergyForm {
    pub allergen: TextareaState,
    pub allergy_type: Option<AllergyType>,
    pub severity: Option<Severity>,
    pub reaction: TextareaState,
    pub onset_date: Option<String>,
    pub notes: TextareaState,
    pub focused_field: AllergyFormField,
    pub is_valid: bool,
    errors: HashMap<AllergyFormField, String>,
    theme: Theme,
    allergy_type_dropdown: DropdownWidget,
    severity_dropdown: DropdownWidget,
}

impl Clone for AllergyForm {
    fn clone(&self) -> Self {
        Self {
            allergen: self.allergen.clone(),
            allergy_type: self.allergy_type,
            severity: self.severity,
            reaction: self.reaction.clone(),
            onset_date: self.onset_date.clone(),
            notes: self.notes.clone(),
            focused_field: self.focused_field,
            is_valid: self.is_valid,
            errors: self.errors.clone(),
            theme: self.theme.clone(),
            allergy_type_dropdown: self.allergy_type_dropdown.clone(),
            severity_dropdown: self.severity_dropdown.clone(),
        }
    }
}

impl AllergyForm {
    pub fn new(theme: Theme) -> Self {
        let allergy_type_options = vec![
            DropdownOption::new("Drug", "Drug"),
            DropdownOption::new("Food", "Food"),
            DropdownOption::new("Environmental", "Environmental"),
            DropdownOption::new("Other", "Other"),
        ];
        let severity_options = vec![
            DropdownOption::new("Mild", "Mild"),
            DropdownOption::new("Moderate", "Moderate"),
            DropdownOption::new("Severe", "Severe"),
        ];

        Self {
            allergen: TextareaState::new("Allergen *").with_height_mode(HeightMode::SingleLine),
            allergy_type: None,
            severity: None,
            reaction: TextareaState::new("Reaction").with_height_mode(HeightMode::SingleLine),
            onset_date: None,
            notes: TextareaState::new("Notes").with_height_mode(HeightMode::FixedLines(3)),
            focused_field: AllergyFormField::Allergen,
            is_valid: false,
            errors: HashMap::new(),
            theme: theme.clone(),
            allergy_type_dropdown: DropdownWidget::new(
                "Allergy Type *",
                allergy_type_options,
                theme.clone(),
            ),
            severity_dropdown: DropdownWidget::new("Severity *", severity_options, theme),
        }
    }

    pub fn focused_field(&self) -> AllergyFormField {
        self.focused_field
    }

    pub fn next_field(&mut self) {
        let fields = AllergyFormField::all();
        if let Some(current_idx) = fields.iter().position(|f| *f == self.focused_field) {
            let next_idx = (current_idx + 1) % fields.len();
            self.focused_field = fields[next_idx];
        }
    }

    pub fn prev_field(&mut self) {
        let fields = AllergyFormField::all();
        if let Some(current_idx) = fields.iter().position(|f| *f == self.focused_field) {
            let prev_idx = if current_idx == 0 {
                fields.len() - 1
            } else {
                current_idx - 1
            };
            self.focused_field = fields[prev_idx];
        }
    }

    pub fn get_value(&self, field: AllergyFormField) -> String {
        match field {
            AllergyFormField::Allergen => self.allergen.value(),
            AllergyFormField::AllergyType => self
                .allergy_type_dropdown
                .selected_value()
                .map(|s: &str| s.to_string())
                .unwrap_or_default(),
            AllergyFormField::Severity => self
                .severity_dropdown
                .selected_value()
                .map(|s: &str| s.to_string())
                .unwrap_or_default(),
            AllergyFormField::Reaction => self.reaction.value(),
            AllergyFormField::OnsetDate => self.onset_date.clone().unwrap_or_default(),
            AllergyFormField::Notes => self.notes.value(),
        }
    }

    pub fn set_value(&mut self, field: AllergyFormField, value: String) {
        match field {
            AllergyFormField::Allergen => {
                self.allergen = TextareaState::new("Allergen *")
                    .with_height_mode(HeightMode::SingleLine)
                    .with_value(value);
            }
            AllergyFormField::AllergyType => {
                self.allergy_type_dropdown.set_value(&value);
                self.allergy_type = value.parse().ok();
            }
            AllergyFormField::Severity => {
                self.severity_dropdown.set_value(&value);
                self.severity = value.parse().ok();
            }
            AllergyFormField::Reaction => {
                self.reaction = TextareaState::new("Reaction")
                    .with_height_mode(HeightMode::SingleLine)
                    .with_value(value);
            }
            AllergyFormField::OnsetDate => {
                self.onset_date = if value.is_empty() { None } else { Some(value) };
            }
            AllergyFormField::Notes => {
                self.notes = TextareaState::new("Notes")
                    .with_height_mode(HeightMode::FixedLines(3))
                    .with_value(value);
            }
        }
        self.validate_field(&field);
    }

    fn validate_field(&mut self, field: &AllergyFormField) {
        self.errors.remove(field);

        let value = self.get_value(*field);

        match field {
            AllergyFormField::Allergen => {
                if value.trim().is_empty() {
                    self.errors
                        .insert(*field, "Allergen is required".to_string());
                }
            }
            AllergyFormField::AllergyType => {
                if value.is_empty() {
                    self.errors.insert(
                        *field,
                        "Allergy type is required (Drug/Food/Environmental/Other)".to_string(),
                    );
                }
            }
            AllergyFormField::Severity => {
                if value.is_empty() {
                    self.errors.insert(
                        *field,
                        "Severity is required (Mild/Moderate/Severe)".to_string(),
                    );
                }
            }
            AllergyFormField::OnsetDate => {
                if !value.is_empty() && parse_date(&value).is_none() {
                    self.errors
                        .insert(*field, "Use dd/mm/yyyy format".to_string());
                }
            }
            _ => {}
        }
    }

    pub fn validate(&mut self) -> bool {
        self.errors.clear();

        for field in AllergyFormField::all() {
            self.validate_field(&field);
        }

        self.is_valid = self.errors.is_empty();
        self.is_valid
    }

    pub fn error(&self, field: AllergyFormField) -> Option<&String> {
        self.errors.get(&field)
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Option<AllergyFormAction> {
        use crossterm::event::{KeyCode, KeyEventKind};

        if key.kind != KeyEventKind::Press {
            return None;
        }

        // Ctrl+Enter submits the form from any field.
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Enter {
            self.validate();
            return Some(AllergyFormAction::Submit);
        }

        match self.focused_field {
            AllergyFormField::AllergyType => {
                if self.allergy_type_dropdown.handle_key(key).is_some() {
                    if let Some(value) = self.allergy_type_dropdown.selected_value() {
                        self.allergy_type = value.parse::<AllergyType>().ok();
                        self.validate_field(&AllergyFormField::AllergyType);
                    }
                    return Some(AllergyFormAction::ValueChanged);
                }
            }
            AllergyFormField::Severity => {
                if self.severity_dropdown.handle_key(key).is_some() {
                    if let Some(value) = self.severity_dropdown.selected_value() {
                        self.severity = value.parse::<Severity>().ok();
                        self.validate_field(&AllergyFormField::Severity);
                    }
                    return Some(AllergyFormAction::ValueChanged);
                }
            }
            // Textarea fields: delegate key handling to TextareaState.
            AllergyFormField::Allergen => {
                let ratatui_key = to_ratatui_key(key);
                let consumed = self.allergen.handle_key(ratatui_key);
                if consumed {
                    self.validate_field(&AllergyFormField::Allergen);
                    return Some(AllergyFormAction::ValueChanged);
                }
            }
            AllergyFormField::Reaction => {
                let ratatui_key = to_ratatui_key(key);
                let consumed = self.reaction.handle_key(ratatui_key);
                if consumed {
                    return Some(AllergyFormAction::ValueChanged);
                }
            }
            AllergyFormField::Notes => {
                let ratatui_key = to_ratatui_key(key);
                let consumed = self.notes.handle_key(ratatui_key);
                if consumed {
                    return Some(AllergyFormAction::ValueChanged);
                }
            }
            _ => {}
        }

        match key.code {
            KeyCode::Tab => {
                if key.modifiers.contains(KeyModifiers::SHIFT) {
                    self.prev_field();
                } else {
                    self.next_field();
                }
                Some(AllergyFormAction::FocusChanged)
            }
            KeyCode::BackTab => {
                self.prev_field();
                Some(AllergyFormAction::FocusChanged)
            }
            KeyCode::Up => {
                self.prev_field();
                Some(AllergyFormAction::FocusChanged)
            }
            KeyCode::Down => {
                self.next_field();
                Some(AllergyFormAction::FocusChanged)
            }
            KeyCode::Enter => {
                self.validate();
                Some(AllergyFormAction::Submit)
            }
            KeyCode::Esc => Some(AllergyFormAction::Cancel),
            _ => None,
        }
    }

    pub fn to_allergy(&self, patient_id: uuid::Uuid, created_by: uuid::Uuid) -> Allergy {
        Allergy {
            id: uuid::Uuid::new_v4(),
            patient_id,
            allergen: self.allergen.value(),
            allergy_type: self.allergy_type.unwrap_or(AllergyType::Other),
            severity: self.severity.unwrap_or(Severity::Moderate),
            reaction: Some(self.reaction.value()).filter(|s| !s.is_empty()),
            onset_date: self.onset_date.as_deref().and_then(|d| parse_date(d)),
            notes: Some(self.notes.value()).filter(|s| !s.is_empty()),
            is_active: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            created_by,
            updated_by: None,
        }
    }
}

impl Widget for AllergyForm {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.is_empty() {
            return;
        }

        let block = Block::default()
            .title(" New Allergy ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.colors.border));

        block.clone().render(area, buf);

        let inner = block.inner(area);
        if inner.is_empty() {
            return;
        }

        let label_width = LABEL_WIDTH;
        let field_start = inner.x + label_width + 2;

        let fields = AllergyFormField::all();

        let mut y = inner.y + 1;
        let max_y = inner.y + inner.height - 2;

        let mut open_dropdown: Option<(DropdownWidget, Rect)> = None;

        for field in fields {
            if y > max_y {
                break;
            }

            let is_focused = field == self.focused_field;

            // TextareaWidget fields: render using TextareaWidget.
            if field.is_textarea() {
                let textarea_state = match field {
                    AllergyFormField::Allergen => &self.allergen,
                    AllergyFormField::Reaction => &self.reaction,
                    AllergyFormField::Notes => &self.notes,
                    _ => unreachable!(),
                };
                let field_height = textarea_state.height();
                let field_area = Rect::new(inner.x + 1, y, inner.width - 2, field_height);
                TextareaWidget::new(textarea_state, self.theme.clone())
                    .focused(is_focused)
                    .render(field_area, buf);
                y += field_height;

                if let Some(error_msg) = self.error(field) {
                    if y <= max_y {
                        let error_style = Style::default().fg(self.theme.colors.error);
                        buf.set_string(inner.x + 1, y, format!("  {}", error_msg), error_style);
                        y += 1;
                    }
                }
                continue;
            }

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

            match field {
                AllergyFormField::AllergyType => {
                    let dropdown = self.allergy_type_dropdown.clone();
                    let dropdown_area = Rect::new(field_start, y, max_value_width, 3);
                    if dropdown.is_open() {
                        open_dropdown = Some((dropdown.clone(), dropdown_area));
                    }
                    dropdown.render(dropdown_area, buf);
                    if let Some(error_msg) = self.error(field) {
                        let error_style = Style::default().fg(self.theme.colors.error);
                        buf.set_string(field_start, y + 3, format!("  {}", error_msg), error_style);
                        y += 1;
                    }
                    y += 3;
                }
                AllergyFormField::Severity => {
                    let dropdown = self.severity_dropdown.clone();
                    let dropdown_area = Rect::new(field_start, y, max_value_width, 3);
                    if dropdown.is_open() {
                        open_dropdown = Some((dropdown.clone(), dropdown_area));
                    }
                    dropdown.render(dropdown_area, buf);
                    if let Some(error_msg) = self.error(field) {
                        let error_style = Style::default().fg(self.theme.colors.error);
                        buf.set_string(field_start, y + 3, format!("  {}", error_msg), error_style);
                        y += 1;
                    }
                    y += 3;
                }
                AllergyFormField::OnsetDate => {
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
                _ => {
                    y += 2;
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
            "Tab: Next | Ctrl+Enter: Submit | Esc: Cancel",
            Style::default().fg(self.theme.colors.disabled),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allergy_form_creation() {
        let theme = Theme::dark();
        let form = AllergyForm::new(theme);

        assert_eq!(form.focused_field(), AllergyFormField::Allergen);
        assert!(!form.is_valid);
        assert!(!form.has_errors());
    }

    #[test]
    fn test_allergy_form_validation_required_fields() {
        let theme = Theme::dark();
        let mut form = AllergyForm::new(theme);

        form.validate();
        assert!(!form.is_valid);
        assert!(form.error(AllergyFormField::Allergen).is_some());
        assert!(form.error(AllergyFormField::AllergyType).is_some());
        assert!(form.error(AllergyFormField::Severity).is_some());
    }

    #[test]
    fn test_allergy_form_validation_passes_when_required_filled() {
        let theme = Theme::dark();
        let mut form = AllergyForm::new(theme);

        form.set_value(AllergyFormField::Allergen, "Penicillin".to_string());
        form.set_value(AllergyFormField::AllergyType, "Drug".to_string());
        form.set_value(AllergyFormField::Severity, "Severe".to_string());

        let valid = form.validate();
        assert!(valid);
        assert!(!form.has_errors());
    }

    #[test]
    fn test_allergy_form_field_navigation() {
        let theme = Theme::dark();
        let mut form = AllergyForm::new(theme);

        assert_eq!(form.focused_field(), AllergyFormField::Allergen);
        form.next_field();
        assert_eq!(form.focused_field(), AllergyFormField::AllergyType);
        form.next_field();
        assert_eq!(form.focused_field(), AllergyFormField::Severity);
        form.prev_field();
        assert_eq!(form.focused_field(), AllergyFormField::AllergyType);
    }

    #[test]
    fn test_allergy_form_onset_date_validation() {
        let theme = Theme::dark();
        let mut form = AllergyForm::new(theme);

        form.set_value(AllergyFormField::OnsetDate, "not-a-date".to_string());
        assert!(form.error(AllergyFormField::OnsetDate).is_some());

        form.set_value(AllergyFormField::OnsetDate, "15/01/2024".to_string());
        assert!(form.error(AllergyFormField::OnsetDate).is_none());
    }

    #[test]
    fn test_allergy_form_all_fields_ordered() {
        let fields = AllergyFormField::all();
        assert_eq!(fields[0], AllergyFormField::Allergen);
        assert_eq!(fields[1], AllergyFormField::AllergyType);
        assert_eq!(fields[2], AllergyFormField::Severity);
        assert_eq!(fields[3], AllergyFormField::Reaction);
        assert_eq!(fields[4], AllergyFormField::OnsetDate);
        assert_eq!(fields[5], AllergyFormField::Notes);
    }

    #[test]
    fn test_allergy_form_textarea_fields_accept_input() {
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

        let theme = Theme::dark();
        let mut form = AllergyForm::new(theme);

        let key = KeyEvent::new(KeyCode::Char('P'), KeyModifiers::NONE);
        let action = form.handle_key(key);
        assert!(action.is_some());

        let key = KeyEvent::new(KeyCode::Char('e'), KeyModifiers::NONE);
        form.handle_key(key);

        assert!(form.get_value(AllergyFormField::Allergen).contains('P'));
    }

    #[test]
    fn test_allergy_form_get_value_uses_textarea() {
        let theme = Theme::dark();
        let mut form = AllergyForm::new(theme);

        form.set_value(AllergyFormField::Allergen, "Penicillin".to_string());
        form.set_value(AllergyFormField::Reaction, "Rash".to_string());
        form.set_value(AllergyFormField::Notes, "Severe reaction noted".to_string());

        assert_eq!(form.get_value(AllergyFormField::Allergen), "Penicillin");
        assert_eq!(form.get_value(AllergyFormField::Reaction), "Rash");
        assert_eq!(
            form.get_value(AllergyFormField::Notes),
            "Severe reaction noted"
        );
    }
}

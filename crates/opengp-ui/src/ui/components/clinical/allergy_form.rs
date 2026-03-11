//! Allergy Form Component
//!
//! Form for creating or editing a patient allergy.

use std::collections::HashMap;

use chrono::NaiveDate;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Borders, Widget};
use uuid::Uuid;

use crate::ui::input::to_ratatui_key;
use crate::ui::layout::LABEL_WIDTH;
use crate::ui::theme::Theme;
use crate::ui::widgets::{
    parse_date, DatePickerAction, DatePickerPopup, DropdownOption, DropdownWidget, FormFieldMeta,
    FormNavigation, HeightMode, ScrollableFormState, TextareaState, TextareaWidget,
};
use opengp_domain::domain::clinical::{Allergy, AllergyType, Severity};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FormMode {
    #[default]
    Create,
    Edit(Uuid),
}

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

impl AllergyFormField {
    pub fn all() -> Vec<AllergyFormField> {
        use strum::IntoEnumIterator;
        AllergyFormField::iter().collect()
    }

    pub fn label(&self) -> &'static str {
        use strum::IntoStaticStr;
        (*self).into()
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

    pub fn is_dropdown(&self) -> bool {
        matches!(
            self,
            AllergyFormField::AllergyType | AllergyFormField::Severity
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
    mode: FormMode,
    allergen: TextareaState,
    allergy_type: Option<AllergyType>,
    severity: Option<Severity>,
    reaction: TextareaState,
    onset_date: Option<NaiveDate>,
    notes: TextareaState,
    focused_field: AllergyFormField,
    is_valid: bool,
    errors: HashMap<AllergyFormField, String>,
    theme: Theme,
    scroll: ScrollableFormState,
    allergy_type_dropdown: DropdownWidget,
    severity_dropdown: DropdownWidget,
    date_picker: DatePickerPopup,
}

impl Clone for AllergyForm {
    fn clone(&self) -> Self {
        Self {
            mode: self.mode,
            allergen: self.allergen.clone(),
            allergy_type: self.allergy_type,
            severity: self.severity,
            reaction: self.reaction.clone(),
            onset_date: self.onset_date,
            notes: self.notes.clone(),
            focused_field: self.focused_field,
            is_valid: self.is_valid,
            errors: self.errors.clone(),
            theme: self.theme.clone(),
            scroll: self.scroll.clone(),
            allergy_type_dropdown: self.allergy_type_dropdown.clone(),
            severity_dropdown: self.severity_dropdown.clone(),
            date_picker: self.date_picker.clone(),
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
            mode: FormMode::Create,
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
            scroll: ScrollableFormState::new(),
            allergy_type_dropdown: DropdownWidget::new(
                "Allergy Type *",
                allergy_type_options,
                theme.clone(),
            ),
            severity_dropdown: DropdownWidget::new("Severity *", severity_options, theme),
            date_picker: DatePickerPopup::new(),
        }
    }

    pub fn from_allergy(allergy: Allergy, theme: Theme) -> Self {
        let mut form = Self::new(theme);
        form.mode = FormMode::Edit(allergy.id);

        form.allergen = TextareaState::new("Allergen *")
            .with_height_mode(HeightMode::SingleLine)
            .with_value(allergy.allergen);

        form.allergy_type = Some(allergy.allergy_type);
        form.allergy_type_dropdown
            .set_value(&allergy.allergy_type.to_string());

        form.severity = Some(allergy.severity);
        form.severity_dropdown
            .set_value(&allergy.severity.to_string());

        if let Some(reaction) = allergy.reaction {
            form.reaction = TextareaState::new("Reaction")
                .with_height_mode(HeightMode::SingleLine)
                .with_value(reaction);
        }

        form.onset_date = allergy.onset_date;

        if let Some(notes) = allergy.notes {
            form.notes = TextareaState::new("Notes")
                .with_height_mode(HeightMode::FixedLines(3))
                .with_value(notes);
        }

        form
    }

    pub fn is_edit_mode(&self) -> bool {
        matches!(self.mode, FormMode::Edit(_))
    }

    pub fn allergy_id(&self) -> Option<Uuid> {
        match self.mode {
            FormMode::Edit(id) => Some(id),
            FormMode::Create => None,
        }
    }

    pub fn focused_field(&self) -> AllergyFormField {
        self.focused_field
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
            AllergyFormField::OnsetDate => self
                .onset_date
                .map(|d| d.format("%d/%m/%Y").to_string())
                .unwrap_or_default(),
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
                let parsed = if value.is_empty() {
                    None
                } else {
                    parse_date(&value)
                };
                self.onset_date = parsed;
                if !value.is_empty() && parsed.is_none() {
                    self.errors.insert(
                        AllergyFormField::OnsetDate,
                        "Use dd/mm/yyyy format".to_string(),
                    );
                    return;
                }
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

    pub fn error(&self, field: AllergyFormField) -> Option<&String> {
        self.errors.get(&field)
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Option<AllergyFormAction> {
        use crossterm::event::{KeyCode, KeyEventKind};

        if key.kind != KeyEventKind::Press {
            return None;
        }

        // Ctrl+S submits the form from any field
        if key.modifiers.contains(KeyModifiers::CONTROL) && matches!(key.code, KeyCode::Char('s')) {
            self.validate();
            return Some(AllergyFormAction::Submit);
        }

        // Esc cancels the form
        if key.code == KeyCode::Esc {
            return Some(AllergyFormAction::Cancel);
        }

        if self.date_picker.is_visible() {
            if let Some(action) = self.date_picker.handle_key(key) {
                match action {
                    DatePickerAction::Selected(date) => {
                        self.onset_date = Some(date);
                        self.validate_field(&AllergyFormField::OnsetDate);
                        return Some(AllergyFormAction::ValueChanged);
                    }
                    DatePickerAction::Dismissed => {
                        return Some(AllergyFormAction::FocusChanged);
                    }
                }
            }
            return Some(AllergyFormAction::FocusChanged);
        }

        if self.focused_field == AllergyFormField::OnsetDate {
            if matches!(key.code, KeyCode::Enter | KeyCode::Char(' ')) {
                let current_value = self.onset_date;
                self.date_picker.open(current_value);
                return Some(AllergyFormAction::FocusChanged);
            }
        }

        match self.focused_field {
            AllergyFormField::AllergyType => {
                if let Some(action) = self.allergy_type_dropdown.handle_key(key) {
                    // Allow Tab/BackTab/Esc to pass through to form's navigation handler
                    match key.code {
                        KeyCode::Tab | KeyCode::BackTab | KeyCode::Esc => {
                            // Return None so caller handles Tab for field navigation
                            return None;
                        }
                        _ => {
                            if let Some(value) = self.allergy_type_dropdown.selected_value() {
                                self.allergy_type = value.parse::<AllergyType>().ok();
                                self.validate_field(&AllergyFormField::AllergyType);
                            }
                            return Some(AllergyFormAction::ValueChanged);
                        }
                    }
                }
            }
            AllergyFormField::Severity => {
                if let Some(action) = self.severity_dropdown.handle_key(key) {
                    // Allow Tab/BackTab/Esc to pass through to form's navigation handler
                    match key.code {
                        KeyCode::Tab | KeyCode::BackTab | KeyCode::Esc => {
                            // Return None so caller handles Tab for field navigation
                            return None;
                        }
                        _ => {
                            if let Some(value) = self.severity_dropdown.selected_value() {
                                self.severity = value.parse::<Severity>().ok();
                                self.validate_field(&AllergyFormField::Severity);
                            }
                            return Some(AllergyFormAction::ValueChanged);
                        }
                    }
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
            KeyCode::PageUp => {
                self.scroll.scroll_up();
                Some(AllergyFormAction::FocusChanged)
            }
            KeyCode::PageDown => {
                self.scroll.scroll_down();
                Some(AllergyFormAction::FocusChanged)
            }
            KeyCode::Enter => None,
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
            onset_date: self.onset_date,
            notes: Some(self.notes.value()).filter(|s| !s.is_empty()),
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

        for field in AllergyFormField::all() {
            self.validate_field(&field);
        }

        self.is_valid = self.errors.is_empty();
        self.is_valid
    }

    fn current_field(&self) -> Self::FormField {
        self.focused_field
    }

    fn fields(&self) -> Vec<Self::FormField> {
        AllergyFormField::all()
    }

    fn set_current_field(&mut self, field: Self::FormField) {
        self.focused_field = field;
    }
}

impl Widget for AllergyForm {
    fn render(mut self, area: Rect, buf: &mut Buffer) {
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

        let mut total_height: u16 = 0;
        for field in &fields {
            if field.is_textarea() {
                total_height += 2;
            } else if field.is_dropdown() {
                total_height += 4;
            } else {
                total_height += 2;
            }
        }
        self.scroll.set_total_height(total_height);
        self.scroll.clamp_offset(inner.height.saturating_sub(2));

        let mut y: i32 = (inner.y as i32) + 1 - (self.scroll.scroll_offset as i32);
        let max_y = inner.y as i32 + inner.height as i32 - 2;

        let mut open_dropdown: Option<(DropdownWidget, Rect)> = None;

        for field in fields {
            let field_height = if field.is_textarea() {
                2i32
            } else if field.is_dropdown() {
                4i32
            } else {
                2i32
            };

            if y + field_height <= inner.y as i32 || y >= max_y {
                y += field_height;
                continue;
            }

            let is_focused = field == self.focused_field;

            if field.is_textarea() {
                let textarea_state = match field {
                    AllergyFormField::Allergen => &self.allergen,
                    AllergyFormField::Reaction => &self.reaction,
                    AllergyFormField::Notes => &self.notes,
                    _ => unreachable!(),
                };
                let field_height = textarea_state.height();
                if y >= inner.y as i32 && y < max_y {
                    let field_area =
                        Rect::new(inner.x + 1, y as u16, inner.width - 2, field_height);
                    TextareaWidget::new(textarea_state, self.theme.clone())
                        .focused(is_focused)
                        .render(field_area, buf);

                    if let Some(error_msg) = self.error(field) {
                        if (y as u16) + field_height <= inner.y + inner.height - 2 {
                            let error_style = Style::default().fg(self.theme.colors.error);
                            buf.set_string(
                                inner.x + 1,
                                (y as u16) + field_height,
                                format!("  {}", error_msg),
                                error_style,
                            );
                        }
                    }
                }
                y += field_height as i32;
                continue;
            }

            let has_error = self.error(field).is_some();

            if y >= inner.y as i32 && y < max_y {
                if !field.is_dropdown() {
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
            }

            let max_value_width = inner.width.saturating_sub(label_width + 4);

            match field {
                AllergyFormField::AllergyType => {
                    let dropdown = self.allergy_type_dropdown.clone();
                    if y >= inner.y as i32 && y < max_y {
                        let dropdown_area = Rect::new(field_start, y as u16, max_value_width, 3);
                        if dropdown.is_open() {
                            open_dropdown = Some((dropdown.clone(), dropdown_area));
                        }
                        dropdown.focused(is_focused).render(dropdown_area, buf);
                        if let Some(error_msg) = self.error(field) {
                            let error_style = Style::default().fg(self.theme.colors.error);
                            buf.set_string(
                                field_start,
                                (y as u16) + 3,
                                format!("  {}", error_msg),
                                error_style,
                            );
                        }
                    }
                    y += 4;
                }
                AllergyFormField::Severity => {
                    let dropdown = self.severity_dropdown.clone();
                    if y >= inner.y as i32 && y < max_y {
                        let dropdown_area = Rect::new(field_start, y as u16, max_value_width, 3);
                        if dropdown.is_open() {
                            open_dropdown = Some((dropdown.clone(), dropdown_area));
                        }
                        dropdown.focused(is_focused).render(dropdown_area, buf);
                        if let Some(error_msg) = self.error(field) {
                            let error_style = Style::default().fg(self.theme.colors.error);
                            buf.set_string(
                                field_start,
                                (y as u16) + 3,
                                format!("  {}", error_msg),
                                error_style,
                            );
                        }
                    }
                    y += 4;
                }
                AllergyFormField::OnsetDate => {
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
                _ => {
                    y += 2;
                }
            }
        }

        if let Some((dropdown, dropdown_area)) = open_dropdown {
            dropdown.render(dropdown_area, buf);
        }

        self.scroll.render_scrollbar(inner, buf);

        let help_y = inner.y + inner.height - 1;
        buf.set_string(
            inner.x + 1,
            help_y,
            "Tab: Next | Ctrl+S: Submit | Esc: Cancel",
            Style::default().fg(self.theme.colors.disabled),
        );

        if self.date_picker.is_visible() {
            self.date_picker.render(area, buf);
        }
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

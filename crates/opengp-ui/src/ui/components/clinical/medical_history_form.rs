//! Medical History Form Component
//!
//! Form for creating or editing a medical history entry for a patient.

use std::collections::HashMap;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use opengp_config::forms::ValidationRules;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Borders, Widget};
use uuid::Uuid;

use crate::ui::input::to_ratatui_key;
use crate::ui::layout::LABEL_WIDTH;
use crate::ui::theme::Theme;
use crate::ui::widgets::{
    parse_date, DropdownAction, DropdownOption, DropdownWidget, DynamicForm, DynamicFormMeta,
    FieldType, FormFieldMeta, FormNavigation, FormValidator, HeightMode, ScrollableFormState,
    TextareaState, TextareaWidget,
};
use opengp_domain::domain::clinical::{ConditionStatus, MedicalHistory, Severity};

const FIELD_CONDITION: &str = "condition";
const FIELD_DIAGNOSIS_DATE: &str = "diagnosis_date";
const FIELD_STATUS: &str = "status";
const FIELD_SEVERITY: &str = "severity";
const FIELD_NOTES: &str = "notes";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FormMode {
    #[default]
    Create,
    Edit(Uuid),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, strum::EnumIter, strum::IntoStaticStr)]
pub enum MedicalHistoryFormField {
    #[strum(to_string = "Condition *")]
    Condition,
    #[strum(to_string = "Diagnosis Date (dd/mm/yyyy)")]
    DiagnosisDate,
    #[strum(to_string = "Status *")]
    Status,
    #[strum(to_string = "Severity")]
    Severity,
    #[strum(to_string = "Notes")]
    Notes,
}

impl MedicalHistoryFormField {
    pub fn all() -> Vec<MedicalHistoryFormField> {
        use strum::IntoEnumIterator;
        MedicalHistoryFormField::iter().collect()
    }

    pub fn label(&self) -> &'static str {
        (*self).into()
    }

    pub fn id(&self) -> &'static str {
        match self {
            MedicalHistoryFormField::Condition => FIELD_CONDITION,
            MedicalHistoryFormField::DiagnosisDate => FIELD_DIAGNOSIS_DATE,
            MedicalHistoryFormField::Status => FIELD_STATUS,
            MedicalHistoryFormField::Severity => FIELD_SEVERITY,
            MedicalHistoryFormField::Notes => FIELD_NOTES,
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            FIELD_CONDITION => Some(MedicalHistoryFormField::Condition),
            FIELD_DIAGNOSIS_DATE => Some(MedicalHistoryFormField::DiagnosisDate),
            FIELD_STATUS => Some(MedicalHistoryFormField::Status),
            FIELD_SEVERITY => Some(MedicalHistoryFormField::Severity),
            FIELD_NOTES => Some(MedicalHistoryFormField::Notes),
            _ => None,
        }
    }

    pub fn is_required(&self) -> bool {
        matches!(
            self,
            MedicalHistoryFormField::Condition | MedicalHistoryFormField::Status
        )
    }

    pub fn is_dropdown(&self) -> bool {
        matches!(
            self,
            MedicalHistoryFormField::Status | MedicalHistoryFormField::Severity
        )
    }
}

pub struct MedicalHistoryForm {
    mode: FormMode,
    textareas: HashMap<String, TextareaState>,
    dropdowns: HashMap<String, DropdownWidget>,
    field_ids: Vec<String>,
    pub focused_field: MedicalHistoryFormField,
    errors: HashMap<String, String>,
    validator: FormValidator,
    theme: Theme,
    scroll: ScrollableFormState,
}

impl Clone for MedicalHistoryForm {
    fn clone(&self) -> Self {
        Self {
            mode: self.mode,
            textareas: self.textareas.clone(),
            dropdowns: self.dropdowns.clone(),
            field_ids: self.field_ids.clone(),
            focused_field: self.focused_field,
            errors: self.errors.clone(),
            validator: build_validator(),
            theme: self.theme.clone(),
            scroll: self.scroll.clone(),
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

impl FormFieldMeta for MedicalHistoryFormField {
    fn label(&self) -> &'static str {
        MedicalHistoryFormField::label(self)
    }

    fn is_required(&self) -> bool {
        MedicalHistoryFormField::is_required(self)
    }
}

impl DynamicFormMeta for MedicalHistoryForm {
    fn label(&self, field_id: &str) -> String {
        MedicalHistoryFormField::from_id(field_id)
            .map(|field| field.label().to_string())
            .unwrap_or_else(|| field_id.to_string())
    }

    fn is_required(&self, field_id: &str) -> bool {
        MedicalHistoryFormField::from_id(field_id)
            .map(|field| field.is_required())
            .unwrap_or(false)
    }

    fn field_type(&self, field_id: &str) -> FieldType {
        match MedicalHistoryFormField::from_id(field_id) {
            Some(MedicalHistoryFormField::DiagnosisDate) => FieldType::Date,
            Some(MedicalHistoryFormField::Status | MedicalHistoryFormField::Severity) => {
                FieldType::Select(vec![])
            }
            _ => FieldType::Text,
        }
    }
}

impl DynamicForm for MedicalHistoryForm {
    fn field_ids(&self) -> &[String] {
        &self.field_ids
    }

    fn current_field(&self) -> &str {
        self.focused_field.id()
    }

    fn set_current_field(&mut self, field_id: &str) {
        if !self.field_ids.iter().any(|id| id == field_id) {
            return;
        }

        if let Some(field) = MedicalHistoryFormField::from_id(field_id) {
            self.focused_field = field;
        }
    }

    fn get_value(&self, field_id: &str) -> String {
        self.get_value_by_id(field_id)
    }

    fn set_value(&mut self, field_id: &str, value: String) {
        self.set_value_by_id(field_id, value)
    }

    fn validate(&mut self) -> bool {
        self.errors.clear();
        for field_id in self.field_ids.clone() {
            self.validate_field_by_id(&field_id);
        }
        self.errors.is_empty()
    }

    fn get_error(&self, field_id: &str) -> Option<&str> {
        self.errors.get(field_id).map(|s| s.as_str())
    }

    fn set_error(&mut self, field_id: &str, error: Option<String>) {
        self.set_error_by_id(field_id, error);
    }
}

impl FormNavigation for MedicalHistoryForm {
    type FormField = MedicalHistoryFormField;

    fn get_error(&self, field: Self::FormField) -> Option<&str> {
        self.errors.get(field.id()).map(|s| s.as_str())
    }

    fn set_error(&mut self, field: Self::FormField, error: Option<String>) {
        self.set_error_by_id(field.id(), error);
    }

    fn validate(&mut self) -> bool {
        <Self as DynamicForm>::validate(self)
    }

    fn current_field(&self) -> Self::FormField {
        self.focused_field
    }

    fn fields(&self) -> Vec<Self::FormField> {
        self.field_ids
            .iter()
            .filter_map(|field_id| MedicalHistoryFormField::from_id(field_id))
            .collect()
    }

    fn set_current_field(&mut self, field: Self::FormField) {
        self.focused_field = field;
    }
}

impl MedicalHistoryForm {
    pub fn new(theme: Theme) -> Self {
        let field_ids = MedicalHistoryFormField::all()
            .into_iter()
            .map(|field| field.id().to_string())
            .collect::<Vec<_>>();

        let mut textareas = HashMap::new();
        textareas.insert(
            FIELD_CONDITION.to_string(),
            make_textarea_state(MedicalHistoryFormField::Condition, None),
        );
        textareas.insert(
            FIELD_DIAGNOSIS_DATE.to_string(),
            make_textarea_state(MedicalHistoryFormField::DiagnosisDate, None),
        );
        textareas.insert(
            FIELD_NOTES.to_string(),
            make_textarea_state(MedicalHistoryFormField::Notes, None),
        );

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

        let mut dropdowns = HashMap::new();
        dropdowns.insert(
            FIELD_STATUS.to_string(),
            DropdownWidget::new("Status *", status_options, theme.clone()),
        );
        dropdowns.insert(
            FIELD_SEVERITY.to_string(),
            DropdownWidget::new("Severity", severity_options, theme.clone()),
        );

        Self {
            mode: FormMode::Create,
            textareas,
            dropdowns,
            field_ids,
            focused_field: MedicalHistoryFormField::Condition,
            errors: HashMap::new(),
            validator: build_validator(),
            theme,
            scroll: ScrollableFormState::new(),
        }
    }

    pub fn from_medical_history(medical_history: MedicalHistory, theme: Theme) -> Self {
        let mut form = Self::new(theme);
        form.mode = FormMode::Edit(medical_history.id);

        form.set_value(
            MedicalHistoryFormField::Condition,
            medical_history.condition.clone(),
        );

        if let Some(diagnosis_date) = medical_history.diagnosis_date {
            form.set_value(
                MedicalHistoryFormField::DiagnosisDate,
                format!("{}", diagnosis_date.format("%d/%m/%Y")),
            );
        }

        form.set_value(
            MedicalHistoryFormField::Status,
            medical_history.status.to_string(),
        );

        if let Some(severity) = medical_history.severity {
            form.set_value(MedicalHistoryFormField::Severity, severity.to_string());
        }

        if let Some(notes) = medical_history.notes {
            form.set_value(MedicalHistoryFormField::Notes, notes);
        }

        form
    }

    pub fn is_edit_mode(&self) -> bool {
        matches!(self.mode, FormMode::Edit(_))
    }

    pub fn medical_history_id(&self) -> Option<Uuid> {
        match self.mode {
            FormMode::Edit(id) => Some(id),
            FormMode::Create => None,
        }
    }

    pub fn focused_field(&self) -> MedicalHistoryFormField {
        self.focused_field
    }

    pub fn set_focus(&mut self, field: MedicalHistoryFormField) {
        self.focused_field = field;
    }

    pub fn get_value(&self, field: MedicalHistoryFormField) -> String {
        self.get_value_by_id(field.id())
    }

    pub fn set_value(&mut self, field: MedicalHistoryFormField, value: String) {
        self.set_value_by_id(field.id(), value);
    }

    fn get_value_by_id(&self, field_id: &str) -> String {
        if let Some(textarea) = self.textareas.get(field_id) {
            return textarea.value();
        }

        if let Some(dropdown) = self.dropdowns.get(field_id) {
            return dropdown.selected_value().unwrap_or("").to_string();
        }

        String::new()
    }

    fn set_value_by_id(&mut self, field_id: &str, value: String) {
        if let Some(field) = MedicalHistoryFormField::from_id(field_id) {
            if let Some(textarea) = self.textareas.get_mut(field_id) {
                let focused = textarea.focused;
                *textarea = make_textarea_state(field, Some(value.clone())).focused(focused);
            } else if let Some(dropdown) = self.dropdowns.get_mut(field_id) {
                dropdown.set_value(&value);
            }
            self.validate_field_by_id(field_id);
        }
    }

    fn set_error_by_id(&mut self, field_id: &str, error: Option<String>) {
        match error {
            Some(message) => {
                self.errors.insert(field_id.to_string(), message);
            }
            None => {
                self.errors.remove(field_id);
            }
        }
    }

    fn validate_field_by_id(&mut self, field_id: &str) {
        self.errors.remove(field_id);
        let value = self.get_value_by_id(field_id);

        let mut errors = self.validator.validate(field_id, &value);

        match field_id {
            FIELD_CONDITION => {
                if value.trim().is_empty() {
                    errors = vec!["Condition is required".to_string()];
                }
            }
            FIELD_STATUS => {
                if value.trim().is_empty() {
                    errors = vec!["Status is required".to_string()];
                } else if value.parse::<ConditionStatus>().is_err() {
                    errors = vec![
                        "Valid values: Active, Resolved, Chronic, Recurring, InRemission"
                            .to_string(),
                    ];
                }
            }
            FIELD_DIAGNOSIS_DATE => {
                if !value.is_empty() && parse_date(&value).is_none() {
                    errors = vec!["Use dd/mm/yyyy format".to_string()];
                }
            }
            FIELD_SEVERITY => {
                if !value.is_empty() && value.parse::<Severity>().is_err() {
                    errors = vec!["Valid values: Mild, Moderate, Severe".to_string()];
                }
            }
            FIELD_NOTES => {}
            _ => {}
        }

        self.set_error_by_id(field_id, errors.into_iter().next());
    }

    pub fn error(&self, field: MedicalHistoryFormField) -> Option<&String> {
        self.errors.get(field.id())
    }

    #[allow(clippy::question_mark)]
    pub fn handle_key(&mut self, key: KeyEvent) -> Option<MedicalHistoryFormAction> {
        use crossterm::event::KeyEventKind;

        if key.kind != KeyEventKind::Press {
            return None;
        }

        if key.modifiers.contains(KeyModifiers::CONTROL) && matches!(key.code, KeyCode::Char('s')) {
            FormNavigation::validate(self);
            return Some(MedicalHistoryFormAction::Submit);
        }

        if matches!(
            self.focused_field,
            MedicalHistoryFormField::Status | MedicalHistoryFormField::Severity
        ) {
            let field_id = self.focused_field.id().to_string();
            let action = {
                let Some(dropdown) = self.dropdowns.get_mut(&field_id) else {
                    return None;
                };
                dropdown.handle_key(key)
            };

            if let Some(action) = action {
                match key.code {
                    KeyCode::Tab | KeyCode::BackTab | KeyCode::Esc => return None,
                    _ => match action {
                        DropdownAction::Selected(_) => {
                            self.validate_field_by_id(&field_id);
                            return Some(MedicalHistoryFormAction::ValueChanged);
                        }
                        DropdownAction::Opened
                        | DropdownAction::Closed
                        | DropdownAction::FocusChanged => {
                            return Some(MedicalHistoryFormAction::FocusChanged);
                        }
                    },
                }
            }
        }

        if matches!(
            self.focused_field,
            MedicalHistoryFormField::Condition
                | MedicalHistoryFormField::DiagnosisDate
                | MedicalHistoryFormField::Notes
        ) {
            let field_id = self.focused_field.id().to_string();
            if let Some(textarea) = self.textareas.get_mut(&field_id) {
                let consumed = textarea.handle_key(to_ratatui_key(key));
                if consumed {
                    self.validate_field_by_id(&field_id);
                    return Some(MedicalHistoryFormAction::ValueChanged);
                }
            }
        }

        match key.code {
            KeyCode::Tab => {
                if key
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::SHIFT)
                {
                    FormNavigation::prev_field(self);
                } else {
                    FormNavigation::next_field(self);
                }
                Some(MedicalHistoryFormAction::FocusChanged)
            }
            KeyCode::BackTab => {
                FormNavigation::prev_field(self);
                Some(MedicalHistoryFormAction::FocusChanged)
            }
            KeyCode::Up => {
                FormNavigation::prev_field(self);
                Some(MedicalHistoryFormAction::FocusChanged)
            }
            KeyCode::Down => {
                FormNavigation::next_field(self);
                Some(MedicalHistoryFormAction::FocusChanged)
            }
            KeyCode::PageUp => {
                self.scroll.scroll_up();
                Some(MedicalHistoryFormAction::FocusChanged)
            }
            KeyCode::PageDown => {
                self.scroll.scroll_down();
                Some(MedicalHistoryFormAction::FocusChanged)
            }
            KeyCode::Enter => None,
            KeyCode::Esc => Some(MedicalHistoryFormAction::Cancel),
            _ => None,
        }
    }

    pub fn to_medical_history(
        &mut self,
        patient_id: uuid::Uuid,
        created_by: uuid::Uuid,
    ) -> Option<MedicalHistory> {
        if !FormNavigation::validate(self) {
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
        let diagnosis_date = parse_date(&self.get_value(MedicalHistoryFormField::DiagnosisDate));

        Some(MedicalHistory {
            id: uuid::Uuid::new_v4(),
            patient_id,
            condition: self.get_value(MedicalHistoryFormField::Condition),
            diagnosis_date,
            status,
            severity,
            notes: Some(self.get_value(MedicalHistoryFormField::Notes))
                .filter(|s: &String| !s.is_empty()),
            is_active: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            created_by,
            updated_by: None,
        })
    }
}

fn make_textarea_state(field: MedicalHistoryFormField, value: Option<String>) -> TextareaState {
    let mut state = match field {
        MedicalHistoryFormField::Condition => {
            TextareaState::new("Condition").with_height_mode(HeightMode::SingleLine)
        }
        MedicalHistoryFormField::DiagnosisDate => {
            TextareaState::new("DiagnosisDate").with_height_mode(HeightMode::SingleLine)
        }
        MedicalHistoryFormField::Notes => {
            TextareaState::new("Notes").with_height_mode(HeightMode::FixedLines(4))
        }
        _ => TextareaState::new("Field").with_height_mode(HeightMode::SingleLine),
    };

    if let Some(value) = value {
        state = state.with_value(value);
    }

    state
}

fn build_validator() -> FormValidator {
    let mut rules = HashMap::new();
    rules.insert(
        FIELD_CONDITION.to_string(),
        ValidationRules {
            required: true,
            ..ValidationRules::default()
        },
    );
    rules.insert(
        FIELD_DIAGNOSIS_DATE.to_string(),
        ValidationRules {
            date_format: Some("dd/mm/yyyy".to_string()),
            ..ValidationRules::default()
        },
    );
    rules.insert(
        FIELD_STATUS.to_string(),
        ValidationRules {
            required: true,
            ..ValidationRules::default()
        },
    );
    rules.insert(FIELD_SEVERITY.to_string(), ValidationRules::default());
    rules.insert(FIELD_NOTES.to_string(), ValidationRules::default());

    FormValidator::new(&rules)
}

impl Widget for MedicalHistoryForm {
    fn render(mut self, area: Rect, buf: &mut Buffer) {
        if area.is_empty() {
            return;
        }

        let title = if self.is_edit_mode() {
            " Edit Medical History "
        } else {
            " New Medical History "
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

        let fields = MedicalHistoryFormField::all();

        let mut total_height: u16 = 0;
        for field in &fields {
            if matches!(field, MedicalHistoryFormField::Notes) {
                total_height += 7;
            } else {
                total_height += 3;
            }
            total_height += 1;
        }
        self.scroll.set_total_height(total_height);
        self.scroll.clamp_offset(inner.height.saturating_sub(2));

        let mut y: i32 = (inner.y as i32) + 1 - (self.scroll.scroll_offset as i32);
        let max_y = inner.y as i32 + inner.height as i32 - 2;

        let mut open_dropdown: Option<(DropdownWidget, Rect)> = None;

        for field in fields {
            let field_height = if matches!(field, MedicalHistoryFormField::Notes) {
                7i32
            } else {
                3i32
            };

            if y + field_height <= inner.y as i32 || y >= max_y {
                y += field_height + 1;
                continue;
            }

            let is_focused = field == self.focused_field;

            if y >= inner.y as i32 && y < max_y && !field.is_dropdown() {
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

            match field {
                MedicalHistoryFormField::Status | MedicalHistoryFormField::Severity => {
                    let field_id = field.id();
                    let Some(dropdown) = self.dropdowns.get(field_id).cloned() else {
                        y += 4;
                        continue;
                    };

                    let dropdown_width = inner.width.saturating_sub(label_width + 4);
                    if y >= inner.y as i32 && y < max_y {
                        let dropdown_area = Rect::new(field_start, y as u16, dropdown_width, 3);
                        if dropdown.is_open() {
                            open_dropdown = Some((dropdown.clone(), dropdown_area));
                        }
                        dropdown.focused(is_focused).render(dropdown_area, buf);
                    }
                    y += 4;
                }
                _ => {
                    let Some(textarea_state) = self.textareas.get(field.id()) else {
                        y += 4;
                        continue;
                    };

                    let height = textarea_state.height();

                    if y >= inner.y as i32 && y < max_y {
                        let textarea_width = inner.width.saturating_sub(label_width + 4);
                        let textarea_area =
                            Rect::new(field_start, y as u16, textarea_width, height);

                        TextareaWidget::new(textarea_state, self.theme.clone())
                            .focused(is_focused)
                            .render(textarea_area, buf);

                        if let Some(error_msg) = self.error(field) {
                            let error_style = Style::default().fg(self.theme.colors.error);
                            buf.set_string(
                                field_start,
                                (y as u16) + height,
                                format!("  {}", error_msg),
                                error_style,
                            );
                        }
                    }

                    y += height as i32 + 2;
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
            "Tab: Next | Shift+Tab: Prev | Ctrl+S: Submit | Esc: Cancel",
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

        FormNavigation::validate(&mut form);
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

        assert!(FormNavigation::validate(&mut form));
        assert!(!form.has_errors());
    }

    #[test]
    fn test_field_navigation() {
        let theme = Theme::dark();
        let mut form = MedicalHistoryForm::new(theme);

        assert_eq!(form.focused_field(), MedicalHistoryFormField::Condition);
        FormNavigation::next_field(&mut form);
        assert_eq!(form.focused_field(), MedicalHistoryFormField::DiagnosisDate);
        FormNavigation::prev_field(&mut form);
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

    #[test]
    fn test_dynamic_form_string_access_matches_enum_access() {
        let theme = Theme::dark();
        let mut form = MedicalHistoryForm::new(theme);

        <MedicalHistoryForm as DynamicForm>::set_value(
            &mut form,
            FIELD_CONDITION,
            "Asthma".to_string(),
        );

        let by_string = <MedicalHistoryForm as DynamicForm>::get_value(&form, FIELD_CONDITION);
        let by_enum = form.get_value(MedicalHistoryFormField::Condition);

        assert_eq!(by_string, by_enum);
    }
}

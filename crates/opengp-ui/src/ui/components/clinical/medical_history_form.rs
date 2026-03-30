use std::collections::HashMap;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use opengp_config::forms::ValidationRules;
use opengp_domain::domain::clinical::{ConditionStatus, MedicalHistory, Severity};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Borders, Widget};
use uuid::Uuid;

use crate::ui::input::to_ratatui_key;
use crate::ui::layout::LABEL_WIDTH;
use crate::ui::shared::FormMode;
use crate::ui::theme::Theme;
use crate::ui::widgets::{
    parse_date, DropdownAction, DropdownOption, DropdownWidget, FormField as FormStateField,
    FormState, FormValidator, HeightMode, TextareaState, TextareaWidget,
};

const FIELD_CONDITION: &str = "condition";
const FIELD_DIAGNOSIS_DATE: &str = "diagnosis_date";
const FIELD_STATUS: &str = "status";
const FIELD_SEVERITY: &str = "severity";
const FIELD_NOTES: &str = "notes";

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
    pub fn all() -> Vec<Self> {
        use strum::IntoEnumIterator;
        Self::iter().collect()
    }
    pub fn label(&self) -> &'static str {
        (*self).into()
    }
    pub fn id(&self) -> &'static str {
        match self {
            Self::Condition => FIELD_CONDITION,
            Self::DiagnosisDate => FIELD_DIAGNOSIS_DATE,
            Self::Status => FIELD_STATUS,
            Self::Severity => FIELD_SEVERITY,
            Self::Notes => FIELD_NOTES,
        }
    }
    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            FIELD_CONDITION => Some(Self::Condition),
            FIELD_DIAGNOSIS_DATE => Some(Self::DiagnosisDate),
            FIELD_STATUS => Some(Self::Status),
            FIELD_SEVERITY => Some(Self::Severity),
            FIELD_NOTES => Some(Self::Notes),
            _ => None,
        }
    }
    fn is_required(&self) -> bool {
        matches!(self, Self::Condition | Self::Status)
    }
    fn is_textarea(&self) -> bool {
        matches!(self, Self::Condition | Self::DiagnosisDate | Self::Notes)
    }
    fn is_dropdown(&self) -> bool {
        matches!(self, Self::Status | Self::Severity)
    }
}

impl FormStateField for MedicalHistoryFormField {
    fn all() -> Vec<Self> {
        Self::all()
    }
    fn label(&self) -> &'static str {
        self.label()
    }
    fn id(&self) -> &'static str {
        self.id()
    }
    fn from_id(id: &str) -> Option<Self> {
        Self::from_id(id)
    }
    fn is_required(&self) -> bool {
        self.is_required()
    }
    fn is_textarea(&self) -> bool {
        self.is_textarea()
    }
    fn is_dropdown(&self) -> bool {
        self.is_dropdown()
    }
}

pub struct MedicalHistoryForm {
    state: FormState<MedicalHistoryFormField>,
    validator: FormValidator,
}

impl Clone for MedicalHistoryForm {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            validator: build_validator(),
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
        let mut state = FormState::new(theme.clone(), MedicalHistoryFormField::Condition);
        state.textareas.insert(
            FIELD_CONDITION.to_string(),
            make_textarea_state(MedicalHistoryFormField::Condition, None),
        );
        state.textareas.insert(
            FIELD_DIAGNOSIS_DATE.to_string(),
            make_textarea_state(MedicalHistoryFormField::DiagnosisDate, None),
        );
        state.textareas.insert(
            FIELD_NOTES.to_string(),
            make_textarea_state(MedicalHistoryFormField::Notes, None),
        );
        state.dropdowns = build_dropdowns(theme);
        Self {
            state,
            validator: build_validator(),
        }
    }

    pub fn is_edit_mode(&self) -> bool {
        matches!(self.state.mode, FormMode::Edit(_))
    }

    pub fn medical_history_id(&self) -> Option<Uuid> {
        match self.state.mode {
            FormMode::Edit(id) => Some(id),
            FormMode::Create => None,
        }
    }

    pub fn focused_field(&self) -> MedicalHistoryFormField {
        self.state.focused_field()
    }

    pub fn get_value(&self, field: MedicalHistoryFormField) -> String {
        self.state.get_value(field)
    }

    pub fn set_value(&mut self, field: MedicalHistoryFormField, value: String) {
        self.state.set_value(field, value);
        self.validate_field(field);
    }

    pub fn validate(&mut self) -> bool {
        for field in self.state.field_order.clone() {
            self.validate_field(field);
        }
        self.state.errors.is_empty()
    }

    pub fn error(&self, field: MedicalHistoryFormField) -> Option<&String> {
        self.state.errors.get(field.id())
    }

    #[allow(clippy::question_mark)]
    pub fn handle_key(&mut self, key: KeyEvent) -> Option<MedicalHistoryFormAction> {
        use crossterm::event::KeyEventKind;

        if key.kind != KeyEventKind::Press {
            return None;
        }
        if key.modifiers.contains(KeyModifiers::CONTROL) && matches!(key.code, KeyCode::Char('s')) {
            self.validate();
            return Some(MedicalHistoryFormAction::Submit);
        }

        let focused = self.state.focused_field();
        if focused.is_dropdown() {
            let field_id = focused.id().to_string();
            let action = {
                let Some(dropdown) = self.state.dropdowns.get_mut(&field_id) else {
                    return None;
                };
                dropdown.handle_key(key)
            };
            if let Some(action) = action {
                match key.code {
                    KeyCode::Tab | KeyCode::BackTab | KeyCode::Esc => return None,
                    _ => match action {
                        DropdownAction::Selected(_) => {
                            self.validate_field(focused);
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

        if focused.is_textarea() {
            let field_id = focused.id().to_string();
            if let Some(textarea) = self.state.textareas.get_mut(&field_id) {
                if textarea.handle_key(to_ratatui_key(key)) {
                    self.validate_field(focused);
                    return Some(MedicalHistoryFormAction::ValueChanged);
                }
            }
        }

        match key.code {
            KeyCode::Tab => {
                if key.modifiers.contains(KeyModifiers::SHIFT) {
                    self.state.prev_field();
                } else {
                    self.state.next_field();
                }
                Some(MedicalHistoryFormAction::FocusChanged)
            }
            KeyCode::BackTab | KeyCode::Up => {
                self.state.prev_field();
                Some(MedicalHistoryFormAction::FocusChanged)
            }
            KeyCode::Down => {
                self.state.next_field();
                Some(MedicalHistoryFormAction::FocusChanged)
            }
            KeyCode::PageUp => {
                self.state.scroll.scroll_up();
                Some(MedicalHistoryFormAction::FocusChanged)
            }
            KeyCode::PageDown => {
                self.state.scroll.scroll_down();
                Some(MedicalHistoryFormAction::FocusChanged)
            }
            KeyCode::Enter => None,
            KeyCode::Esc => Some(MedicalHistoryFormAction::Cancel),
            _ => None,
        }
    }

    pub fn to_medical_history(
        &mut self,
        patient_id: Uuid,
        created_by: Uuid,
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
        let diagnosis_date = parse_date(&self.get_value(MedicalHistoryFormField::DiagnosisDate));
        Some(MedicalHistory {
            id: Uuid::new_v4(),
            patient_id,
            condition: self.get_value(MedicalHistoryFormField::Condition),
            diagnosis_date,
            status,
            severity,
            notes: Some(self.get_value(MedicalHistoryFormField::Notes)).filter(|s| !s.is_empty()),
            is_active: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            created_by,
            updated_by: None,
        })
    }

    fn validate_field(&mut self, field: MedicalHistoryFormField) {
        let field_id = field.id();
        self.state.errors.remove(field_id);
        let value = self.state.get_value(field);
        let mut errors = self.validator.validate(field_id, &value);
        match field {
            MedicalHistoryFormField::Condition => {
                if value.trim().is_empty() {
                    errors = vec!["Condition is required".to_string()];
                }
            }
            MedicalHistoryFormField::Status => {
                if value.trim().is_empty() {
                    errors = vec!["Status is required".to_string()];
                } else if value.parse::<ConditionStatus>().is_err() {
                    errors = vec![
                        "Valid values: Active, Resolved, Chronic, Recurring, InRemission"
                            .to_string(),
                    ];
                }
            }
            MedicalHistoryFormField::DiagnosisDate => {
                if !value.is_empty() && parse_date(&value).is_none() {
                    errors = vec!["Use dd/mm/yyyy format".to_string()];
                }
            }
            MedicalHistoryFormField::Severity => {
                if !value.is_empty() && value.parse::<Severity>().is_err() {
                    errors = vec!["Valid values: Mild, Moderate, Severe".to_string()];
                }
            }
            MedicalHistoryFormField::Notes => {}
        }
        self.set_error_by_id(field_id, errors.into_iter().next());
    }

    fn set_error_by_id(&mut self, field_id: &str, error: Option<String>) {
        if let Some(textarea) = self.state.textareas.get_mut(field_id) {
            textarea.set_error(error.clone());
        }
        match error {
            Some(message) => {
                self.state.errors.insert(field_id.to_string(), message);
            }
            None => {
                self.state.errors.remove(field_id);
            }
        }
    }
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
            .border_style(Style::default().fg(self.state.theme.colors.border));
        block.clone().render(area, buf);
        let inner = block.inner(area);
        if inner.is_empty() {
            return;
        }

        let label_width = LABEL_WIDTH;
        let field_start = inner.x + label_width + 2;
        let fields = MedicalHistoryFormField::all();
        let total_height: u16 = fields
            .iter()
            .map(|field| {
                if matches!(field, MedicalHistoryFormField::Notes) {
                    8
                } else {
                    4
                }
            })
            .sum();
        self.state.scroll.set_total_height(total_height);
        self.state
            .scroll
            .clamp_offset(inner.height.saturating_sub(2));

        let mut y: i32 = (inner.y as i32) + 1 - (self.state.scroll.scroll_offset as i32);
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

            let is_focused = field == self.state.focused_field();
            if y >= inner.y as i32 && y < max_y && !field.is_dropdown() {
                let label_style = if is_focused {
                    Style::default()
                        .fg(self.state.theme.colors.primary)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(self.state.theme.colors.foreground)
                };
                buf.set_string(inner.x + 1, y as u16, field.label(), label_style);
                if is_focused {
                    buf.set_string(
                        field_start - 1,
                        y as u16,
                        ">",
                        Style::default().fg(self.state.theme.colors.primary),
                    );
                }
            }

            if field.is_dropdown() {
                let Some(dropdown) = self.state.dropdowns.get(field.id()).cloned() else {
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
                continue;
            }

            let Some(textarea_state) = self.state.textareas.get(field.id()) else {
                y += 4;
                continue;
            };
            let height = textarea_state.height();
            if y >= inner.y as i32 && y < max_y {
                let textarea_width = inner.width.saturating_sub(label_width + 4);
                let textarea_area = Rect::new(field_start, y as u16, textarea_width, height);
                TextareaWidget::new(textarea_state, self.state.theme.clone())
                    .focused(is_focused)
                    .render(textarea_area, buf);
                if let Some(error_msg) = self.error(field) {
                    let error_style = Style::default().fg(self.state.theme.colors.error);
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

        if let Some((dropdown, dropdown_area)) = open_dropdown {
            dropdown.render(dropdown_area, buf);
        }
        self.state.scroll.render_scrollbar(inner, buf);
        let help_y = inner.y + inner.height - 1;
        buf.set_string(
            inner.x + 1,
            help_y,
            "Tab: Next | Shift+Tab: Prev | Ctrl+S: Submit | Esc: Cancel",
            Style::default().fg(self.state.theme.colors.disabled),
        );
    }
}

fn build_dropdowns(theme: Theme) -> HashMap<String, DropdownWidget> {
    let mut dropdowns = HashMap::new();
    dropdowns.insert(
        FIELD_STATUS.to_string(),
        DropdownWidget::new(
            "Status *",
            vec![
                DropdownOption::new("Active", "Active"),
                DropdownOption::new("Resolved", "Resolved"),
                DropdownOption::new("Chronic", "Chronic"),
                DropdownOption::new("Recurring", "Recurring"),
                DropdownOption::new("InRemission", "InRemission"),
            ],
            theme.clone(),
        ),
    );
    dropdowns.insert(
        FIELD_SEVERITY.to_string(),
        DropdownWidget::new(
            "Severity",
            vec![
                DropdownOption::new("Mild", "Mild"),
                DropdownOption::new("Moderate", "Moderate"),
                DropdownOption::new("Severe", "Severe"),
            ],
            theme,
        ),
    );
    dropdowns
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

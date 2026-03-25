//! Allergy Form Component
//!
//! Form for creating or editing a patient allergy.

use std::collections::HashMap;

use chrono::NaiveDate;
use crossterm::event::{KeyEvent, KeyModifiers};
use opengp_config::forms::ValidationRules;
use opengp_domain::domain::clinical::{Allergy, AllergyType, Severity};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Borders, Widget};
use uuid::Uuid;

use crate::ui::input::to_ratatui_key;
use crate::ui::layout::LABEL_WIDTH;
use crate::ui::theme::Theme;
use crate::ui::widgets::{
    format_date, parse_date, DatePickerAction, DatePickerPopup, DropdownAction, DropdownOption,
    DropdownWidget, DynamicForm, DynamicFormMeta, FormFieldMeta, FormNavigation, FormValidator,
    HeightMode, ScrollableFormState, TextareaState, TextareaWidget,
};

const FIELD_ALLERGEN: &str = "allergen";
const FIELD_ALLERGY_TYPE: &str = "allergy_type";
const FIELD_SEVERITY: &str = "severity";
const FIELD_REACTION: &str = "reaction";
const FIELD_ONSET_DATE: &str = "onset_date";
const FIELD_NOTES: &str = "notes";

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
        (*self).into()
    }

    pub fn id(&self) -> &'static str {
        match self {
            AllergyFormField::Allergen => FIELD_ALLERGEN,
            AllergyFormField::AllergyType => FIELD_ALLERGY_TYPE,
            AllergyFormField::Severity => FIELD_SEVERITY,
            AllergyFormField::Reaction => FIELD_REACTION,
            AllergyFormField::OnsetDate => FIELD_ONSET_DATE,
            AllergyFormField::Notes => FIELD_NOTES,
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            FIELD_ALLERGEN => Some(AllergyFormField::Allergen),
            FIELD_ALLERGY_TYPE => Some(AllergyFormField::AllergyType),
            FIELD_SEVERITY => Some(AllergyFormField::Severity),
            FIELD_REACTION => Some(AllergyFormField::Reaction),
            FIELD_ONSET_DATE => Some(AllergyFormField::OnsetDate),
            FIELD_NOTES => Some(AllergyFormField::Notes),
            _ => None,
        }
    }

    pub fn is_required(&self) -> bool {
        matches!(
            self,
            AllergyFormField::Allergen | AllergyFormField::AllergyType | AllergyFormField::Severity
        )
    }

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
    errors: HashMap<String, String>,
    focused_field: String,
    field_ids: Vec<String>,
    textareas: HashMap<String, TextareaState>,
    dropdowns: HashMap<String, DropdownWidget>,
    allergy_type: Option<AllergyType>,
    severity: Option<Severity>,
    onset_date: Option<NaiveDate>,
    is_valid: bool,
    validator: FormValidator,
    theme: Theme,
    scroll: ScrollableFormState,
    date_picker: DatePickerPopup,
}

impl Clone for AllergyForm {
    fn clone(&self) -> Self {
        Self {
            mode: self.mode,
            errors: self.errors.clone(),
            focused_field: self.focused_field.clone(),
            field_ids: self.field_ids.clone(),
            textareas: self.textareas.clone(),
            dropdowns: self.dropdowns.clone(),
            allergy_type: self.allergy_type,
            severity: self.severity,
            onset_date: self.onset_date,
            is_valid: self.is_valid,
            validator: build_validator(),
            theme: self.theme.clone(),
            scroll: self.scroll.clone(),
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

        let mut form = Self {
            mode: FormMode::Create,
            errors: HashMap::new(),
            focused_field: FIELD_ALLERGEN.to_string(),
            field_ids: AllergyFormField::all()
                .into_iter()
                .map(|field| field.id().to_string())
                .collect(),
            textareas: HashMap::new(),
            dropdowns: HashMap::new(),
            allergy_type: None,
            severity: None,
            onset_date: None,
            is_valid: false,
            validator: FormValidator::new(&HashMap::new()),
            theme: theme.clone(),
            scroll: ScrollableFormState::new(),
            date_picker: DatePickerPopup::new(),
        };

        form.textareas.insert(
            FIELD_ALLERGEN.to_string(),
            TextareaState::new("Allergen *").with_height_mode(HeightMode::SingleLine),
        );
        form.textareas.insert(
            FIELD_REACTION.to_string(),
            TextareaState::new("Reaction").with_height_mode(HeightMode::SingleLine),
        );
        form.textareas.insert(
            FIELD_NOTES.to_string(),
            TextareaState::new("Notes").with_height_mode(HeightMode::FixedLines(3)),
        );

        form.dropdowns.insert(
            FIELD_ALLERGY_TYPE.to_string(),
            DropdownWidget::new("Allergy Type *", allergy_type_options, theme.clone()),
        );
        form.dropdowns.insert(
            FIELD_SEVERITY.to_string(),
            DropdownWidget::new("Severity *", severity_options, theme),
        );

        form.validator = build_validator();
        form
    }

    pub fn from_allergy(allergy: Allergy, theme: Theme) -> Self {
        let mut form = Self::new(theme);
        form.mode = FormMode::Edit(allergy.id);

        form.set_value(AllergyFormField::Allergen, allergy.allergen);
        form.set_value(
            AllergyFormField::AllergyType,
            allergy.allergy_type.to_string(),
        );
        form.set_value(AllergyFormField::Severity, allergy.severity.to_string());

        if let Some(reaction) = allergy.reaction {
            form.set_value(AllergyFormField::Reaction, reaction);
        }

        form.onset_date = allergy.onset_date;

        if let Some(notes) = allergy.notes {
            form.set_value(AllergyFormField::Notes, notes);
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
        AllergyFormField::from_id(&self.focused_field).unwrap_or(AllergyFormField::Allergen)
    }

    pub fn get_value(&self, field: AllergyFormField) -> String {
        self.get_value_by_id(field.id())
    }

    pub fn set_value(&mut self, field: AllergyFormField, value: String) {
        self.set_value_by_id(field.id(), value)
    }

    fn get_value_by_id(&self, field_id: &str) -> String {
        if let Some(textarea) = self.textareas.get(field_id) {
            return textarea.value();
        }

        if let Some(dropdown) = self.dropdowns.get(field_id) {
            return dropdown.selected_value().unwrap_or("").to_string();
        }

        if field_id == FIELD_ONSET_DATE {
            return self.onset_date.map(format_date).unwrap_or_default();
        }

        String::new()
    }

    fn set_value_by_id(&mut self, field_id: &str, value: String) {
        if let Some(textarea) = self.textareas.get_mut(field_id) {
            let label = textarea.label.clone();
            let height_mode = textarea.height_mode.clone();
            let max_length = textarea.max_length;
            let focused = textarea.focused;

            let mut updated = TextareaState::new(label)
                .with_height_mode(height_mode)
                .with_value(value.clone())
                .focused(focused);
            if let Some(limit) = max_length {
                updated = updated.max_length(limit);
            }
            *textarea = updated;
        } else if let Some(dropdown) = self.dropdowns.get_mut(field_id) {
            dropdown.set_value(&value);
        } else if field_id == FIELD_ONSET_DATE {
            let parsed = if value.trim().is_empty() {
                None
            } else {
                parse_date(&value)
            };
            self.onset_date = parsed;
            if !value.trim().is_empty() && parsed.is_none() {
                self.set_error_by_id(FIELD_ONSET_DATE, Some("Use dd/mm/yyyy format".to_string()));
                self.is_valid = false;
                return;
            }
        }

        self.sync_domain_enum_fields(field_id, &value);
        self.validate_field_by_id(field_id);
    }

    fn sync_domain_enum_fields(&mut self, field_id: &str, value: &str) {
        match field_id {
            FIELD_ALLERGY_TYPE => self.allergy_type = value.parse::<AllergyType>().ok(),
            FIELD_SEVERITY => self.severity = value.parse::<Severity>().ok(),
            _ => {}
        }
    }

    fn validate_field_by_id(&mut self, field_id: &str) {
        self.errors.remove(field_id);

        let value = self.get_value_by_id(field_id);
        let mut errors = self.validator.validate(field_id, &value);

        if field_id == FIELD_ALLERGEN
            && errors.iter().any(|error| error == "This field is required")
        {
            errors = vec!["Allergen is required".to_string()];
        }

        if field_id == FIELD_ALLERGY_TYPE
            && errors.iter().any(|error| error == "This field is required")
        {
            errors = vec!["Allergy type is required (Drug/Food/Environmental/Other)".to_string()];
        }

        if field_id == FIELD_SEVERITY
            && errors.iter().any(|error| error == "This field is required")
        {
            errors = vec!["Severity is required (Mild/Moderate/Severe)".to_string()];
        }

        if field_id == FIELD_ONSET_DATE && !value.trim().is_empty() && parse_date(&value).is_none()
        {
            errors = vec!["Use dd/mm/yyyy format".to_string()];
        }

        let error_msg = errors.into_iter().next();
        self.set_error_by_id(field_id, error_msg.clone());
        if let Some(textarea) = self.textareas.get_mut(field_id) {
            textarea.set_error(error_msg);
        }

        self.is_valid = self.errors.is_empty();
    }

    fn set_error_by_id(&mut self, field_id: &str, error: Option<String>) {
        match error {
            Some(msg) => {
                self.errors.insert(field_id.to_string(), msg);
            }
            None => {
                self.errors.remove(field_id);
            }
        }
    }

    fn focused_textarea_mut(&mut self) -> Option<&mut TextareaState> {
        self.textareas.get_mut(&self.focused_field)
    }

    fn textarea_for(&self, field: AllergyFormField) -> Option<&TextareaState> {
        self.textareas.get(field.id())
    }

    fn dropdown_for(&self, field: AllergyFormField) -> Option<&DropdownWidget> {
        self.dropdowns.get(field.id())
    }

    fn handle_dropdown_key(&mut self, key: KeyEvent) -> Option<Option<AllergyFormAction>> {
        let field_id = self.focused_field.clone();
        if !self.dropdowns.contains_key(&field_id) {
            return None;
        }

        let mut selected_value: Option<String> = None;
        let action = {
            let dropdown = self.dropdowns.get_mut(&field_id)?;
            dropdown.handle_key(key)
        };

        if let Some(action) = action {
            match key.code {
                crossterm::event::KeyCode::Tab
                | crossterm::event::KeyCode::BackTab
                | crossterm::event::KeyCode::Esc => return None,
                _ => match action {
                    DropdownAction::Selected(_) | DropdownAction::Closed => {
                        selected_value = self
                            .dropdowns
                            .get(&field_id)
                            .and_then(|dropdown| dropdown.selected_value().map(|v| v.to_string()));
                    }
                    DropdownAction::Opened | DropdownAction::FocusChanged => {
                        return Some(Some(AllergyFormAction::ValueChanged));
                    }
                },
            }
        } else {
            match key.code {
                crossterm::event::KeyCode::Tab
                | crossterm::event::KeyCode::BackTab
                | crossterm::event::KeyCode::Esc => return None,
                _ => return Some(None),
            }
        }

        if let Some(value) = selected_value {
            self.set_value_by_id(&field_id, value);
        }

        Some(Some(AllergyFormAction::ValueChanged))
    }

    pub fn error(&self, field: AllergyFormField) -> Option<&String> {
        self.errors.get(field.id())
    }

    pub fn validate(&mut self) -> bool {
        FormNavigation::validate(self)
    }

    pub fn next_field(&mut self) {
        FormNavigation::next_field(self);
    }

    pub fn prev_field(&mut self) {
        FormNavigation::prev_field(self);
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Option<AllergyFormAction> {
        use crossterm::event::{KeyCode, KeyEventKind};

        if key.kind != KeyEventKind::Press {
            return None;
        }

        if key.modifiers.contains(KeyModifiers::CONTROL) && matches!(key.code, KeyCode::Char('s')) {
            FormNavigation::validate(self);
            return Some(AllergyFormAction::Submit);
        }

        if key.code == KeyCode::Esc {
            return Some(AllergyFormAction::Cancel);
        }

        if self.date_picker.is_visible() {
            if let Some(action) = self.date_picker.handle_key(key) {
                match action {
                    DatePickerAction::Selected(date) => {
                        self.set_value_by_id(FIELD_ONSET_DATE, format_date(date));
                        return Some(AllergyFormAction::ValueChanged);
                    }
                    DatePickerAction::Dismissed => {
                        return Some(AllergyFormAction::FocusChanged);
                    }
                }
            }
            return Some(AllergyFormAction::FocusChanged);
        }

        if self.focused_field == FIELD_ONSET_DATE
            && matches!(key.code, KeyCode::Enter | KeyCode::Char(' '))
        {
            self.date_picker.open(self.onset_date);
            return Some(AllergyFormAction::FocusChanged);
        }

        if let Some(dropdown_action) = self.handle_dropdown_key(key) {
            return dropdown_action;
        }

        if !self.dropdowns.contains_key(&self.focused_field) {
            let ratatui_key = to_ratatui_key(key);
            if let Some(textarea) = self.focused_textarea_mut() {
                let consumed = textarea.handle_key(ratatui_key);
                if consumed {
                    let field_id = self.focused_field.clone();
                    self.validate_field_by_id(&field_id);
                    return Some(AllergyFormAction::ValueChanged);
                }
            }
        }

        match key.code {
            KeyCode::Tab => {
                if key.modifiers.contains(KeyModifiers::SHIFT) {
                    FormNavigation::prev_field(self);
                } else {
                    FormNavigation::next_field(self);
                }
                Some(AllergyFormAction::FocusChanged)
            }
            KeyCode::BackTab => {
                FormNavigation::prev_field(self);
                Some(AllergyFormAction::FocusChanged)
            }
            KeyCode::Up => {
                FormNavigation::prev_field(self);
                Some(AllergyFormAction::FocusChanged)
            }
            KeyCode::Down => {
                FormNavigation::next_field(self);
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
            allergen: self.get_value(AllergyFormField::Allergen),
            allergy_type: self.allergy_type.unwrap_or(AllergyType::Other),
            severity: self.severity.unwrap_or(Severity::Moderate),
            reaction: Some(self.get_value(AllergyFormField::Reaction)).filter(|s| !s.is_empty()),
            onset_date: self.onset_date,
            notes: Some(self.get_value(AllergyFormField::Notes)).filter(|s| !s.is_empty()),
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

impl DynamicFormMeta for AllergyForm {
    fn label(&self, field_id: &str) -> String {
        AllergyFormField::from_id(field_id)
            .map(|field| field.label().to_string())
            .unwrap_or_else(|| field_id.to_string())
    }

    fn is_required(&self, field_id: &str) -> bool {
        AllergyFormField::from_id(field_id)
            .map(|field| field.is_required())
            .unwrap_or(false)
    }

    fn field_type(&self, field_id: &str) -> crate::ui::widgets::FieldType {
        match AllergyFormField::from_id(field_id) {
            Some(AllergyFormField::AllergyType | AllergyFormField::Severity) => {
                crate::ui::widgets::FieldType::Select(vec![])
            }
            Some(AllergyFormField::OnsetDate) => crate::ui::widgets::FieldType::Date,
            _ => crate::ui::widgets::FieldType::Text,
        }
    }
}

impl DynamicForm for AllergyForm {
    fn field_ids(&self) -> &[String] {
        &self.field_ids
    }

    fn current_field(&self) -> &str {
        &self.focused_field
    }

    fn set_current_field(&mut self, field_id: &str) {
        if self.field_ids.iter().any(|id| id == field_id) {
            self.focused_field = field_id.to_string();
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
        self.is_valid = self.errors.is_empty();
        self.is_valid
    }

    fn get_error(&self, field_id: &str) -> Option<&str> {
        self.errors.get(field_id).map(|s| s.as_str())
    }

    fn set_error(&mut self, field_id: &str, error: Option<String>) {
        self.set_error_by_id(field_id, error);
    }
}

impl FormNavigation for AllergyForm {
    type FormField = AllergyFormField;

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
        self.focused_field()
    }

    fn fields(&self) -> Vec<Self::FormField> {
        self.field_ids
            .iter()
            .filter_map(|field_id| AllergyFormField::from_id(field_id))
            .collect()
    }

    fn set_current_field(&mut self, field: Self::FormField) {
        self.focused_field = field.id().to_string();
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
                total_height += self
                    .textarea_for(*field)
                    .map(|state| state.height())
                    .unwrap_or(2);
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
                self.textarea_for(field)
                    .map(|state| state.height())
                    .unwrap_or(2) as i32
            } else if field.is_dropdown() {
                4
            } else {
                2
            };

            if y + field_height <= inner.y as i32 || y >= max_y {
                y += field_height;
                continue;
            }

            let is_focused = field.id() == self.focused_field;

            if field.is_textarea() {
                let Some(textarea_state) = self.textarea_for(field) else {
                    y += field_height;
                    continue;
                };

                let textarea_height = textarea_state.height();
                if y >= inner.y as i32 && y < max_y {
                    let field_area =
                        Rect::new(inner.x + 1, y as u16, inner.width - 2, textarea_height);
                    TextareaWidget::new(textarea_state, self.theme.clone())
                        .focused(is_focused)
                        .render(field_area, buf);

                    if let Some(error_msg) = self.error(field) {
                        if (y as u16) + textarea_height <= inner.y + inner.height - 2 {
                            let error_style = Style::default().fg(self.theme.colors.error);
                            buf.set_string(
                                inner.x + 1,
                                (y as u16) + textarea_height,
                                format!("  {}", error_msg),
                                error_style,
                            );
                        }
                    }
                }
                y += textarea_height as i32;
                continue;
            }

            let has_error = self.error(field).is_some();

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

            let max_value_width = inner.width.saturating_sub(label_width + 4);

            match field {
                AllergyFormField::AllergyType | AllergyFormField::Severity => {
                    let Some(dropdown) = self.dropdown_for(field).cloned() else {
                        y += 4;
                        continue;
                    };

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

fn build_validator() -> FormValidator {
    let mut rules: HashMap<String, ValidationRules> = HashMap::new();
    rules.insert(
        FIELD_ALLERGEN.to_string(),
        ValidationRules {
            required: true,
            ..ValidationRules::default()
        },
    );
    rules.insert(
        FIELD_ALLERGY_TYPE.to_string(),
        ValidationRules {
            required: true,
            ..ValidationRules::default()
        },
    );
    rules.insert(
        FIELD_SEVERITY.to_string(),
        ValidationRules {
            required: true,
            ..ValidationRules::default()
        },
    );
    rules.insert(
        FIELD_ONSET_DATE.to_string(),
        ValidationRules {
            date_format: Some("dd/mm/yyyy".to_string()),
            ..ValidationRules::default()
        },
    );

    FormValidator::new(&rules)
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

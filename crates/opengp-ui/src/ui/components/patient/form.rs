use std::collections::HashMap;

use crossterm::event::{KeyCode, KeyEvent, MouseEvent, MouseEventKind};
use opengp_config::forms::{FieldDefinition, FieldType as ConfigFieldType};
use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};
use ratatui::style::Style;
use ratatui::widgets::{Block, Borders, Widget};
use uuid::Uuid;

use crate::ui::input::to_ratatui_key;
use crate::ui::layout::LABEL_WIDTH;
use crate::ui::theme::Theme;
use crate::ui::view_models::PatientFormData;
use crate::ui::widgets::{
    format_date, parse_date, DatePickerAction, DatePickerPopup, DropdownAction, DropdownWidget,
    DynamicForm, DynamicFormMeta, FormNavigation, FormState, FormValidator, TextareaState,
    TextareaWidget,
};

#[path = "patient_form_defs.rs"]
mod defs;
use defs::*;
#[path = "patient_form_data_ops.rs"]
mod data_ops;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FormMode {
    #[default]
    Create,
    Edit(Uuid),
}

pub use defs::PatientFormField;

pub type FormField = PatientFormField;

pub struct PatientForm {
    mode: FormMode,
    data: PatientFormData,
    field_ids: Vec<String>,
    field_configs: HashMap<String, FieldDefinition>,
    saving: bool,
    validator: FormValidator,
    date_picker: DatePickerPopup,
    state: FormState<PatientFormField>,
}

impl Clone for PatientForm {
    fn clone(&self) -> Self {
        Self {
            mode: self.mode,
            data: self.data.clone(),
            field_ids: self.field_ids.clone(),
            field_configs: self.field_configs.clone(),
            saving: self.saving,
            validator: build_validator(&self.field_configs),
            date_picker: self.date_picker.clone(),
            state: self.state.clone(),
        }
    }
}

impl FormNavigation for PatientForm {
    type FormField = PatientFormField;

    fn get_error(&self, field: Self::FormField) -> Option<&str> {
        self.state.errors.get(field.id()).map(String::as_str)
    }

    fn set_error(&mut self, field: Self::FormField, error: Option<String>) {
        self.set_error_by_id(field.id(), error);
    }

    fn validate(&mut self) -> bool {
        DynamicForm::validate(self)
    }

    fn current_field(&self) -> Self::FormField {
        self.state.focused_field
    }

    fn fields(&self) -> Vec<Self::FormField> {
        self.state.field_order.clone()
    }

    fn set_current_field(&mut self, field: Self::FormField) {
        self.state.focused_field = field;
    }
}

impl DynamicFormMeta for PatientForm {
    fn label(&self, field_id: &str) -> String {
        self.field_configs
            .get(field_id)
            .map(|field| field.label.clone())
            .unwrap_or_else(|| field_id.to_string())
    }

    fn is_required(&self, field_id: &str) -> bool {
        self.field_configs
            .get(field_id)
            .map(|field| field.required)
            .unwrap_or(false)
    }

    fn field_type(&self, field_id: &str) -> crate::ui::widgets::FieldType {
        match self
            .field_configs
            .get(field_id)
            .map(|field| &field.field_type)
        {
            Some(ConfigFieldType::Date) => crate::ui::widgets::FieldType::Date,
            Some(ConfigFieldType::Select) => crate::ui::widgets::FieldType::Select(vec![]),
            _ => crate::ui::widgets::FieldType::Text,
        }
    }
}

impl DynamicForm for PatientForm {
    fn field_ids(&self) -> &[String] {
        &self.field_ids
    }

    fn current_field(&self) -> &str {
        self.state.focused_field.id()
    }

    fn set_current_field(&mut self, field_id: &str) {
        if self.field_ids.iter().any(|id| id == field_id) {
            if let Some(field) = PatientFormField::from_id(field_id) {
                self.state.focused_field = field;
            }
        }
    }

    fn get_value(&self, field_id: &str) -> String {
        self.get_value_by_id(field_id)
    }

    fn set_value(&mut self, field_id: &str, value: String) {
        self.set_value_by_id(field_id, value)
    }

    fn validate(&mut self) -> bool {
        self.state.errors.clear();
        for field_id in self.field_ids.clone() {
            self.validate_field_by_id(&field_id);
        }
        self.state.errors.is_empty()
    }

    fn get_error(&self, field_id: &str) -> Option<&str> {
        self.state.errors.get(field_id).map(String::as_str)
    }

    fn set_error(&mut self, field_id: &str, error: Option<String>) {
        self.set_error_by_id(field_id, error);
    }
}

impl PatientForm {
    pub fn new(theme: Theme) -> Self {
        let field_definitions = load_patient_field_definitions();
        let field_ids: Vec<String> = field_definitions
            .iter()
            .filter(|field| field.visible && field.navigable)
            .map(|field| field.id.clone())
            .collect();
        let field_configs: HashMap<String, FieldDefinition> = field_definitions
            .into_iter()
            .map(|field| (field.id.clone(), field))
            .collect();

        let default_focus = if field_ids.iter().any(|id| id == FIELD_FIRST_NAME) {
            PatientFormField::FirstName
        } else {
            field_ids
                .iter()
                .find_map(|id| PatientFormField::from_id(id))
                .unwrap_or(PatientFormField::FirstName)
        };
        let mut state = FormState::new(theme, default_focus);

        state.field_order = field_ids
            .iter()
            .filter_map(|id| PatientFormField::from_id(id))
            .collect();

        for field_id in &field_ids {
            if let Some(field) = field_configs.get(field_id) {
                match field.field_type {
                    ConfigFieldType::Select => {
                        state.dropdowns.insert(
                            field.id.clone(),
                            DropdownWidget::new(
                                field.label.as_str(),
                                dropdown_options(field),
                                state.theme.clone(),
                            ),
                        );
                    }
                    _ => {
                        state
                            .textareas
                            .insert(field.id.clone(), make_textarea_state(field, None));
                    }
                }
            }
        }

        let validator = build_validator(&field_configs);
        Self {
            mode: FormMode::Create,
            data: PatientFormData::empty(),
            field_ids,
            field_configs,
            saving: false,
            validator,
            date_picker: DatePickerPopup::new(),
            state,
        }
    }

    pub fn is_edit_mode(&self) -> bool {
        matches!(self.mode, FormMode::Edit(_))
    }

    pub fn patient_id(&self) -> Option<Uuid> {
        match self.mode {
            FormMode::Edit(id) => Some(id),
            FormMode::Create => None,
        }
    }

    pub fn get_value(&self, field: FormField) -> String {
        self.get_value_by_id(field.id())
    }

    pub fn set_value(&mut self, field: FormField, value: String) {
        self.set_value_by_id(field.id(), value);
    }

    fn get_value_by_id(&self, field_id: &str) -> String {
        self.state.get_value_by_id(field_id)
    }

    fn set_value_by_id(&mut self, field_id: &str, value: String) {
        self.state.set_value_by_id(field_id, value.clone());
        self.sync_data_for_field(field_id, &value);
        self.validate_field_by_id(field_id);
    }

    fn sync_data_for_field(&mut self, field_id: &str, value: &str) {
        match field_id {
            FIELD_GENDER => {
                if let Ok(gender) = value.parse() {
                    self.data.gender = gender;
                }
            }
            FIELD_CONCESSION_TYPE => {
                self.data.concession_type = value.parse().ok();
            }
            FIELD_INTERPRETER_REQUIRED => {
                self.data.interpreter_required = value == "Yes";
            }
            FIELD_ATSI_STATUS => {
                self.data.aboriginal_torres_strait_islander = value.parse().ok();
            }
            _ => {}
        }
    }

    fn focused_field_id(&self) -> &'static str {
        self.state.focused_field.id()
    }

    fn focused_textarea_mut(&mut self) -> Option<&mut TextareaState> {
        self.state.textareas.get_mut(self.focused_field_id())
    }

    fn textarea_for(&self, field_id: &str) -> Option<&TextareaState> {
        self.state.textareas.get(field_id)
    }

    pub fn focused_field(&self) -> FormField {
        self.state.focused_field
    }

    pub fn set_focus(&mut self, field: FormField) {
        self.state.focused_field = field;
    }

    fn get_field_position(&self, field_id: &str) -> (u16, u16) {
        let mut y: u16 = 0;
        for id in &self.field_ids {
            if id == field_id {
                return (y, self.get_field_height(id));
            }
            y += self.get_field_height(id) + 1;
        }
        (0, 0)
    }

    fn get_field_height(&self, field_id: &str) -> u16 {
        if self.state.dropdowns.contains_key(field_id) {
            4
        } else if let Some(textarea) = self.textarea_for(field_id) {
            textarea.height()
        } else {
            1
        }
    }

    pub fn is_saving(&self) -> bool {
        self.saving
    }

    pub fn set_saving(&mut self, saving: bool) {
        self.saving = saving;
    }

    fn validate_field_by_id(&mut self, field_id: &str) {
        self.state.errors.remove(field_id);

        let value = self.get_value_by_id(field_id);
        let mut errors = self.validator.validate(field_id, &value);

        if field_id == FIELD_MEDICARE_NUMBER && !value.is_empty() {
            if value.len() != 10 {
                errors = vec!["Medicare number must be 10 digits".to_string()];
            } else if !value.chars().all(|c| c.is_ascii_digit()) {
                errors = vec!["Medicare number must contain only digits".to_string()];
            }
        }

        if matches!(field_id, FIELD_DATE_OF_BIRTH | FIELD_MEDICARE_EXPIRY)
            && !value.trim().is_empty()
            && parse_date(&value).is_none()
        {
            errors = vec!["Use dd/mm/yyyy format".to_string()];
        }

        let error_msg = errors.into_iter().next();
        self.set_error_by_id(field_id, error_msg.clone());
        if let Some(textarea) = self.state.textareas.get_mut(field_id) {
            textarea.set_error(error_msg);
        }
    }

    fn set_error_by_id(&mut self, field_id: &str, error: Option<String>) {
        match error {
            Some(msg) => {
                self.state.errors.insert(field_id.to_string(), msg);
            }
            None => {
                self.state.errors.remove(field_id);
            }
        }
    }

    pub fn error(&self, field: FormField) -> Option<&String> {
        self.state.errors.get(field.id())
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Option<PatientFormAction> {
        use crossterm::event::{KeyEventKind, KeyModifiers};

        if key.kind != KeyEventKind::Press || self.saving {
            return None;
        }

        if key.modifiers.contains(KeyModifiers::CONTROL) && matches!(key.code, KeyCode::Char('s')) {
            FormNavigation::validate(self);
            return Some(PatientFormAction::Submit);
        }

        if self.date_picker.is_visible() {
            if let Some(action) = self.date_picker.handle_key(key) {
                match action {
                    DatePickerAction::Selected(date) => {
                        self.set_value_by_id(FIELD_DATE_OF_BIRTH, format_date(date));
                        return Some(PatientFormAction::ValueChanged);
                    }
                    DatePickerAction::Dismissed => return Some(PatientFormAction::FocusChanged),
                }
            }
            return Some(PatientFormAction::FocusChanged);
        }

        if self.focused_field() == FormField::DateOfBirth
            && matches!(key.code, KeyCode::Enter | KeyCode::Char(' '))
        {
            let current_value = parse_date(&self.get_value_by_id(FIELD_DATE_OF_BIRTH));
            self.date_picker.open(current_value);
            return Some(PatientFormAction::FocusChanged);
        }

        if let Some(dropdown_action) = self.handle_dropdown_key(key) {
            return dropdown_action;
        }

        if !self.state.dropdowns.contains_key(self.focused_field_id()) {
            let ratatui_key = to_ratatui_key(key);
            if let Some(textarea) = self.focused_textarea_mut() {
                if textarea.handle_key(ratatui_key) {
                    let field = self.focused_field_id().to_string();
                    self.validate_field_by_id(&field);
                    return Some(PatientFormAction::ValueChanged);
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
                Some(PatientFormAction::FocusChanged)
            }
            KeyCode::BackTab | KeyCode::Up => {
                FormNavigation::prev_field(self);
                Some(PatientFormAction::FocusChanged)
            }
            KeyCode::Down => {
                FormNavigation::next_field(self);
                Some(PatientFormAction::FocusChanged)
            }
            KeyCode::PageUp => {
                self.state.scroll.scroll_up();
                Some(PatientFormAction::ValueChanged)
            }
            KeyCode::PageDown => {
                self.state.scroll.scroll_down();
                Some(PatientFormAction::ValueChanged)
            }
            KeyCode::Esc => Some(PatientFormAction::Cancel),
            _ => None,
        }
    }

    fn handle_dropdown_key(&mut self, key: KeyEvent) -> Option<Option<PatientFormAction>> {
        let field_id = self.focused_field_id().to_string();
        if !self.state.dropdowns.contains_key(&field_id) {
            return None;
        }

        let action = {
            let dropdown = self.state.dropdowns.get_mut(&field_id)?;
            dropdown.handle_key(key)
        };

        if let Some(action) = action {
            match key.code {
                KeyCode::Tab | KeyCode::BackTab | KeyCode::Esc => return None,
                _ => match action {
                    DropdownAction::Selected(_) | DropdownAction::Closed => {
                        let selected_value =
                            self.state.dropdowns.get(&field_id).and_then(|dropdown| {
                                dropdown.selected_value().map(ToString::to_string)
                            });
                        if let Some(value) = selected_value {
                            self.set_value_by_id(&field_id, value);
                        }
                    }
                    DropdownAction::Opened | DropdownAction::FocusChanged => {
                        return Some(Some(PatientFormAction::ValueChanged));
                    }
                },
            }
        } else {
            match key.code {
                KeyCode::Tab | KeyCode::BackTab | KeyCode::Esc => return None,
                _ => return Some(None),
            }
        }

        Some(Some(PatientFormAction::ValueChanged))
    }

    pub fn handle_mouse(&mut self, mouse: MouseEvent, area: Rect) -> Option<PatientFormAction> {
        if mouse.kind != MouseEventKind::Up(crossterm::event::MouseButton::Left)
            || !area.contains(Position::new(mouse.column, mouse.row))
        {
            return None;
        }

        let click_pos = Position::new(mouse.column, mouse.row);
        let inner = Rect::new(area.x + 1, area.y + 1, area.width - 2, area.height - 2);
        if !inner.contains(click_pos) {
            return None;
        }

        let mut y = inner.y + 1;
        let max_y = inner.y + inner.height - 2;
        for field_id in &self.field_ids {
            if y > max_y {
                break;
            }

            let field_height = self.get_field_height(field_id);
            let field_area = Rect::new(inner.x + 1, y, inner.width - 2, field_height);
            if field_area.contains(click_pos) {
                if let Some(field) = PatientFormField::from_id(field_id) {
                    if field != self.state.focused_field {
                        self.state.focused_field = field;
                        return Some(PatientFormAction::FocusChanged);
                    }
                }
            }

            y += field_height + 1;
        }

        None
    }
}

impl Widget for PatientForm {
    fn render(mut self, area: Rect, buf: &mut Buffer) {
        if area.is_empty() {
            return;
        }

        let block = Block::default()
            .title(if self.is_edit_mode() {
                " Edit Patient "
            } else {
                " New Patient "
            })
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.state.theme.colors.border));

        block.clone().render(area, buf);
        let inner = block.inner(area);
        if inner.is_empty() {
            return;
        }

        let label_width = LABEL_WIDTH;
        let field_start = inner.x + label_width + 2;

        let fields = self.field_ids.clone();
        let mut total_height: u16 = 0;
        for field_id in &fields {
            total_height += self.get_field_height(field_id) + 1;
        }
        self.state.scroll.set_total_height(total_height);
        self.state
            .scroll
            .clamp_offset(inner.height.saturating_sub(2));

        let (focused_y, focused_height) = self.get_field_position(self.focused_field_id());
        self.state.scroll.scroll_to_field(
            focused_y,
            focused_height,
            inner.height.saturating_sub(2),
        );

        let mut y: i32 = (inner.y as i32) + 1 - (self.state.scroll.scroll_offset as i32);
        let max_y = inner.y as i32 + inner.height as i32 - 2;
        let mut open_dropdown: Option<(DropdownWidget, Rect)> = None;

        for field_id in fields {
            let field_height = self.get_field_height(&field_id) as i32;
            if y + field_height <= inner.y as i32 || y >= max_y {
                y += field_height + 1;
                continue;
            }

            let is_focused = field_id == self.focused_field_id();
            if let Some(dropdown) = self.state.dropdowns.get(&field_id).cloned() {
                if y >= inner.y as i32 && y < max_y {
                    let dropdown_area = Rect::new(
                        field_start,
                        y as u16,
                        inner.width.saturating_sub(label_width + 4),
                        3,
                    );
                    if dropdown.is_open() {
                        open_dropdown = Some((dropdown.clone(), dropdown_area));
                    }
                    dropdown.focused(is_focused).render(dropdown_area, buf);
                }
                y += 4;
                continue;
            }

            if let Some(textarea) = self.state.textareas.get(&field_id) {
                let textarea_height = textarea.height() as i32;
                if y >= inner.y as i32 && y < max_y {
                    let textarea_area = Rect::new(
                        inner.x + 1,
                        y as u16,
                        inner.width - 2,
                        textarea_height as u16,
                    );
                    TextareaWidget::new(textarea, self.state.theme.clone())
                        .focused(is_focused)
                        .render(textarea_area, buf);
                }
                y += textarea_height + 1;
            }
        }

        if let Some((dropdown, dropdown_area)) = open_dropdown {
            dropdown.render(dropdown_area, buf);
        }

        self.state.scroll.render_scrollbar(inner, buf);
        let help_y = inner.y + inner.height - 1;
        buf.set_string(
            inner.x + 1,
            help_y,
            "Tab: Next | Ctrl+S: Submit | Esc: Cancel",
            Style::default().fg(self.state.theme.colors.disabled),
        );

        if self.date_picker.is_visible() {
            self.date_picker.render(area, buf);
        }
    }
}

trait EmptyToNone {
    fn empty_to_none(self) -> Option<String>;
}

impl EmptyToNone for String {
    fn empty_to_none(self) -> Option<String> {
        if self.is_empty() {
            None
        } else {
            Some(self)
        }
    }
}

fn or_default(s: String, default: &str) -> String {
    if s.is_empty() {
        default.to_string()
    } else {
        s
    }
}

#[derive(Debug, Clone)]
pub enum PatientFormAction {
    FocusChanged,
    ValueChanged,
    Submit,
    Cancel,
    SaveComplete,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::widgets::FormField as SharedFormFieldTrait;

    #[test]
    fn form_creation_sets_defaults() {
        let form = PatientForm::new(Theme::dark());
        assert!(!form.is_edit_mode());
        assert_eq!(form.focused_field(), FormField::FirstName);
        assert!(!form.has_errors());
    }

    #[test]
    fn required_field_validation_sets_errors() {
        let mut form = PatientForm::new(Theme::dark());
        FormNavigation::validate(&mut form);
        assert!(form.error(FormField::FirstName).is_some());
        assert!(form.error(FormField::LastName).is_some());
    }

    #[test]
    fn enum_alias_and_trait_contract_remain_usable() {
        fn assert_trait<T: SharedFormFieldTrait>() {}
        assert_trait::<PatientFormField>();

        let mut form = PatientForm::new(Theme::dark());
        form.set_value(FormField::FirstName, "Alice".to_string());
        assert_eq!(form.get_value(FormField::FirstName), "Alice");
        assert_eq!(form.state.focused_field, PatientFormField::FirstName);
    }

    #[test]
    fn to_new_patient_data_valid_and_invalid_paths() {
        let mut form = PatientForm::new(Theme::dark());

        form.set_value(FormField::FirstName, "Alice".to_string());
        form.set_value(FormField::LastName, "Smith".to_string());
        form.set_value(FormField::DateOfBirth, "15/05/1990".to_string());
        form.set_value(FormField::Gender, "Female".to_string());
        form.set_value(FormField::PreferredLanguage, "English".to_string());

        let result = form.to_new_patient_data();
        assert!(result.is_some());

        let mut invalid = PatientForm::new(Theme::dark());
        invalid.set_value(FormField::FirstName, "Alice".to_string());
        invalid.set_value(FormField::LastName, "".to_string());
        assert!(invalid.to_new_patient_data().is_none());
    }
}

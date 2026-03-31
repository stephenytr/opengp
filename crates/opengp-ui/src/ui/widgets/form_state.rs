use std::collections::HashMap;
use std::hash::Hash;

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use crate::shared::{FormAction, FormMode};
use crate::theme::Theme;

use super::{DropdownWidget, ScrollableFormState, TextareaState};

pub trait FormField: Copy + Eq + Hash + strum::IntoEnumIterator + Into<&'static str> {
    fn all() -> Vec<Self>;
    fn label(&self) -> &'static str;
    fn id(&self) -> &'static str;
    fn from_id(id: &str) -> Option<Self>;
    fn is_required(&self) -> bool;
    fn is_textarea(&self) -> bool;
    fn is_dropdown(&self) -> bool;
}

#[derive(Clone)]
pub struct FormState<F: FormField> {
    pub textareas: HashMap<String, TextareaState>,
    pub dropdowns: HashMap<String, DropdownWidget>,
    pub focused_field: F,
    pub field_order: Vec<F>,
    pub errors: HashMap<String, String>,
    pub theme: Theme,
    pub scroll: ScrollableFormState,
    pub mode: FormMode,
}

impl<F: FormField> FormState<F> {
    pub fn new(theme: Theme, focused_field: F) -> Self {
        Self {
            textareas: HashMap::new(),
            dropdowns: HashMap::new(),
            focused_field,
            field_order: F::all(),
            errors: HashMap::new(),
            theme,
            scroll: ScrollableFormState::new(),
            mode: FormMode::Create,
        }
    }

    pub fn get_value(&self, field: F) -> String {
        self.get_value_by_id(field.id())
    }

    pub fn set_value(&mut self, field: F, value: String) {
        self.set_value_by_id(field.id(), value);
    }

    pub fn get_value_by_id(&self, field_id: &str) -> String {
        if let Some(field) = F::from_id(field_id) {
            if field.is_textarea() {
                return self
                    .textareas
                    .get(field_id)
                    .map(TextareaState::value)
                    .unwrap_or_default();
            }

            if field.is_dropdown() {
                return self
                    .dropdowns
                    .get(field_id)
                    .and_then(|dropdown| dropdown.selected_value())
                    .unwrap_or_default()
                    .to_string();
            }
        }

        if let Some(textarea) = self.textareas.get(field_id) {
            return textarea.value();
        }

        if let Some(dropdown) = self.dropdowns.get(field_id) {
            return dropdown.selected_value().unwrap_or_default().to_string();
        }

        String::new()
    }

    pub fn set_value_by_id(&mut self, field_id: &str, value: String) {
        if let Some(field) = F::from_id(field_id) {
            let mut handled = false;
            if field.is_textarea() {
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
                    handled = true;
                }
            } else if field.is_dropdown() {
                if let Some(dropdown) = self.dropdowns.get_mut(field_id) {
                    dropdown.set_value(&value);
                    handled = true;
                }
            }

            if handled {
                let _ = self.validate_field(field);
                return;
            }
        }

        if let Some(textarea) = self.textareas.get_mut(field_id) {
            let label = textarea.label.clone();
            let height_mode = textarea.height_mode.clone();
            let max_length = textarea.max_length;
            let focused = textarea.focused;

            let mut updated = TextareaState::new(label)
                .with_height_mode(height_mode)
                .with_value(value)
                .focused(focused);
            if let Some(limit) = max_length {
                updated = updated.max_length(limit);
            }
            *textarea = updated;
        } else if let Some(dropdown) = self.dropdowns.get_mut(field_id) {
            dropdown.set_value(&value);
        }

        if let Some(field) = F::from_id(field_id) {
            let _ = self.validate_field(field);
        }
    }

    pub fn validate_field(&mut self, field: F) -> bool {
        let field_id = field.id();
        self.errors.remove(field_id);

        let value = self.get_value(field);
        let error = if field.is_required() && value.trim().is_empty() {
            Some(format!("{} is required", field.label()))
        } else {
            None
        };

        if let Some(textarea) = self.textareas.get_mut(field_id) {
            textarea.set_error(error.clone());
        }

        match error {
            Some(message) => {
                self.errors.insert(field_id.to_string(), message);
                false
            }
            None => true,
        }
    }

    pub fn handle_navigation_key(&mut self, key: KeyEvent) -> Option<FormAction> {
        if key.kind != KeyEventKind::Press {
            return None;
        }

        if key.modifiers.contains(KeyModifiers::CONTROL) && matches!(key.code, KeyCode::Char('s')) {
            for field in self.field_order.clone() {
                let _ = self.validate_field(field);
            }
            return Some(FormAction::Submit);
        }

        if key.code == KeyCode::Esc {
            return Some(FormAction::Cancel);
        }

        match key.code {
            KeyCode::Tab => {
                if key.modifiers.contains(KeyModifiers::SHIFT) {
                    self.prev_field();
                } else {
                    self.next_field();
                }
                Some(FormAction::FocusChanged)
            }
            KeyCode::BackTab | KeyCode::Up => {
                self.prev_field();
                Some(FormAction::FocusChanged)
            }
            KeyCode::Down => {
                self.next_field();
                Some(FormAction::FocusChanged)
            }
            _ => None,
        }
    }

    pub fn focused_field(&self) -> F {
        self.focused_field
    }

    pub fn next_field(&mut self) {
        if self.field_order.is_empty() {
            return;
        }

        if let Some(current_idx) = self
            .field_order
            .iter()
            .position(|field| *field == self.focused_field)
        {
            let next_idx = (current_idx + 1) % self.field_order.len();
            self.focused_field = self.field_order[next_idx];
        }
    }

    pub fn prev_field(&mut self) {
        if self.field_order.is_empty() {
            return;
        }

        if let Some(current_idx) = self
            .field_order
            .iter()
            .position(|field| *field == self.focused_field)
        {
            let prev_idx = if current_idx == 0 {
                self.field_order.len() - 1
            } else {
                current_idx - 1
            };
            self.focused_field = self.field_order[prev_idx];
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::ui::widgets::DropdownOption;
    use crossterm::event::{KeyCode, KeyEventKind, KeyModifiers};
    use strum::IntoEnumIterator;

    const FIELD_FIRST_NAME: &str = "first_name";
    const FIELD_STATUS: &str = "status";
    const FIELD_NOTES: &str = "notes";

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, strum::EnumIter, strum::IntoStaticStr)]
    enum TestField {
        #[strum(to_string = "First Name *")]
        FirstName,
        #[strum(to_string = "Status")]
        Status,
        #[strum(to_string = "Notes")]
        Notes,
    }

    impl FormField for TestField {
        fn all() -> Vec<Self> {
            Self::iter().collect()
        }

        fn label(&self) -> &'static str {
            (*self).into()
        }

        fn id(&self) -> &'static str {
            match self {
                TestField::FirstName => FIELD_FIRST_NAME,
                TestField::Status => FIELD_STATUS,
                TestField::Notes => FIELD_NOTES,
            }
        }

        fn from_id(id: &str) -> Option<Self> {
            match id {
                FIELD_FIRST_NAME => Some(TestField::FirstName),
                FIELD_STATUS => Some(TestField::Status),
                FIELD_NOTES => Some(TestField::Notes),
                _ => None,
            }
        }

        fn is_required(&self) -> bool {
            matches!(self, TestField::FirstName)
        }

        fn is_textarea(&self) -> bool {
            matches!(self, TestField::FirstName | TestField::Notes)
        }

        fn is_dropdown(&self) -> bool {
            matches!(self, TestField::Status)
        }
    }

    fn press(code: KeyCode) -> KeyEvent {
        let mut key = KeyEvent::new(code, KeyModifiers::NONE);
        key.kind = KeyEventKind::Press;
        key
    }

    fn build_state(focused_field: TestField) -> FormState<TestField> {
        let theme = Theme::dark();
        let mut state = FormState::new(theme.clone(), focused_field);
        state.textareas.insert(
            FIELD_FIRST_NAME.to_string(),
            TextareaState::new("First Name *"),
        );
        state
            .textareas
            .insert(FIELD_NOTES.to_string(), TextareaState::new("Notes"));
        state.dropdowns.insert(
            FIELD_STATUS.to_string(),
            DropdownWidget::new(
                "Status",
                vec![
                    DropdownOption::new("active", "Active"),
                    DropdownOption::new("inactive", "Inactive"),
                ],
                theme,
            ),
        );
        state
    }

    #[test]
    fn form_state_trait_contract_form_state_form_field_methods_work() {
        let all = TestField::all();
        assert_eq!(
            all,
            vec![TestField::FirstName, TestField::Status, TestField::Notes]
        );
        assert_eq!(TestField::FirstName.label(), "First Name *");
        assert_eq!(TestField::FirstName.id(), FIELD_FIRST_NAME);
        assert_eq!(TestField::from_id(FIELD_STATUS), Some(TestField::Status));
        assert!(TestField::FirstName.is_required());
        assert!(TestField::Notes.is_textarea());
        assert!(TestField::Status.is_dropdown());
    }

    #[test]
    fn form_state_navigation_form_state_tab_moves_focus_forward() {
        let mut state = build_state(TestField::FirstName);
        assert_eq!(state.focused_field(), TestField::FirstName);
        let action = state.handle_navigation_key(press(KeyCode::Tab));
        assert_eq!(action, Some(FormAction::FocusChanged));
        assert_eq!(state.focused_field(), TestField::Status);
    }

    #[test]
    fn form_state_navigation_form_state_shift_tab_moves_focus_back() {
        let mut state = build_state(TestField::Status);
        let action = state.handle_navigation_key(KeyEvent::new(KeyCode::Tab, KeyModifiers::SHIFT));
        assert_eq!(action, Some(FormAction::FocusChanged));
        assert_eq!(state.focused_field(), TestField::FirstName);
    }

    #[test]
    fn form_state_navigation_form_state_up_and_down_move_focus() {
        let mut state = build_state(TestField::Status);

        let down = state.handle_navigation_key(press(KeyCode::Down));
        assert_eq!(down, Some(FormAction::FocusChanged));
        assert_eq!(state.focused_field(), TestField::Notes);

        let up = state.handle_navigation_key(press(KeyCode::Up));
        assert_eq!(up, Some(FormAction::FocusChanged));
        assert_eq!(state.focused_field(), TestField::Status);
    }

    #[test]
    fn form_state_navigation_form_state_esc_cancels() {
        let mut state = build_state(TestField::FirstName);
        let action = state.handle_navigation_key(press(KeyCode::Esc));
        assert_eq!(action, Some(FormAction::Cancel));
    }

    #[test]
    fn form_state_navigation_form_state_ctrl_s_submits_and_validates() {
        let mut state = build_state(TestField::FirstName);
        let action =
            state.handle_navigation_key(KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL));

        assert_eq!(action, Some(FormAction::Submit));
        assert_eq!(
            state.errors.get(FIELD_FIRST_NAME),
            Some(&"First Name * is required".to_string())
        );
    }

    #[test]
    fn form_state_values_form_state_get_set_and_validate_field() {
        let mut state = build_state(TestField::FirstName);

        state.set_value(TestField::FirstName, "Alice".to_string());
        assert_eq!(state.get_value(TestField::FirstName), "Alice");

        assert!(state.validate_field(TestField::FirstName));

        state.set_value(TestField::FirstName, "   ".to_string());
        assert!(!state.validate_field(TestField::FirstName));
        assert_eq!(
            state.errors.get(FIELD_FIRST_NAME),
            Some(&"First Name * is required".to_string())
        );
    }

    #[test]
    fn form_state_navigation_form_state_next_prev_wrap() {
        let mut state = build_state(TestField::FirstName);

        state.next_field();
        assert_eq!(state.focused_field(), TestField::Status);
        state.next_field();
        assert_eq!(state.focused_field(), TestField::Notes);
        state.next_field();
        assert_eq!(state.focused_field(), TestField::FirstName);

        state.prev_field();
        assert_eq!(state.focused_field(), TestField::Notes);
    }
}

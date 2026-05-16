use std::collections::HashMap;
use std::hash::Hash;

use crossterm::event::{Event, KeyEvent, KeyEventKind};
use rat_event::ct_event;

use crate::shared::{FormAction, FormMode};
use crate::theme::Theme;

use super::{DropdownWidget, ScrollableFormState, TextareaState};

/// Describes a logical field in a form that is driven by a [`FormState`].
///
/// Implementors map enum variants to concrete widget configuration such as
/// labels, identifiers, and whether the field is required or uses a textarea
/// or dropdown.
pub trait FormField: Copy + Eq + Hash + strum::IntoEnumIterator + Into<&'static str> {
    /// Returns all fields in the order they should be focused.
    fn all() -> Vec<Self>;

    /// Human friendly label shown next to the field in the UI.
    fn label(&self) -> &'static str;

    /// Stable identifier for this field, used as the key in form state maps.
    fn id(&self) -> &'static str;

    /// Looks up a field by its identifier string.
    fn from_id(id: &str) -> Option<Self>;

    /// Returns true if the field must have a non empty value to be valid.
    fn is_required(&self) -> bool;

    /// Returns true if the field is rendered as a textarea widget.
    fn is_textarea(&self) -> bool;

    /// Returns true if the field is rendered as a dropdown widget.
    fn is_dropdown(&self) -> bool;
}

/// Shared state for forms that are built from [`FormField`] definitions.
///
/// This struct owns the widget state for all fields in a form, tracks focus
/// and validation errors, and translates keyboard input into high level
/// [`FormAction`] values.
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
    /// Creates a new form state for the given theme and initial focused field.
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

    /// Returns the current textual value for the given field.
    pub fn get_value(&self, field: F) -> String {
        self.get_value_by_id(field.id())
    }

    /// Sets the textual value for the given field and revalidates it.
    pub fn set_value(&mut self, field: F, value: String) {
        self.set_value_by_id(field.id(), value);
    }

    /// Returns the current textual value for a field identified by its id.
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

    /// Sets the textual value for the field identified by the given id.
    ///
    /// This updates the underlying widget state and revalidates the field if
    /// it can be resolved from the id.
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

    /// Validates a single field and updates the error map.
    ///
    /// Returns true if the field is valid after validation.
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

    /// Handles a navigation key press and updates focus or triggers actions.
    ///
    /// Returns a [`FormAction`] when the key should be handled by the caller,
    /// such as submit or cancel, and `None` when the key is ignored.
    pub fn handle_navigation_key(&mut self, key: KeyEvent) -> Option<FormAction> {
        if key.kind != KeyEventKind::Press {
            return None;
        }

        let event = Event::Key(key);
        match &event {
            ct_event!(key press CONTROL-'s') => {
                for field in self.field_order.clone() {
                    let _ = self.validate_field(field);
                }
                Some(FormAction::Submit)
            }
            ct_event!(keycode press Esc) => Some(FormAction::Cancel),
            ct_event!(keycode press Tab) => {
                self.next_field();
                Some(FormAction::FocusChanged)
            }
            ct_event!(keycode press SHIFT-Tab) => {
                self.prev_field();
                Some(FormAction::FocusChanged)
            }
            ct_event!(keycode press BackTab) => {
                self.prev_field();
                Some(FormAction::FocusChanged)
            }
            ct_event!(keycode press Up) => {
                self.prev_field();
                Some(FormAction::FocusChanged)
            }
            ct_event!(keycode press Down) => {
                self.next_field();
                Some(FormAction::FocusChanged)
            }
            _ => None,
        }
    }

    /// Returns the field that is currently focused.
    pub fn focused_field(&self) -> F {
        self.focused_field
    }

    /// Moves focus to the next field, wrapping at the end.
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

    /// Moves focus to the previous field, wrapping at the start.
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

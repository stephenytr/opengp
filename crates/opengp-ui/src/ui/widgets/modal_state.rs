use crossterm::event::{Event, KeyEvent, KeyEventKind};
use rat_event::ct_event;
use rat_focus::{FocusBuilder, FocusFlag, HasFocus};
use ratatui::layout::Rect;

use crate::shared::ModalAction;
use crate::theme::Theme;

/// Describes a logical button in a modal dialog that is driven by a [`ModalState`].
///
/// Implementors map enum variants to human-readable labels and define the full
/// set of buttons available in the modal.
pub trait ModalButton: Copy + Eq + PartialEq {
    /// Returns all buttons in the order they should be focused.
    fn all() -> Vec<Self>;

    /// Human friendly label shown on the button in the UI.
    fn label(&self) -> &'static str;
}

/// Shared state for modal dialogs that are built from [`ModalButton`] definitions.
///
/// This struct tracks which button is currently focused and translates keyboard
/// input into high level [`ModalAction`] values.
#[derive(Clone)]
pub struct ModalState<B: ModalButton> {
    pub focused_button: B,
    pub button_order: Vec<B>,
    pub theme: Theme,
    pub focus: FocusFlag,
}

impl<B: ModalButton> ModalState<B> {
    /// Creates a new modal state for the given theme and initial focused button.
    pub fn new(theme: Theme, focused_button: B) -> Self {
        Self {
            focused_button,
            button_order: B::all(),
            theme,
            focus: FocusFlag::default(),
        }
    }

    /// Returns the button that is currently focused.
    pub fn focused_button(&self) -> B {
        self.focused_button
    }

    /// Moves focus to the next button, wrapping at the end.
    pub fn next_button(&mut self) {
        if self.button_order.is_empty() {
            return;
        }

        if let Some(current_idx) = self
            .button_order
            .iter()
            .position(|button| *button == self.focused_button)
        {
            let next_idx = (current_idx + 1) % self.button_order.len();
            self.focused_button = self.button_order[next_idx];
        }
    }

    /// Moves focus to the previous button, wrapping at the start.
    pub fn prev_button(&mut self) {
        if self.button_order.is_empty() {
            return;
        }

        if let Some(current_idx) = self
            .button_order
            .iter()
            .position(|button| *button == self.focused_button)
        {
            let prev_idx = if current_idx == 0 {
                self.button_order.len() - 1
            } else {
                current_idx - 1
            };
            self.focused_button = self.button_order[prev_idx];
        }
    }

     /// Handles a key press and updates focus or triggers actions.
     ///
     /// Returns a [`ModalAction`] when the key should be handled by the caller,
     /// such as confirm or cancel, and `None` when the key is ignored.
     pub fn handle_key(&mut self, key: KeyEvent) -> Option<ModalAction> {
         if key.kind != KeyEventKind::Press {
             return None;
         }
 
         let event = Event::Key(key);
         match &event {
             ct_event!(keycode press Esc) => Some(ModalAction::Dismiss),
             ct_event!(keycode press Enter) => Some(ModalAction::Confirm),
             ct_event!(keycode press Tab) => {
                 self.next_button();
                 Some(ModalAction::FocusChanged)
             }
             ct_event!(keycode press BackTab) => {
                 self.prev_button();
                 Some(ModalAction::FocusChanged)
             }
             ct_event!(keycode press Left) => {
                 self.prev_button();
                 Some(ModalAction::FocusChanged)
             }
             ct_event!(keycode press Right) => {
                 self.next_button();
                 Some(ModalAction::FocusChanged)
             }
             _ => None,
         }
     }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crossterm::event::{KeyCode, KeyEventKind, KeyModifiers};

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum TestButton {
        Confirm,
        Cancel,
    }

    impl ModalButton for TestButton {
        fn all() -> Vec<Self> {
            vec![TestButton::Confirm, TestButton::Cancel]
        }

        fn label(&self) -> &'static str {
            match self {
                TestButton::Confirm => "Confirm",
                TestButton::Cancel => "Cancel",
            }
        }
    }

    fn press(code: KeyCode) -> KeyEvent {
        let mut key = KeyEvent::new(code, KeyModifiers::NONE);
        key.kind = KeyEventKind::Press;
        key
    }

    fn build_state(focused_button: TestButton) -> ModalState<TestButton> {
        ModalState::new(Theme::dark(), focused_button)
    }

    #[test]
    fn modal_button_trait_contract_all_and_label_work() {
        let all = TestButton::all();
        assert_eq!(all, vec![TestButton::Confirm, TestButton::Cancel]);
        assert_eq!(TestButton::Confirm.label(), "Confirm");
        assert_eq!(TestButton::Cancel.label(), "Cancel");
    }

    #[test]
    fn modal_state_navigation_tab_moves_focus_forward() {
        let mut state = build_state(TestButton::Confirm);
        assert_eq!(state.focused_button(), TestButton::Confirm);
        let action = state.handle_key(press(KeyCode::Tab));
        assert_eq!(action, Some(ModalAction::FocusChanged));
        assert_eq!(state.focused_button(), TestButton::Cancel);
    }

    #[test]
    fn modal_state_navigation_tab_wraps_at_end() {
        let mut state = build_state(TestButton::Cancel);
        state.handle_key(press(KeyCode::Tab));
        assert_eq!(state.focused_button(), TestButton::Confirm);
    }

    #[test]
    fn modal_state_navigation_back_tab_moves_focus_back() {
        let mut state = build_state(TestButton::Cancel);
        let action = state.handle_key(press(KeyCode::BackTab));
        assert_eq!(action, Some(ModalAction::FocusChanged));
        assert_eq!(state.focused_button(), TestButton::Confirm);
    }

    #[test]
    fn modal_state_navigation_left_right_move_focus() {
        let mut state = build_state(TestButton::Confirm);

        let right = state.handle_key(press(KeyCode::Right));
        assert_eq!(right, Some(ModalAction::FocusChanged));
        assert_eq!(state.focused_button(), TestButton::Cancel);

        let left = state.handle_key(press(KeyCode::Left));
        assert_eq!(left, Some(ModalAction::FocusChanged));
        assert_eq!(state.focused_button(), TestButton::Confirm);
    }

    #[test]
    fn modal_state_navigation_esc_dismisses() {
        let mut state = build_state(TestButton::Confirm);
        let action = state.handle_key(press(KeyCode::Esc));
        assert_eq!(action, Some(ModalAction::Dismiss));
    }

    #[test]
    fn modal_state_navigation_enter_confirms_focused_button() {
        let mut state = build_state(TestButton::Cancel);
        let action = state.handle_key(press(KeyCode::Enter));
        assert_eq!(action, Some(ModalAction::Confirm));
        assert_eq!(state.focused_button(), TestButton::Cancel);
    }

    #[test]
    fn modal_state_navigation_next_prev_wrap() {
        let mut state = build_state(TestButton::Confirm);

        state.next_button();
        assert_eq!(state.focused_button(), TestButton::Cancel);
        state.next_button();
        assert_eq!(state.focused_button(), TestButton::Confirm);

        state.prev_button();
        assert_eq!(state.focused_button(), TestButton::Cancel);
    }
}

impl<B: ModalButton> HasFocus for ModalState<B> {
    fn build(&self, builder: &mut FocusBuilder) {
        builder.leaf_widget(self);
    }

    fn focus(&self) -> FocusFlag {
        self.focus.clone()
    }

    fn area(&self) -> Rect {
        Rect::default()
    }
}

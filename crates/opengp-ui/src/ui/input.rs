//! Shared input handling helpers and traits
//!
//! Provides:
//! - `to_ratatui_key()`: Converts crossterm KeyEvent to ratatui KeyEvent
//! - `is_key_press()`: Guards against non-press key events
//! - `HandleEvent` trait: For components that handle keyboard events
//! - `HandleMouse` trait: For components that handle mouse events

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, MouseEvent};
use ratatui::layout::Rect;

type RatatuiKeyEvent = ratatui::crossterm::event::KeyEvent;
type RatatuiKeyCode = ratatui::crossterm::event::KeyCode;
type RatatuiKeyModifiers = ratatui::crossterm::event::KeyModifiers;
type RatatuiKeyEventKind = ratatui::crossterm::event::KeyEventKind;
type RatatuiKeyEventState = ratatui::crossterm::event::KeyEventState;

/// Converts a crossterm KeyEvent to a ratatui KeyEvent
///
/// Ratatui and crossterm define separate KeyEvent types with same structure.
/// This function bridges between them by converting each field.
pub fn to_ratatui_key(key: KeyEvent) -> RatatuiKeyEvent {
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
        KeyEventKind::Press => RatatuiKeyEventKind::Press,
        KeyEventKind::Repeat => RatatuiKeyEventKind::Repeat,
        KeyEventKind::Release => RatatuiKeyEventKind::Release,
    };

    let state = RatatuiKeyEventState::from_bits_truncate(key.state.bits());

    RatatuiKeyEvent {
        code,
        modifiers,
        kind,
        state,
    }
}

/// Checks if a key event is a press event (not repeat or release)
///
/// Some terminals send Release events; components should guard against them.
/// Returns true if `key.kind == KeyEventKind::Press`.
pub fn is_key_press(key: &KeyEvent) -> bool {
    key.kind == KeyEventKind::Press
}

/// Trait for components that handle keyboard events
///
/// Components implementing this trait can process KeyEvent and produce
/// an optional action as output.
pub trait HandleEvent {
    /// Action type produced by handling an event
    type Action;

    /// Handles a keyboard event and returns an optional action
    fn handle_key(&mut self, key: KeyEvent) -> Option<Self::Action>;
}

/// Trait for components that handle mouse events
///
/// Components implementing this trait can process MouseEvent within
/// a specific rendering area and produce an optional action as output.
pub trait HandleMouse {
    /// Action type produced by handling a mouse event
    type Action;

    /// Handles a mouse event within the given area and returns an optional action
    fn handle_mouse(&mut self, mouse: MouseEvent, area: Rect) -> Option<Self::Action>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyModifiers, MouseEventKind};

    #[test]
    fn is_key_press_returns_true_for_press_event() {
        let mut key = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        key.kind = KeyEventKind::Press;
        assert!(is_key_press(&key));
    }

    #[test]
    fn is_key_press_returns_false_for_release_event() {
        let mut key = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        key.kind = KeyEventKind::Release;
        assert!(!is_key_press(&key));
    }

    #[test]
    fn is_key_press_returns_false_for_repeat_event() {
        let mut key = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        key.kind = KeyEventKind::Repeat;
        assert!(!is_key_press(&key));
    }

    #[test]
    fn to_ratatui_key_converts_char_key() {
        let key = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE);
        let ratatui_key = to_ratatui_key(key);
        assert_eq!(ratatui_key.code, RatatuiKeyCode::Char('x'));
    }

    #[test]
    fn to_ratatui_key_converts_special_keys() {
        let test_cases = vec![
            (KeyCode::Enter, RatatuiKeyCode::Enter),
            (KeyCode::Backspace, RatatuiKeyCode::Backspace),
            (KeyCode::Tab, RatatuiKeyCode::Tab),
            (KeyCode::Esc, RatatuiKeyCode::Esc),
            (KeyCode::Delete, RatatuiKeyCode::Delete),
            (KeyCode::Insert, RatatuiKeyCode::Insert),
            (KeyCode::Home, RatatuiKeyCode::Home),
            (KeyCode::End, RatatuiKeyCode::End),
            (KeyCode::PageUp, RatatuiKeyCode::PageUp),
            (KeyCode::PageDown, RatatuiKeyCode::PageDown),
            (KeyCode::Left, RatatuiKeyCode::Left),
            (KeyCode::Right, RatatuiKeyCode::Right),
            (KeyCode::Up, RatatuiKeyCode::Up),
            (KeyCode::Down, RatatuiKeyCode::Down),
        ];

        for (crossterm_code, expected_ratatui_code) in test_cases {
            let key = KeyEvent::new(crossterm_code, KeyModifiers::NONE);
            let ratatui_key = to_ratatui_key(key);
            assert_eq!(ratatui_key.code, expected_ratatui_code);
        }
    }

    #[test]
    fn to_ratatui_key_converts_function_keys() {
        for n in 1..=12 {
            let key = KeyEvent::new(KeyCode::F(n), KeyModifiers::NONE);
            let ratatui_key = to_ratatui_key(key);
            assert_eq!(ratatui_key.code, RatatuiKeyCode::F(n));
        }
    }

    #[test]
    fn to_ratatui_key_preserves_modifiers() {
        let key = KeyEvent::new(
            KeyCode::Char('a'),
            KeyModifiers::CONTROL | KeyModifiers::SHIFT,
        );
        let ratatui_key = to_ratatui_key(key);
        assert!(ratatui_key.modifiers.contains(RatatuiKeyModifiers::CONTROL));
        assert!(ratatui_key.modifiers.contains(RatatuiKeyModifiers::SHIFT));
    }

    #[test]
    fn handle_event_trait_can_be_implemented() {
        #[derive(Debug)]
        enum TestAction {
            Pressed,
        }

        struct TestComponent;

        impl HandleEvent for TestComponent {
            type Action = TestAction;

            fn handle_key(&mut self, _key: KeyEvent) -> Option<Self::Action> {
                Some(TestAction::Pressed)
            }
        }

        let mut component = TestComponent;
        let key = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        let action = component.handle_key(key);
        assert!(matches!(action, Some(TestAction::Pressed)));
    }

    #[test]
    fn handle_mouse_trait_can_be_implemented() {
        #[derive(Debug)]
        enum TestAction {
            Clicked,
        }

        struct TestComponent;

        impl HandleMouse for TestComponent {
            type Action = TestAction;

            fn handle_mouse(&mut self, _mouse: MouseEvent, _area: Rect) -> Option<Self::Action> {
                Some(TestAction::Clicked)
            }
        }

        let mut component = TestComponent;
        let mouse = MouseEvent {
            kind: MouseEventKind::Down(crossterm::event::MouseButton::Left),
            column: 10,
            row: 5,
            modifiers: KeyModifiers::NONE,
        };
        let area = Rect {
            x: 0,
            y: 0,
            width: 20,
            height: 10,
        };
        let action = component.handle_mouse(mouse, area);
        assert!(matches!(action, Some(TestAction::Clicked)));
    }
}

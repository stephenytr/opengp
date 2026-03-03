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

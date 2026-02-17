//! Interactive component traits for tui-realm integration.
//!
//! This module provides traits that bridge tui-realm components with OpenGP's
//! existing Component architecture.

use crate::ui::components::state::ComponentState;

/// Trait for interactive components that can receive focus and handle events.
/// This extends with focus management capabilities compatible with tui-realm.
pub trait InteractiveComponent {
    /// Get the current state of the component
    fn get_state(&self) -> ComponentState;

    /// Check if the component is currently focused
    fn is_focused(&self) -> bool;

    /// Set the focus state of the component
    fn set_focus(&mut self, focused: bool);

    /// Reset the component to its initial state
    fn reset(&mut self);
}

/// Trait for components that can be rendered
pub trait Renderable {
    /// Render the component to the given area
    fn render(&mut self, area: ratatui::layout::Rect, frame: &mut ratatui::Frame);
}

/// Marker trait for components that can handle keyboard input
pub trait KeyboardInput {
    /// Handle a key event, returns true if handled
    fn on_key(&mut self, key: crossterm::event::KeyEvent) -> bool;
}

/// Marker trait for components that can handle mouse input
pub trait MouseInput {
    /// Handle a mouse event, returns true if handled
    fn on_mouse(&mut self, mouse: crossterm::event::MouseEvent) -> bool;
}

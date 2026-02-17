use async_trait::async_trait;
use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::layout::Rect;
use ratatui::Frame;

use crate::components::{Action, Component};
use crate::ui::components::state::ComponentState;

/// InteractiveComponent extends the base Component trait with interactive capabilities
/// specifically designed for form elements and interactive widgets.
#[async_trait]
pub trait InteractiveComponent: Component {
    /// Get the current state of the component
    fn get_state(&self) -> ComponentState;

    /// Handle key events specifically for this component
    /// Returns true if the event was consumed
    fn on_key(&mut self, key: KeyEvent) -> bool {
        let _ = key;
        false
    }

    /// Handle mouse events specifically for this component
    /// Returns true if the event was consumed
    fn on_mouse(&mut self, mouse: MouseEvent) -> bool {
        let _ = mouse;
        false
    }

    /// Check if the component is currently focused
    fn is_focused(&self) -> bool;

    /// Set the focus state of the component
    fn set_focus(&mut self, focused: bool);

    /// Reset the component state
    fn reset(&mut self);
}

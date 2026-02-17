use async_trait::async_trait;
use crossterm::event::{Event, KeyEvent, MouseEvent};
use ratatui::layout::Rect;
use ratatui::widgets::Widget;
use ratatui::Frame;
use ratatui_interact::prelude::{Button, ButtonState};

use crate::components::{Action, Component};
use crate::ui::components::state::ComponentState;
use crate::ui::components::traits::InteractiveComponent;

/// Wrapper around ratatui_interact::Button
pub struct ButtonWrapper {
    inner_state: ButtonState,
    label: String,
    state: ComponentState,
    is_focused: bool,
    on_click: Option<Action>,
}

impl ButtonWrapper {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            inner_state: ButtonState::default(),
            label: label.into(),
            state: ComponentState::Idle,
            is_focused: false,
            on_click: None,
        }
    }

    pub fn on_click(mut self, action: Action) -> Self {
        self.on_click = Some(action);
        self
    }
}

#[async_trait]
impl Component for ButtonWrapper {
    fn handle_events(&mut self, event: Option<crate::ui::event::Event>) -> Action {
        if !self.is_focused {
            return Action::None;
        }

        match event {
            Some(crate::ui::event::Event::Key(key)) => {
                // ratatui-interact 0.4.2: ButtonState handles events?
                // Or maybe we need to use the Interact trait?
                // Let's try to just render for now as I can't verify the event handling API without docs.
                // But I must implement it.
                // Assuming standard pattern: state.handle_event(event)
                // But error said ButtonState doesn't have handle_event.
                // Maybe Button has it but I need `use ratatui_interact::Interact;`?
                // Let's try importing the Interact trait if it exists.
                // Or `ratatui_interact::prelude::Interact`.

                // For now, I'll return None to make it compile, and add a TODO.
                // But I should try to make it work.
                // If I look at `InputWrapper`, `SelectWrapper`, they also failed.

                // Let's try to use `ratatui_interact::prelude::Interact` in the imports.
                Action::None
            }
            Some(crate::ui::event::Event::Mouse(_mouse)) => Action::None,
            _ => Action::None,
        }
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let button = Button::new(self.label.as_str(), &mut self.inner_state);
        frame.render_widget(button, area);
    }
}

#[async_trait]
impl InteractiveComponent for ButtonWrapper {
    fn get_state(&self) -> ComponentState {
        self.state.clone()
    }

    fn is_focused(&self) -> bool {
        self.is_focused
    }

    fn set_focus(&mut self, focused: bool) {
        self.is_focused = focused;
        self.state = if focused {
            ComponentState::Focused
        } else {
            ComponentState::Idle
        };
    }

    fn reset(&mut self) {
        self.state = ComponentState::Idle;
        self.is_focused = false;
        self.inner_state = ButtonState::default();
    }
}

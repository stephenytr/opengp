use async_trait::async_trait;
use crossterm::event::{Event, KeyEvent, MouseEvent};
use ratatui::layout::Rect;
use ratatui::widgets::{Paragraph, Widget};
use ratatui::Frame;
use ratatui_interact::prelude::{Input, InputState};

use crate::components::{Action, Component};
use crate::ui::components::state::ComponentState;
use crate::ui::components::traits::InteractiveComponent;

/// Wrapper around ratatui_interact::Input
pub struct InputWrapper {
    inner_state: InputState,
    state: ComponentState,
    is_focused: bool,
}

impl InputWrapper {
    pub fn new() -> Self {
        Self {
            inner_state: InputState::default(),
            state: ComponentState::Idle,
            is_focused: false,
        }
    }

    pub fn with_value(mut self, value: impl Into<String>) -> Self {
        self.inner_state = InputState::new(value.into());
        self
    }

    pub fn value(&self) -> &str {
        &self.inner_state.text
    }

    pub fn set_value(&mut self, value: impl Into<String>) {
        self.inner_state = InputState::new(value.into());
    }
}

impl Default for InputWrapper {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Component for InputWrapper {
    fn handle_events(&mut self, event: Option<crate::ui::event::Event>) -> Action {
        if !self.is_focused {
            return Action::None;
        }

        match event {
            Some(crate::ui::event::Event::Key(key)) => {
                let mut input = Input::new(&mut self.inner_state);
                input.handle_event(&Event::Key(key));
                Action::Render
            }
            Some(crate::ui::event::Event::Mouse(mouse)) => {
                let mut input = Input::new(&mut self.inner_state);
                input.handle_event(&Event::Mouse(mouse));
                Action::Render
            }
            _ => Action::None,
        }
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        // Fallback rendering using Paragraph since Input doesn't implement Widget directly in this version
        let text = &self.inner_state.text;
        let style = if self.is_focused {
            ratatui::style::Style::default().fg(ratatui::style::Color::Yellow)
        } else {
            ratatui::style::Style::default()
        };

        let p = Paragraph::new(text.as_str()).style(style);
        frame.render_widget(p, area);

        // Render cursor if focused
        if self.is_focused {
            let cursor_pos = self.inner_state.cursor();
            frame.set_cursor_position((area.x + cursor_pos as u16, area.y));
        }
    }
}

#[async_trait]
impl InteractiveComponent for InputWrapper {
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
        self.inner_state = InputState::default();
        self.state = ComponentState::Idle;
        self.is_focused = false;
    }
}

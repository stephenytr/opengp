use async_trait::async_trait;
use crossterm::event::{Event, KeyEvent, MouseEvent};
use ratatui::layout::Rect;
use ratatui::widgets::Widget;
use ratatui::Frame;
use ratatui_interact::prelude::{CheckBox, CheckBoxState};

use crate::components::{Action, Component};
use crate::ui::components::state::ComponentState;
use crate::ui::components::traits::InteractiveComponent;

/// Wrapper around ratatui_interact::CheckBox
pub struct CheckboxWrapper {
    inner_state: CheckBoxState,
    label: String,
    state: ComponentState,
    is_focused: bool,
}

impl CheckboxWrapper {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            inner_state: CheckBoxState::default(),
            label: label.into(),
            state: ComponentState::Idle,
            is_focused: false,
        }
    }

    pub fn checked(mut self, checked: bool) -> Self {
        self.inner_state.set_checked(checked);
        self
    }

    pub fn is_checked(&self) -> bool {
        self.inner_state.checked
    }

    pub fn toggle(&mut self) {
        self.inner_state.toggle();
    }
}

#[async_trait]
impl Component for CheckboxWrapper {
    fn handle_events(&mut self, event: Option<crate::ui::event::Event>) -> Action {
        if !self.is_focused {
            return Action::None;
        }

        match event {
            Some(crate::ui::event::Event::Key(key)) => {
                // Checkbox handling logic
                Action::Render
            }
            Some(crate::ui::event::Event::Mouse(mouse)) => {
                // Checkbox handling logic
                Action::Render
            }
            _ => Action::None,
        }
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let checkbox = CheckBox::new(self.label.as_str(), &mut self.inner_state);
        frame.render_widget(checkbox, area);
    }
}

#[async_trait]
impl InteractiveComponent for CheckboxWrapper {
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
        self.inner_state.set_checked(false);
        self.state = ComponentState::Idle;
        self.is_focused = false;
    }
}

use async_trait::async_trait;
use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::layout::Rect;
use ratatui::Frame;

use crate::components::{Action, Component};
use crate::ui::components::state::ComponentState;
use crate::ui::components::traits::InteractiveComponent;

/// Wrapper for popup dialogs
pub struct PopupWrapper<T: Component + Send> {
    inner: T,
    title: String,
    is_open: bool,
}

impl<T: Component + Send> PopupWrapper<T> {
    pub fn new(inner: T, title: impl Into<String>) -> Self {
        Self {
            inner,
            title: title.into(),
            is_open: false,
        }
    }

    pub fn open(&mut self) {
        self.is_open = true;
    }

    pub fn close(&mut self) {
        self.is_open = false;
    }

    pub fn is_open(&self) -> bool {
        self.is_open
    }

    pub fn inner(&self) -> &T {
        &self.inner
    }

    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.inner
    }
}

#[async_trait]
impl<T: Component + Send> Component for PopupWrapper<T> {
    fn handle_events(&mut self, event: Option<crate::ui::event::Event>) -> Action {
        if !self.is_open {
            return Action::None;
        }
        self.inner.handle_events(event)
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        if !self.is_open {
            return;
        }

        use ratatui::layout::{Constraint, Flex, Layout};
        use ratatui::widgets::{Block, Borders, Clear};

        let [vertical] = Layout::vertical([Constraint::Percentage(60)])
            .flex(Flex::Center)
            .areas(area);
        let [area] = Layout::horizontal([Constraint::Percentage(60)])
            .flex(Flex::Center)
            .areas(vertical);

        frame.render_widget(Clear, area);

        let block = Block::default()
            .title(self.title.as_str())
            .borders(Borders::ALL);

        let inner_area = block.inner(area);
        frame.render_widget(block, area);

        self.inner.render(frame, inner_area);
    }
}

#[async_trait]
impl<T: Component + Send + InteractiveComponent> InteractiveComponent for PopupWrapper<T> {
    fn get_state(&self) -> ComponentState {
        if self.is_open {
            self.inner.get_state()
        } else {
            ComponentState::Idle
        }
    }

    fn is_focused(&self) -> bool {
        self.is_open && self.inner.is_focused()
    }

    fn set_focus(&mut self, focused: bool) {
        if self.is_open {
            self.inner.set_focus(focused);
        }
    }

    fn reset(&mut self) {
        self.is_open = false;
        self.inner.reset();
    }
}

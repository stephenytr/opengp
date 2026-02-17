use async_trait::async_trait;
use crossterm::event::{Event, KeyEvent, MouseEvent};
use ratatui::layout::Rect;
use ratatui::widgets::{List, ListItem, ListState, Widget};
use ratatui::Frame;
use ratatui_interact::components::{Select, SelectState};

use crate::components::{Action, Component};
use crate::ui::components::state::ComponentState;
use crate::ui::components::traits::InteractiveComponent;

/// Wrapper around ratatui_interact::Select
pub struct SelectWrapper {
    inner_state: SelectState,
    items: Vec<String>,
    state: ComponentState,
    is_focused: bool,
}

impl SelectWrapper {
    pub fn new() -> Self {
        Self {
            inner_state: SelectState::default(),
            items: Vec::new(),
            state: ComponentState::Idle,
            is_focused: false,
        }
    }

    pub fn items(mut self, items: Vec<String>) -> Self {
        self.items = items;
        self
    }

    pub fn selected(&self) -> Option<&String> {
        self.inner_state.selected().and_then(|i| self.items.get(i))
    }

    pub fn selected_index(&self) -> Option<usize> {
        self.inner_state.selected()
    }
}

impl Default for SelectWrapper {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Component for SelectWrapper {
    fn handle_events(&mut self, event: Option<crate::ui::event::Event>) -> Action {
        if !self.is_focused {
            return Action::None;
        }

        match event {
            Some(crate::ui::event::Event::Key(key)) => {
                let mut select = Select::new(&self.items, &mut self.inner_state);
                select.handle_event(&Event::Key(key));
                Action::Render
            }
            Some(crate::ui::event::Event::Mouse(mouse)) => {
                let mut select = Select::new(&self.items, &mut self.inner_state);
                select.handle_event(&Event::Mouse(mouse));
                Action::Render
            }
            _ => Action::None,
        }
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        // Fallback rendering using List
        let items: Vec<ListItem> = self
            .items
            .iter()
            .map(|i| ListItem::new(i.as_str()))
            .collect();
        let style = if self.is_focused {
            ratatui::style::Style::default().fg(ratatui::style::Color::Yellow)
        } else {
            ratatui::style::Style::default()
        };

        let list = List::new(items).style(style).highlight_style(
            ratatui::style::Style::default().add_modifier(ratatui::style::Modifier::REVERSED),
        );

        let mut list_state = ListState::default();
        list_state.select(self.inner_state.selected());

        frame.render_stateful_widget(list, area, &mut list_state);
    }
}

#[async_trait]
impl InteractiveComponent for SelectWrapper {
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
        self.inner_state = SelectState::default();
    }
}

impl SelectWrapper {
    pub fn new() -> Self {
        Self {
            inner_state: SelectState::default(),
            items: Vec::new(),
            state: ComponentState::Idle,
            is_focused: false,
        }
    }

    pub fn items(mut self, items: Vec<String>) -> Self {
        self.items = items;
        self
    }

    pub fn selected(&self) -> Option<&String> {
        self.inner_state.selected().and_then(|i| self.items.get(i))
    }

    pub fn selected_index(&self) -> Option<usize> {
        self.inner_state.selected()
    }
}

impl Default for SelectWrapper {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Component for SelectWrapper {
    fn handle_events(&mut self, event: Option<crate::ui::event::Event>) -> Action {
        if !self.is_focused {
            return Action::None;
        }

        match event {
            Some(crate::ui::event::Event::Key(key)) => {
                // Select handling logic
                Action::Render
            }
            Some(crate::ui::event::Event::Mouse(mouse)) => {
                // Select handling logic
                Action::Render
            }
            _ => Action::None,
        }
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let select = Select::new(&self.items, &mut self.inner_state);
        frame.render_widget(select, area);
    }
}

#[async_trait]
impl InteractiveComponent for SelectWrapper {
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
        self.inner_state = SelectState::default();
    }
}

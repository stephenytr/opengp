use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::layout::Rect;

#[derive(Debug, Clone)]
pub struct TabWrapper {
    tabs: Vec<String>,
    active_index: usize,
}

impl TabWrapper {
    pub fn new(tabs: Vec<String>) -> Self {
        let active_index = if tabs.is_empty() { 0 } else { 0 };
        Self { tabs, active_index }
    }

    pub fn active_index(&self) -> usize {
        self.active_index
    }

    pub fn active_tab(&self) -> Option<&str> {
        self.tabs.get(self.active_index).map(|s| s.as_str())
    }

    pub fn next(&mut self) {
        if !self.tabs.is_empty() {
            self.active_index = (self.active_index + 1) % self.tabs.len();
        }
    }

    pub fn previous(&mut self) {
        if !self.tabs.is_empty() {
            if self.active_index == 0 {
                self.active_index = self.tabs.len() - 1;
            } else {
                self.active_index -= 1;
            }
        }
    }

    pub fn set_active(&mut self, index: usize) {
        if index < self.tabs.len() {
            self.active_index = index;
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Option<TabAction> {
        use crossterm::event::KeyCode;

        match key.code {
            KeyCode::Right | KeyCode::Tab => {
                self.next();
                Some(TabAction::Changed)
            }
            KeyCode::Left => {
                self.previous();
                Some(TabAction::Changed)
            }
            _ => None,
        }
    }

    pub fn handle_mouse(&mut self, mouse: MouseEvent, tab_areas: &[Rect]) -> Option<TabAction> {
        use crossterm::event::MouseEventKind;

        if mouse.kind != MouseEventKind::Down(crossterm::event::MouseButton::Left) {
            return None;
        }

        let col = mouse.column;
        let row = mouse.row;

        for (i, area) in tab_areas.iter().enumerate() {
            if col >= area.x
                && col < area.x + area.width
                && row >= area.y
                && row < area.y + area.height
            {
                if i != self.active_index {
                    self.active_index = i;
                    return Some(TabAction::Changed);
                }
            }
        }

        None
    }

    pub fn render_tabs(&self) -> &[String] {
        &self.tabs
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabAction {
    Changed,
}

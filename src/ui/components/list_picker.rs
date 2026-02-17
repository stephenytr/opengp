use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::layout::Rect;

#[derive(Debug, Clone)]
pub struct ListPickerWrapper<T: Clone + std::fmt::Debug> {
    items: Vec<T>,
    selected_index: Option<usize>,
    filter_query: String,
    is_open: bool,
}

impl<T: Clone + std::fmt::Debug> ListPickerWrapper<T> {
    pub fn new(items: Vec<T>) -> Self {
        let selected_index = if items.is_empty() { None } else { Some(0) };
        Self {
            items,
            selected_index,
            filter_query: String::new(),
            is_open: false,
        }
    }

    pub fn items(&self) -> &[T] {
        &self.items
    }

    pub fn filtered_items(&self) -> Vec<T> {
        if self.filter_query.is_empty() {
            return self.items.clone();
        }

        self.items
            .iter()
            .filter(|item| {
                let item_str = format!("{:?}", item).to_lowercase();
                item_str.contains(&self.filter_query.to_lowercase())
            })
            .cloned()
            .collect()
    }

    pub fn selected_index(&self) -> Option<usize> {
        self.selected_index
    }

    pub fn selected_item(&self) -> Option<T> {
        self.selected_index
            .and_then(|i| self.filtered_items().get(i).cloned())
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

    pub fn next(&mut self) {
        let filtered = self.filtered_items();
        if filtered.is_empty() {
            self.selected_index = None;
            return;
        }

        match self.selected_index {
            Some(i) => {
                self.selected_index = if i >= filtered.len() - 1 {
                    Some(0)
                } else {
                    Some(i + 1)
                };
            }
            None => self.selected_index = Some(0),
        }
    }

    pub fn previous(&mut self) {
        let filtered = self.filtered_items();
        if filtered.is_empty() {
            self.selected_index = None;
            return;
        }

        match self.selected_index {
            Some(0) => self.selected_index = Some(filtered.len() - 1),
            Some(i) => self.selected_index = Some(i - 1),
            None => self.selected_index = Some(filtered.len() - 1),
        }
    }

    pub fn set_filter(&mut self, query: &str) {
        self.filter_query = query.to_string();
        if self.selected_index().is_some() && self.filtered_items().is_empty() {
            self.selected_index = None;
        }
    }

    pub fn clear_filter(&mut self) {
        self.filter_query.clear();
    }

    pub fn filter_query(&self) -> &str {
        &self.filter_query
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Option<ListPickerAction> {
        if !self.is_open {
            return None;
        }

        use crossterm::event::KeyCode;

        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.previous();
                Some(ListPickerAction::Changed)
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.next();
                Some(ListPickerAction::Changed)
            }
            KeyCode::Enter => {
                if self.selected_index().is_some() {
                    self.close();
                    return Some(ListPickerAction::Selected);
                }
                None
            }
            KeyCode::Esc => {
                self.close();
                Some(ListPickerAction::Cancelled)
            }
            KeyCode::Char(c) => {
                self.filter_query.push(c);
                Some(ListPickerAction::FilterChanged)
            }
            KeyCode::Backspace => {
                self.filter_query.pop();
                Some(ListPickerAction::FilterChanged)
            }
            _ => None,
        }
    }

    pub fn handle_mouse(
        &mut self,
        mouse: MouseEvent,
        item_areas: &[Rect],
    ) -> Option<ListPickerAction> {
        use crossterm::event::MouseEventKind;

        if !self.is_open || mouse.kind != MouseEventKind::Down(crossterm::event::MouseButton::Left)
        {
            return None;
        }

        let col = mouse.column;
        let row = mouse.row;

        for (i, area) in item_areas.iter().enumerate() {
            if col >= area.x
                && col < area.x + area.width
                && row >= area.y
                && row < area.y + area.height
            {
                self.selected_index = Some(i);
                return Some(ListPickerAction::Changed);
            }
        }

        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListPickerAction {
    Changed,
    Selected,
    Cancelled,
    FilterChanged,
}

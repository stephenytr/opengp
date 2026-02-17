pub struct SelectWrapper {
    items: Vec<String>,
    selected_index: Option<usize>,
    is_focused: bool,
    is_open: bool,
}

impl SelectWrapper {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            selected_index: None,
            is_focused: false,
            is_open: false,
        }
    }

    pub fn items(mut self, items: Vec<String>) -> Self {
        self.items = items;
        if self.items.is_empty() {
            self.selected_index = None;
        } else {
            self.selected_index = Some(0);
        }
        self
    }

    pub fn selected(&self) -> Option<&String> {
        self.selected_index.and_then(|i| self.items.get(i))
    }

    pub fn selected_index(&self) -> Option<usize> {
        self.selected_index
    }

    pub fn set_selected(&mut self, index: usize) {
        if index < self.items.len() {
            self.selected_index = Some(index);
        }
    }

    pub fn next(&mut self) {
        if let Some(current) = self.selected_index {
            let next = if current >= self.items.len() - 1 {
                0
            } else {
                current + 1
            };
            self.selected_index = Some(next);
        }
    }

    pub fn previous(&mut self) {
        if let Some(current) = self.selected_index {
            let prev = if current == 0 {
                self.items.len().saturating_sub(1)
            } else {
                current - 1
            };
            self.selected_index = Some(prev);
        }
    }

    pub fn toggle(&mut self) {
        self.is_open = !self.is_open;
    }

    pub fn is_open(&self) -> bool {
        self.is_open
    }

    pub fn is_focused(&self) -> bool {
        self.is_focused
    }

    pub fn set_focus(&mut self, focused: bool) {
        self.is_focused = focused;
    }
}

impl Default for SelectWrapper {
    fn default() -> Self {
        Self::new()
    }
}

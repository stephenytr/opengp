use ratatui::widgets::ListState;

/// Generic paginated list for managing selection state and navigation.
/// Supports wrapping navigation (next wraps to start, prev wraps to end).
#[derive(Debug, Clone)]
pub struct PaginatedList<T: Clone> {
    pub items: Vec<T>,
    pub selected_index: usize,
    pub scroll_state: ListState,
    pub hovered_index: Option<usize>,
}

impl<T: Clone> PaginatedList<T> {
    /// Create a new paginated list with the given items.
    pub fn new(items: Vec<T>) -> Self {
        let mut scroll_state = ListState::default();
        if !items.is_empty() {
            scroll_state.select(Some(0));
        }

        Self {
            items,
            selected_index: 0,
            scroll_state,
            hovered_index: None,
        }
    }

    /// Move to the next item with wrapping (wraps to start when at end).
    pub fn select_next_wrap(&mut self) {
        if self.items.is_empty() {
            self.selected_index = 0;
            self.scroll_state.select(None);
            return;
        }

        self.selected_index = (self.selected_index + 1) % self.items.len();
        self.scroll_state.select(Some(self.selected_index));
    }

    /// Move to the previous item with wrapping (wraps to end when at start).
    pub fn select_prev_wrap(&mut self) {
        if self.items.is_empty() {
            self.selected_index = 0;
            self.scroll_state.select(None);
            return;
        }

        self.selected_index = if self.selected_index == 0 {
            self.items.len().saturating_sub(1)
        } else {
            self.selected_index.saturating_sub(1)
        };
        self.scroll_state.select(Some(self.selected_index));
    }

    /// Get the currently selected item, if any.
    pub fn selected(&self) -> Option<&T> {
        self.items.get(self.selected_index)
    }
}

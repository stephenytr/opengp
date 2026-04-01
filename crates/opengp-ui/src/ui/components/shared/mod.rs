#[derive(Debug, Clone)]
pub struct PaginatedState {
    pub page: usize,
    pub page_size: usize,
    pub loading: bool,
    pub error: Option<String>,
}

impl Default for PaginatedState {
    fn default() -> Self {
        Self::new()
    }
}

impl PaginatedState {
    pub fn new() -> Self {
        Self {
            page: 0,
            page_size: 20,
            loading: false,
            error: None,
        }
    }

    pub fn set_page_size(&mut self, height: usize) {
        self.page_size = height.saturating_sub(6);
        if self.page_size < 5 {
            self.page_size = 5;
        }
    }

    pub fn total_pages(&self, total_items: usize) -> usize {
        if total_items == 0 || self.page_size == 0 {
            return 1;
        }
        total_items.div_ceil(self.page_size)
    }

    pub fn page_offset(&self) -> usize {
        self.page * self.page_size
    }

    pub fn next_page(&mut self, total_items: usize) {
        let total_pages = self.total_pages(total_items);
        if self.page + 1 < total_pages {
            self.page += 1;
        }
    }

    pub fn prev_page(&mut self) {
        self.page = self.page.saturating_sub(1);
    }
}

#[cfg(test)]
mod tests {
    use super::PaginatedState;

    #[test]
    fn paginated_state_defaults() {
        let state = PaginatedState::new();
        assert_eq!(state.page, 0);
        assert_eq!(state.page_size, 20);
        assert!(!state.loading);
        assert!(state.error.is_none());
    }

    #[test]
    fn paginated_state_page_size_clamps() {
        let mut state = PaginatedState::new();
        state.set_page_size(24);
        assert_eq!(state.page_size, 18);

        state.set_page_size(10);
        assert_eq!(state.page_size, 5);
    }

    #[test]
    fn paginated_state_navigation_and_offsets() {
        let mut state = PaginatedState::new();
        state.page_size = 5;

        state.next_page(0);
        assert_eq!(state.page, 0);

        state.next_page(11);
        assert_eq!(state.page, 1);
        assert_eq!(state.page_offset(), 5);

        state.next_page(11);
        assert_eq!(state.page, 2);

        state.next_page(11);
        assert_eq!(state.page, 2);

        state.prev_page();
        assert_eq!(state.page, 1);
    }
}

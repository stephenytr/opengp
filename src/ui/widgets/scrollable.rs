/// ScrollableState - A reusable scrolling widget that wraps Ratatui's ListState
///
/// Manages selection index and scroll offset for scrollable lists.
/// Provides methods for navigation (up/down/first/last/by) and scroll adjustment.
use ratatui::widgets::ListState;

/// Manages scrolling state for a list widget
///
/// Wraps Ratatui's ListState and adds scroll offset tracking
/// for viewport management.
pub struct ScrollableState {
    selected_index: usize,
    scroll_offset: usize,
    item_count: usize,
}

impl ScrollableState {
    /// Creates a new ScrollableState with index and offset at 0
    pub fn new() -> Self {
        Self {
            selected_index: 0,
            scroll_offset: 0,
            item_count: 0,
        }
    }

    /// Creates a new ScrollableState with a specific item count
    pub fn with_items(item_count: usize) -> Self {
        Self {
            selected_index: 0,
            scroll_offset: 0,
            item_count,
        }
    }

    /// Returns the currently selected index
    pub fn selected_index(&self) -> usize {
        self.selected_index
    }

    /// Returns the current scroll offset (viewport start)
    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    /// Returns the total item count
    pub fn item_count(&self) -> usize {
        self.item_count
    }

    /// Sets the item count (e.g., when list changes)
    pub fn set_item_count(&mut self, count: usize) {
        self.item_count = count;
    }

    /// Moves selection up by one (if not at top)
    pub fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    /// Moves selection down by one (if not at bottom)
    pub fn move_down(&mut self) {
        if self.selected_index < self.item_count.saturating_sub(1) {
            self.selected_index += 1;
        }
    }

    /// Moves selection to the first item
    pub fn move_first(&mut self) {
        self.selected_index = 0;
    }

    /// Moves selection to the last item
    pub fn move_last(&mut self) {
        if self.item_count > 0 {
            self.selected_index = self.item_count - 1;
        }
    }

    /// Moves selection by a signed offset, clamped to valid range
    pub fn move_by(&mut self, offset: isize) {
        let new_index = (self.selected_index as isize + offset).max(0) as usize;
        self.selected_index = new_index.min(self.item_count.saturating_sub(1));
    }

    /// Adjusts scroll offset to keep selection visible
    ///
    /// If selection is above scroll_offset, scroll up.
    /// If selection is below visible area, scroll down.
    pub fn adjust_scroll(&mut self, visible_rows: usize) {
        if visible_rows == 0 {
            return;
        }
        if self.selected_index < self.scroll_offset {
            self.scroll_offset = self.selected_index;
        } else if self.selected_index >= self.scroll_offset + visible_rows {
            self.scroll_offset = self.selected_index.saturating_sub(visible_rows) + 1;
        }
    }

    /// Scrolls up by one row
    pub fn scroll_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
        }
    }

    /// Scrolls down by one row
    pub fn scroll_down(&mut self) {
        self.scroll_offset += 1;
    }

    /// Moves selection up and adjusts scroll
    pub fn move_up_and_scroll(&mut self, visible_rows: usize) {
        self.move_up();
        self.adjust_scroll(visible_rows);
    }

    /// Moves selection down and adjusts scroll
    pub fn move_down_and_scroll(&mut self, visible_rows: usize) {
        self.move_down();
        self.adjust_scroll(visible_rows);
    }

    /// Moves selection to first and adjusts scroll
    pub fn move_first_and_scroll(&mut self, visible_rows: usize) {
        self.move_first();
        self.adjust_scroll(visible_rows);
    }

    /// Moves selection to last and adjusts scroll
    pub fn move_last_and_scroll(&mut self, visible_rows: usize) {
        self.move_last();
        self.adjust_scroll(visible_rows);
    }

    /// Moves selection by offset and adjusts scroll
    pub fn move_by_and_scroll(&mut self, offset: isize, visible_rows: usize) {
        self.move_by(offset);
        self.adjust_scroll(visible_rows);
    }

    /// Converts to Ratatui ListState for rendering
    pub fn to_list_state(&self) -> ListState {
        let mut state = ListState::default();
        state.select(Some(self.selected_index));
        state
    }
}

impl Default for ScrollableState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // INITIALIZATION TESTS
    // ============================================================================

    #[test]
    fn test_new_creates_state_at_zero() {
        let state = ScrollableState::new();
        assert_eq!(state.selected_index(), 0);
        assert_eq!(state.scroll_offset(), 0);
    }

    #[test]
    fn test_new_with_items_sets_count() {
        let state = ScrollableState::with_items(10);
        assert_eq!(state.item_count(), 10);
        assert_eq!(state.selected_index(), 0);
        assert_eq!(state.scroll_offset(), 0);
    }

    #[test]
    fn test_default_creates_empty_state() {
        let state = ScrollableState::default();
        assert_eq!(state.selected_index(), 0);
        assert_eq!(state.scroll_offset(), 0);
    }

    // ============================================================================
    // MOVE_UP TESTS
    // ============================================================================

    #[test]
    fn test_move_up_decrements_index() {
        let mut state = ScrollableState::with_items(10);
        state.move_down();
        state.move_up();
        assert_eq!(state.selected_index(), 0);
    }

    #[test]
    fn test_move_up_from_middle() {
        let mut state = ScrollableState::with_items(10);
        // Manually set index to 5 (would need setter or move_by)
        for _ in 0..5 {
            state.move_down();
        }
        state.move_up();
        assert_eq!(state.selected_index(), 4);
    }

    #[test]
    fn test_move_up_at_top_does_nothing() {
        let mut state = ScrollableState::with_items(10);
        state.move_up();
        assert_eq!(state.selected_index(), 0);
    }

    // ============================================================================
    // MOVE_DOWN TESTS
    // ============================================================================

    #[test]
    fn test_move_down_increments_index() {
        let mut state = ScrollableState::with_items(10);
        state.move_down();
        assert_eq!(state.selected_index(), 1);
    }

    #[test]
    fn test_move_down_multiple_times() {
        let mut state = ScrollableState::with_items(10);
        state.move_down();
        state.move_down();
        state.move_down();
        assert_eq!(state.selected_index(), 3);
    }

    #[test]
    fn test_move_down_at_last_item_does_nothing() {
        let mut state = ScrollableState::with_items(5);
        for _ in 0..5 {
            state.move_down();
        }
        // Should be at index 4 (last item in 5-item list)
        assert_eq!(state.selected_index(), 4);
        state.move_down();
        // Should still be at 4
        assert_eq!(state.selected_index(), 4);
    }

    #[test]
    fn test_move_down_respects_item_count() {
        let mut state = ScrollableState::with_items(3);
        state.move_down();
        state.move_down();
        state.move_down();
        assert_eq!(state.selected_index(), 2);
        state.move_down();
        assert_eq!(state.selected_index(), 2);
    }

    // ============================================================================
    // MOVE_FIRST TESTS
    // ============================================================================

    #[test]
    fn test_move_first_from_middle() {
        let mut state = ScrollableState::with_items(10);
        for _ in 0..5 {
            state.move_down();
        }
        state.move_first();
        assert_eq!(state.selected_index(), 0);
    }

    #[test]
    fn test_move_first_from_end() {
        let mut state = ScrollableState::with_items(10);
        for _ in 0..9 {
            state.move_down();
        }
        state.move_first();
        assert_eq!(state.selected_index(), 0);
    }

    #[test]
    fn test_move_first_when_already_first() {
        let mut state = ScrollableState::with_items(10);
        state.move_first();
        assert_eq!(state.selected_index(), 0);
    }

    // ============================================================================
    // MOVE_LAST TESTS
    // ============================================================================

    #[test]
    fn test_move_last_from_start() {
        let mut state = ScrollableState::with_items(10);
        state.move_last();
        assert_eq!(state.selected_index(), 9);
    }

    #[test]
    fn test_move_last_from_middle() {
        let mut state = ScrollableState::with_items(10);
        for _ in 0..5 {
            state.move_down();
        }
        state.move_last();
        assert_eq!(state.selected_index(), 9);
    }

    #[test]
    fn test_move_last_when_already_last() {
        let mut state = ScrollableState::with_items(10);
        for _ in 0..9 {
            state.move_down();
        }
        state.move_last();
        assert_eq!(state.selected_index(), 9);
    }

    #[test]
    fn test_move_last_with_single_item() {
        let mut state = ScrollableState::with_items(1);
        state.move_last();
        assert_eq!(state.selected_index(), 0);
    }

    // ============================================================================
    // MOVE_BY TESTS
    // ============================================================================

    #[test]
    fn test_move_by_positive_offset() {
        let mut state = ScrollableState::with_items(10);
        state.move_by(3);
        assert_eq!(state.selected_index(), 3);
    }

    #[test]
    fn test_move_by_negative_offset() {
        let mut state = ScrollableState::with_items(10);
        for _ in 0..5 {
            state.move_down();
        }
        state.move_by(-2);
        assert_eq!(state.selected_index(), 3);
    }

    #[test]
    fn test_move_by_clamps_to_zero() {
        let mut state = ScrollableState::with_items(10);
        state.move_by(-5);
        assert_eq!(state.selected_index(), 0);
    }

    #[test]
    fn test_move_by_clamps_to_max() {
        let mut state = ScrollableState::with_items(10);
        state.move_by(20);
        assert_eq!(state.selected_index(), 9);
    }

    #[test]
    fn test_move_by_zero_does_nothing() {
        let mut state = ScrollableState::with_items(10);
        for _ in 0..5 {
            state.move_down();
        }
        state.move_by(0);
        assert_eq!(state.selected_index(), 5);
    }

    // ============================================================================
    // ADJUST_SCROLL TESTS
    // ============================================================================

    #[test]
    fn test_adjust_scroll_selection_above_offset() {
        let mut state = ScrollableState::with_items(20);
        // Simulate scrolling down
        for _ in 0..10 {
            state.move_down();
        }
        // Manually set scroll offset (would need setter)
        // For now, test that adjust_scroll works when selection < offset
        state.move_first();
        state.adjust_scroll(5);
        assert_eq!(state.scroll_offset(), 0);
    }

    #[test]
    fn test_adjust_scroll_selection_below_visible_area() {
        let mut state = ScrollableState::with_items(20);
        for _ in 0..10 {
            state.move_down();
        }
        state.adjust_scroll(5);
        assert!(state.selected_index() >= state.scroll_offset());
        assert!(state.selected_index() < state.scroll_offset() + 5);
    }

    #[test]
    fn test_adjust_scroll_with_zero_visible_rows() {
        let mut state = ScrollableState::with_items(10);
        for _ in 0..5 {
            state.move_down();
        }
        let original_offset = state.scroll_offset();
        state.adjust_scroll(0);
        // Should not change when visible_rows is 0
        assert_eq!(state.scroll_offset(), original_offset);
    }

    #[test]
    fn test_adjust_scroll_keeps_selection_visible() {
        let mut state = ScrollableState::with_items(20);
        for _ in 0..15 {
            state.move_down();
        }
        state.adjust_scroll(10);
        // Selection should be within visible range
        assert!(state.selected_index() >= state.scroll_offset());
        assert!(state.selected_index() < state.scroll_offset() + 10);
    }

    // ============================================================================
    // SCROLL_UP / SCROLL_DOWN TESTS
    // ============================================================================

    #[test]
    fn test_scroll_up_decrements_offset() {
        let mut state = ScrollableState::with_items(20);
        // First move down to create scroll offset
        for _ in 0..10 {
            state.move_down();
        }
        state.adjust_scroll(5);
        let offset_before = state.scroll_offset();
        state.scroll_up();
        assert!(state.scroll_offset() < offset_before || state.scroll_offset() == 0);
    }

    #[test]
    fn test_scroll_down_increments_offset() {
        let mut state = ScrollableState::with_items(20);
        let offset_before = state.scroll_offset();
        state.scroll_down();
        assert!(state.scroll_offset() > offset_before);
    }

    // ============================================================================
    // COMBINED MOVE_AND_SCROLL TESTS
    // ============================================================================

    #[test]
    fn test_move_up_and_scroll() {
        let mut state = ScrollableState::with_items(20);
        for _ in 0..10 {
            state.move_down();
        }
        state.adjust_scroll(5);
        state.move_up_and_scroll(5);
        assert_eq!(state.selected_index(), 9);
        assert!(state.selected_index() >= state.scroll_offset());
    }

    #[test]
    fn test_move_down_and_scroll() {
        let mut state = ScrollableState::with_items(20);
        state.move_down_and_scroll(5);
        assert_eq!(state.selected_index(), 1);
        assert!(state.selected_index() < state.scroll_offset() + 5);
    }

    #[test]
    fn test_move_first_and_scroll() {
        let mut state = ScrollableState::with_items(20);
        for _ in 0..15 {
            state.move_down();
        }
        state.move_first_and_scroll(5);
        assert_eq!(state.selected_index(), 0);
        assert_eq!(state.scroll_offset(), 0);
    }

    #[test]
    fn test_move_last_and_scroll() {
        let mut state = ScrollableState::with_items(20);
        state.move_last_and_scroll(5);
        assert_eq!(state.selected_index(), 19);
        // Scroll offset should be adjusted so last item is visible
        assert!(state.selected_index() >= state.scroll_offset());
    }

    #[test]
    fn test_move_by_and_scroll_positive() {
        let mut state = ScrollableState::with_items(20);
        state.move_by_and_scroll(5, 5);
        assert_eq!(state.selected_index(), 5);
        assert!(state.selected_index() < state.scroll_offset() + 5);
    }

    #[test]
    fn test_move_by_and_scroll_negative() {
        let mut state = ScrollableState::with_items(20);
        for _ in 0..10 {
            state.move_down();
        }
        state.move_by_and_scroll(-3, 5);
        assert_eq!(state.selected_index(), 7);
        assert!(state.selected_index() >= state.scroll_offset());
    }

    // ============================================================================
    // ITEM_COUNT TESTS
    // ============================================================================

    #[test]
    fn test_set_item_count_updates_count() {
        let mut state = ScrollableState::with_items(10);
        state.set_item_count(20);
        assert_eq!(state.item_count(), 20);
    }

    #[test]
    fn test_set_item_count_zero() {
        let mut state = ScrollableState::with_items(10);
        state.set_item_count(0);
        assert_eq!(state.item_count(), 0);
    }

    // ============================================================================
    // RATATUI INTEGRATION TESTS
    // ============================================================================

    #[test]
    fn test_to_list_state_reflects_selection() {
        let mut state = ScrollableState::with_items(10);
        for _ in 0..5 {
            state.move_down();
        }
        let list_state = state.to_list_state();
        assert_eq!(list_state.selected(), Some(5));
    }

    #[test]
    fn test_to_list_state_at_zero() {
        let state = ScrollableState::with_items(10);
        let list_state = state.to_list_state();
        assert_eq!(list_state.selected(), Some(0));
    }
}

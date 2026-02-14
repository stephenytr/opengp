//! Reusable list selection widget with keyboard navigation.
//!
//! This module provides a generic `ListSelector<T>` that manages list selection state
//! for TUI widgets. It wraps navigation (wraps to top when at bottom, wraps to bottom
//! when at top) and provides a clean API for working with selectable lists.
//!
//! # Usage
//!
//! ```rust
//! use opengp::ui::widgets::ListSelector;
//!
//! let mut selector = ListSelector::new(vec!["apple", "banana", "cherry"]);
//! assert_eq!(selector.selected(), Some(&"apple"));
//!
//! selector.next();  // wraps to first
//! selector.previous();  // wraps to last
//! assert_eq!(selector.selected(), Some(&"cherry"));
//! ```
//!
//! # Integration with Ratatui
//!
//! The `ListSelector` uses `ratatui::widgets::TableState` for rendering integration:
//!
//! ```rust
//! use ratatui::widgets::Table;
//! use ratatui::Frame;
//! use opengp::ui::widgets::ListSelector;
//!
//! fn render_list(frame: &mut Frame, area: ratatui::layout::Rect) {
//!     let mut selector = ListSelector::new(vec!["a", "b", "c"]);
//!     let table = Table::new([], [])
//!         .highlight_symbol(">> ");
//!
//!     frame.render_stateful_widget(table, area, selector.state_mut());
//! }
//! ```

use ratatui::widgets::TableState;

/// Generic list selection state manager with wrapping navigation.
///
/// This struct manages the selection state for a list of items, providing
/// keyboard-like navigation methods that wrap around the list. It integrates
/// with Ratatui's `TableState` for rendering.
///
/// # Type Parameters
///
/// * `T` - The type of items in the list
///
/// # Notes
///
/// - Selection is automatically set to the first item when created with non-empty list
/// - Navigation wraps: `next()` at end wraps to start, `previous()` at start wraps to end
/// - Empty lists have no selection
#[derive(Debug, Clone)]
pub struct ListSelector<T> {
    items: Vec<T>,
    state: TableState,
}

impl<T> ListSelector<T> {
    /// Creates a new `ListSelector` with the given items.
    ///
    /// If the items slice is non-empty, the first item is automatically selected.
    ///
    /// # Arguments
    ///
    /// * `items` - A vector of items to manage selection for
    ///
    /// # Examples
    ///
    /// ```rust
    /// use opengp::ui::widgets::ListSelector;
    ///
    /// let selector = ListSelector::new(vec![1, 2, 3]);
    /// assert_eq!(selector.selected(), Some(&1));
    /// assert_eq!(selector.items().len(), 3);
    /// ```
    ///
    /// # Empty List
    ///
    /// ```rust
    /// use opengp::ui::widgets::ListSelector;
    ///
    /// let selector = ListSelector::<i32>::new(vec![]);
    /// assert_eq!(selector.selected(), None);
    /// ```
    pub fn new(items: Vec<T>) -> Self {
        let mut state = TableState::default();
        if !items.is_empty() {
            state.select(Some(0));
        }

        Self { items, state }
    }

    /// Moves selection to the next item, wrapping to the first item if at the end.
    ///
    /// # Behavior
    ///
    /// - If the list is empty, this is a no-op
    /// - If at the last item, wraps to the first item (index 0)
    /// - Otherwise, advances selection by one
    ///
    /// # Examples
    ///
    /// ```rust
    /// use opengp::ui::widgets::ListSelector;
    ///
    /// let mut selector = ListSelector::new(vec!["a", "b", "c"]);
    /// assert_eq!(selector.selected(), Some(&"a"));
    ///
    /// selector.next();
    /// assert_eq!(selector.selected(), Some(&"b"));
    ///
    /// selector.next();
    /// selector.next();  // wraps
    /// assert_eq!(selector.selected(), Some(&"a"));
    /// ```
    pub fn next(&mut self) {
        if self.items.is_empty() {
            return;
        }

        let current = self.state.selected().unwrap_or(0);
        let next = if current >= self.items.len() - 1 {
            0
        } else {
            current + 1
        };
        self.state.select(Some(next));
    }

    /// Moves selection to the previous item, wrapping to the last item if at the start.
    ///
    /// # Behavior
    ///
    /// - If the list is empty, this is a no-op
    /// - If at the first item, wraps to the last item
    /// - Otherwise, moves selection back by one
    ///
    /// # Examples
    ///
    /// ```rust
    /// use opengp::ui::widgets::ListSelector;
    ///
    /// let mut selector = ListSelector::new(vec!["a", "b", "c"]);
    /// assert_eq!(selector.selected(), Some(&"a"));
    ///
    /// selector.previous();  // wraps
    /// assert_eq!(selector.selected(), Some(&"c"));
    /// ```
    pub fn previous(&mut self) {
        if self.items.is_empty() {
            return;
        }

        let current = self.state.selected().unwrap_or(0);
        let prev = if current == 0 {
            self.items.len() - 1
        } else {
            current - 1
        };
        self.state.select(Some(prev));
    }

    /// Moves selection to the first item.
    ///
    /// # Behavior
    ///
    /// - If the list is empty, this is a no-op
    /// - Always selects index 0
    ///
    /// # Examples
    ///
    /// ```rust
    /// use opengp::ui::widgets::ListSelector;
    ///
    /// let mut selector = ListSelector::new(vec!["a", "b", "c"]);
    /// selector.next();
    /// selector.next();
    /// assert_eq!(selector.selected(), Some(&"c"));
    ///
    /// selector.select_first();
    /// assert_eq!(selector.selected(), Some(&"a"));
    /// ```
    pub fn select_first(&mut self) {
        if !self.items.is_empty() {
            self.state.select(Some(0));
        }
    }

    /// Moves selection to the last item.
    ///
    /// # Behavior
    ///
    /// - If the list is empty, this is a no-op
    /// - Always selects the last index
    ///
    /// # Examples
    ///
    /// ```rust
    /// use opengp::ui::widgets::ListSelector;
    ///
    /// let mut selector = ListSelector::new(vec!["a", "b", "c"]);
    /// assert_eq!(selector.selected(), Some(&"a"));
    ///
    /// selector.select_last();
    /// assert_eq!(selector.selected(), Some(&"c"));
    /// ```
    pub fn select_last(&mut self) {
        if !self.items.is_empty() {
            self.state.select(Some(self.items.len() - 1));
        }
    }

    /// Returns a reference to the currently selected item.
    ///
    /// # Returns
    ///
    /// * `Some(&T)` - Reference to the selected item
    /// * `None` - If the list is empty or no item is selected
    ///
    /// # Examples
    ///
    /// ```rust
    /// use opengp::ui::widgets::ListSelector;
    ///
    /// let selector = ListSelector::new(vec![1, 2, 3]);
    /// assert_eq!(selector.selected(), Some(&1));
    ///
    /// let empty: ListSelector<i32> = ListSelector::new(vec![]);
    /// assert_eq!(empty.selected(), None);
    /// ```
    pub fn selected(&self) -> Option<&T> {
        self.state.selected().and_then(|i| self.items.get(i))
    }

    /// Returns a reference to all items in the list.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use opengp::ui::widgets::ListSelector;
    ///
    /// let selector = ListSelector::new(vec!["a", "b", "c"]);
    /// assert_eq!(selector.items(), &["a", "b", "c"]);
    /// ```
    pub fn items(&self) -> &Vec<T> {
        &self.items
    }

    /// Returns a mutable reference to the underlying `TableState`.
    ///
    /// This is useful for rendering with Ratatui widgets that require
    /// a `TableState` reference.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use opengp::ui::widgets::ListSelector;
    /// use ratatui::widgets::Table;
    ///
    /// let mut selector = ListSelector::new(vec!["a", "b", "c"]);
    /// let state = selector.state_mut();
    /// ```
    pub fn state_mut(&mut self) -> &mut TableState {
        &mut self.state
    }

    /// Returns the number of items in the list.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use opengp::ui::widgets::ListSelector;
    ///
    /// let selector = ListSelector::new(vec!["a", "b", "c"]);
    /// assert_eq!(selector.len(), 3);
    ///
    /// let empty: ListSelector<i32> = ListSelector::new(vec![]);
    /// assert_eq!(empty.len(), 0);
    /// ```
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Returns `true` if the list contains no items.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use opengp::ui::widgets::ListSelector;
    ///
    /// let selector = ListSelector::new(vec!["a"]);
    /// assert!(!selector.is_empty());
    ///
    /// let empty: ListSelector<i32> = ListSelector::new(vec![]);
    /// assert!(empty.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Returns the index of the currently selected item.
    ///
    /// # Returns
    ///
    /// * `Some(usize)` - Index of the selected item
    /// * `None` - If the list is empty or no item is selected
    ///
    /// # Examples
    ///
    /// ```rust
    /// use opengp::ui::widgets::ListSelector;
    ///
    /// let selector = ListSelector::new(vec!["a", "b", "c"]);
    /// assert_eq!(selector.selected_index(), Some(0));
    /// ```
    pub fn selected_index(&self) -> Option<usize> {
        self.state.selected()
    }

    /// Clears the current selection.
    ///
    /// After calling this, `selected()` will return `None` until a new
    /// selection is made via navigation methods.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use opengp::ui::widgets::ListSelector;
    ///
    /// let mut selector = ListSelector::new(vec!["a", "b", "c"]);
    /// assert!(selector.selected().is_some());
    ///
    /// selector.clear_selection();
    /// assert!(selector.selected().is_none());
    /// ```
    pub fn clear_selection(&mut self) {
        self.state.select(None);
    }

    /// Replaces the items in the list, resetting selection to the first item.
    ///
    /// If the new items vector is empty, selection is cleared.
    ///
    /// # Arguments
    ///
    /// * `items` - New vector of items
    ///
    /// # Examples
    ///
    /// ```rust
    /// use opengp::ui::widgets::ListSelector;
    ///
    /// let mut selector = ListSelector::new(vec!["a", "b"]);
    /// selector.set_items(vec!["x", "y", "z"]);
    /// assert_eq!(selector.selected(), Some(&"x"));
    /// ```
    pub fn set_items(&mut self, items: Vec<T>) {
        self.items = items;
        if !self.items.is_empty() {
            self.state.select(Some(0));
        } else {
            self.state.select(None);
        }
    }
}

impl<T: Clone> ListSelector<T> {
    /// Returns a clone of the currently selected item.
    ///
    /// This is a convenience method when you need to extract the value
    /// rather than a reference.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use opengp::ui::widgets::ListSelector;
    ///
    /// let selector = ListSelector::new(vec![String::from("hello")]);
    /// let value = selector.selected_cloned();
    /// assert_eq!(value, Some(String::from("hello")));
    /// ```
    pub fn selected_cloned(&self) -> Option<T> {
        self.selected().cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // === Constructor Tests ===

    #[test]
    fn test_new_with_items_selects_first() {
        let selector = ListSelector::new(vec![1, 2, 3]);
        assert_eq!(selector.selected(), Some(&1));
    }

    #[test]
    fn test_new_with_empty_list_has_no_selection() {
        let selector: ListSelector<i32> = ListSelector::new(vec![]);
        assert_eq!(selector.selected(), None);
    }

    #[test]
    fn test_new_preserves_item_order() {
        let items = vec!["first", "second", "third"];
        let selector = ListSelector::new(items.clone());
        assert_eq!(selector.items(), &items);
    }

    // === Next Navigation Tests ===

    #[test]
    fn test_next_advances_selection() {
        let mut selector = ListSelector::new(vec!["a", "b", "c"]);
        selector.next();
        assert_eq!(selector.selected(), Some(&"b"));
    }

    #[test]
    fn test_next_wraps_to_first_at_end() {
        let mut selector = ListSelector::new(vec!["a", "b", "c"]);
        selector.next(); // b
        selector.next(); // c
        selector.next(); // wraps to a
        assert_eq!(selector.selected(), Some(&"a"));
    }

    #[test]
    fn test_next_on_empty_list_is_noop() {
        let mut selector: ListSelector<i32> = ListSelector::new(vec![]);
        selector.next();
        assert_eq!(selector.selected(), None);
    }

    // === Previous Navigation Tests ===

    #[test]
    fn test_previous_goes_back() {
        let mut selector = ListSelector::new(vec!["a", "b", "c"]);
        selector.next(); // b
        selector.previous();
        assert_eq!(selector.selected(), Some(&"a"));
    }

    #[test]
    fn test_previous_wraps_to_last_at_start() {
        let mut selector = ListSelector::new(vec!["a", "b", "c"]);
        selector.previous(); // wraps to c
        assert_eq!(selector.selected(), Some(&"c"));
    }

    #[test]
    fn test_previous_on_empty_list_is_noop() {
        let mut selector: ListSelector<i32> = ListSelector::new(vec![]);
        selector.previous();
        assert_eq!(selector.selected(), None);
    }

    // === Select First/Last Tests ===

    #[test]
    fn test_select_first() {
        let mut selector = ListSelector::new(vec!["a", "b", "c"]);
        selector.next();
        selector.next();
        selector.select_first();
        assert_eq!(selector.selected(), Some(&"a"));
    }

    #[test]
    fn test_select_last() {
        let mut selector = ListSelector::new(vec!["a", "b", "c"]);
        selector.select_last();
        assert_eq!(selector.selected(), Some(&"c"));
    }

    #[test]
    fn test_select_first_on_empty_list_is_noop() {
        let mut selector: ListSelector<i32> = ListSelector::new(vec![]);
        selector.select_first();
        assert_eq!(selector.selected(), None);
    }

    #[test]
    fn test_select_last_on_empty_list_is_noop() {
        let mut selector: ListSelector<i32> = ListSelector::new(vec![]);
        selector.select_last();
        assert_eq!(selector.selected(), None);
    }

    // === Wrapping Edge Cases ===

    #[test]
    fn test_wrap_single_item_list() {
        let mut selector = ListSelector::new(vec!["only"]);
        selector.next();
        assert_eq!(selector.selected(), Some(&"only"));
        selector.previous();
        assert_eq!(selector.selected(), Some(&"only"));
    }

    #[test]
    fn test_wrap_two_item_list() {
        let mut selector = ListSelector::new(vec!["a", "b"]);
        assert_eq!(selector.selected(), Some(&"a"));

        selector.next();
        assert_eq!(selector.selected(), Some(&"b"));

        selector.next();
        assert_eq!(selector.selected(), Some(&"a"));

        selector.previous();
        assert_eq!(selector.selected(), Some(&"b"));
    }

    // === Length and Empty Tests ===

    #[test]
    fn test_len() {
        let selector = ListSelector::new(vec![1, 2, 3, 4, 5]);
        assert_eq!(selector.len(), 5);
    }

    #[test]
    fn test_len_empty_list() {
        let selector: ListSelector<i32> = ListSelector::new(vec![]);
        assert_eq!(selector.len(), 0);
    }

    #[test]
    fn test_is_empty() {
        let empty: ListSelector<i32> = ListSelector::new(vec![]);
        assert!(empty.is_empty());

        let non_empty = ListSelector::new(vec![1]);
        assert!(!non_empty.is_empty());
    }

    // === Index Tests ===

    #[test]
    fn test_selected_index() {
        let selector = ListSelector::new(vec!["a", "b", "c"]);
        assert_eq!(selector.selected_index(), Some(0));
    }

    #[test]
    fn test_selected_index_after_navigation() {
        let mut selector = ListSelector::new(vec!["a", "b", "c"]);
        selector.next();
        selector.next();
        assert_eq!(selector.selected_index(), Some(2));
    }

    #[test]
    fn test_selected_index_empty_list() {
        let selector: ListSelector<i32> = ListSelector::new(vec![]);
        assert_eq!(selector.selected_index(), None);
    }

    // === Clear Selection Tests ===

    #[test]
    fn test_clear_selection() {
        let mut selector = ListSelector::new(vec!["a", "b", "c"]);
        selector.clear_selection();
        assert_eq!(selector.selected(), None);
        assert_eq!(selector.selected_index(), None);
    }

    // === Set Items Tests ===

    #[test]
    fn test_set_items() {
        let mut selector = ListSelector::new(vec!["a"]);
        selector.set_items(vec!["x", "y"]);
        assert_eq!(selector.selected(), Some(&"x"));
        assert_eq!(selector.len(), 2);
    }

    #[test]
    fn test_set_items_resets_to_first() {
        let mut selector = ListSelector::new(vec!["a", "b", "c"]);
        selector.next();
        selector.next(); // at c
        selector.set_items(vec!["x"]);
        assert_eq!(selector.selected(), Some(&"x"));
    }

    #[test]
    fn test_set_items_empty_clears_selection() {
        let mut selector = ListSelector::new(vec!["a"]);
        selector.set_items(vec![]);
        assert_eq!(selector.selected(), None);
    }

    // === Selected Cloned Tests ===

    #[test]
    fn test_selected_cloned() {
        let selector = ListSelector::new(vec![String::from("hello")]);
        let cloned = selector.selected_cloned();
        assert_eq!(cloned, Some(String::from("hello")));
    }

    #[test]
    fn test_selected_cloned_empty_list() {
        let selector: ListSelector<String> = ListSelector::new(vec![]);
        assert_eq!(selector.selected_cloned(), None);
    }

    // === State Mut Tests ===

    #[test]
    fn test_state_mut_returns_table_state() {
        let mut selector = ListSelector::new(vec!["a", "b"]);
        let _state = selector.state_mut();
        // Just verify it compiles and returns a mutable reference
    }

    // === Complex Navigation Tests ===

    #[test]
    fn test_full_navigation_cycle() {
        let mut selector = ListSelector::new(vec!["a", "b", "c", "d"]);

        // Start at a
        assert_eq!(selector.selected(), Some(&"a"));

        // Navigate forward: a -> b -> c -> d -> a (wrap)
        selector.next(); // b
        selector.next(); // c
        selector.next(); // d
        selector.next(); // wraps to a

        // Navigate backward: a -> d (wrap) -> c -> b -> a
        selector.previous(); // wraps to d
        selector.previous(); // c
        selector.previous(); // b
        selector.previous(); // a

        assert_eq!(selector.selected(), Some(&"a"));
    }

    #[test]
    fn test_mixed_first_last_navigation() {
        let mut selector = ListSelector::new(vec!["a", "b", "c", "d", "e"]);

        selector.select_last(); // e
        assert_eq!(selector.selected(), Some(&"e"));

        selector.select_first(); // a
        assert_eq!(selector.selected(), Some(&"a"));

        selector.select_last(); // e
        assert_eq!(selector.selected(), Some(&"e"));
    }
}

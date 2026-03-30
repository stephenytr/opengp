//! Navigation standardization contract for all list-like components
//!
//! This module defines the standard navigation behavior that ALL scrollable/list components
//! should follow to provide a consistent user experience.
//!
//! ## Navigation Contract
//!
//! ### Keyboard Navigation (Vim + Arrow Keys)
//! - **Up**: Arrow Up or 'k' - Move selection up one row
//! - **Down**: Arrow Down or 'j' - Move selection down one row
//! - **Home**: Home key - Jump to first item
//! - **End**: End key - Jump to last item
//! - **Page Up**: PageUp key - Jump up by visible height
//! - **Page Down**: PageDown key - Jump down by visible height
//!
//! ### Mouse Navigation
//! - **Scroll Up/Down**: Mouse wheel for viewport scrolling
//! - **Click to Select**: Left-click on list item to select it
//!
//! ### Scroll Management
//! - Use `ScrollableState` for ALL scrollable components (no manual scroll_offset)
//! - Selection is automatically kept visible within viewport
//! - Scroll offset adjusts to follow selection movement
//!
//! ## Implementation Pattern
//!
//! ```ignore
//! use crate::ui::widgets::{ScrollableState, list_nav};
//! use crossterm::event::{KeyEvent, MouseEvent};
//!
//! pub struct MyListComponent {
//!     items: Vec<Item>,
//!     scrollable: ScrollableState,  // Use this, not manual scroll_offset
//! }
//!
//! impl MyListComponent {
//!     pub fn handle_key(&mut self, key: KeyEvent, visible_rows: usize) {
//!         if let Some(action) = list_nav::list_handle_key(key, &mut self.scrollable, visible_rows) {
//!             // Handle action (typically: trigger render update)
//!         }
//!     }
//!
//!     pub fn handle_mouse(&mut self, mouse: MouseEvent, area: Rect, visible_rows: usize) {
//!         let header_height = 1; // e.g., for column header
//!         if let Some(action) = list_nav::list_handle_mouse(
//!             mouse,
//!             area,
//!             header_height,
//!             &mut self.scrollable,
//!             visible_rows,
//!         ) {
//!             // Handle action
//!         }
//!     }
//! }
//! ```
//!
//! ## ScrollableState API
//!
//! Core methods:
//! - `move_up()`, `move_down()` - Change selection
//! - `move_first()`, `move_last()` - Jump to bounds
//! - `move_by(offset)` - Signed offset movement
//! - `adjust_scroll(visible_rows)` - Keep selection visible
//! - `scroll_up()`, `scroll_down()` - Scroll viewport independent of selection
//! - `move_*_and_scroll()` - Combined movement + scroll adjustment
//!
//! Properties:
//! - `selected_index()` - Current selection position
//! - `scroll_offset()` - Viewport top position
//! - `item_count()` - Total items
//! - `set_item_count(count)` - Update total when list changes
//!
//! ## Common Patterns
//!
//! ### Pagination (e.g., consultation list)
//! - Store pagination state separately (current page, per-page count)
//! - Update `scrollable.set_item_count()` when fetching new page
//! - Keep ScrollableState as single-page viewport
//! - Map selected_index + page offset to actual item
//!
//! ### Filtering
//! - Filter before setting item count
//! - Reset scrollable to start (selected_index=0) on filter change
//! - Example: `scrollable.move_first_and_scroll(visible_rows)`
//!
//! ### Forms with Scrollable Fields
//! - Use ScrollableState for field list (not form offset)
//! - Render form fields starting from `scroll_offset()`
//! - Highlight `selected_index()` field

pub mod scroll_management {
    //! Best practices for scroll and selection management

    /// When selection changes, call `adjust_scroll(visible_rows)` to ensure visibility.
    /// This prevents the selected item from scrolling out of view.
    pub const PRINCIPLE_KEEP_SELECTION_VISIBLE: &str =
        "After any selection change, call adjust_scroll with the visible_rows";

    /// All components must use the same key bindings for consistency.
    /// Use `list_nav::list_handle_key()` which provides the standard set.
    pub const PRINCIPLE_VIM_BINDINGS: &str =
        "Vim bindings (j/k for down/up) must be used in ALL list components";

    /// ScrollableState must be the single source of truth for scroll state.
    /// Never maintain manual scroll_offset in component state.
    pub const PRINCIPLE_SINGLE_SOURCE_OF_TRUTH: &str =
        "Use ScrollableState for all scrolling; never duplicate scroll_offset";

    /// When component data changes (e.g., filtered list), reset scrolling to top.
    pub const PRINCIPLE_RESET_ON_DATA_CHANGE: &str =
        "When items change, reset scrollable: move_first_and_scroll()";
}

pub mod migration_guide {
    //! Guide for refactoring components from manual scroll_offset to ScrollableState

    /// Step-by-step refactoring checklist:
    /// 1. Replace `scroll_offset: usize` with `scrollable: ScrollableState` in struct
    /// 2. In constructors, initialize: `scrollable: ScrollableState::with_items(count)`
    /// 3. When item count changes, call: `scrollable.set_item_count(new_count)`
    /// 4. Replace all scroll_offset accesses:
    ///    - For rendering: use `scrollable.scroll_offset()` (read-only)
    ///    - For updates: use `scrollable.move_*()` methods
    /// 5. Replace key handling:
    ///    - Old: manual if/match on KeyCode
    ///    - New: call `list_nav::list_handle_key(key, &mut scrollable, visible_rows)`
    /// 6. After key handling: call `scrollable.adjust_scroll(visible_rows)` if needed
    /// 7. Update render to skip `scroll_offset` lines: use `.skip(scrollable.scroll_offset())`
    pub const MIGRATION_STEPS: &str = "See guide above";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_documentation_exists() {
        // This test verifies the documentation constants are accessible
        assert!(!scroll_management::PRINCIPLE_KEEP_SELECTION_VISIBLE.is_empty());
        assert!(!scroll_management::PRINCIPLE_VIM_BINDINGS.is_empty());
        assert!(!migration_guide::MIGRATION_STEPS.is_empty());
    }
}

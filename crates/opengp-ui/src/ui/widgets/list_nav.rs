//! Shared list keyboard and mouse navigation helpers
//!
//! Provides reusable functions for handling standard list navigation:
//! - Arrow keys (up/down), vim keys (j/k), Home/End, PageUp/PageDown
//! - Mouse wheel scrolling and click-to-select
//!
//! Works with `ScrollableState` for managing selection and scroll offset.

use crossterm::event::{KeyCode, KeyEvent, MouseEvent, MouseEventKind};
use ratatui::layout::{Position, Rect};

use crate::ui::widgets::scrollable::ScrollableState;

/// Actions that can result from list navigation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListNavAction {
    /// Selection or scroll position changed
    SelectionChanged,
    /// Custom index selected (e.g., via mouse click)
    SelectIndex(usize),
}

/// Handles keyboard navigation for a scrollable list
///
/// Processes arrow keys, vim keys, Home/End, PageUp/PageDown.
/// Updates `scrollable` state in-place and returns an optional action.
///
/// # Arguments
/// * `key` - The keyboard event
/// * `scrollable` - Mutable reference to ScrollableState
/// * `visible_rows` - Number of visible rows in the viewport
///
/// # Returns
/// `Some(ListNavAction)` if navigation occurred, `None` otherwise
pub fn list_handle_key(
    key: KeyEvent,
    scrollable: &mut ScrollableState,
    visible_rows: usize,
) -> Option<ListNavAction> {
    use crossterm::event::KeyEventKind;

    // Ignore non-press key events
    if key.kind != KeyEventKind::Press {
        return None;
    }

    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            scrollable.move_up_and_scroll(visible_rows);
            Some(ListNavAction::SelectionChanged)
        }
        KeyCode::Down | KeyCode::Char('j') => {
            scrollable.move_down_and_scroll(visible_rows);
            Some(ListNavAction::SelectionChanged)
        }
        KeyCode::Home => {
            scrollable.move_first_and_scroll(visible_rows);
            Some(ListNavAction::SelectionChanged)
        }
        KeyCode::End => {
            scrollable.move_last_and_scroll(visible_rows);
            Some(ListNavAction::SelectionChanged)
        }
        KeyCode::PageUp => {
            scrollable.move_by_and_scroll(-(visible_rows as isize), visible_rows);
            Some(ListNavAction::SelectionChanged)
        }
        KeyCode::PageDown => {
            scrollable.move_by_and_scroll(visible_rows as isize, visible_rows);
            Some(ListNavAction::SelectionChanged)
        }
        _ => None,
    }
}

/// Handles mouse navigation for a scrollable list
///
/// Processes mouse wheel (scroll up/down) and left-click to select.
/// Updates `scrollable` state in-place and returns an optional action.
///
/// # Arguments
/// * `mouse` - The mouse event
/// * `area` - The rendering area of the list
/// * `header_height` - Number of rows reserved for header (e.g., 1 for column titles)
/// * `scrollable` - Mutable reference to ScrollableState
/// * `visible_rows` - Number of visible rows in the viewport
///
/// # Returns
/// `Some(ListNavAction)` if navigation occurred, `None` otherwise
pub fn list_handle_mouse(
    mouse: MouseEvent,
    area: Rect,
    header_height: u16,
    scrollable: &mut ScrollableState,
    visible_rows: usize,
) -> Option<ListNavAction> {
    // Handle mouse wheel scrolling
    if let MouseEventKind::ScrollUp = mouse.kind {
        for _ in 0..3 {
            scrollable.scroll_up();
        }
        return Some(ListNavAction::SelectionChanged);
    }
    if let MouseEventKind::ScrollDown = mouse.kind {
        let max_scroll = scrollable.item_count().saturating_sub(visible_rows);
        for _ in 0..3 {
            if scrollable.scroll_offset() < max_scroll {
                scrollable.scroll_down();
            }
        }
        return Some(ListNavAction::SelectionChanged);
    }

    // Only handle left-click
    if mouse.kind != MouseEventKind::Up(crossterm::event::MouseButton::Left) {
        return None;
    }

    // Check if click is within the list area
    if !area.contains(Position::new(mouse.column, mouse.row)) {
        return None;
    }

    // Check if click is in the header area (skip it)
    if mouse.row < area.y + header_height {
        return None;
    }

    // Calculate which row was clicked (relative to list content area)
    let row_index = (mouse.row - area.y - header_height) as usize;

    // Account for scroll offset to get actual item index
    let actual_index = scrollable.scroll_offset() + row_index;

    // Only select if within list bounds
    if actual_index < scrollable.item_count() {
        // Move selection to clicked item
        let current_index = scrollable.selected_index();
        let offset = actual_index as isize - current_index as isize;
        scrollable.move_by(offset);
        Some(ListNavAction::SelectionChanged)
    } else {
        None
    }
}

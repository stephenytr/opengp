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

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::MouseButton;

    // ============================================================================
    // KEYBOARD NAVIGATION TESTS
    // ============================================================================

    #[test]
    fn test_list_handle_key_up_arrow_moves_selection_up() {
        let mut scrollable = ScrollableState::with_items(10);
        scrollable.move_down();
        scrollable.move_down();

        let key = KeyEvent::new(KeyCode::Up, crossterm::event::KeyModifiers::NONE);

        let action = list_handle_key(key, &mut scrollable, 5);
        assert_eq!(action, Some(ListNavAction::SelectionChanged));
        assert_eq!(scrollable.selected_index(), 1);
    }

    #[test]
    fn test_list_handle_key_k_moves_selection_up() {
        let mut scrollable = ScrollableState::with_items(10);
        scrollable.move_down();
        scrollable.move_down();

        let key = KeyEvent::new(KeyCode::Char('k'), crossterm::event::KeyModifiers::NONE);

        let action = list_handle_key(key, &mut scrollable, 5);
        assert_eq!(action, Some(ListNavAction::SelectionChanged));
        assert_eq!(scrollable.selected_index(), 1);
    }

    #[test]
    fn test_list_handle_key_down_arrow_moves_selection_down() {
        let mut scrollable = ScrollableState::with_items(10);

        let key = KeyEvent::new(KeyCode::Down, crossterm::event::KeyModifiers::NONE);

        let action = list_handle_key(key, &mut scrollable, 5);
        assert_eq!(action, Some(ListNavAction::SelectionChanged));
        assert_eq!(scrollable.selected_index(), 1);
    }

    #[test]
    fn test_list_handle_key_j_moves_selection_down() {
        let mut scrollable = ScrollableState::with_items(10);

        let key = KeyEvent::new(KeyCode::Char('j'), crossterm::event::KeyModifiers::NONE);

        let action = list_handle_key(key, &mut scrollable, 5);
        assert_eq!(action, Some(ListNavAction::SelectionChanged));
        assert_eq!(scrollable.selected_index(), 1);
    }

    #[test]
    fn test_list_handle_key_home_moves_to_first() {
        let mut scrollable = ScrollableState::with_items(10);
        for _ in 0..5 {
            scrollable.move_down();
        }

        let key = KeyEvent::new(KeyCode::Home, crossterm::event::KeyModifiers::NONE);

        let action = list_handle_key(key, &mut scrollable, 5);
        assert_eq!(action, Some(ListNavAction::SelectionChanged));
        assert_eq!(scrollable.selected_index(), 0);
    }

    #[test]
    fn test_list_handle_key_end_moves_to_last() {
        let mut scrollable = ScrollableState::with_items(10);

        let key = KeyEvent::new(KeyCode::End, crossterm::event::KeyModifiers::NONE);

        let action = list_handle_key(key, &mut scrollable, 5);
        assert_eq!(action, Some(ListNavAction::SelectionChanged));
        assert_eq!(scrollable.selected_index(), 9);
    }

    #[test]
    fn test_list_handle_key_page_up_moves_by_visible_rows() {
        let mut scrollable = ScrollableState::with_items(20);
        for _ in 0..10 {
            scrollable.move_down();
        }

        let key = KeyEvent::new(KeyCode::PageUp, crossterm::event::KeyModifiers::NONE);

        let action = list_handle_key(key, &mut scrollable, 5);
        assert_eq!(action, Some(ListNavAction::SelectionChanged));
        assert_eq!(scrollable.selected_index(), 5);
    }

    #[test]
    fn test_list_handle_key_page_down_moves_by_visible_rows() {
        let mut scrollable = ScrollableState::with_items(20);

        let key = KeyEvent::new(KeyCode::PageDown, crossterm::event::KeyModifiers::NONE);

        let action = list_handle_key(key, &mut scrollable, 5);
        assert_eq!(action, Some(ListNavAction::SelectionChanged));
        assert_eq!(scrollable.selected_index(), 5);
    }

    #[test]
    fn test_list_handle_key_non_navigation_key_returns_none() {
        let mut scrollable = ScrollableState::with_items(10);

        let key = KeyEvent::new(KeyCode::Char('a'), crossterm::event::KeyModifiers::NONE);

        let action = list_handle_key(key, &mut scrollable, 5);
        assert_eq!(action, None);
        assert_eq!(scrollable.selected_index(), 0);
    }

    #[test]
    fn test_list_handle_key_release_event_returns_none() {
        use crossterm::event::KeyEventKind;

        let mut scrollable = ScrollableState::with_items(10);

        let mut key = KeyEvent::new(KeyCode::Up, crossterm::event::KeyModifiers::NONE);
        key.kind = KeyEventKind::Release;

        let action = list_handle_key(key, &mut scrollable, 5);
        assert_eq!(action, None);
        assert_eq!(scrollable.selected_index(), 0);
    }

    // ============================================================================
    // MOUSE NAVIGATION TESTS
    // ============================================================================

    #[test]
    fn test_list_handle_mouse_scroll_up_reduces_offset() {
        let mut scrollable = ScrollableState::with_items(20);
        scrollable.scroll_down();
        scrollable.scroll_down();
        scrollable.scroll_down();
        let initial_offset = scrollable.scroll_offset();

        let mouse = MouseEvent {
            kind: MouseEventKind::ScrollUp,
            column: 10,
            row: 5,
            modifiers: crossterm::event::KeyModifiers::NONE,
        };

        let area = Rect {
            x: 0,
            y: 0,
            width: 20,
            height: 10,
        };

        let action = list_handle_mouse(mouse, area, 1, &mut scrollable, 5);
        assert_eq!(action, Some(ListNavAction::SelectionChanged));
        assert!(scrollable.scroll_offset() < initial_offset);
    }

    #[test]
    fn test_list_handle_mouse_scroll_down_increases_offset() {
        let mut scrollable = ScrollableState::with_items(20);

        let mouse = MouseEvent {
            kind: MouseEventKind::ScrollDown,
            column: 10,
            row: 5,
            modifiers: crossterm::event::KeyModifiers::NONE,
        };

        let area = Rect {
            x: 0,
            y: 0,
            width: 20,
            height: 10,
        };

        let action = list_handle_mouse(mouse, area, 1, &mut scrollable, 5);
        assert_eq!(action, Some(ListNavAction::SelectionChanged));
        assert!(scrollable.scroll_offset() > 0);
    }

    #[test]
    fn test_list_handle_mouse_scroll_down_capped_at_max() {
        let mut scrollable = ScrollableState::with_items(10);

        let mouse = MouseEvent {
            kind: MouseEventKind::ScrollDown,
            column: 10,
            row: 5,
            modifiers: crossterm::event::KeyModifiers::NONE,
        };

        let area = Rect {
            x: 0,
            y: 0,
            width: 20,
            height: 10,
        };

        // Scroll down multiple times
        for _ in 0..10 {
            let _ = list_handle_mouse(mouse, area, 1, &mut scrollable, 5);
        }

        // Should be capped at max_scroll (10 - 5 = 5)
        assert!(scrollable.scroll_offset() <= 5);
    }

    #[test]
    fn test_list_handle_mouse_left_click_selects_row() {
        let mut scrollable = ScrollableState::with_items(20);

        let mouse = MouseEvent {
            kind: MouseEventKind::Up(MouseButton::Left),
            column: 10,
            row: 5,
            modifiers: crossterm::event::KeyModifiers::NONE,
        };

        let area = Rect {
            x: 0,
            y: 0,
            width: 20,
            height: 10,
        };

        let action = list_handle_mouse(mouse, area, 1, &mut scrollable, 5);
        assert_eq!(action, Some(ListNavAction::SelectionChanged));
        assert_eq!(scrollable.selected_index(), 4);
    }

    #[test]
    fn test_list_handle_mouse_left_click_accounts_for_scroll_offset() {
        let mut scrollable = ScrollableState::with_items(20);
        scrollable.scroll_down();
        scrollable.scroll_down();
        scrollable.scroll_down();

        let mouse = MouseEvent {
            kind: MouseEventKind::Up(MouseButton::Left),
            column: 10,
            row: 5,
            modifiers: crossterm::event::KeyModifiers::NONE,
        };

        let area = Rect {
            x: 0,
            y: 0,
            width: 20,
            height: 10,
        };

        let action = list_handle_mouse(mouse, area, 1, &mut scrollable, 5);
        assert_eq!(action, Some(ListNavAction::SelectionChanged));
        assert_eq!(scrollable.selected_index(), 7);
    }

    #[test]
    fn test_list_handle_mouse_left_click_outside_area_returns_none() {
        let mut scrollable = ScrollableState::with_items(20);

        let mouse = MouseEvent {
            kind: MouseEventKind::Up(MouseButton::Left),
            column: 50,
            row: 5,
            modifiers: crossterm::event::KeyModifiers::NONE,
        };

        let area = Rect {
            x: 0,
            y: 0,
            width: 20,
            height: 10,
        };

        let action = list_handle_mouse(mouse, area, 1, &mut scrollable, 5);
        assert_eq!(action, None);
        assert_eq!(scrollable.selected_index(), 0);
    }

    #[test]
    fn test_list_handle_mouse_left_click_on_header_returns_none() {
        let mut scrollable = ScrollableState::with_items(20);

        let mouse = MouseEvent {
            kind: MouseEventKind::Up(MouseButton::Left),
            column: 10,
            row: 0,
            modifiers: crossterm::event::KeyModifiers::NONE,
        };

        let area = Rect {
            x: 0,
            y: 0,
            width: 20,
            height: 10,
        };

        let action = list_handle_mouse(mouse, area, 1, &mut scrollable, 5);
        assert_eq!(action, None);
        assert_eq!(scrollable.selected_index(), 0);
    }

    #[test]
    fn test_list_handle_mouse_right_click_returns_none() {
        let mut scrollable = ScrollableState::with_items(20);

        let mouse = MouseEvent {
            kind: MouseEventKind::Up(MouseButton::Right),
            column: 10,
            row: 5,
            modifiers: crossterm::event::KeyModifiers::NONE,
        };

        let area = Rect {
            x: 0,
            y: 0,
            width: 20,
            height: 10,
        };

        let action = list_handle_mouse(mouse, area, 1, &mut scrollable, 5);
        assert_eq!(action, None);
        assert_eq!(scrollable.selected_index(), 0);
    }
}

//! Simple mouse interaction utilities
//!
//! This module provides minimal, tested helpers for mouse interactions.
//! No complex traits - just functions that work.

use crossterm::event::{MouseEvent, MouseEventKind};
use ratatui::layout::Rect;

/// Check if a mouse event is within a rectangle
pub fn is_event_in_rect(event: &MouseEvent, rect: Rect) -> bool {
    event.column >= rect.x
        && event.column < rect.x + rect.width
        && event.row >= rect.y
        && event.row < rect.y + rect.height
}

/// Check if mouse event is a click (Down event)
pub fn is_click(event: &MouseEvent) -> bool {
    matches!(event.kind, MouseEventKind::Down(_))
}

/// Check if mouse event is a scroll up
pub fn is_scroll_up(event: &MouseEvent) -> bool {
    matches!(event.kind, MouseEventKind::ScrollUp)
}

/// Check if mouse event is a scroll down
pub fn is_scroll_down(event: &MouseEvent) -> bool {
    matches!(event.kind, MouseEventKind::ScrollDown)
}

/// Calculate row index from mouse Y position within a table
///
/// Returns None if click is outside table bounds or on header.
///
/// # Arguments
/// * `event` - The mouse event
/// * `table_rect` - The rectangle of the entire table widget
/// * `header_height` - Height of the header in rows (usually 1)
/// * `row_count` - Total number of data rows
pub fn table_row_from_click(
    event: &MouseEvent,
    table_rect: Rect,
    header_height: u16,
    row_count: usize,
) -> Option<usize> {
    if !is_event_in_rect(event, table_rect) {
        return None;
    }

    // Account for border (1) + header
    let content_start = table_rect.y + 1 + header_height;
    let relative_y = event.row.saturating_sub(content_start);

    let row_index = relative_y as usize;
    if row_index < row_count {
        Some(row_index)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyModifiers, MouseButton, MouseEventKind};

    fn create_mouse_event(kind: MouseEventKind, column: u16, row: u16) -> MouseEvent {
        MouseEvent {
            kind,
            column,
            row,
            modifiers: KeyModifiers::empty(),
        }
    }

    #[test]
    fn test_is_event_in_rect() {
        let rect = Rect::new(10, 10, 20, 10);

        // Inside
        let event = create_mouse_event(MouseEventKind::Down(MouseButton::Left), 15, 15);
        assert!(is_event_in_rect(&event, rect));

        // Outside - left
        let event = create_mouse_event(MouseEventKind::Down(MouseButton::Left), 5, 15);
        assert!(!is_event_in_rect(&event, rect));

        // Outside - right
        let event = create_mouse_event(MouseEventKind::Down(MouseButton::Left), 35, 15);
        assert!(!is_event_in_rect(&event, rect));

        // On edge (exclusive)
        let event = create_mouse_event(MouseEventKind::Down(MouseButton::Left), 30, 15);
        assert!(!is_event_in_rect(&event, rect));
    }

    #[test]
    fn test_is_click() {
        let click = create_mouse_event(MouseEventKind::Down(MouseButton::Left), 0, 0);
        assert!(is_click(&click));

        let up = create_mouse_event(MouseEventKind::Up(MouseButton::Left), 0, 0);
        assert!(!is_click(&up));

        let moved = create_mouse_event(MouseEventKind::Moved, 0, 0);
        assert!(!is_click(&moved));
    }

    #[test]
    fn test_scroll_detection() {
        let scroll_up = create_mouse_event(MouseEventKind::ScrollUp, 0, 0);
        assert!(is_scroll_up(&scroll_up));
        assert!(!is_scroll_down(&scroll_up));

        let scroll_down = create_mouse_event(MouseEventKind::ScrollDown, 0, 0);
        assert!(is_scroll_down(&scroll_down));
        assert!(!is_scroll_up(&scroll_down));
    }

    #[test]
    fn test_table_row_from_click() {
        let table_rect = Rect::new(0, 0, 50, 20);

        // Click on first data row (after header at row 1, border at row 0)
        // table_rect.y=0, border=1, header=1, so content starts at row 2
        let event = create_mouse_event(MouseEventKind::Down(MouseButton::Left), 10, 2);
        assert_eq!(table_row_from_click(&event, table_rect, 1, 10), Some(0));

        // Click on third data row (row 4)
        let event = create_mouse_event(MouseEventKind::Down(MouseButton::Left), 10, 4);
        assert_eq!(table_row_from_click(&event, table_rect, 1, 10), Some(2));

        // Click outside table
        let event = create_mouse_event(MouseEventKind::Down(MouseButton::Left), 10, 25);
        assert_eq!(table_row_from_click(&event, table_rect, 1, 10), None);

        // Click beyond row count (clicking row 3 maps to row_index 1, but only 1 row exists)
        let event = create_mouse_event(MouseEventKind::Down(MouseButton::Left), 10, 3);
        assert_eq!(table_row_from_click(&event, table_rect, 1, 1), None);
    }
}

//! Debug helper for mouse coordinate verification
//!
//! Use this to verify hit regions align with rendered positions.
//! Enable with environment variable: MOUSE_DEBUG=1

use crossterm::event::MouseEvent;
use ratatui::layout::Rect;
use tracing::debug;

/// Check if debug logging is enabled via MOUSE_DEBUG environment variable
fn is_debug_enabled() -> bool {
    std::env::var("MOUSE_DEBUG").is_ok()
}

/// Log a hit region with its calculated rectangle
///
/// Only logs when MOUSE_DEBUG environment variable is set.
pub fn log_hit_region(name: &str, rect: Rect, action: &str) {
    if is_debug_enabled() {
        debug!(
            "HIT_REGION: {} at x={},y={},w={},h={} -> {}",
            name, rect.x, rect.y, rect.width, rect.height, action
        );
    }
}

/// Log mouse event coordinates
///
/// Only logs when MOUSE_DEBUG environment variable is set.
pub fn log_mouse_event(event: &MouseEvent, context: &str) {
    if is_debug_enabled() {
        debug!(
            "MOUSE_EVENT: {:?} at ({}, {}) in {}",
            event.kind, event.column, event.row, context
        );
    }
}

/// Verify a coordinate is within a rectangle
///
/// Returns true if (x, y) is within the bounds of rect.
/// Also logs the hit test when MOUSE_DEBUG is enabled.
pub fn is_in_rect(x: u16, y: u16, rect: Rect) -> bool {
    let result = x >= rect.x && x < rect.x + rect.width && y >= rect.y && y < rect.y + rect.height;

    if is_debug_enabled() {
        debug!(
            "HIT_TEST: ({}, {}) in rect [{},{},{},{}] = {}",
            x, y, rect.x, rect.y, rect.width, rect.height, result
        );
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_in_rect_inside() {
        let rect = Rect::new(10, 10, 20, 10);
        assert!(is_in_rect(15, 15, rect));
        assert!(is_in_rect(10, 10, rect)); // Top-left corner
        assert!(is_in_rect(29, 19, rect)); // Bottom-right corner (exclusive)
    }

    #[test]
    fn test_is_in_rect_outside() {
        let rect = Rect::new(10, 10, 20, 10);
        assert!(!is_in_rect(5, 15, rect)); // Left of rect
        assert!(!is_in_rect(35, 15, rect)); // Right of rect
        assert!(!is_in_rect(15, 5, rect)); // Above rect
        assert!(!is_in_rect(15, 25, rect)); // Below rect
        assert!(!is_in_rect(30, 15, rect)); // On right edge (exclusive)
        assert!(!is_in_rect(15, 20, rect)); // On bottom edge (exclusive)
    }

    #[test]
    fn test_is_in_rect_zero_size() {
        let rect = Rect::new(10, 10, 0, 10);
        assert!(!is_in_rect(10, 15, rect)); // Zero width means nothing is inside

        let rect = Rect::new(10, 10, 20, 0);
        assert!(!is_in_rect(15, 10, rect)); // Zero height means nothing is inside
    }
}

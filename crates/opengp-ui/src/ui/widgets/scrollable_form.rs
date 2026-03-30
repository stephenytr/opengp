//! ScrollableFormState - Manages scrolling for form content that exceeds viewport height
//!
//! Provides vertical scrolling functionality for forms when content is larger than the visible area.
//! Automatically handles focus changes to keep focused fields visible.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;

use crate::ui::theme::Theme;

/// Manages scrolling state for form content
///
/// Tracks scroll offset (how many rows scrolled down) and total content height.
/// Provides methods for manual scrolling and automatic scroll-to-focus.
#[derive(Clone, Debug)]
pub struct ScrollableFormState {
    pub scroll_offset: u16,
    pub total_content_height: u16,
}

impl ScrollableFormState {
    /// Creates a new ScrollableFormState at top position
    pub fn new() -> Self {
        Self {
            scroll_offset: 0,
            total_content_height: 0,
        }
    }

    /// Scrolls up by one row (decrements offset, clamped to 0)
    pub fn scroll_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
        }
    }

    /// Scrolls down by one row
    pub fn scroll_down(&mut self) {
        self.scroll_offset += 1;
    }

    /// Adjusts scroll offset to keep a field visible
    ///
    /// # Arguments
    /// * `field_y` - Absolute y position of the field's top
    /// * `field_height` - Height of the field in rows
    /// * `visible_height` - Height of the visible viewport (form inner area)
    ///
    /// If field is above current viewport (field_y < scroll_offset), scrolls up.
    /// If field is below viewport, scrolls down to show it.
    pub fn scroll_to_field(&mut self, field_y: u16, field_height: u16, visible_height: u16) {
        if visible_height == 0 {
            return;
        }

        let field_bottom = field_y + field_height;
        let viewport_bottom = self.scroll_offset + visible_height;

        // Field is above visible area - scroll up to show it
        if field_y < self.scroll_offset {
            self.scroll_offset = field_y;
        }
        // Field is below visible area - scroll down to show it
        else if field_bottom > viewport_bottom {
            self.scroll_offset = field_bottom.saturating_sub(visible_height);
        }
    }

    /// Returns adjusted y position for rendering, accounting for scroll offset
    ///
    /// Subtracts scroll_offset from the y coordinate.
    /// If result is negative (field is above viewport), it will be off-screen.
    pub fn apply_offset(&self, y: i32) -> i32 {
        y - self.scroll_offset as i32
    }

    /// Renders a simple scrollbar on the right edge of the area
    ///
    /// Shows a vertical bar indicator that represents the current scroll position.
    /// Only visible when content exceeds viewport height.
    pub fn render_scrollbar(&self, area: Rect, buf: &mut Buffer, theme: &Theme) {
        if area.height == 0 || self.total_content_height <= area.height {
            return; // No scrollbar needed
        }

        let scrollbar_x = area.right().saturating_sub(1);
        let scrollbar_height = area.height as usize;
        let total_height = self.total_content_height as usize;

        // Calculate scrollbar position (which rows of the scrollbar are "filled")
        let scroll_ratio = self.scroll_offset as f32 / (total_height - scrollbar_height) as f32;
        let scrollbar_pos = (scroll_ratio * (scrollbar_height - 1) as f32) as u16;

        // Draw scrollbar track and position indicator
        for row in 0..area.height {
            let y = area.y + row;
            let char = if row == scrollbar_pos {
                '█' // Full block for current position
            } else if row < scrollbar_pos {
                '▄' // Lower half block for above position
            } else {
                '▀' // Upper half block for below position
            };

            buf.set_string(
                scrollbar_x,
                y,
                char.to_string(),
                Style::default().fg(theme.colors.text_dim),
            );
        }
    }

    /// Sets the total content height (should be called before render with total form height)
    pub fn set_total_height(&mut self, height: u16) {
        self.total_content_height = height;
    }

    /// Clamps scroll offset to valid range based on content and viewport heights
    pub fn clamp_offset(&mut self, visible_height: u16) {
        if self.total_content_height > visible_height {
            let max_offset = self.total_content_height - visible_height;
            if self.scroll_offset > max_offset {
                self.scroll_offset = max_offset;
            }
        } else {
            self.scroll_offset = 0;
        }
    }
}

impl Default for ScrollableFormState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_starts_at_zero() {
        let state = ScrollableFormState::new();
        assert_eq!(state.scroll_offset, 0);
        assert_eq!(state.total_content_height, 0);
    }

    #[test]
    fn test_scroll_up_at_zero_does_nothing() {
        let mut state = ScrollableFormState::new();
        state.scroll_up();
        assert_eq!(state.scroll_offset, 0);
    }

    #[test]
    fn test_scroll_up_decrements() {
        let mut state = ScrollableFormState::new();
        state.scroll_down();
        state.scroll_down();
        state.scroll_up();
        assert_eq!(state.scroll_offset, 1);
    }

    #[test]
    fn test_scroll_down_increments() {
        let mut state = ScrollableFormState::new();
        state.scroll_down();
        assert_eq!(state.scroll_offset, 1);
        state.scroll_down();
        assert_eq!(state.scroll_offset, 2);
    }

    #[test]
    fn test_scroll_to_field_above_viewport() {
        let mut state = ScrollableFormState::new();
        state.scroll_offset = 5;
        state.scroll_to_field(2, 1, 10); // field at y=2, height=1, viewport=10
        assert_eq!(state.scroll_offset, 2); // Should scroll up to show field
    }

    #[test]
    fn test_scroll_to_field_below_viewport() {
        let mut state = ScrollableFormState::new();
        state.scroll_to_field(15, 1, 10); // field at y=15, viewport=10
        assert_eq!(state.scroll_offset, 6); // Should scroll to 15 - 10 + 1 = 6
    }

    #[test]
    fn test_scroll_to_field_in_viewport() {
        let mut state = ScrollableFormState::new();
        state.scroll_offset = 5;
        state.scroll_to_field(7, 1, 10); // field at y=7, within viewport 5-14
        assert_eq!(state.scroll_offset, 5); // Should not change
    }

    #[test]
    fn test_apply_offset_subtracts_scroll() {
        let state = ScrollableFormState {
            scroll_offset: 5,
            total_content_height: 50,
        };
        assert_eq!(state.apply_offset(10), 5); // 10 - 5 = 5
        assert_eq!(state.apply_offset(3), -2); // 3 - 5 = -2 (off-screen)
    }

    #[test]
    fn test_clamp_offset_when_content_fits() {
        let mut state = ScrollableFormState {
            scroll_offset: 10,
            total_content_height: 20,
        };
        state.clamp_offset(30); // visible_height > total_height
        assert_eq!(state.scroll_offset, 0); // Should clamp to 0
    }

    #[test]
    fn test_clamp_offset_when_content_overflows() {
        let mut state = ScrollableFormState {
            scroll_offset: 25,
            total_content_height: 50,
        };
        state.clamp_offset(20); // visible_height=20, total=50, max_offset=30
        assert_eq!(state.scroll_offset, 25); // Within range, no change
    }

    #[test]
    fn test_clamp_offset_when_exceeds_max() {
        let mut state = ScrollableFormState {
            scroll_offset: 35,
            total_content_height: 50,
        };
        state.clamp_offset(20); // max_offset = 50 - 20 = 30
        assert_eq!(state.scroll_offset, 30); // Clamped to 30
    }

    #[test]
    fn test_default_equals_new() {
        let default = ScrollableFormState::default();
        let new = ScrollableFormState::new();
        assert_eq!(default.scroll_offset, new.scroll_offset);
    }
}

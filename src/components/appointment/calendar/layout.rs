//! Layout calculation helper for appointment calendar mouse hit testing
//!
//! This module provides layout-aware position calculations for the appointment calendar,
//! eliminating hardcoded offsets and ensuring mouse hit detection matches Ratatui's
//! actual rendering.

use ratatui::layout::{Constraint, Direction, Layout, Rect};

/// Calendar layout calculator that replicates Ratatui's layout logic
///
/// This struct encapsulates all position calculations for the appointment calendar,
/// ensuring mouse hit detection matches the actual rendered layout.
#[derive(Debug, Clone)]
pub struct CalendarLayout {
    grid_area: Rect,
    num_practitioners: usize,
    time_column_width: u16,
    practitioner_column_width: u16,
}

impl CalendarLayout {
    /// Create a layout calculator from the root area and number of visible practitioners
    ///
    /// # Arguments
    /// * `root_area` - The root Rect for the entire calendar component
    /// * `num_practitioners` - Number of visible practitioners (after filtering)
    ///
    /// # Returns
    /// A `CalendarLayout` that can perform hit testing
    pub fn new(root_area: Rect, num_practitioners: usize) -> Self {
        let horizontal_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(30), Constraint::Min(50)])
            .split(root_area);

        let day_schedule_area = horizontal_chunks[1];

        let vertical_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(day_schedule_area);

        let grid_area = vertical_chunks[1];

        Self {
            grid_area,
            num_practitioners,
            time_column_width: 8,
            practitioner_column_width: 15,
        }
    }

    /// Perform hit testing to find which practitioner column and time slot was clicked
    ///
    /// # Arguments
    /// * `col` - The absolute column position (from mouse event)
    /// * `row` - The absolute row position (from mouse event)
    ///
    /// # Returns
    /// * `Some((practitioner_idx, slot_idx))` - The practitioner and slot indices if hit
    /// * `None` - If the click was outside the grid or in the time column
    pub fn hit_test(&self, col: u16, row: u16) -> Option<(usize, usize)> {
        // Check if click is within the grid area
        if !self.is_in_grid_area(col, row) {
            return None;
        }

        // Convert absolute coordinates to grid-relative coordinates
        let grid_col = col.saturating_sub(self.grid_area.x);
        let grid_row = row.saturating_sub(self.grid_area.y);

        // Calculate practitioner column index
        let practitioner_idx = self.calculate_practitioner_index(grid_col)?;

        // Calculate time slot index
        let slot_idx = self.calculate_slot_index(grid_row)?;

        Some((practitioner_idx, slot_idx))
    }

    /// Check if a coordinate is within the grid area
    fn is_in_grid_area(&self, col: u16, row: u16) -> bool {
        col >= self.grid_area.x
            && col < self.grid_area.x + self.grid_area.width
            && row >= self.grid_area.y
            && row < self.grid_area.y + self.grid_area.height
    }

    /// Calculate which practitioner column was clicked
    ///
    /// Returns `None` if click was in the time column or outside valid columns
    fn calculate_practitioner_index(&self, grid_col: u16) -> Option<usize> {
        // Table structure (Ratatui Table with Borders::ALL):
        // ┌─────────┬───────────────┬───────────────┬───────────────┐
        // │ Time    │ Practitioner1 │ Practitioner2 │ Practitioner3 │
        // ├─────────┼───────────────┼───────────────┼───────────────┤
        // │ 08:00   │ ...           │ ...           │ ...           │
        // └─────────┴───────────────┴───────────────┴───────────────┘
        //
        // Layout breakdown:
        // - 1 char: left border
        // - 8 chars: time column (Constraint::Length(8))
        // - 1 char: column separator
        // - 15 chars: practitioner column (Constraint::Min(15))
        // - 1 char: column separator
        // - ...repeat for each practitioner
        // - 1 char: right border

        let border_width = 1u16;

        // Check if click is in left border or time column
        if grid_col < border_width + self.time_column_width {
            return None; // Click was in time column or left border
        }

        // Calculate position relative to first practitioner column start
        // Position after: border + time_column + separator
        let first_column_start = border_width + self.time_column_width + 1;

        if grid_col < first_column_start {
            return None; // Click was in the separator after time column
        }

        let offset_from_first_column = grid_col - first_column_start;

        // Each practitioner column is: width + 1 separator (except last has right border)
        let column_stride = self.practitioner_column_width + 1; // column + separator

        let practitioner_idx = (offset_from_first_column / column_stride) as usize;
        let position_in_column = offset_from_first_column % column_stride;

        // Check if click is within the column content (not in separator)
        if position_in_column >= self.practitioner_column_width {
            return None; // Click was in column separator
        }

        // Check if practitioner index is valid
        if practitioner_idx >= self.num_practitioners {
            return None; // Click was beyond last practitioner column
        }

        Some(practitioner_idx)
    }

    /// Calculate which time slot row was clicked
    ///
    /// Returns `None` if click was in the header or outside valid slots
    fn calculate_slot_index(&self, grid_row: u16) -> Option<usize> {
        // Ratatui Table widget rendering with Block borders:
        // The grid_area already has Block borders applied.
        // Within grid_area:
        // - Row 0: Top border
        // - Row 1+: Data rows start immediately (header is internal to Table)
        //
        // The Table widget renders header + data internally, but for hit testing
        // we only care about the data rows since header is handled by the widget.

        let border_height = 1u16;
        let slot_height = 2u16;

        if grid_row < border_height {
            return None;
        }

        let data_row_offset = grid_row - border_height;
        let slot_idx = (data_row_offset / slot_height) as usize;

        const MAX_SLOTS: usize = 40;
        if slot_idx >= MAX_SLOTS {
            return None;
        }

        Some(slot_idx)
    }

    /// Get the grid area (useful for debugging)
    pub fn grid_area(&self) -> Rect {
        self.grid_area
    }

    /// Get the number of practitioners (useful for validation)
    pub fn num_practitioners(&self) -> usize {
        self.num_practitioners
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layout_calculation() {
        let root_area = Rect::new(0, 0, 150, 50);
        let num_practitioners = 3;

        let layout = CalendarLayout::new(root_area, num_practitioners);

        assert_eq!(layout.num_practitioners(), 3);
        assert!(layout.grid_area().height > 0);
    }

    #[test]
    fn test_hit_test_time_column() {
        let root_area = Rect::new(0, 0, 150, 50);
        let layout = CalendarLayout::new(root_area, 3);

        // Click in time column (grid_area.x + 5 should be within time column)
        let col = layout.grid_area.x + 5;
        let row = layout.grid_area.y + 5;

        assert_eq!(layout.hit_test(col, row), None);
    }

    #[test]
    fn test_hit_test_first_practitioner() {
        let root_area = Rect::new(0, 0, 150, 50);
        let layout = CalendarLayout::new(root_area, 3);

        // Click in first practitioner column
        // Position: border(1) + time_col(8) + sep(1) + 5 chars into first column
        let col = layout.grid_area.x + 1 + 8 + 1 + 5;

        // Click in first data row (after border only)
        // Position: border(1) + 1 char into first slot
        let row = layout.grid_area.y + 1 + 1;

        assert_eq!(layout.hit_test(col, row), Some((0, 0)));
    }

    #[test]
    fn test_hit_test_second_practitioner() {
        let root_area = Rect::new(0, 0, 150, 50);
        let layout = CalendarLayout::new(root_area, 3);

        // Click in second practitioner column
        // Position: border(1) + time_col(8) + sep(1) + first_col(15) + sep(1) + 5 chars into second column
        let col = layout.grid_area.x + 1 + 8 + 1 + 15 + 1 + 5;

        // Click in first data row (after border only)
        let row = layout.grid_area.y + 1 + 1;

        assert_eq!(layout.hit_test(col, row), Some((1, 0)));
    }

    #[test]
    fn test_hit_test_column_separator() {
        let root_area = Rect::new(0, 0, 150, 50);
        let layout = CalendarLayout::new(root_area, 3);

        // Click exactly on separator between first and second column
        // Position: border(1) + time_col(8) + sep(1) + first_col(15) = separator position
        let col = layout.grid_area.x + 1 + 8 + 1 + 15;
        let row = layout.grid_area.y + 5;

        assert_eq!(layout.hit_test(col, row), None);
    }

    #[test]
    fn test_hit_test_different_slots() {
        let root_area = Rect::new(0, 0, 150, 50);
        let layout = CalendarLayout::new(root_area, 3);

        let col = layout.grid_area.x + 1 + 8 + 1 + 5;

        let row1 = layout.grid_area.y + 1;
        assert_eq!(layout.hit_test(col, row1), Some((0, 0)));

        let row2 = layout.grid_area.y + 1 + 2;
        assert_eq!(layout.hit_test(col, row2), Some((0, 1)));

        let row3 = layout.grid_area.y + 1 + 4;
        assert_eq!(layout.hit_test(col, row3), Some((0, 2)));
    }
}

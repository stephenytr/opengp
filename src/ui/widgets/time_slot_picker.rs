//! Reusable time slot picker widget with keyboard navigation.
//!
//! This module provides a `TimeSlotPicker` widget that manages time slot selection
//! with availability indicators. It displays 15-minute appointment slots from 8:00 AM
//! to 5:45 PM with keyboard navigation support.
//!
//! # Usage
//!
//! ```rust
//! use opengp::ui::widgets::TimeSlotPicker;
//!
//! let mut picker = TimeSlotPicker::new();
//! assert_eq!(picker.selected_time(), "08:00");
//!
//! picker.next();
//! assert_eq!(picker.selected_time(), "08:15");
//!
//! picker.prev();
//! assert_eq!(picker.selected_time(), "08:00");
//!
//! // Mark some slots as unavailable
//! let mut availability = vec![true; 40];
//! availability[0] = false; // 08:00 is booked
//! let picker = TimeSlotPicker::new().with_availability(availability);
//! ```

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Time slot picker state manager.
///
/// This struct manages the state of a time slot picker including available time slots,
/// selected slot index, and availability status for each slot.
#[derive(Debug, Clone)]
pub struct TimeSlotPickerState {
    /// List of time slots in "HH:MM" format (e.g., "08:00", "08:15", ..., "17:45")
    time_slots: Vec<String>,
    /// Index of the currently selected time slot (0-39)
    selected_index: usize,
    /// Availability status for each slot (true=available, false=booked)
    availability: Vec<bool>,
}

impl TimeSlotPickerState {
    /// Creates a new time slot picker state with default time slots (08:00-17:45).
    ///
    /// All slots are marked as available by default.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use opengp::ui::widgets::TimeSlotPickerState;
    ///
    /// let state = TimeSlotPickerState::new();
    /// assert_eq!(state.time_slots().len(), 40);
    /// assert_eq!(state.selected_index(), 0);
    /// ```
    pub fn new() -> Self {
        let time_slots = Self::generate_time_slots();
        let availability = vec![true; time_slots.len()];

        Self {
            time_slots,
            selected_index: 0,
            availability,
        }
    }

    /// Returns the list of all time slots.
    pub fn time_slots(&self) -> &[String] {
        &self.time_slots
    }

    /// Returns the currently selected time slot index (0-39).
    pub fn selected_index(&self) -> usize {
        self.selected_index
    }

    /// Returns the currently selected time slot as a string (e.g., "08:00").
    pub fn selected_time(&self) -> &str {
        &self.time_slots[self.selected_index]
    }

    /// Returns the availability status for all slots.
    pub fn availability(&self) -> &[bool] {
        &self.availability
    }

    /// Returns whether the selected slot is available.
    pub fn is_selected_available(&self) -> bool {
        self.availability[self.selected_index]
    }

    /// Moves selection to the next time slot, wrapping to the first slot if at the end.
    pub fn next(&mut self) {
        self.selected_index = (self.selected_index + 1) % self.time_slots.len();
    }

    /// Moves selection to the previous time slot, wrapping to the last slot if at the beginning.
    pub fn prev(&mut self) {
        if self.selected_index == 0 {
            self.selected_index = self.time_slots.len() - 1;
        } else {
            self.selected_index -= 1;
        }
    }

    /// Updates the availability status for all slots.
    ///
    /// # Arguments
    ///
    /// * `availability` - A vector of boolean values (true=available, false=booked)
    ///
    /// # Panics
    ///
    /// Panics if the length of `availability` does not match the number of time slots.
    pub fn set_availability(&mut self, availability: Vec<bool>) {
        assert_eq!(
            availability.len(),
            self.time_slots.len(),
            "Availability vector length must match number of time slots"
        );
        self.availability = availability;
    }

    /// Generates the list of time slots from 08:00 to 17:45 in 15-minute intervals.
    ///
    /// Returns a vector of 40 time slot strings in "HH:MM" format.
    fn generate_time_slots() -> Vec<String> {
        let mut slots = Vec::new();
        for hour in 8..18 {
            for minute in [0, 15, 30, 45] {
                slots.push(format!("{:02}:{:02}", hour, minute));
            }
        }
        slots
    }
}

impl Default for TimeSlotPickerState {
    fn default() -> Self {
        Self::new()
    }
}

/// Time slot picker widget.
///
/// This widget wraps `TimeSlotPickerState` and provides rendering and interaction
/// methods for selecting appointment time slots.
#[derive(Debug, Clone)]
pub struct TimeSlotPicker {
    state: TimeSlotPickerState,
}

impl TimeSlotPicker {
    /// Creates a new time slot picker widget with default settings.
    ///
    /// All slots are marked as available by default.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use opengp::ui::widgets::TimeSlotPicker;
    ///
    /// let picker = TimeSlotPicker::new();
    /// assert_eq!(picker.selected_time(), "08:00");
    /// ```
    pub fn new() -> Self {
        Self {
            state: TimeSlotPickerState::new(),
        }
    }

    /// Creates a new time slot picker with custom availability status.
    ///
    /// # Arguments
    ///
    /// * `availability` - A vector of 40 boolean values (true=available, false=booked)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use opengp::ui::widgets::TimeSlotPicker;
    ///
    /// let mut availability = vec![true; 40];
    /// availability[0] = false; // 08:00 is booked
    /// let picker = TimeSlotPicker::new().with_availability(availability);
    /// assert!(!picker.is_selected_available());
    /// ```
    pub fn with_availability(mut self, availability: Vec<bool>) -> Self {
        self.state.set_availability(availability);
        self
    }

    /// Returns the currently selected time slot as a string (e.g., "08:00").
    pub fn selected_time(&self) -> &str {
        self.state.selected_time()
    }

    /// Returns the currently selected time slot index (0-39).
    pub fn selected_index(&self) -> usize {
        self.state.selected_index()
    }

    /// Returns whether the selected slot is available.
    pub fn is_selected_available(&self) -> bool {
        self.state.is_selected_available()
    }

    /// Moves selection to the next time slot, wrapping to the first slot if at the end.
    pub fn next(&mut self) {
        self.state.next();
    }

    /// Moves selection to the previous time slot, wrapping to the last slot if at the beginning.
    pub fn prev(&mut self) {
        self.state.prev();
    }

    /// Updates the availability status for all slots.
    ///
    /// # Arguments
    ///
    /// * `availability` - A vector of 40 boolean values (true=available, false=booked)
    pub fn set_availability(&mut self, availability: Vec<bool>) {
        self.state.set_availability(availability);
    }

    /// Renders the time slot picker widget.
    ///
    /// Displays a scrollable list of time slots with:
    /// - Available slots: White text, selectable
    /// - Unavailable slots: DarkGray text with strikethrough
    /// - Selected slot: Yellow background + Black text + Bold
    ///
    /// # Arguments
    ///
    /// * `frame` - The frame to render into
    /// * `area` - The area to render the widget in
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        if area.height < 3 {
            return; // Not enough space to render
        }

        // Calculate visible slots (show ~10-15 slots at a time)
        let visible_height = (area.height as usize).saturating_sub(2); // Account for borders
        let scroll_offset = if self.state.selected_index >= visible_height / 2 {
            self.state.selected_index - visible_height / 2
        } else {
            0
        };

        let mut lines = Vec::new();

        // Render visible time slots
        for (display_idx, slot_idx) in (scroll_offset..self.state.time_slots.len())
            .enumerate()
            .take(visible_height)
        {
            let time_slot = &self.state.time_slots[slot_idx];
            let is_available = self.state.availability[slot_idx];
            let is_selected = slot_idx == self.state.selected_index;

            let style = if is_selected {
                // Selected: Yellow background + Black text + Bold
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else if is_available {
                // Available: White text
                Style::default().fg(Color::White)
            } else {
                // Unavailable: DarkGray text with strikethrough
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::CROSSED_OUT)
            };

            let line = Line::from(Span::styled(format!(" {} ", time_slot), style));
            lines.push(line);
        }

        // Create the paragraph widget
        let paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Time Slots ")
                    .title_alignment(ratatui::layout::Alignment::Center),
            )
            .style(Style::default().fg(Color::White));

        frame.render_widget(paragraph, area);
    }
}

impl Default for TimeSlotPicker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_slot_generation() {
        let state = TimeSlotPickerState::new();
        let slots = state.time_slots();

        // Should have 40 slots (8 hours * 4 slots per hour)
        assert_eq!(slots.len(), 40);

        // First slot should be 08:00
        assert_eq!(slots[0], "08:00");

        // Last slot should be 17:45
        assert_eq!(slots[39], "17:45");

        // Check some intermediate slots
        assert_eq!(slots[1], "08:15");
        assert_eq!(slots[4], "09:00");
        assert_eq!(slots[8], "10:00");
    }

    #[test]
    fn test_time_slot_format() {
        let state = TimeSlotPickerState::new();
        let slots = state.time_slots();

        // All slots should be in HH:MM format
        for slot in slots {
            assert_eq!(slot.len(), 5);
            assert_eq!(slot.chars().nth(2), Some(':'));
        }
    }

    #[test]
    fn test_initial_state() {
        let state = TimeSlotPickerState::new();

        assert_eq!(state.selected_index(), 0);
        assert_eq!(state.selected_time(), "08:00");
        assert!(state.is_selected_available());
    }

    #[test]
    fn test_next_navigation() {
        let mut state = TimeSlotPickerState::new();

        assert_eq!(state.selected_index(), 0);
        state.next();
        assert_eq!(state.selected_index(), 1);
        assert_eq!(state.selected_time(), "08:15");

        state.next();
        assert_eq!(state.selected_index(), 2);
        assert_eq!(state.selected_time(), "08:30");
    }

    #[test]
    fn test_prev_navigation() {
        let mut state = TimeSlotPickerState::new();

        state.next();
        state.next();
        assert_eq!(state.selected_index(), 2);

        state.prev();
        assert_eq!(state.selected_index(), 1);
        assert_eq!(state.selected_time(), "08:15");

        state.prev();
        assert_eq!(state.selected_index(), 0);
        assert_eq!(state.selected_time(), "08:00");
    }

    #[test]
    fn test_next_wraparound() {
        let mut state = TimeSlotPickerState::new();

        // Move to last slot
        for _ in 0..39 {
            state.next();
        }
        assert_eq!(state.selected_index(), 39);
        assert_eq!(state.selected_time(), "17:45");

        // Next should wrap to first slot
        state.next();
        assert_eq!(state.selected_index(), 0);
        assert_eq!(state.selected_time(), "08:00");
    }

    #[test]
    fn test_prev_wraparound() {
        let mut state = TimeSlotPickerState::new();

        assert_eq!(state.selected_index(), 0);

        // Prev should wrap to last slot
        state.prev();
        assert_eq!(state.selected_index(), 39);
        assert_eq!(state.selected_time(), "17:45");
    }

    #[test]
    fn test_availability_default() {
        let state = TimeSlotPickerState::new();

        // All slots should be available by default
        for is_available in state.availability() {
            assert!(*is_available);
        }
    }

    #[test]
    fn test_set_availability() {
        let mut state = TimeSlotPickerState::new();

        let mut availability = vec![true; 40];
        availability[0] = false; // 08:00 is booked
        availability[5] = false; // 09:15 is booked

        state.set_availability(availability);

        assert!(!state.availability()[0]);
        assert!(state.availability()[1]);
        assert!(!state.availability()[5]);
        assert!(state.availability()[6]);
    }

    #[test]
    fn test_is_selected_available() {
        let mut state = TimeSlotPickerState::new();

        assert!(state.is_selected_available());

        let mut availability = vec![true; 40];
        availability[0] = false;
        state.set_availability(availability);

        assert!(!state.is_selected_available());

        state.next();
        assert!(state.is_selected_available());
    }

    #[test]
    fn test_widget_creation() {
        let picker = TimeSlotPicker::new();

        assert_eq!(picker.selected_time(), "08:00");
        assert_eq!(picker.selected_index(), 0);
        assert!(picker.is_selected_available());
    }

    #[test]
    fn test_widget_with_availability() {
        let mut availability = vec![true; 40];
        availability[0] = false;
        availability[10] = false;

        let picker = TimeSlotPicker::new().with_availability(availability);

        assert!(!picker.is_selected_available());

        let mut picker = picker;
        picker.next();
        assert!(picker.is_selected_available());
    }

    #[test]
    fn test_widget_navigation() {
        let mut picker = TimeSlotPicker::new();

        assert_eq!(picker.selected_time(), "08:00");

        picker.next();
        assert_eq!(picker.selected_time(), "08:15");

        picker.prev();
        assert_eq!(picker.selected_time(), "08:00");
    }

    #[test]
    fn test_widget_set_availability() {
        let mut picker = TimeSlotPicker::new();

        let mut availability = vec![true; 40];
        availability[0] = false;

        picker.set_availability(availability);

        assert!(!picker.is_selected_available());
    }

    #[test]
    fn test_multiple_navigation_cycles() {
        let mut state = TimeSlotPickerState::new();

        // Navigate forward through all slots
        for i in 0..40 {
            assert_eq!(state.selected_index(), i);
            state.next();
        }

        // Should wrap back to 0
        assert_eq!(state.selected_index(), 0);

        // Navigate backward through all slots
        for i in (0..40).rev() {
            state.prev();
            assert_eq!(state.selected_index(), i);
        }
    }

    #[test]
    fn test_availability_vector_length_validation() {
        let mut state = TimeSlotPickerState::new();

        // Should panic with wrong length
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            state.set_availability(vec![true; 39]); // Wrong length
        }));

        assert!(result.is_err());
    }

    #[test]
    fn test_state_clone() {
        let state1 = TimeSlotPickerState::new();
        let state2 = state1.clone();

        assert_eq!(state1.selected_index(), state2.selected_index());
        assert_eq!(state1.selected_time(), state2.selected_time());
    }

    #[test]
    fn test_widget_clone() {
        let picker1 = TimeSlotPicker::new();
        let picker2 = picker1.clone();

        assert_eq!(picker1.selected_time(), picker2.selected_time());
        assert_eq!(picker1.selected_index(), picker2.selected_index());
    }

    #[test]
    fn test_default_trait() {
        let state = TimeSlotPickerState::default();
        assert_eq!(state.selected_index(), 0);

        let picker = TimeSlotPicker::default();
        assert_eq!(picker.selected_index(), 0);
    }
}

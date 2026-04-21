//! Inline Picker Widget
//!
//! A reusable helper that owns both DatePickerPopup and TimePickerPopup,
//! handles key delegation, and renders overlays.
//!
//! This widget provides a unified interface for date and time selection,
//! delegating key events to whichever picker is currently visible.

use chrono::{NaiveDate, NaiveTime};
use crossterm::event::KeyEvent;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::Widget;

use super::{DatePickerAction, DatePickerPopup, TimePickerAction, TimePickerPopup};
use crate::ui::theme::Theme;

/// Actions returned by the inline picker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InlinePickerAction {
    /// User selected a date
    DateSelected(NaiveDate),
    /// User selected a time
    TimeSelected(NaiveTime),
    /// User dismissed the picker without selecting
    Dismissed,
}

/// A reusable inline picker that owns both date and time pickers.
///
/// Handles key delegation to the visible picker and renders overlays.
/// Only one picker should be visible at a time.
#[derive(Debug, Clone)]
pub struct InlinePicker {
    date_picker: DatePickerPopup,
    time_picker: TimePickerPopup,
}

impl InlinePicker {
    /// Create a new inline picker.
    pub fn new(theme: Theme) -> Self {
        Self {
            date_picker: DatePickerPopup::new(theme.clone()),
            time_picker: TimePickerPopup::with_theme(theme),
        }
    }

    /// Open the date picker with an optional current date value.
    pub fn open_date_picker(&mut self, current: Option<NaiveDate>) {
        self.date_picker.open(current);
    }

    /// Open the time picker with practitioner, date, and duration.
    pub fn open_time_picker(&mut self, practitioner_id: i64, date: NaiveDate, duration: u32) {
        self.time_picker.open(practitioner_id, date, duration);
    }

    /// Check if either picker is currently visible.
    pub fn is_visible(&self) -> bool {
        self.date_picker.is_visible() || self.time_picker.is_visible()
    }

    /// Handle a key event, delegating to the visible picker.
    ///
    /// Returns `Some(InlinePickerAction)` if a picker returned an action.
    /// Returns `None` if the key was consumed for navigation or neither picker is visible.
    pub fn handle_key(&mut self, key: KeyEvent) -> Option<InlinePickerAction> {
        // Check date picker first
        if self.date_picker.is_visible() {
            if let Some(action) = self.date_picker.handle_key(key) {
                return Some(match action {
                    DatePickerAction::Selected(date) => InlinePickerAction::DateSelected(date),
                    DatePickerAction::Dismissed => InlinePickerAction::Dismissed,
                });
            }
            return None;
        }

        // Check time picker next
        if self.time_picker.is_visible() {
            if let Some(action) = self.time_picker.handle_key(key) {
                return Some(match action {
                    TimePickerAction::Selected(time) => InlinePickerAction::TimeSelected(time),
                    TimePickerAction::Dismissed => InlinePickerAction::Dismissed,
                });
            }
            return None;
        }

        None
    }

    /// Set booked time slots for the time picker.
    pub fn set_booked_slots(&mut self, slots: Vec<NaiveTime>) {
        self.time_picker.set_booked_slots(slots);
    }

    /// Close both pickers.
    pub fn close(&mut self) {
        self.date_picker.close();
        self.time_picker.close();
    }
}

impl Widget for InlinePicker {
    fn render(mut self, area: Rect, buf: &mut Buffer) {
        // Render the date picker if visible
        if self.date_picker.is_visible() {
            self.date_picker.render(area, buf);
        }
        // Render the time picker if visible
        else if self.time_picker.is_visible() {
            self.time_picker.render(area, buf);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyModifiers};

    #[test]
    fn test_new_picker_not_visible() {
        let picker = InlinePicker::new(Theme::default());
        assert!(!picker.is_visible());
    }

    #[test]
    fn test_open_date_picker_sets_visibility() {
        let mut picker = InlinePicker::new(Theme::default());
        let date = NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();
        picker.open_date_picker(Some(date));
        assert!(picker.is_visible());
        assert!(picker.date_picker.is_visible());
        assert!(!picker.time_picker.is_visible());
    }

    #[test]
    fn test_open_time_picker_sets_visibility() {
        let mut picker = InlinePicker::new(Theme::default());
        let date = NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();
        picker.open_time_picker(1, date, 30);
        assert!(picker.is_visible());
        assert!(!picker.date_picker.is_visible());
        assert!(picker.time_picker.is_visible());
    }

    #[test]
    fn test_close_hides_both_pickers() {
        let mut picker = InlinePicker::new(Theme::default());
        let date = NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();
        picker.open_date_picker(Some(date));
        picker.close();
        assert!(!picker.is_visible());
    }

    #[test]
    fn test_handle_key_returns_none_when_not_visible() {
        let mut picker = InlinePicker::new(Theme::default());
        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let action = picker.handle_key(key);
        assert!(action.is_none());
    }

    #[test]
    fn test_handle_key_delegates_to_date_picker() {
        let mut picker = InlinePicker::new(Theme::default());
        let date = NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();
        picker.open_date_picker(Some(date));

        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        let action = picker.handle_key(key);

        assert!(matches!(action, Some(InlinePickerAction::Dismissed)));
    }

    #[test]
    fn test_handle_key_date_selected() {
        let mut picker = InlinePicker::new(Theme::default());
        let date = NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();
        picker.open_date_picker(Some(date));

        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let action = picker.handle_key(key);

        assert!(matches!(action, Some(InlinePickerAction::DateSelected(_))));
    }

    #[test]
    fn test_handle_key_delegates_to_time_picker() {
        let mut picker = InlinePicker::new(Theme::default());
        let date = NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();
        picker.open_time_picker(1, date, 30);

        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        let action = picker.handle_key(key);

        assert!(matches!(action, Some(InlinePickerAction::Dismissed)));
    }

    #[test]
    fn test_handle_key_time_selected() {
        let mut picker = InlinePicker::new(Theme::default());
        let date = NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();
        picker.open_time_picker(1, date, 30);

        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let action = picker.handle_key(key);

        assert!(matches!(action, Some(InlinePickerAction::TimeSelected(_))));
    }

    #[test]
    fn test_navigation_in_date_picker_returns_none() {
        let mut picker = InlinePicker::new(Theme::default());
        let date = NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();
        picker.open_date_picker(Some(date));

        let key = KeyEvent::new(KeyCode::Right, KeyModifiers::NONE);
        let action = picker.handle_key(key);

        assert!(action.is_none());
        assert!(picker.date_picker.is_visible());
    }

    #[test]
    fn test_set_booked_slots_updates_time_picker() {
        let mut picker = InlinePicker::new(Theme::default());
        let slots = vec![NaiveTime::from_hms_opt(9, 0, 0).unwrap()];
        picker.set_booked_slots(slots);
        // Just verify the method doesn't panic
        // The time picker's internal state is private
    }

    #[test]
    fn test_date_picker_can_be_opened_after_closing() {
        let mut picker = InlinePicker::new(Theme::default());
        let date = NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();
        picker.open_date_picker(Some(date));
        picker.close();
        assert!(!picker.is_visible());

        picker.open_date_picker(Some(date));
        assert!(picker.is_visible());
    }

    #[test]
    fn test_switching_between_pickers() {
        let mut picker = InlinePicker::new(Theme::default());
        let date = NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();

        // Open date picker
        picker.open_date_picker(Some(date));
        assert!(picker.date_picker.is_visible());
        assert!(!picker.time_picker.is_visible());

        // Close and open time picker
        picker.close();
        picker.open_time_picker(1, date, 30);
        assert!(!picker.date_picker.is_visible());
        assert!(picker.time_picker.is_visible());
    }
}

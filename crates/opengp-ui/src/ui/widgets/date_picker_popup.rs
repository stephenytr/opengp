//! Date Picker Popup Widget
//!
//! Reusable calendar popup for selecting dates in form fields.
//! Provides keyboard-driven date selection with centered overlay rendering.

use chrono::{Datelike, NaiveDate};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::{Clear, Widget};

use super::CalendarWidget;

/// Actions returned by the date picker popup.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatePickerAction {
    /// User selected a date
    Selected(NaiveDate),
    /// User dismissed the popup without selecting
    Dismissed,
}

/// A reusable date picker popup component.
///
/// Renders a centered calendar overlay that allows keyboard navigation
/// and date selection. Can be opened with an initial date and returns
/// the selected date or dismissal action.
#[derive(Debug, Clone)]
pub struct DatePickerPopup {
    calendar: CalendarWidget,
    visible: bool,
    initial_date: Option<NaiveDate>,
}

impl DatePickerPopup {
    /// Create a new date picker popup.
    pub fn new() -> Self {
        Self {
            calendar: CalendarWidget::new(),
            visible: false,
            initial_date: None,
        }
    }

    /// Open the date picker with an optional current date value.
    pub fn open(&mut self, current_value: Option<NaiveDate>) {
        self.calendar = CalendarWidget::show_date_picker(current_value);
        self.initial_date = current_value;
        self.visible = true;
    }

    /// Close the date picker.
    pub fn close(&mut self) {
        self.visible = false;
    }

    /// Check if the date picker is currently visible.
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Handle a key event while the popup is open.
    ///
    /// Returns `Some(DatePickerAction)` if the user selected a date or dismissed the popup.
    /// Returns `None` if the key was consumed for navigation.
    pub fn handle_key(&mut self, key: KeyEvent) -> Option<DatePickerAction> {
        if !self.visible {
            return None;
        }

        match key.code {
            KeyCode::Enter | KeyCode::Char(' ') => {
                let selected = self.calendar.focused_date;
                self.visible = false;
                Some(DatePickerAction::Selected(selected))
            }
            KeyCode::Esc => {
                self.visible = false;
                Some(DatePickerAction::Dismissed)
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.calendar.focused_date.day() > 7 {
                    self.calendar.focused_date -= chrono::Duration::days(7);
                } else {
                    self.calendar.prev_month();
                    let (year, month) = self.calendar.current_month;
                    let day = self.calendar.focused_date.day();
                    if let Some(new_date) = chrono::NaiveDate::from_ymd_opt(year, month, day) {
                        self.calendar.focused_date = new_date;
                    } else if let Some(new_date) = chrono::NaiveDate::from_ymd_opt(year, month, 28)
                    {
                        self.calendar.focused_date = new_date;
                    }
                }
                None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let (year, month) = self.calendar.current_month;
                let days_in_month = self.days_in_month(year, month);
                if self.calendar.focused_date.day() + 7 <= days_in_month {
                    self.calendar.focused_date += chrono::Duration::days(7);
                } else {
                    self.calendar.next_month();
                    let (new_year, new_month) = self.calendar.current_month;
                    let day = self.calendar.focused_date.day();
                    if let Some(new_date) =
                        chrono::NaiveDate::from_ymd_opt(new_year, new_month, day)
                    {
                        self.calendar.focused_date = new_date;
                    } else if let Some(new_date) =
                        chrono::NaiveDate::from_ymd_opt(new_year, new_month, 1)
                    {
                        self.calendar.focused_date = new_date;
                    }
                }
                None
            }
            KeyCode::Left | KeyCode::Char('h') => {
                let prev = self.calendar.focused_date - chrono::Duration::days(1);
                self.calendar.focused_date = prev;
                let current_month = self.calendar.current_month.1;
                if prev.month() != current_month {
                    self.calendar.prev_month();
                }
                None
            }
            KeyCode::Right | KeyCode::Char('l') => {
                let next = self.calendar.focused_date + chrono::Duration::days(1);
                self.calendar.focused_date = next;
                let current_month = self.calendar.current_month.1;
                if next.month() != current_month {
                    self.calendar.next_month();
                }
                None
            }
            _ => None,
        }
    }

    /// Render the date picker popup as a centered overlay.
    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        if !self.visible || area.is_empty() {
            return;
        }

        let popup_width = 27.min(area.width.saturating_sub(4));
        let popup_height = 13.min(area.height.saturating_sub(2));

        if popup_width < 21 || popup_height < 9 {
            return;
        }

        let popup_x = if area.width > popup_width {
            area.x + (area.width - popup_width) / 2
        } else {
            area.x
        };
        let popup_y = if area.height > popup_height {
            area.y + (area.height - popup_height) / 2
        } else {
            area.y
        };

        let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

        Clear.render(popup_area, buf);

        self.calendar.render_calendar(
            popup_area,
            buf,
            crate::ui::widgets::CalendarMode::DatePicker,
        );
    }

    /// Helper to calculate days in a month.
    fn days_in_month(&self, year: i32, month: u32) -> u32 {
        let (next_year, next_month) = if month == 12 {
            (year + 1, 1)
        } else {
            (year, month + 1)
        };

        let start = NaiveDate::from_ymd_opt(year, month, 1).expect("valid month start");
        let next_start =
            NaiveDate::from_ymd_opt(next_year, next_month, 1).expect("valid month start");

        (next_start - start).num_days() as u32
    }
}

impl Default for DatePickerPopup {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[test]
    fn test_new_popup_not_visible() {
        let popup = DatePickerPopup::new();
        assert!(!popup.is_visible());
    }

    #[test]
    fn test_open_sets_visible_with_date() {
        let mut popup = DatePickerPopup::new();
        let date = NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();
        popup.open(Some(date));
        assert!(popup.is_visible());
        assert_eq!(popup.calendar.focused_date, date);
        assert_eq!(popup.initial_date, Some(date));
    }

    #[test]
    fn test_open_sets_visible_without_date() {
        let mut popup = DatePickerPopup::new();
        popup.open(None);
        assert!(popup.is_visible());
        assert_eq!(popup.initial_date, None);
    }

    #[test]
    fn test_close_clears_visibility() {
        let mut popup = DatePickerPopup::new();
        let date = NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();
        popup.open(Some(date));
        popup.close();
        assert!(!popup.is_visible());
    }

    #[test]
    fn test_handle_key_enter_confirms_selection() {
        let mut popup = DatePickerPopup::new();
        let date = NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();
        popup.open(Some(date));

        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let action = popup.handle_key(key);

        assert!(matches!(action, Some(DatePickerAction::Selected(_))));
        assert!(!popup.is_visible());
    }

    #[test]
    fn test_handle_key_space_confirms_selection() {
        let mut popup = DatePickerPopup::new();
        let date = NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();
        popup.open(Some(date));

        let key = KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE);
        let action = popup.handle_key(key);

        assert!(matches!(action, Some(DatePickerAction::Selected(_))));
        assert!(!popup.is_visible());
    }

    #[test]
    fn test_handle_key_esc_dismisses() {
        let mut popup = DatePickerPopup::new();
        let date = NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();
        popup.open(Some(date));

        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        let action = popup.handle_key(key);

        assert!(matches!(action, Some(DatePickerAction::Dismissed)));
        assert!(!popup.is_visible());
    }

    #[test]
    fn test_handle_key_up_navigates_previous_week() {
        let mut popup = DatePickerPopup::new();
        let date = NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();
        popup.open(Some(date));

        let key = KeyEvent::new(KeyCode::Up, KeyModifiers::NONE);
        let action = popup.handle_key(key);

        assert!(action.is_none());
        assert!(popup.is_visible());
        assert_eq!(
            popup.calendar.focused_date,
            NaiveDate::from_ymd_opt(2024, 3, 8).unwrap()
        );
    }

    #[test]
    fn test_handle_key_k_navigates_previous_week() {
        let mut popup = DatePickerPopup::new();
        let date = NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();
        popup.open(Some(date));

        let key = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE);
        let action = popup.handle_key(key);

        assert!(action.is_none());
        assert_eq!(
            popup.calendar.focused_date,
            NaiveDate::from_ymd_opt(2024, 3, 8).unwrap()
        );
    }

    #[test]
    fn test_handle_key_down_navigates_next_week() {
        let mut popup = DatePickerPopup::new();
        let date = NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();
        popup.open(Some(date));

        let key = KeyEvent::new(KeyCode::Down, KeyModifiers::NONE);
        let action = popup.handle_key(key);

        assert!(action.is_none());
        assert_eq!(
            popup.calendar.focused_date,
            NaiveDate::from_ymd_opt(2024, 3, 22).unwrap()
        );
    }

    #[test]
    fn test_handle_key_j_navigates_next_week() {
        let mut popup = DatePickerPopup::new();
        let date = NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();
        popup.open(Some(date));

        let key = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
        let action = popup.handle_key(key);

        assert!(action.is_none());
        assert_eq!(
            popup.calendar.focused_date,
            NaiveDate::from_ymd_opt(2024, 3, 22).unwrap()
        );
    }

    #[test]
    fn test_handle_key_left_navigates_previous_day() {
        let mut popup = DatePickerPopup::new();
        let date = NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();
        popup.open(Some(date));

        let key = KeyEvent::new(KeyCode::Left, KeyModifiers::NONE);
        let action = popup.handle_key(key);

        assert!(action.is_none());
        assert_eq!(
            popup.calendar.focused_date,
            NaiveDate::from_ymd_opt(2024, 3, 14).unwrap()
        );
    }

    #[test]
    fn test_handle_key_h_navigates_previous_day() {
        let mut popup = DatePickerPopup::new();
        let date = NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();
        popup.open(Some(date));

        let key = KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE);
        let action = popup.handle_key(key);

        assert!(action.is_none());
        assert_eq!(
            popup.calendar.focused_date,
            NaiveDate::from_ymd_opt(2024, 3, 14).unwrap()
        );
    }

    #[test]
    fn test_handle_key_right_navigates_next_day() {
        let mut popup = DatePickerPopup::new();
        let date = NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();
        popup.open(Some(date));

        let key = KeyEvent::new(KeyCode::Right, KeyModifiers::NONE);
        let action = popup.handle_key(key);

        assert!(action.is_none());
        assert_eq!(
            popup.calendar.focused_date,
            NaiveDate::from_ymd_opt(2024, 3, 16).unwrap()
        );
    }

    #[test]
    fn test_handle_key_l_navigates_next_day() {
        let mut popup = DatePickerPopup::new();
        let date = NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();
        popup.open(Some(date));

        let key = KeyEvent::new(KeyCode::Char('l'), KeyModifiers::NONE);
        let action = popup.handle_key(key);

        assert!(action.is_none());
        assert_eq!(
            popup.calendar.focused_date,
            NaiveDate::from_ymd_opt(2024, 3, 16).unwrap()
        );
    }

    #[test]
    fn test_handle_key_up_at_month_boundary_wraps_to_previous_month() {
        let mut popup = DatePickerPopup::new();
        let date = NaiveDate::from_ymd_opt(2024, 3, 5).unwrap();
        popup.open(Some(date));

        let key = KeyEvent::new(KeyCode::Up, KeyModifiers::NONE);
        popup.handle_key(key);

        assert_eq!(popup.calendar.current_month.1, 2);
        assert_eq!(popup.calendar.focused_date.month(), 2);
    }

    #[test]
    fn test_handle_key_down_at_month_boundary_wraps_to_next_month() {
        let mut popup = DatePickerPopup::new();
        let date = NaiveDate::from_ymd_opt(2024, 3, 28).unwrap();
        popup.open(Some(date));

        let key = KeyEvent::new(KeyCode::Down, KeyModifiers::NONE);
        popup.handle_key(key);

        assert_eq!(popup.calendar.current_month.1, 4);
        assert_eq!(popup.calendar.focused_date.month(), 4);
    }

    #[test]
    fn test_handle_key_left_at_month_boundary_wraps_to_previous_month() {
        let mut popup = DatePickerPopup::new();
        let date = NaiveDate::from_ymd_opt(2024, 3, 1).unwrap();
        popup.open(Some(date));

        let key = KeyEvent::new(KeyCode::Left, KeyModifiers::NONE);
        popup.handle_key(key);

        assert_eq!(popup.calendar.current_month.1, 2);
        assert_eq!(
            popup.calendar.focused_date,
            NaiveDate::from_ymd_opt(2024, 2, 29).unwrap()
        );
    }

    #[test]
    fn test_handle_key_right_at_month_boundary_wraps_to_next_month() {
        let mut popup = DatePickerPopup::new();
        let date = NaiveDate::from_ymd_opt(2024, 3, 31).unwrap();
        popup.open(Some(date));

        let key = KeyEvent::new(KeyCode::Right, KeyModifiers::NONE);
        popup.handle_key(key);

        assert_eq!(popup.calendar.current_month.1, 4);
        assert_eq!(
            popup.calendar.focused_date,
            NaiveDate::from_ymd_opt(2024, 4, 1).unwrap()
        );
    }

    #[test]
    fn test_handle_key_when_not_visible_returns_none() {
        let mut popup = DatePickerPopup::new();
        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let action = popup.handle_key(key);

        assert!(action.is_none());
    }

    #[test]
    fn test_handle_key_unknown_key_returns_none() {
        let mut popup = DatePickerPopup::new();
        let date = NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();
        popup.open(Some(date));

        let key = KeyEvent::new(KeyCode::F(1), KeyModifiers::NONE);
        let action = popup.handle_key(key);

        assert!(action.is_none());
        assert!(popup.is_visible());
    }

    #[test]
    fn test_selected_date_is_returned_on_confirmation() {
        let mut popup = DatePickerPopup::new();
        let date = NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();
        popup.open(Some(date));

        let key = KeyEvent::new(KeyCode::Right, KeyModifiers::NONE);
        popup.handle_key(key);

        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let action = popup.handle_key(key);

        if let Some(DatePickerAction::Selected(selected)) = action {
            assert_eq!(selected, NaiveDate::from_ymd_opt(2024, 3, 16).unwrap());
        } else {
            panic!("Expected DatePickerAction::Selected");
        }
    }

    #[test]
    fn test_days_in_month_february_leap_year() {
        let popup = DatePickerPopup::new();
        let days = popup.days_in_month(2024, 2);
        assert_eq!(days, 29);
    }

    #[test]
    fn test_days_in_month_february_non_leap_year() {
        let popup = DatePickerPopup::new();
        let days = popup.days_in_month(2023, 2);
        assert_eq!(days, 28);
    }

    #[test]
    fn test_days_in_month_april() {
        let popup = DatePickerPopup::new();
        let days = popup.days_in_month(2024, 4);
        assert_eq!(days, 30);
    }

    #[test]
    fn test_days_in_month_december() {
        let popup = DatePickerPopup::new();
        let days = popup.days_in_month(2024, 12);
        assert_eq!(days, 31);
    }

    #[test]
    fn test_default_creates_new_popup() {
        let popup = DatePickerPopup::default();
        assert!(!popup.is_visible());
    }
}

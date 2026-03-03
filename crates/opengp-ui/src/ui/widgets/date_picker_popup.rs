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
                    self.calendar.focused_date =
                        self.calendar.focused_date - chrono::Duration::days(7);
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
                if self.calendar.focused_date.day() + 7 <= days_in_month as u32 {
                    self.calendar.focused_date =
                        self.calendar.focused_date + chrono::Duration::days(7);
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
                let current_month = self.calendar.current_month.1 as u32;
                if prev.month() != current_month {
                    self.calendar.prev_month();
                }
                None
            }
            KeyCode::Right | KeyCode::Char('l') => {
                let next = self.calendar.focused_date + chrono::Duration::days(1);
                self.calendar.focused_date = next;
                let current_month = self.calendar.current_month.1 as u32;
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

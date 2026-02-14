//! Reusable month calendar widget with keyboard navigation.
//!
//! This module provides a `MonthCalendar` widget that manages month calendar state
//! and rendering. It handles day selection, month navigation, and provides a clean
//! API for integrating calendar functionality into TUI components.
//!
//! # Usage
//!
//! ```rust
//! use opengp::ui::widgets::MonthCalendar;
//! use chrono::NaiveDate;
//!
//! let mut calendar = MonthCalendar::new(NaiveDate::from_ymd_opt(2026, 2, 14).unwrap());
//! assert_eq!(calendar.selected_date().day(), 14);
//!
//! calendar.next_day();
//! assert_eq!(calendar.selected_date().day(), 15);
//!
//! calendar.prev_day();
//! assert_eq!(calendar.selected_date().day(), 14);
//! ```

use chrono::{Datelike, Duration, NaiveDate, Weekday};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Month calendar state manager.
///
/// This struct manages the state of a month calendar including the current month,
/// selected day, and today's date for highlighting.
#[derive(Debug, Clone)]
pub struct MonthCalendarState {
    /// First day of the current month (always day 1)
    current_month: NaiveDate,
    /// Selected day (1-31)
    selected_day: u32,
    /// Today's date for highlighting
    today: NaiveDate,
}

impl MonthCalendarState {
    /// Creates a new month calendar state with the given date.
    ///
    /// The initial month is set to the month of the given date, and the selected
    /// day is set to the day of the given date.
    ///
    /// # Arguments
    ///
    /// * `date` - The initial date to display
    ///
    /// # Examples
    ///
    /// ```rust
    /// use opengp::ui::widgets::MonthCalendarState;
    /// use chrono::NaiveDate;
    ///
    /// let state = MonthCalendarState::new(NaiveDate::from_ymd_opt(2026, 2, 14).unwrap());
    /// assert_eq!(state.selected_day(), 14);
    /// ```
    pub fn new(date: NaiveDate) -> Self {
        let current_month = date.with_day(1).unwrap();
        let selected_day = date.day();
        let today = chrono::Local::now().naive_local().date();

        Self {
            current_month,
            selected_day,
            today,
        }
    }

    /// Returns the selected day (1-31).
    pub fn selected_day(&self) -> u32 {
        self.selected_day
    }

    /// Returns the current month's first day.
    pub fn current_month(&self) -> NaiveDate {
        self.current_month
    }

    /// Returns today's date.
    pub fn today(&self) -> NaiveDate {
        self.today
    }

    /// Moves selection to the next day, wrapping to the first day of the next month if needed.
    pub fn next_day(&mut self) {
        let days_in_month =
            Self::days_in_month(self.current_month.year(), self.current_month.month());

        if self.selected_day >= days_in_month {
            self.selected_day = 1;
            self.current_month += Duration::days(32);
            self.current_month = self.current_month.with_day(1).unwrap();
        } else {
            self.selected_day += 1;
        }
    }

    /// Moves selection to the previous day, wrapping to the last day of the previous month if needed.
    pub fn prev_day(&mut self) {
        if self.selected_day <= 1 {
            self.current_month -= Duration::days(1);
            self.current_month = self.current_month.with_day(1).unwrap();
            self.selected_day =
                Self::days_in_month(self.current_month.year(), self.current_month.month());
        } else {
            self.selected_day -= 1;
        }
    }

    /// Moves to the next month, keeping the selected day if possible.
    pub fn next_month(&mut self) {
        self.current_month += Duration::days(32);
        self.current_month = self.current_month.with_day(1).unwrap();

        let days_in_month =
            Self::days_in_month(self.current_month.year(), self.current_month.month());
        if self.selected_day > days_in_month {
            self.selected_day = days_in_month;
        }
    }

    /// Moves to the previous month, keeping the selected day if possible.
    pub fn prev_month(&mut self) {
        self.current_month -= Duration::days(1);
        self.current_month = self.current_month.with_day(1).unwrap();

        let days_in_month =
            Self::days_in_month(self.current_month.year(), self.current_month.month());
        if self.selected_day > days_in_month {
            self.selected_day = days_in_month;
        }
    }

    /// Jumps to today's date.
    pub fn select_today(&mut self) {
        self.current_month = self.today.with_day(1).unwrap();
        self.selected_day = self.today.day();
    }

    /// Returns the number of days in the given month.
    fn days_in_month(year: i32, month: u32) -> u32 {
        match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 => {
                if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) {
                    29
                } else {
                    28
                }
            }
            _ => 0,
        }
    }
}

/// Month calendar widget.
///
/// This widget manages the rendering and interaction of a month calendar.
/// It uses `MonthCalendarState` to track state and provides methods for
/// keyboard navigation and date selection.
#[derive(Debug, Clone)]
pub struct MonthCalendar {
    state: MonthCalendarState,
}

impl MonthCalendar {
    /// Creates a new month calendar widget with the given initial date.
    ///
    /// # Arguments
    ///
    /// * `initial_date` - The date to display initially
    ///
    /// # Examples
    ///
    /// ```rust
    /// use opengp::ui::widgets::MonthCalendar;
    /// use chrono::NaiveDate;
    ///
    /// let calendar = MonthCalendar::new(NaiveDate::from_ymd_opt(2026, 2, 14).unwrap());
    /// assert_eq!(calendar.selected_date().day(), 14);
    /// ```
    pub fn new(initial_date: NaiveDate) -> Self {
        Self {
            state: MonthCalendarState::new(initial_date),
        }
    }

    /// Returns the currently selected date.
    pub fn selected_date(&self) -> NaiveDate {
        self.state
            .current_month
            .with_day(self.state.selected_day)
            .unwrap()
    }

    /// Moves selection to the next day.
    pub fn next_day(&mut self) {
        self.state.next_day();
    }

    /// Moves selection to the previous day.
    pub fn prev_day(&mut self) {
        self.state.prev_day();
    }

    /// Moves to the next month.
    pub fn next_month(&mut self) {
        self.state.next_month();
    }

    /// Moves to the previous month.
    pub fn prev_month(&mut self) {
        self.state.prev_month();
    }

    /// Jumps to today's date.
    pub fn select_today(&mut self) {
        self.state.select_today();
    }

    /// Handles keyboard events for month calendar navigation.
    ///
    /// This method provides standardized keybind handling for all month calendar
    /// instances. It returns `true` if the key was handled, `false` otherwise.
    ///
    /// # Keybinds (according to KEYBINDS.md)
    ///
    /// - `↑` / `↓`: Navigate to previous/next day
    /// - `h` / `l`: Navigate to previous/next month
    /// - `t`: Jump to today's date
    ///
    /// # Arguments
    ///
    /// * `key` - The keyboard event to handle
    ///
    /// # Returns
    ///
    /// * `true` if the key was handled and the calendar state was updated
    /// * `false` if the key was not recognized as a calendar navigation key
    ///
    /// # Examples
    ///
    /// ```rust
    /// use opengp::ui::widgets::MonthCalendar;
    /// use chrono::NaiveDate;
    /// use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    ///
    /// let mut calendar = MonthCalendar::new(NaiveDate::from_ymd_opt(2026, 2, 14).unwrap());
    /// let key = KeyEvent::new(KeyCode::Char('h'), KeyModifiers::empty());
    ///
    /// if calendar.handle_key_event(key) {
    ///     // Calendar was updated, render the change
    /// }
    /// ```
    pub fn handle_key_event(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Up => {
                self.prev_day();
                true
            }
            KeyCode::Down => {
                self.next_day();
                true
            }
            KeyCode::Char('h') => {
                self.prev_month();
                true
            }
            KeyCode::Char('l') => {
                self.next_month();
                true
            }
            KeyCode::Char('t') => {
                self.select_today();
                true
            }
            _ => false,
        }
    }

    /// Returns a reference to the internal state.
    pub fn state(&self) -> &MonthCalendarState {
        &self.state
    }

    /// Returns a mutable reference to the internal state.
    pub fn state_mut(&mut self) -> &mut MonthCalendarState {
        &mut self.state
    }

    /// Renders the month calendar widget.
    ///
    /// # Arguments
    ///
    /// * `frame` - The frame to render to
    /// * `area` - The area to render within
    /// * `is_focused` - Whether the calendar has focus (affects border color)
    pub fn render(&self, frame: &mut Frame, area: Rect, is_focused: bool) {
        let month_year = format!(
            "{} {}",
            Self::get_month_name(self.state.current_month.month()),
            self.state.current_month.year()
        );

        let first_weekday = self.state.current_month.weekday();
        let days_in_month = Self::days_in_month(
            self.state.current_month.year(),
            self.state.current_month.month(),
        );

        let mut lines = vec![Line::from(vec![
            Span::styled(
                "Mon ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Tue ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Wed ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Thu ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Fri ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("Sat ", Style::default().fg(Color::Cyan)),
            Span::styled("Sun ", Style::default().fg(Color::Cyan)),
        ])];

        let mut current_day = 1;
        let first_day_offset = Self::first_weekday_offset(first_weekday);
        let mut is_first_week = true;

        while current_day <= days_in_month {
            let mut day_cells = Vec::new();

            for day_of_week in 0..7 {
                if (is_first_week && day_of_week < first_day_offset) || current_day > days_in_month
                {
                    day_cells.push(Span::raw("   "));
                } else {
                    let is_today = self.state.today.year() == self.state.current_month.year()
                        && self.state.today.month() == self.state.current_month.month()
                        && self.state.today.day() == current_day;

                    let is_selected = current_day == self.state.selected_day;
                    let is_weekend = day_of_week >= 5;

                    let style = if is_selected {
                        Style::default()
                            .fg(Color::Black)
                            .bg(Color::Yellow)
                            .add_modifier(Modifier::BOLD)
                    } else if is_today {
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD)
                    } else if is_weekend {
                        Style::default().fg(Color::Cyan)
                    } else {
                        Style::default().fg(Color::White)
                    };

                    day_cells.push(Span::styled(format!("{:2} ", current_day), style));
                    current_day += 1;
                }
            }

            lines.push(Line::from(day_cells));
            is_first_week = false;
        }

        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("↑↓", Style::default().fg(Color::Cyan)),
            Span::raw(": Day  "),
            Span::styled("h/l", Style::default().fg(Color::Cyan)),
            Span::raw(": Month"),
        ]));
        lines.push(Line::from(vec![
            Span::styled("t", Style::default().fg(Color::Cyan)),
            Span::raw(": Today  "),
            Span::styled("n", Style::default().fg(Color::Cyan)),
            Span::raw(": New  "),
            Span::styled("Enter", Style::default().fg(Color::Cyan)),
            Span::raw(": Day View"),
        ]));

        let paragraph = Paragraph::new(lines).block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" {} ", month_year))
                .border_style(if is_focused {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default().fg(Color::White)
                }),
        );

        frame.render_widget(paragraph, area);
    }

    /// Returns the name of the month for the given month number.
    fn get_month_name(month: u32) -> &'static str {
        match month {
            1 => "January",
            2 => "February",
            3 => "March",
            4 => "April",
            5 => "May",
            6 => "June",
            7 => "July",
            8 => "August",
            9 => "September",
            10 => "October",
            11 => "November",
            12 => "December",
            _ => "Unknown",
        }
    }

    /// Returns the number of days in the given month.
    fn days_in_month(year: i32, month: u32) -> u32 {
        MonthCalendarState::days_in_month(year, month)
    }

    /// Returns the weekday offset for the first day of the month.
    /// Monday = 0, Tuesday = 1, ..., Sunday = 6
    fn first_weekday_offset(weekday: Weekday) -> usize {
        match weekday {
            Weekday::Mon => 0,
            Weekday::Tue => 1,
            Weekday::Wed => 2,
            Weekday::Thu => 3,
            Weekday::Fri => 4,
            Weekday::Sat => 5,
            Weekday::Sun => 6,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_new_sets_correct_date() {
        let date = NaiveDate::from_ymd_opt(2026, 2, 14).unwrap();
        let state = MonthCalendarState::new(date);

        assert_eq!(state.selected_day(), 14);
        assert_eq!(state.current_month().month(), 2);
        assert_eq!(state.current_month().year(), 2026);
    }

    #[test]
    fn test_state_next_day_within_month() {
        let date = NaiveDate::from_ymd_opt(2026, 2, 14).unwrap();
        let mut state = MonthCalendarState::new(date);

        state.next_day();
        assert_eq!(state.selected_day(), 15);
        assert_eq!(state.current_month().month(), 2);
    }

    #[test]
    fn test_state_next_day_wraps_to_next_month() {
        let date = NaiveDate::from_ymd_opt(2026, 2, 28).unwrap();
        let mut state = MonthCalendarState::new(date);

        state.next_day();
        assert_eq!(state.selected_day(), 1);
        assert_eq!(state.current_month().month(), 3);
    }

    #[test]
    fn test_state_prev_day_within_month() {
        let date = NaiveDate::from_ymd_opt(2026, 2, 14).unwrap();
        let mut state = MonthCalendarState::new(date);

        state.prev_day();
        assert_eq!(state.selected_day(), 13);
        assert_eq!(state.current_month().month(), 2);
    }

    #[test]
    fn test_state_prev_day_wraps_to_prev_month() {
        let date = NaiveDate::from_ymd_opt(2026, 2, 1).unwrap();
        let mut state = MonthCalendarState::new(date);

        state.prev_day();
        assert_eq!(state.selected_day(), 31);
        assert_eq!(state.current_month().month(), 1);
    }

    #[test]
    fn test_state_next_month_keeps_day_if_valid() {
        let date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
        let mut state = MonthCalendarState::new(date);

        state.next_month();
        assert_eq!(state.selected_day(), 15);
        assert_eq!(state.current_month().month(), 2);
    }

    #[test]
    fn test_state_next_month_clamps_day_if_invalid() {
        let date = NaiveDate::from_ymd_opt(2026, 1, 31).unwrap();
        let mut state = MonthCalendarState::new(date);

        state.next_month();
        assert_eq!(state.selected_day(), 28);
        assert_eq!(state.current_month().month(), 2);
    }

    #[test]
    fn test_state_prev_month_keeps_day_if_valid() {
        let date = NaiveDate::from_ymd_opt(2026, 3, 15).unwrap();
        let mut state = MonthCalendarState::new(date);

        state.prev_month();
        assert_eq!(state.selected_day(), 15);
        assert_eq!(state.current_month().month(), 2);
    }

    #[test]
    fn test_state_prev_month_clamps_day_if_invalid() {
        let date = NaiveDate::from_ymd_opt(2026, 3, 31).unwrap();
        let mut state = MonthCalendarState::new(date);

        state.prev_month();
        assert_eq!(state.selected_day(), 28);
        assert_eq!(state.current_month().month(), 2);
    }

    #[test]
    fn test_state_select_today() {
        let date = NaiveDate::from_ymd_opt(2026, 2, 14).unwrap();
        let mut state = MonthCalendarState::new(date);

        state.next_month();
        assert_eq!(state.current_month().month(), 3);

        state.select_today();
        assert_eq!(state.current_month().month(), state.today().month());
        assert_eq!(state.selected_day(), state.today().day());
    }

    #[test]
    fn test_days_in_month_january() {
        assert_eq!(MonthCalendarState::days_in_month(2026, 1), 31);
    }

    #[test]
    fn test_days_in_month_february_non_leap() {
        assert_eq!(MonthCalendarState::days_in_month(2026, 2), 28);
    }

    #[test]
    fn test_days_in_month_february_leap() {
        assert_eq!(MonthCalendarState::days_in_month(2024, 2), 29);
    }

    #[test]
    fn test_days_in_month_april() {
        assert_eq!(MonthCalendarState::days_in_month(2026, 4), 30);
    }

    #[test]
    fn test_widget_new_sets_correct_date() {
        let date = NaiveDate::from_ymd_opt(2026, 2, 14).unwrap();
        let calendar = MonthCalendar::new(date);

        assert_eq!(calendar.selected_date().day(), 14);
    }

    #[test]
    fn test_widget_next_day() {
        let date = NaiveDate::from_ymd_opt(2026, 2, 14).unwrap();
        let mut calendar = MonthCalendar::new(date);

        calendar.next_day();
        assert_eq!(calendar.selected_date().day(), 15);
    }

    #[test]
    fn test_widget_prev_day() {
        let date = NaiveDate::from_ymd_opt(2026, 2, 14).unwrap();
        let mut calendar = MonthCalendar::new(date);

        calendar.prev_day();
        assert_eq!(calendar.selected_date().day(), 13);
    }

    #[test]
    fn test_widget_next_month() {
        let date = NaiveDate::from_ymd_opt(2026, 2, 14).unwrap();
        let mut calendar = MonthCalendar::new(date);

        calendar.next_month();
        assert_eq!(calendar.selected_date().month(), 3);
    }

    #[test]
    fn test_widget_prev_month() {
        let date = NaiveDate::from_ymd_opt(2026, 2, 14).unwrap();
        let mut calendar = MonthCalendar::new(date);

        calendar.prev_month();
        assert_eq!(calendar.selected_date().month(), 1);
    }

    #[test]
    fn test_widget_select_today() {
        let date = NaiveDate::from_ymd_opt(2026, 2, 14).unwrap();
        let mut calendar = MonthCalendar::new(date);

        calendar.next_month();
        calendar.select_today();

        let today = chrono::Local::now().naive_local().date();
        assert_eq!(calendar.selected_date().month(), today.month());
        assert_eq!(calendar.selected_date().day(), today.day());
    }

    #[test]
    fn test_widget_state_access() {
        let date = NaiveDate::from_ymd_opt(2026, 2, 14).unwrap();
        let calendar = MonthCalendar::new(date);

        let state = calendar.state();
        assert_eq!(state.selected_day(), 14);
    }

    #[test]
    fn test_widget_state_mut_access() {
        let date = NaiveDate::from_ymd_opt(2026, 2, 14).unwrap();
        let mut calendar = MonthCalendar::new(date);

        calendar.state_mut().next_day();
        assert_eq!(calendar.selected_date().day(), 15);
    }

    #[test]
    fn test_get_month_name() {
        assert_eq!(MonthCalendar::get_month_name(1), "January");
        assert_eq!(MonthCalendar::get_month_name(2), "February");
        assert_eq!(MonthCalendar::get_month_name(12), "December");
    }

    #[test]
    fn test_first_weekday_offset() {
        assert_eq!(MonthCalendar::first_weekday_offset(Weekday::Mon), 0);
        assert_eq!(MonthCalendar::first_weekday_offset(Weekday::Tue), 1);
        assert_eq!(MonthCalendar::first_weekday_offset(Weekday::Sun), 6);
    }
}

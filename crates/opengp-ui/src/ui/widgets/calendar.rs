//! Calendar Widget Module
//!
//! Provides a calendar widget for date selection in the OpenGP TUI.
//! Supports month navigation and date selection.

use chrono::{Datelike, Local, NaiveDate};
use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Widget};
use ratatui::{buffer::Buffer, layout::Rect};
use std::collections::HashMap;
use time::{Date, Month};

/// Actions that can be triggered by the calendar widget
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CalendarAction {
    /// Select a specific date
    SelectDate,
    /// Navigate to next month
    NextMonth,
    /// Navigate to previous month
    PrevMonth,
    /// Close the calendar
    Close,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CalendarMode {
    Scheduling,
    DatePicker,
}

/// Trait for styling dates in the calendar.
#[allow(dead_code)]
pub trait DateStyler {
    /// Return the `Style` that should be applied to the provided date.
    fn get_style(&self, date: Date) -> Style;
}

/// Convert a `chrono::NaiveDate` into `time::Date` for styling helpers.
#[allow(dead_code)]
pub fn chrono_to_time(date: NaiveDate) -> Date {
    // SAFETY: date.month() is 1-12, always valid Month value
    #[allow(clippy::expect_used)]
    let month = Month::try_from(date.month() as u8).expect("invalid month");
    // SAFETY: chrono::NaiveDate is guaranteed to be valid calendar date
    #[allow(clippy::expect_used)]
    Date::from_calendar_date(date.year(), month, date.day() as u8)
        .expect("chrono NaiveDate is always valid")
}

/// Calendar widget for date selection
#[derive(Debug, Clone)]
pub struct CalendarWidget {
    /// Currently displayed month (year, month)
    pub current_month: (i32, u32),
    /// Selected date
    pub selected_date: Option<NaiveDate>,
    /// Focused date for navigation
    pub focused_date: NaiveDate,
    /// Error messages
    pub errors: HashMap<String, String>,
}

impl CalendarWidget {
    /// Create a new calendar widget with today's date
    pub fn new() -> Self {
        let today = Local::now().naive_local().date();
        Self {
            current_month: (today.year(), today.month()),
            selected_date: None,
            focused_date: today,
            errors: HashMap::new(),
        }
    }

    /// Create a calendar widget with a specific initial date
    pub fn with_date(date: NaiveDate) -> Self {
        Self {
            current_month: (date.year(), date.month()),
            selected_date: None,
            focused_date: date,
            errors: HashMap::new(),
        }
    }

    /// Get the currently displayed month and year
    pub fn current_month(&self) -> (i32, u32) {
        self.current_month
    }

    /// Get the selected date
    pub fn selected_date(&self) -> Option<NaiveDate> {
        self.selected_date
    }

    /// Set the selected date
    pub fn set_selected_date(&mut self, date: NaiveDate) {
        self.selected_date = Some(date);
    }

    /// Navigate to the next month
    pub fn next_month(&mut self) {
        let (year, month) = self.current_month;
        let (new_year, new_month) = if month == 12 {
            (year + 1, 1)
        } else {
            (year, month + 1)
        };
        self.current_month = (new_year, new_month);
    }

    /// Navigate to the previous month
    pub fn prev_month(&mut self) {
        let (year, month) = self.current_month;
        let (new_year, new_month) = if month == 1 {
            (year - 1, 12)
        } else {
            (year, month - 1)
        };
        self.current_month = (new_year, new_month);
    }

    pub fn handle_mouse(&mut self, mouse: MouseEvent, area: Rect) -> Option<CalendarAction> {
        if let MouseEventKind::Up(MouseButton::Left) = mouse.kind {
            if let Some(day_index) = self.get_day_index_at(mouse.column, mouse.row, area) {
                if let Some(date) = self.day_at_index(day_index) {
                    self.focused_date = date;
                    self.selected_date = Some(date);
                    return Some(CalendarAction::SelectDate);
                }
            }
        }

        None
    }

    /// Create a date picker mode calendar with the given initial date
    pub fn show_date_picker(current_value: Option<NaiveDate>) -> Self {
        let initial_date = current_value.unwrap_or_else(|| Local::now().naive_local().date());
        Self {
            current_month: (initial_date.year(), initial_date.month()),
            selected_date: current_value,
            focused_date: initial_date,
            errors: HashMap::new(),
        }
    }

    /// Render the calendar widget with mode-specific styling
    pub fn render_calendar(&self, area: Rect, buf: &mut Buffer, mode: CalendarMode) {
        if area.is_empty() || area.width < 21 || area.height < 9 {
            return;
        }

        match mode {
            CalendarMode::Scheduling => self.render_scheduling_mode(area, buf),
            CalendarMode::DatePicker => self.render_date_picker_mode(area, buf),
        }
    }

    fn render_scheduling_mode(&self, area: Rect, buf: &mut Buffer) {
        // Create the border block with month/year title
        let title = self.format_month_title();
        let block = Block::default()
            .title(format!(" {} ", title))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Gray));

        block.clone().render(area, buf);
        let inner = block.inner(area);

        // Render weekday header
        self.render_weekday_header(inner, buf);

        // Render dates with appointment styling
        self.render_dates_scheduling(inner, buf);
    }

    fn render_date_picker_mode(&self, area: Rect, buf: &mut Buffer) {
        // Compact rendering for date picker mode
        let title = self.format_month_title();
        let block = Block::default()
            .title(format!(" {} ", title))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow));

        block.clone().render(area, buf);
        let inner = block.inner(area);

        // Render compact weekday header
        self.render_weekday_header(inner, buf);

        // Render dates with date picker styling
        self.render_dates_picker(inner, buf);
    }

    fn format_month_title(&self) -> String {
        let (year, month) = self.current_month;
        let month_name = match month {
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
        };
        format!("{} {}", month_name, year)
    }

    fn render_weekday_header(&self, area: Rect, buf: &mut Buffer) {
        if area.height < 2 {
            return;
        }

        let cell_width = (area.width as usize / 7).max(2);
        let weekdays = ["Mo", "Tu", "We", "Th", "Fr", "Sa", "Sun"];
        let style = Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD);

        for (i, weekday) in weekdays.iter().enumerate() {
            let x = area.x + (i as u16 * cell_width as u16);
            if x < area.right() {
                let label = if cell_width >= 3 {
                    *weekday
                } else {
                    &weekday[..2]
                };
                buf.set_string(x, area.y, label, style);
            }
        }
    }

    fn render_dates_scheduling(&self, area: Rect, buf: &mut Buffer) {
        let start_y = area.y.saturating_add(2);
        if start_y >= area.bottom() {
            return;
        }

        let cell_width = (area.width as usize / 7).max(2);
        let row_height = 2usize;
        let (_year, month) = self.current_month;

        for day_index in 0..42 {
            if let Some(date) = self.day_at_index(day_index) {
                let row = start_y + (day_index as u16 / 7) * row_height as u16;
                let col = area.x + ((day_index % 7) as u16 * cell_width as u16);

                if row >= area.bottom() || col + 2 > area.right() {
                    break;
                }

                let style = if Some(date) == self.selected_date {
                    Style::default()
                        .bg(Color::Blue)
                        .fg(Color::Black)
                        .add_modifier(Modifier::BOLD)
                } else if date == self.focused_date {
                    Style::default().fg(Color::Yellow)
                } else if date.month() != month {
                    Style::default().fg(Color::DarkGray)
                } else {
                    Style::default().fg(Color::White)
                };

                buf.set_string(
                    col,
                    row,
                    format!("{:>width$}", date.day(), width = cell_width),
                    style,
                );
            }
        }
    }

    fn render_dates_picker(&self, area: Rect, buf: &mut Buffer) {
        let start_y = area.y.saturating_add(2);
        if start_y >= area.bottom() {
            return;
        }

        let cell_width = (area.width as usize / 7).max(2);
        let row_height = 2usize;
        let (_year, month) = self.current_month;

        for day_index in 0..42 {
            if let Some(date) = self.day_at_index(day_index) {
                let row = start_y + (day_index as u16 / 7) * row_height as u16;
                let col = area.x + ((day_index % 7) as u16 * cell_width as u16);

                if row >= area.bottom() || col + 2 > area.right() {
                    break;
                }

                let style = if Some(date) == self.selected_date {
                    Style::default()
                        .bg(Color::Magenta)
                        .fg(Color::White)
                        .add_modifier(Modifier::UNDERLINED)
                } else if date == self.focused_date {
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::UNDERLINED)
                } else if date.month() != month {
                    Style::default().fg(Color::DarkGray)
                } else {
                    Style::default().fg(Color::White)
                };

                buf.set_string(
                    col,
                    row,
                    format!("{:>width$}", date.day(), width = cell_width),
                    style,
                );
            }
        }
    }

    fn day_at_index(&self, index: usize) -> Option<NaiveDate> {
        let (year, month) = self.current_month;
        let first_of_month = NaiveDate::from_ymd_opt(year, month, 1)?;
        let first_weekday = (first_of_month.weekday().number_from_monday() - 1) as i32;
        let day_number = index as i32 + 1 - first_weekday;

        if day_number < 1 || day_number > self.days_in_month(year, month) as i32 {
            return None;
        }

        NaiveDate::from_ymd_opt(year, month, day_number as u32)
    }

    fn get_day_index_at(&self, column: u16, row: u16, area: Rect) -> Option<usize> {
        let border_left = 2u16;
        let start_y = area.y.saturating_add(3);
        let start_x = area.x.saturating_add(border_left);
        let inner_width = area.width.saturating_sub(2);
        let cell_width = ((inner_width as usize) / 7).max(2) as u16;
        let row_height = 2u16;

        if row < start_y {
            return None;
        }

        let inner_right = area.x + area.width.saturating_sub(1);
        let inner_bottom = area.y + area.height.saturating_sub(1);

        if column < start_x || row >= inner_bottom || column >= inner_right {
            return None;
        }

        let dy = row - start_y;
        let dx = column - start_x;

        let row_idx = (dy / row_height) as usize;
        let col = (dx / cell_width) as usize;

        if col > 6 || row_idx > 5 {
            return None;
        }

        Some(row_idx * 7 + col)
    }

    fn days_in_month(&self, year: i32, month: u32) -> u32 {
        let (next_year, next_month) = if month == 12 {
            (year + 1, 1)
        } else {
            (year, month + 1)
        };

        // SAFETY: month is valid (1-12) and day 1 is always valid
        #[allow(clippy::expect_used)]
        let start = NaiveDate::from_ymd_opt(year, month, 1).expect("valid month start");
        // SAFETY: next_month is valid (1-12) and day 1 is always valid
        #[allow(clippy::expect_used)]
        let next_start =
            NaiveDate::from_ymd_opt(next_year, next_month, 1).expect("valid month start");

        (next_start - start).num_days() as u32
    }

    /// Clear any error messages
    pub fn clear_errors(&mut self) {
        self.errors.clear();
    }

    /// Add an error message
    pub fn add_error(&mut self, key: impl Into<String>, message: impl Into<String>) {
        self.errors.insert(key.into(), message.into());
    }
}

impl Default for CalendarWidget {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
fn time_to_naive(date: Date) -> NaiveDate {
    let (year, month, day) = date.to_calendar_date();
    // SAFETY: time::Date is guaranteed to be valid calendar date
    #[allow(clippy::expect_used)]
    NaiveDate::from_ymd_opt(year, month as u32, day as u32)
        .expect("time::Date to chrono::NaiveDate")
}

/// Styler that highlights dates with appointments.
#[derive(Debug, Clone, Default)]
pub struct AppointmentStyler {
    indicators: HashMap<Date, Color>,
}

impl AppointmentStyler {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_indicators(indicators: HashMap<Date, Color>) -> Self {
        Self { indicators }
    }

    pub fn add_indicator(&mut self, date: Date, color: Color) {
        self.indicators.insert(date, color);
    }

    pub fn remove_indicator(&mut self, date: Date) {
        self.indicators.remove(&date);
    }
}

impl DateStyler for CalendarWidget {
    fn get_style(&self, date: Date) -> Style {
        let target = time_to_naive(date);

        if Some(target) == self.selected_date {
            Style::default()
                .bg(Color::Blue)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        }
    }
}

impl DateStyler for AppointmentStyler {
    fn get_style(&self, date: Date) -> Style {
        if let Some(color) = self.indicators.get(&date) {
            Style::default().fg(*color).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        }
    }
}

/// Styler used by simple date picker forms.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DatePickerStyler {
    base_style: Style,
    focus_style: Style,
    focus_date: Option<Date>,
}

impl DatePickerStyler {
    #[allow(dead_code)]
    pub fn new(base_style: Style, focus_style: Style) -> Self {
        Self {
            base_style,
            focus_style,
            focus_date: None,
        }
    }

    #[allow(dead_code)]
    pub fn with_focus(base_style: Style, focus_style: Style, focus_date: Date) -> Self {
        Self {
            base_style,
            focus_style,
            focus_date: Some(focus_date),
        }
    }

    #[allow(dead_code)]
    pub fn set_focus(&mut self, focus: Option<Date>) {
        self.focus_date = focus;
    }
}

impl Default for DatePickerStyler {
    fn default() -> Self {
        let base = Style::default();
        let focus = base.add_modifier(Modifier::UNDERLINED);
        Self::new(base, focus)
    }
}

impl DateStyler for DatePickerStyler {
    fn get_style(&self, date: Date) -> Style {
        if Some(date) == self.focus_date {
            self.focus_style
        } else {
            self.base_style
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyModifiers;
    use time::Month;

    #[test]
    fn test_calendar_widget_new() {
        let calendar = CalendarWidget::new();
        assert_eq!(calendar.selected_date, None);
        assert!(calendar.errors.is_empty());
    }

    #[test]
    fn test_calendar_widget_with_date() {
        let date = NaiveDate::from_ymd_opt(2026, 2, 25).unwrap();
        let calendar = CalendarWidget::with_date(date);
        assert_eq!(calendar.focused_date, date);
        assert_eq!(calendar.current_month, (2026, 2));
    }

    #[test]
    fn test_set_selected_date() {
        let mut calendar = CalendarWidget::new();
        let date = NaiveDate::from_ymd_opt(2026, 3, 15).unwrap();
        calendar.set_selected_date(date);
        assert_eq!(calendar.selected_date(), Some(date));
    }

    #[test]
    fn test_next_month() {
        let mut calendar = CalendarWidget::with_date(NaiveDate::from_ymd_opt(2026, 2, 25).unwrap());
        calendar.next_month();
        assert_eq!(calendar.current_month, (2026, 3));
    }

    #[test]
    fn test_next_month_year_wrap() {
        let mut calendar =
            CalendarWidget::with_date(NaiveDate::from_ymd_opt(2026, 12, 25).unwrap());
        calendar.next_month();
        assert_eq!(calendar.current_month, (2027, 1));
    }

    #[test]
    fn test_prev_month() {
        let mut calendar = CalendarWidget::with_date(NaiveDate::from_ymd_opt(2026, 3, 25).unwrap());
        calendar.prev_month();
        assert_eq!(calendar.current_month, (2026, 2));
    }

    #[test]
    fn test_prev_month_year_wrap() {
        let mut calendar = CalendarWidget::with_date(NaiveDate::from_ymd_opt(2026, 1, 25).unwrap());
        calendar.prev_month();
        assert_eq!(calendar.current_month, (2025, 12));
    }

    #[test]
    fn test_navigation_actions_move_months() {
        let mut calendar = CalendarWidget::with_date(NaiveDate::from_ymd_opt(2026, 5, 1).unwrap());
        calendar.next_month();
        assert_eq!(calendar.current_month, (2026, 6));
        calendar.prev_month();
        assert_eq!(calendar.current_month, (2026, 5));
    }

    #[test]
    fn test_handle_mouse_selects_current_month_date() {
        let mut calendar = CalendarWidget::with_date(NaiveDate::from_ymd_opt(2026, 2, 10).unwrap());
        let area = Rect::new(0, 0, 30, 12);
        let first_of_month = NaiveDate::from_ymd_opt(2026, 2, 1).unwrap();
        let offset = (first_of_month.weekday().number_from_monday() - 1) as u16;
        let inner_width = area.width.saturating_sub(2);
        let cell_width = ((inner_width as usize) / 7).max(2) as u16;
        let column = area.x + 2 + offset * cell_width;
        let row = area.y + 3;
        let mouse = MouseEvent {
            kind: MouseEventKind::Up(MouseButton::Left),
            column,
            row,
            modifiers: KeyModifiers::NONE,
        };

        assert_eq!(
            calendar.handle_mouse(mouse, area),
            Some(CalendarAction::SelectDate)
        );
        assert_eq!(calendar.selected_date(), Some(first_of_month));
    }

    #[test]
    fn test_error_handling() {
        let mut calendar = CalendarWidget::new();
        calendar.add_error("date", "Invalid date");
        assert_eq!(
            calendar.errors.get("date"),
            Some(&"Invalid date".to_string())
        );
        calendar.clear_errors();
        assert!(calendar.errors.is_empty());
    }

    #[test]
    fn test_chrono_to_time_conversion_helper() {
        let naive = NaiveDate::from_ymd_opt(2026, 2, 25).unwrap();
        let converted = chrono_to_time(naive);

        assert_eq!(converted.year(), 2026);
        assert_eq!(converted.month(), Month::February);
        assert_eq!(converted.day(), 25);
    }

    #[test]
    fn test_calendar_widget_date_styler_highlights_selected() {
        let date = NaiveDate::from_ymd_opt(2026, 4, 5).unwrap();
        let mut calendar = CalendarWidget::with_date(date);
        calendar.set_selected_date(date);

        let style = calendar.get_style(chrono_to_time(date));
        let expected = Style::default()
            .bg(Color::Blue)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD);

        assert_eq!(style, expected);
    }

    #[test]
    fn test_appointment_styler_applies_indicator() {
        let mut styler = AppointmentStyler::new();
        let appointment_date = chrono_to_time(NaiveDate::from_ymd_opt(2026, 5, 11).unwrap());
        styler.add_indicator(appointment_date, Color::Green);

        let indicator_style = Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD);
        assert_eq!(styler.get_style(appointment_date), indicator_style);

        let other_date = chrono_to_time(NaiveDate::from_ymd_opt(2026, 5, 12).unwrap());
        assert_eq!(styler.get_style(other_date), Style::default());
    }

    #[test]
    fn test_date_picker_styler_focus_style() {
        let base_style = Style::default();
        let focus_style = Style::default()
            .bg(Color::Magenta)
            .add_modifier(Modifier::UNDERLINED);
        let focus_date = chrono_to_time(NaiveDate::from_ymd_opt(2026, 6, 1).unwrap());

        let mut styler = DatePickerStyler::new(base_style.clone(), focus_style.clone());
        styler.set_focus(Some(focus_date));

        assert_eq!(styler.get_style(focus_date), focus_style);

        let other_date = chrono_to_time(NaiveDate::from_ymd_opt(2026, 6, 2).unwrap());
        assert_eq!(styler.get_style(other_date), base_style);
    }

    #[test]
    fn test_show_date_picker_with_none() {
        let picker = CalendarWidget::show_date_picker(None);
        let today = Local::now().naive_local().date();
        assert_eq!(picker.focused_date, today);
        assert_eq!(picker.selected_date, None);
        assert_eq!(picker.current_month, (today.year(), today.month()));
    }

    #[test]
    fn test_show_date_picker_with_date() {
        let target_date = NaiveDate::from_ymd_opt(2026, 5, 15).unwrap();
        let picker = CalendarWidget::show_date_picker(Some(target_date));

        assert_eq!(picker.focused_date, target_date);
        assert_eq!(picker.selected_date, Some(target_date));
        assert_eq!(picker.current_month, (2026, 5));
    }

    #[test]
    fn test_date_picker_mode_render() {
        let date = NaiveDate::from_ymd_opt(2026, 3, 10).unwrap();
        let mut calendar = CalendarWidget::with_date(date);
        calendar.set_selected_date(date);

        let area = Rect::new(0, 0, 25, 12);
        let mut buf = Buffer::empty(area);

        calendar.render_calendar(area, &mut buf, CalendarMode::DatePicker);

        assert!(!buf.content.is_empty());
    }

    #[test]
    fn test_scheduling_mode_render() {
        let date = NaiveDate::from_ymd_opt(2026, 3, 10).unwrap();
        let mut calendar = CalendarWidget::with_date(date);
        calendar.set_selected_date(date);

        let area = Rect::new(0, 0, 25, 12);
        let mut buf = Buffer::empty(area);

        calendar.render_calendar(area, &mut buf, CalendarMode::Scheduling);

        assert!(!buf.content.is_empty());
    }

    #[test]
    fn test_format_month_title() {
        let calendar = CalendarWidget::with_date(NaiveDate::from_ymd_opt(2026, 7, 15).unwrap());
        let title = calendar.format_month_title();
        assert_eq!(title, "July 2026");
    }

    #[test]
    fn test_render_calendar_small_area_ignored() {
        let calendar = CalendarWidget::new();
        let small_area = Rect::new(0, 0, 10, 5);
        let mut buf = Buffer::empty(small_area);

        calendar.render_calendar(small_area, &mut buf, CalendarMode::DatePicker);
    }

    #[test]
    fn test_date_picker_preserves_selected_date() {
        let target_date = NaiveDate::from_ymd_opt(2026, 2, 28).unwrap();
        let picker = CalendarWidget::show_date_picker(Some(target_date));

        assert_eq!(picker.selected_date(), Some(target_date));
    }

    #[test]
    fn test_date_picker_month_navigation() {
        let target_date = NaiveDate::from_ymd_opt(2026, 6, 15).unwrap();
        let mut picker = CalendarWidget::show_date_picker(Some(target_date));

        picker.next_month();
        assert_eq!(picker.current_month, (2026, 7));

        picker.prev_month();
        assert_eq!(picker.current_month, (2026, 6));
    }
}

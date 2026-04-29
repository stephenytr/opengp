//! Date Picker Popup Widget
//!
//! Reusable calendar popup for selecting dates in form fields.
//! Provides keyboard-driven date selection with centered overlay rendering.

use chrono::{Datelike, NaiveDate};
use crossterm::event::{KeyCode, KeyEvent, MouseEvent, MouseEventKind};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::{Clear, Widget};
use rat_focus::{FocusFlag, HasFocus, FocusBuilder};

use super::CalendarWidget;
use crate::ui::shared::hover_style;
use crate::ui::theme::Theme;

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
    theme: Theme,
    hovered_index: Option<usize>,
    popup_area: Option<Rect>,
    pub focus: FocusFlag,
}

impl DatePickerPopup {
    /// Create a new date picker popup.
    pub fn new(theme: Theme) -> Self {
        Self {
            calendar: CalendarWidget::new(theme.clone()),
            visible: false,
            initial_date: None,
            theme,
            hovered_index: None,
            popup_area: None,
            focus: FocusFlag::default(),
        }
    }

    /// Open the date picker with an optional current date value.
    pub fn open(&mut self, current_value: Option<NaiveDate>) {
        self.calendar = CalendarWidget::show_date_picker(current_value, self.theme.clone());
        self.initial_date = current_value;
        self.visible = true;
    }

    /// Close the date picker.
    pub fn close(&mut self) {
        self.visible = false;
        self.hovered_index = None;
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

    /// Handle mouse events while the popup is open.
    ///
    /// Returns `Some(DatePickerAction)` if user clicked a date.
    /// Returns `None` for movement or other interactions.
    pub fn handle_mouse(&mut self, mouse: MouseEvent) -> Option<DatePickerAction> {
        if !self.visible {
            self.hovered_index = None;
            return None;
        }

        match mouse.kind {
            MouseEventKind::Moved => {
                // Track hover state based on popup area
                if let Some(area) = self.popup_area {
                    self.hovered_index = self.calendar.get_day_index_at(mouse.column, mouse.row, area);
                }
                None
            }
            MouseEventKind::Up(crossterm::event::MouseButton::Left) => {
                // Click selects the date
                if let Some(area) = self.popup_area {
                    if let Some(day_index) = self.calendar.get_day_index_at(mouse.column, mouse.row, area) {
                        if let Some(date) = self.calendar.day_at_index(day_index) {
                            self.calendar.focused_date = date;
                            self.calendar.selected_date = Some(date);
                            self.visible = false;
                            self.hovered_index = None;
                            return Some(DatePickerAction::Selected(date));
                        }
                    }
                }
                None
            }
            _ => {
                // Clear hover on other mouse events (like scroll leaving area)
                if let Some(area) = self.popup_area {
                    if mouse.column < area.x
                        || mouse.column >= area.x + area.width
                        || mouse.row < area.y
                        || mouse.row >= area.y + area.height
                    {
                        self.hovered_index = None;
                    }
                }
                None
            }
        }
    }

    /// Render the date picker popup as a centered overlay.
    pub fn render(&mut self, area: Rect, buf: &mut Buffer) {
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
        self.popup_area = Some(popup_area);

        Clear.render(popup_area, buf);

        self.render_calendar_with_hover(popup_area, buf);
    }

    fn render_calendar_with_hover(&self, area: Rect, buf: &mut Buffer) {
        use ratatui::style::Style;

        if area.is_empty() || area.width < 21 || area.height < 9 {
            return;
        }

        let (year, month) = self.calendar.current_month;
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
        let title = format!("{} {}", month_name, year);

        let block = ratatui::widgets::Block::default()
            .title(format!(" {} ", title))
            .borders(ratatui::widgets::Borders::ALL)
            .border_style(Style::default().fg(self.theme.colors.warning));

        block.clone().render(area, buf);
        let inner = block.inner(area);

        self.render_weekday_header(inner, buf);
        self.render_dates_with_hover(inner, buf);
    }

    fn render_weekday_header(&self, area: Rect, buf: &mut Buffer) {
        if area.height < 2 {
            return;
        }

        let cell_width = (area.width as usize / 7).max(2);
        let weekdays = ["Mo", "Tu", "We", "Th", "Fr", "Sa", "Sun"];
        let style = ratatui::style::Style::default()
            .fg(self.theme.colors.foreground)
            .add_modifier(ratatui::style::Modifier::BOLD);

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

    fn render_dates_with_hover(&self, area: Rect, buf: &mut Buffer) {
        let start_y = area.y.saturating_add(2);
        if start_y >= area.bottom() {
            return;
        }

        let cell_width = (area.width as usize / 7).max(2);
        let row_height = 2usize;
        let (_year, month) = self.calendar.current_month;

        for day_index in 0..42 {
            if let Some(date) = self.calendar.day_at_index(day_index) {
                let row = start_y + (day_index as u16 / 7) * row_height as u16;
                let col = area.x + ((day_index % 7) as u16 * cell_width as u16);

                if row >= area.bottom() || col + 2 > area.right() {
                    break;
                }

                let style = if self.hovered_index == Some(day_index) {
                    hover_style(&self.theme)
                } else if Some(date) == self.calendar.selected_date {
                    ratatui::style::Style::default()
                        .bg(self.theme.colors.secondary)
                        .fg(self.theme.colors.foreground)
                        .add_modifier(ratatui::style::Modifier::UNDERLINED)
                } else if date == self.calendar.focused_date {
                    ratatui::style::Style::default()
                        .fg(self.theme.colors.primary)
                        .add_modifier(ratatui::style::Modifier::UNDERLINED)
                } else if date.month() != month {
                    ratatui::style::Style::default().fg(self.theme.colors.text_dim)
                } else {
                    ratatui::style::Style::default().fg(self.theme.colors.foreground)
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

    /// Helper to calculate days in a month.
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
}

impl Default for DatePickerPopup {
    fn default() -> Self {
        Self::new(Theme::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[test]
    fn test_new_popup_not_visible() {
        let popup = DatePickerPopup::new(Theme::default());
        assert!(!popup.is_visible());
    }

    #[test]
    fn test_open_sets_visible_with_date() {
        let mut popup = DatePickerPopup::new(Theme::default());
        let date = NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();
        popup.open(Some(date));
        assert!(popup.is_visible());
        assert_eq!(popup.calendar.focused_date, date);
        assert_eq!(popup.initial_date, Some(date));
    }

    #[test]
    fn test_open_sets_visible_without_date() {
        let mut popup = DatePickerPopup::new(Theme::default());
        popup.open(None);
        assert!(popup.is_visible());
        assert_eq!(popup.initial_date, None);
    }

    #[test]
    fn test_close_clears_visibility() {
        let mut popup = DatePickerPopup::new(Theme::default());
        let date = NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();
        popup.open(Some(date));
        popup.close();
        assert!(!popup.is_visible());
    }

    #[test]
    fn test_handle_key_enter_confirms_selection() {
        let mut popup = DatePickerPopup::new(Theme::default());
        let date = NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();
        popup.open(Some(date));

        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let action = popup.handle_key(key);

        assert!(matches!(action, Some(DatePickerAction::Selected(_))));
        assert!(!popup.is_visible());
    }

    #[test]
    fn test_handle_key_space_confirms_selection() {
        let mut popup = DatePickerPopup::new(Theme::default());
        let date = NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();
        popup.open(Some(date));

        let key = KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE);
        let action = popup.handle_key(key);

        assert!(matches!(action, Some(DatePickerAction::Selected(_))));
        assert!(!popup.is_visible());
    }

    #[test]
    fn test_handle_key_esc_dismisses() {
        let mut popup = DatePickerPopup::new(Theme::default());
        let date = NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();
        popup.open(Some(date));

        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        let action = popup.handle_key(key);

        assert!(matches!(action, Some(DatePickerAction::Dismissed)));
        assert!(!popup.is_visible());
    }

    #[test]
    fn test_handle_key_up_navigates_previous_week() {
        let mut popup = DatePickerPopup::new(Theme::default());
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
        let mut popup = DatePickerPopup::new(Theme::default());
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
        let mut popup = DatePickerPopup::new(Theme::default());
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
        let mut popup = DatePickerPopup::new(Theme::default());
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
        let mut popup = DatePickerPopup::new(Theme::default());
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
        let mut popup = DatePickerPopup::new(Theme::default());
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
        let mut popup = DatePickerPopup::new(Theme::default());
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
        let mut popup = DatePickerPopup::new(Theme::default());
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
        let mut popup = DatePickerPopup::new(Theme::default());
        let date = NaiveDate::from_ymd_opt(2024, 3, 5).unwrap();
        popup.open(Some(date));

        let key = KeyEvent::new(KeyCode::Up, KeyModifiers::NONE);
        popup.handle_key(key);

        assert_eq!(popup.calendar.current_month.1, 2);
        assert_eq!(popup.calendar.focused_date.month(), 2);
    }

    #[test]
    fn test_handle_key_down_at_month_boundary_wraps_to_next_month() {
        let mut popup = DatePickerPopup::new(Theme::default());
        let date = NaiveDate::from_ymd_opt(2024, 3, 28).unwrap();
        popup.open(Some(date));

        let key = KeyEvent::new(KeyCode::Down, KeyModifiers::NONE);
        popup.handle_key(key);

        assert_eq!(popup.calendar.current_month.1, 4);
        assert_eq!(popup.calendar.focused_date.month(), 4);
    }

    #[test]
    fn test_handle_key_left_at_month_boundary_wraps_to_previous_month() {
        let mut popup = DatePickerPopup::new(Theme::default());
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
        let mut popup = DatePickerPopup::new(Theme::default());
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
        let mut popup = DatePickerPopup::new(Theme::default());
        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let action = popup.handle_key(key);

        assert!(action.is_none());
    }

    #[test]
    fn test_handle_key_unknown_key_returns_none() {
        let mut popup = DatePickerPopup::new(Theme::default());
        let date = NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();
        popup.open(Some(date));

        let key = KeyEvent::new(KeyCode::F(1), KeyModifiers::NONE);
        let action = popup.handle_key(key);

        assert!(action.is_none());
        assert!(popup.is_visible());
    }

    #[test]
    fn test_selected_date_is_returned_on_confirmation() {
        let mut popup = DatePickerPopup::new(Theme::default());
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
        let popup = DatePickerPopup::new(Theme::default());
        let days = popup.days_in_month(2024, 2);
        assert_eq!(days, 29);
    }

    #[test]
    fn test_days_in_month_february_non_leap_year() {
        let popup = DatePickerPopup::new(Theme::default());
        let days = popup.days_in_month(2023, 2);
        assert_eq!(days, 28);
    }

    #[test]
    fn test_days_in_month_april() {
        let popup = DatePickerPopup::new(Theme::default());
        let days = popup.days_in_month(2024, 4);
        assert_eq!(days, 30);
    }

    #[test]
    fn test_days_in_month_december() {
        let popup = DatePickerPopup::new(Theme::default());
        let days = popup.days_in_month(2024, 12);
        assert_eq!(days, 31);
    }

    #[test]
    fn test_default_creates_new_popup() {
        let popup = DatePickerPopup::default();
        assert!(!popup.is_visible());
    }
}

impl HasFocus for DatePickerPopup {
    fn build(&self, builder: &mut FocusBuilder) {
        builder.leaf_widget(self);
    }

    fn focus(&self) -> FocusFlag {
        self.focus.clone()
    }

    fn area(&self) -> Rect {
        Rect::default()
    }
}

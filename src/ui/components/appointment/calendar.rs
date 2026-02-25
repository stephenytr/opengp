use chrono::Datelike;
use chrono::NaiveDate;
use crossterm::event::{KeyEvent, MouseEvent, MouseEventKind};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::Stylize;
use ratatui::style::Style;
use ratatui::widgets::{Block, Borders, Widget};

use crate::ui::keybinds::{Action, KeyContext, KeybindRegistry};
use crate::ui::theme::Theme;

#[derive(Debug, Clone)]
pub struct CalendarDay {
    pub date: NaiveDate,
    pub is_current_month: bool,
    pub is_today: bool,
    pub is_selected: bool,
    pub has_appointments: bool,
}

#[derive(Debug, Clone)]
pub enum CalendarAction {
    SelectDate(NaiveDate),
    FocusDate(NaiveDate),
    MonthChanged(NaiveDate),
    GoToToday,
}

#[derive(Debug, Clone)]
pub struct Calendar {
    pub current_month: NaiveDate,
    pub selected_date: Option<NaiveDate>,
    pub focused_date: NaiveDate,
    pub days: Vec<CalendarDay>,
    pub theme: Theme,
    pub focused: bool,
}

impl Calendar {
    pub fn new(theme: Theme) -> Self {
        let today = chrono::Utc::now().date_naive();
        let current_month = NaiveDate::from_ymd_opt(today.year(), today.month(), 1).unwrap();

        let mut calendar = Self {
            current_month,
            selected_date: Some(today),
            focused_date: today,
            days: Vec::new(),
            theme,
            focused: false,
        };
        calendar.rebuild_days();
        calendar
    }

    pub fn previous_month(&self) -> NaiveDate {
        if self.current_month.month() == 1 {
            NaiveDate::from_ymd_opt(self.current_month.year() - 1, 12, 1).unwrap()
        } else {
            NaiveDate::from_ymd_opt(self.current_month.year(), self.current_month.month() - 1, 1)
                .unwrap()
        }
    }

    pub fn next_month(&self) -> NaiveDate {
        if self.current_month.month() == 12 {
            NaiveDate::from_ymd_opt(self.current_month.year() + 1, 1, 1).unwrap()
        } else {
            NaiveDate::from_ymd_opt(self.current_month.year(), self.current_month.month() + 1, 1)
                .unwrap()
        }
    }

    pub fn rebuild_days(&mut self) {
        self.days.clear();

        let today = chrono::Utc::now().date_naive();
        let first_of_month = self.current_month;

        let first_weekday = first_of_month.weekday().num_days_from_monday();

        let start_date = first_of_month - chrono::Duration::days(first_weekday as i64);

        for i in 0..42 {
            let date = start_date + chrono::Duration::days(i as i64);
            let is_current_month = date.month() == self.current_month.month();
            let is_today = date == today;
            let is_selected = self.selected_date.map_or(false, |d| d == date);

            self.days.push(CalendarDay {
                date,
                is_current_month,
                is_today,
                is_selected,
                has_appointments: false,
            });
        }
    }

    pub fn set_appointment_counts(&mut self, _counts: std::collections::HashMap<NaiveDate, u32>) {
        for day in &mut self.days {
            day.has_appointments = false;
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Option<CalendarAction> {
        use crossterm::event::KeyEventKind;

        if key.kind != KeyEventKind::Press {
            return None;
        }

        let registry = KeybindRegistry::global();

        if let Some(keybind) = registry.lookup(key, KeyContext::Calendar) {
            return match keybind.action {
                Action::PrevDay => {
                    if self.focused_date.day() > 1 {
                        self.focused_date =
                            self.focused_date.pred_opt().unwrap_or(self.focused_date);
                    }
                    Some(CalendarAction::FocusDate(self.focused_date))
                }
                Action::NextDay => {
                    self.focused_date = self.focused_date.succ_opt().unwrap_or(self.focused_date);
                    Some(CalendarAction::FocusDate(self.focused_date))
                }
                Action::PrevWeek => {
                    self.focused_date = self.focused_date - chrono::Duration::days(7);
                    Some(CalendarAction::FocusDate(self.focused_date))
                }
                Action::NextWeek => {
                    self.focused_date = self.focused_date + chrono::Duration::days(7);
                    Some(CalendarAction::FocusDate(self.focused_date))
                }
                Action::Today => {
                    let today = chrono::Utc::now().date_naive();
                    self.current_month =
                        NaiveDate::from_ymd_opt(today.year(), today.month(), 1).unwrap();
                    self.focused_date = today;
                    self.selected_date = Some(today);
                    self.rebuild_days();
                    Some(CalendarAction::GoToToday)
                }
                Action::PrevMonth => {
                    self.current_month = self.previous_month();
                    self.rebuild_days();
                    Some(CalendarAction::MonthChanged(self.current_month))
                }
                Action::NextMonth => {
                    self.current_month = self.next_month();
                    self.rebuild_days();
                    Some(CalendarAction::MonthChanged(self.current_month))
                }
                Action::Enter => {
                    self.selected_date = Some(self.focused_date);
                    self.rebuild_days();
                    Some(CalendarAction::SelectDate(self.focused_date))
                }
                _ => None,
            };
        }
        None
    }

    pub fn handle_mouse(&mut self, mouse: MouseEvent, area: Rect) -> Option<CalendarAction> {
        match mouse.kind {
            MouseEventKind::ScrollUp => {
                self.current_month = self.next_month();
                self.rebuild_days();
                Some(CalendarAction::MonthChanged(self.current_month))
            }
            MouseEventKind::ScrollDown => {
                self.current_month = self.previous_month();
                self.rebuild_days();
                Some(CalendarAction::MonthChanged(self.current_month))
            }
            MouseEventKind::Up(crossterm::event::MouseButton::Left) => {
                if let Some(day_index) = self.get_day_index_at(mouse.column, mouse.row, area) {
                    if let Some(day) = self.days.get(day_index) {
                        if day.is_current_month {
                            let selected = day.date;
                            self.focused_date = selected;
                            self.selected_date = Some(selected);
                            self.rebuild_days();
                            return Some(CalendarAction::SelectDate(selected));
                        }
                    }
                }
                None
            }
            _ => None,
        }
    }

    fn get_day_index_at(&self, column: u16, row: u16, area: Rect) -> Option<usize> {
        let grid_x = area.x + 2;
        let grid_y = area.y + 3;

        if column < grid_x || row < grid_y {
            return None;
        }

        let inner_right = area.x + area.width.saturating_sub(2);
        let inner_bottom = area.y + area.height.saturating_sub(2);

        if column >= inner_right || row >= inner_bottom {
            return None;
        }

        let dx = (column - grid_x) as usize;
        let dy = (row - grid_y) as usize;

        if dx % 3 >= 2 || dy % 2 != 0 {
            return None;
        }

        let col = dx / 3;
        let row_idx = dy / 2;

        if col > 6 || row_idx > 5 {
            return None;
        }

        Some(row_idx * 7 + col)
    }

    fn render_weekday_header(&self, area: Rect, buf: &mut Buffer) {
        let weekdays = ["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"];

        for (i, day) in weekdays.iter().enumerate() {
            let x = area.x + 1 + (i * 3) as u16;
            let style = if i >= 5 {
                Style::default().fg(self.theme.colors.disabled)
            } else {
                Style::default().fg(self.theme.colors.foreground).bold()
            };
            buf.set_string(x, area.y, day, style);
        }
    }

    fn render_days(&self, area: Rect, buf: &mut Buffer) {
        for (i, day) in self.days.iter().enumerate() {
            let row = i / 7;
            let col = i % 7;

            let x = area.x + 1 + (col * 3) as u16;
            let y = area.y + 2 + (row * 2) as u16;

            let style = self.get_day_style(day);

            let day_str = format!("{:>2}", day.date.day());
            buf.set_string(x, y, day_str, style);

            if day.is_selected {
                buf.set_string(x - 1, y, "[", style);
                buf.set_string(x + 2, y, "]", style);
            }

            if day.has_appointments {
                buf.set_string(
                    x + 2,
                    y,
                    "•",
                    Style::default().fg(self.theme.colors.primary),
                );
            }
        }
    }

    fn get_day_style(&self, day: &CalendarDay) -> Style {
        let mut style = Style::default();

        if !day.is_current_month {
            style = style.fg(self.theme.colors.disabled);
        } else if day.is_today {
            style = style.fg(self.theme.colors.primary).bold();
        } else if day.is_selected {
            style = style
                .bg(self.theme.colors.selected)
                .fg(self.theme.colors.foreground);
        } else {
            style = style.fg(self.theme.colors.foreground);
        }

        style
    }
}

impl Widget for Calendar {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.is_empty() || area.width < 21 || area.height < 9 {
            return;
        }

        let border_style = if self.focused {
            Style::default().fg(self.theme.colors.primary)
        } else {
            Style::default().fg(self.theme.colors.border)
        };

        let block = Block::default()
            .title(format!(" {} ", self.current_month.format("%B %Y")))
            .borders(Borders::ALL)
            .border_style(border_style);

        block.clone().render(area, buf);

        let inner = block.inner(area);

        self.render_weekday_header(inner, buf);
        self.render_days(inner, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_calendar() -> Calendar {
        Calendar::new(Theme::dark())
    }

    #[test]
    fn test_calendar_initial_state() {
        let calendar = create_test_calendar();
        let today = chrono::Utc::now().date_naive();

        assert_eq!(calendar.current_month.year(), today.year());
        assert_eq!(calendar.current_month.month(), today.month());
        assert_eq!(calendar.current_month.day(), 1);

        assert!(calendar.selected_date.is_some());
        assert_eq!(calendar.selected_date.unwrap(), today);

        assert_eq!(calendar.focused_date, today);

        assert_eq!(calendar.days.len(), 42);
    }

    #[test]
    fn test_calendar_navigation_next_month() {
        let mut calendar = create_test_calendar();
        let original_month = calendar.current_month;

        let next = calendar.next_month();

        if original_month.month() == 12 {
            assert_eq!(next.year(), original_month.year() + 1);
            assert_eq!(next.month(), 1);
        } else {
            assert_eq!(next.year(), original_month.year());
            assert_eq!(next.month(), original_month.month() + 1);
        }
        assert_eq!(next.day(), 1);

        calendar.current_month = next;
        calendar.rebuild_days();

        assert_eq!(calendar.current_month, next);
    }

    #[test]
    fn test_calendar_navigation_prev_month() {
        let mut calendar = create_test_calendar();
        let original_month = calendar.current_month;

        let prev = calendar.previous_month();

        if original_month.month() == 1 {
            assert_eq!(prev.year(), original_month.year() - 1);
            assert_eq!(prev.month(), 12);
        } else {
            assert_eq!(prev.year(), original_month.year());
            assert_eq!(prev.month(), original_month.month() - 1);
        }
        assert_eq!(prev.day(), 1);

        calendar.current_month = prev;
        calendar.rebuild_days();

        assert_eq!(calendar.current_month, prev);
    }

    #[test]
    fn test_calendar_today_highlighting() {
        let calendar = create_test_calendar();
        let today = chrono::Utc::now().date_naive();

        let today_days: Vec<&CalendarDay> =
            calendar.days.iter().filter(|day| day.is_today).collect();

        assert_eq!(today_days.len(), 1);

        let today_day = today_days[0];
        assert_eq!(today_day.date, today);
        assert!(today_day.is_current_month);
    }

    #[test]
    fn test_calendar_date_selection() {
        let mut calendar = create_test_calendar();
        let today = chrono::Utc::now().date_naive();

        let selected_date = NaiveDate::from_ymd_opt(
            calendar.current_month.year(),
            calendar.current_month.month(),
            15,
        )
        .unwrap();

        calendar.selected_date = Some(selected_date);
        calendar.rebuild_days();

        let selected_days: Vec<&CalendarDay> =
            calendar.days.iter().filter(|day| day.is_selected).collect();

        assert_eq!(selected_days.len(), 1);
        assert_eq!(selected_days[0].date, selected_date);
        assert!(selected_days[0].is_current_month);

        let today_still_selected = calendar
            .days
            .iter()
            .any(|day| day.is_today && day.is_selected);

        if today.day() != 15 {
            assert!(!today_still_selected);
        }
    }
}

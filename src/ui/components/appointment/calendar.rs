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

        let start_date = if first_weekday == 0 {
            first_of_month
        } else {
            first_of_month.pred_opt().unwrap()
        };

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
        let registry = KeybindRegistry::new();

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
        let grid_x = area.x + 1;
        let grid_y = area.y + 2;

        if column < grid_x || row < grid_y {
            return None;
        }

        let col = (column - grid_x) as usize / 4;
        let row_idx = (row - grid_y) as usize / 3;

        if col > 6 || row_idx > 5 {
            return None;
        }

        Some(row_idx * 7 + col)
    }

    fn render_weekday_header(&self, area: Rect, buf: &mut Buffer) {
        let weekdays = ["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"];

        for (i, day) in weekdays.iter().enumerate() {
            let x = area.x + 1 + (i * 4) as u16;
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

            let x = area.x + 1 + (col * 4) as u16;
            let y = area.y + 2 + (row * 3) as u16;

            let style = self.get_day_style(day);

            let day_str = format!("{:>2}", day.date.day());
            buf.set_string(x, y, day_str, style);

            if day.has_appointments {
                buf.set_string(
                    x + 2,
                    y,
                    "•",
                    Style::default().fg(self.theme.colors.primary),
                );
            }

            if day.is_selected {
                buf.set_string(x - 1, y, "[", style);
                buf.set_string(x + 2, y, "]", style);
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
        if area.is_empty() {
            return;
        }

        let block = Block::default()
            .title(format!(" {} ", self.current_month.format("%B %Y")))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.colors.border));

        block.clone().render(area, buf);

        let inner = block.inner(area);

        self.render_weekday_header(inner, buf);
        self.render_days(inner, buf);
    }
}

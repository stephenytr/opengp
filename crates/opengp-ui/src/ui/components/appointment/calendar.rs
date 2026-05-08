use chrono::Datelike;
use chrono::NaiveDate;
use crossterm::event::{KeyEvent, MouseEvent, MouseEventKind};
use rat_focus::{FocusBuilder, FocusFlag, HasFocus};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::Widget;

use crate::ui::input::DoubleClickDetector;
use crate::ui::keybinds::{Action, KeyContext, KeybindRegistry};
use crate::ui::theme::Theme;
use crate::ui::widgets::{CalendarMode, CalendarWidget};
use std::collections::HashMap;

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
    widget: CalendarWidget,
    pub theme: Theme,
    pub focused: bool,
    pub days: Vec<CalendarDay>,
    pub appointment_indicators: HashMap<NaiveDate, char>,
    pub current_month: NaiveDate,
    pub selected_date: Option<NaiveDate>,
    pub focused_date: NaiveDate,
    pub hovered_day: Option<NaiveDate>,
    pub double_click_detector: DoubleClickDetector,
    pub focus: FocusFlag,
}

impl Calendar {
    #[allow(clippy::unwrap_used)]
    pub fn new(theme: Theme) -> Self {
        let today = chrono::Utc::now().date_naive();
        let mut widget = CalendarWidget::with_date(today, theme.clone());
        widget.set_selected_date(today); // Set selected date to today
                                         // SAFETY: today.month() is 1-12 and day 1 is always valid
        let current_month = NaiveDate::from_ymd_opt(today.year(), today.month(), 1).unwrap();

        let mut calendar = Self {
            widget,
            theme,
            focused: false,
            days: Vec::new(),
            appointment_indicators: HashMap::new(),
            current_month,
            selected_date: Some(today),
            focused_date: today,
            hovered_day: None,
            double_click_detector: DoubleClickDetector::default(),
            focus: FocusFlag::default(),
        };
        calendar.rebuild_days();
        calendar
    }

    pub fn set_selected_date(&mut self, date: NaiveDate) {
        self.widget.set_selected_date(date);
        self.selected_date = Some(date);
        self.rebuild_days();
    }

    pub fn set_current_month(&mut self, date: NaiveDate) {
        let year = date.year();
        let month = date.month();
        self.widget.current_month = (year, month);
        self.current_month = date;
        self.rebuild_days();
    }

    pub fn set_focused_date(&mut self, date: NaiveDate) {
        self.widget.focused_date = date;
        self.focused_date = date;
        self.rebuild_days();
    }

    pub fn previous_month(&mut self) {
        self.widget.prev_month();
        self.rebuild_days();
    }

    pub fn next_month(&mut self) {
        self.widget.next_month();
        self.rebuild_days();
    }

    #[allow(clippy::unwrap_used)]
    pub fn rebuild_days(&mut self) {
        self.days.clear();

        let (year, month) = self.widget.current_month;
        // SAFETY: month comes from CalendarWidget which validates 1-12
        self.current_month = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
        self.selected_date = self.widget.selected_date();
        self.focused_date = self.widget.focused_date;

        let today = chrono::Utc::now().date_naive();
        let current_month_naive = self.current_month;

        let first_weekday = current_month_naive.weekday().num_days_from_monday();

        let start_date = current_month_naive - chrono::Duration::days(first_weekday as i64);

        for i in 0..42 {
            let date = start_date + chrono::Duration::days(i as i64);
            let is_current_month = date.month() == current_month_naive.month();
            let is_today = date == today;
            let is_selected = self.selected_date == Some(date);

            self.days.push(CalendarDay {
                date,
                is_current_month,
                is_today,
                is_selected,
                has_appointments: self.appointment_indicators.contains_key(&date),
            });
        }
    }

    pub fn set_appointment_counts(&mut self, counts: HashMap<NaiveDate, u32>) {
        self.appointment_indicators.clear();
        for (date, count) in counts {
            if count > 0 {
                self.appointment_indicators.insert(date, '•');
            }
        }
        self.rebuild_days();
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
                    self.widget.focused_date = self
                        .widget
                        .focused_date
                        .pred_opt()
                        .unwrap_or(self.widget.focused_date);
                    self.rebuild_days();
                    Some(CalendarAction::FocusDate(self.widget.focused_date))
                }
                Action::NextDay => {
                    self.widget.focused_date = self
                        .widget
                        .focused_date
                        .succ_opt()
                        .unwrap_or(self.widget.focused_date);
                    self.rebuild_days();
                    Some(CalendarAction::FocusDate(self.widget.focused_date))
                }
                Action::PrevWeek | Action::NavigateUp => {
                    self.widget.focused_date -= chrono::Duration::days(7);
                    self.rebuild_days();
                    Some(CalendarAction::FocusDate(self.widget.focused_date))
                }
                Action::NextWeek | Action::NavigateDown => {
                    self.widget.focused_date += chrono::Duration::days(7);
                    self.rebuild_days();
                    Some(CalendarAction::FocusDate(self.widget.focused_date))
                }
                Action::Today => {
                    let today = chrono::Utc::now().date_naive();
                    self.widget.current_month = (today.year(), today.month());
                    self.widget.focused_date = today;
                    self.widget.set_selected_date(today);
                    self.rebuild_days();
                    Some(CalendarAction::GoToToday)
                }
                Action::PrevMonth => {
                    self.widget.prev_month();
                    self.rebuild_days();
                    Some(CalendarAction::MonthChanged(self.current_month))
                }
                Action::NextMonth => {
                    self.widget.next_month();
                    self.rebuild_days();
                    Some(CalendarAction::MonthChanged(self.current_month))
                }
                Action::Enter => {
                    let focused = self.widget.focused_date;
                    self.widget.set_selected_date(focused);
                    self.rebuild_days();
                    Some(CalendarAction::SelectDate(focused))
                }
                _ => None,
            };
        }
        None
    }

    pub fn handle_mouse(&mut self, mouse: MouseEvent, area: Rect) -> Option<CalendarAction> {
        match mouse.kind {
            MouseEventKind::Moved => {
                // Track which day cell is being hovered
                if let Some(day) = self.widget.day_at_index(
                    self.widget
                        .get_day_index_at(mouse.column, mouse.row, area)
                        .unwrap_or(0),
                ) {
                    if day.month() == self.current_month.month() {
                        self.hovered_day = Some(day);
                    } else {
                        self.hovered_day = None;
                    }
                } else {
                    self.hovered_day = None;
                }
                None
            }
            MouseEventKind::ScrollUp | MouseEventKind::ScrollDown => None,
            MouseEventKind::Down(crossterm::event::MouseButton::Left) => {
                // Double-click detection
                if self.double_click_detector.check_double_click_now(&mouse) {
                    if let Some(day) = self.widget.day_at_index(
                        self.widget
                            .get_day_index_at(mouse.column, mouse.row, area)
                            .unwrap_or(0),
                    ) {
                        self.widget.set_selected_date(day);
                        self.hovered_day = None;
                        self.rebuild_days();
                        Some(CalendarAction::SelectDate(day))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            MouseEventKind::Up(crossterm::event::MouseButton::Left) => {
                if let Some(_action) = self.widget.handle_mouse(mouse, area) {
                    let selected = self.widget.focused_date;
                    self.rebuild_days();
                    Some(CalendarAction::SelectDate(selected))
                } else {
                    None
                }
            }
            _ => {
                // Clear hover when mouse leaves
                self.hovered_day = None;
                None
            }
        }
    }
}

impl Widget for Calendar {
    fn render(mut self, area: Rect, buf: &mut Buffer) {
        // Sync hovered_day to widget's hovered_day_index for proper hover styling
        if let Some(hovered) = self.hovered_day {
            // Find the index of the hovered day in the 42-day grid
            for (index, day) in self.days.iter().enumerate() {
                if day.date == hovered {
                    self.widget.hovered_day_index = Some(index);
                    break;
                }
            }
        } else {
            self.widget.hovered_day_index = None;
        }

        self.widget
            .render_calendar(area, buf, CalendarMode::Scheduling);
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

        calendar.next_month();
        let next = calendar.current_month;

        if original_month.month() == 12 {
            assert_eq!(next.year(), original_month.year() + 1);
            assert_eq!(next.month(), 1);
        } else {
            assert_eq!(next.year(), original_month.year());
            assert_eq!(next.month(), original_month.month() + 1);
        }
        assert_eq!(next.day(), 1);
    }

    #[test]
    fn test_calendar_navigation_prev_month() {
        let mut calendar = create_test_calendar();
        let original_month = calendar.current_month;

        calendar.previous_month();
        let prev = calendar.current_month;

        if original_month.month() == 1 {
            assert_eq!(prev.year(), original_month.year() - 1);
            assert_eq!(prev.month(), 12);
        } else {
            assert_eq!(prev.year(), original_month.year());
            assert_eq!(prev.month(), original_month.month() - 1);
        }
        assert_eq!(prev.day(), 1);
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

        calendar.set_selected_date(selected_date);

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

impl HasFocus for Calendar {
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

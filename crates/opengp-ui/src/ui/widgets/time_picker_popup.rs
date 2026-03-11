//! Time Picker Popup Widget
//!
//! Reusable time slot picker popup for selecting appointment times.
//! Provides keyboard-driven time selection with centered overlay rendering.

use chrono::{NaiveDate, NaiveTime, Timelike};
use crossterm::event::{KeyCode, KeyEvent};
use opengp_config::CalendarConfig;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style, Stylize};
use ratatui::symbols::border;
use ratatui::widgets::{Block, Clear, Widget};

use crate::ui::theme::Theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimePickerAction {
    Selected(NaiveTime),
    Dismissed,
}

#[derive(Debug, Clone)]
pub struct TimePickerPopup {
    is_visible: bool,
    selected_time: NaiveTime,
    selected_row: u8,
    selected_col: u8,
    viewport_start_hour: u8,
    booked_slots: Vec<NaiveTime>,
    duration: u32,
    practitioner_id: Option<i64>,
    date: Option<NaiveDate>,
    scroll_offset: u8,
    config: CalendarConfig,
    theme: Theme,
}

const GRID_COLS: u8 = 4;
const GRID_ROWS: u8 = 6;

impl TimePickerPopup {
    pub fn new() -> Self {
        let config = CalendarConfig::default();
        Self {
            is_visible: false,
            selected_time: NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            selected_row: 0,
            selected_col: 0,
            viewport_start_hour: config.viewport_start_hour,
            booked_slots: Vec::new(),
            duration: 30,
            practitioner_id: None,
            date: None,
            scroll_offset: 0,
            config,
            theme: Theme::default(),
        }
    }

    pub fn with_theme(theme: Theme) -> Self {
        let config = CalendarConfig::default();
        Self {
            is_visible: false,
            selected_time: NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            selected_row: 0,
            selected_col: 0,
            viewport_start_hour: config.viewport_start_hour,
            booked_slots: Vec::new(),
            duration: 30,
            practitioner_id: None,
            date: None,
            scroll_offset: 0,
            config,
            theme,
        }
    }

    pub fn open(&mut self, practitioner_id: i64, date: NaiveDate, duration: u32) {
        self.practitioner_id = Some(practitioner_id);
        self.date = Some(date);
        self.duration = duration;
        self.selected_time =
            NaiveTime::from_hms_opt(self.config.viewport_start_hour as u32, 0, 0).unwrap();
        self.selected_row = 0;
        self.selected_col = 0;
        self.viewport_start_hour = self.config.viewport_start_hour;
        self.scroll_offset = 0;
        self.booked_slots.clear();
        self.is_visible = true;
    }

    pub fn close(&mut self) {
        self.is_visible = false;
    }

    pub fn is_visible(&self) -> bool {
        self.is_visible
    }

    pub fn selected_time(&self) -> NaiveTime {
        self.selected_time
    }

    pub fn set_booked_slots(&mut self, slots: Vec<NaiveTime>) {
        self.booked_slots = slots;
    }

    fn is_slot_booked(&self, time: NaiveTime) -> bool {
        self.booked_slots.iter().any(|&slot| slot == time)
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Option<TimePickerAction> {
        if !self.is_visible {
            return None;
        }

        match key.code {
            KeyCode::Enter | KeyCode::Char(' ') => {
                if !self.is_slot_booked(self.selected_time) {
                    let selected = self.selected_time;
                    self.is_visible = false;
                    return Some(TimePickerAction::Selected(selected));
                }
                None
            }
            KeyCode::Esc => {
                self.is_visible = false;
                Some(TimePickerAction::Dismissed)
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.move_selection_up();
                None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.move_selection_down();
                None
            }
            KeyCode::Left | KeyCode::Char('h') => {
                self.move_selection_left();
                None
            }
            KeyCode::Right | KeyCode::Char('l') => {
                self.move_selection_right();
                None
            }
            _ => None,
        }
    }

    fn move_selection_up(&mut self) {
        if self.selected_row > 0 {
            self.selected_row -= 1;
            self.update_time_from_grid();
        }
    }

    fn move_selection_down(&mut self) {
        let max_row = self.grid_max_row();
        if self.selected_row < max_row {
            self.selected_row += 1;
            self.update_time_from_grid();
        }
    }

    fn move_selection_left(&mut self) {
        if self.selected_col > 0 {
            self.selected_col -= 1;
            self.update_time_from_grid();
        }
    }

    fn move_selection_right(&mut self) {
        if self.selected_col < GRID_COLS - 1 {
            self.selected_col += 1;
            self.update_time_from_grid();
        }
    }

    fn grid_max_row(&self) -> u8 {
        let hours_range = self.config.max_hour - self.config.viewport_start_hour;
        (hours_range as u8).min(GRID_ROWS - 1)
    }

    fn update_time_from_grid(&mut self) {
        let hour = self.config.viewport_start_hour as u32 + self.selected_row as u32;
        let minute = self.selected_col as u32 * 15;
        if let Some(time) = NaiveTime::from_hms_opt(hour, minute, 0) {
            self.selected_time = time;
        }
    }

    fn update_grid_from_time(&mut self) {
        let hour = self.selected_time.hour() as i32;
        let start_hour = self.config.viewport_start_hour as i32;
        self.selected_row = ((hour - start_hour) as u8).min(GRID_ROWS - 1);
        self.selected_col = (self.selected_time.minute() / 15) as u8;
    }

    fn total_slots(&self) -> u8 {
        ((self.config.max_hour - self.config.min_hour) * 4) as u8
    }

    fn slot_to_time(&self, slot: u8) -> String {
        let total_minutes = (self.config.min_hour as u16 * 60) + (slot as u16 * 15);
        let hour = total_minutes / 60;
        let minute = total_minutes % 60;
        format!("{:02}:{:02}", hour, minute)
    }

    fn time_to_slot(&self, time: NaiveTime) -> Option<u8> {
        let hour = time.hour() as u8;
        let minute = time.minute() as u8;
        if hour < self.config.min_hour || hour >= self.config.max_hour {
            return None;
        }
        let hour_offset = hour - self.config.min_hour;
        let slot = hour_offset * 4 + minute / 15;
        Some(slot)
    }

    fn ensure_visible(&mut self) {
        let min_hour = self.config.min_hour as u32;
        let max_hour = self.config.max_hour as u32;
        let viewport_hours = self.viewport_end_hour() - self.viewport_start_hour;

        if self.selected_time.hour() < self.viewport_start_hour as u32 {
            self.viewport_start_hour = self.selected_time.hour() as u8;
        } else if self.selected_time.hour() >= self.viewport_end_hour() as u32 {
            self.viewport_start_hour =
                (self.selected_time.hour() as u8 + 1).saturating_sub(viewport_hours);
            self.viewport_start_hour = self.viewport_start_hour.max(min_hour as u8);
        }
    }

    fn viewport_end_hour(&self) -> u8 {
        let min_hour = self.config.min_hour as u32;
        let max_hour = self.config.max_hour as u32;
        let viewport_hours = 8;
        (self.viewport_start_hour + viewport_hours).min(max_hour as u8)
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        if !self.is_visible || area.is_empty() {
            return;
        }

        let popup_width = 44.min(area.width.saturating_sub(4));
        let popup_height = 16.min(area.height.saturating_sub(2));

        if popup_width < 20 || popup_height < 8 {
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

        self.render_time_slots(popup_area, buf);
    }

    fn render_time_slots(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered()
            .title(" Select Time ")
            .border_set(border::ROUNDED);

        block.clone().render(area, buf);

        let inner_area = block.inner(area);

        if inner_area.is_empty() {
            return;
        }

        let cell_width = 10u16;
        let start_x = inner_area.x + 1;
        let start_y = inner_area.y + 1;
        let max_row = self.grid_max_row();

        for row in 0..=max_row {
            let hour = self.config.viewport_start_hour as u32 + row as u32;
            for col in 0..GRID_COLS {
                let minute = col as u32 * 15;
                if hour >= self.config.max_hour as u32 {
                    break;
                }

                if let Some(time) = NaiveTime::from_hms_opt(hour, minute, 0) {
                    let is_selected = row == self.selected_row && col == self.selected_col;
                    let is_booked = self.is_slot_booked(time);
                    let time_str = format!("{:02}:{:02}", hour, minute);

                    let style = if is_selected {
                        Style::new().fg(self.theme.colors.primary).bold().reversed()
                    } else if is_booked {
                        Style::new().fg(Color::DarkGray).dim()
                    } else {
                        Style::new().fg(self.theme.colors.foreground)
                    };

                    let display = if is_booked {
                        format!("{}[X]", time_str)
                    } else {
                        format!("{}[ ]", time_str)
                    };

                    let x = start_x + (col as u16 * cell_width);
                    let y = start_y + (row as u16 * 2);

                    if x < inner_area.x + inner_area.width && y < inner_area.y + inner_area.height {
                        buf.set_string(x, y, display, style);
                    }
                }
            }
        }

        let help_text = "Arrow keys: navigate  ↵ Confirm  Esc Cancel";
        let help_x = inner_area.x + 1;
        let help_y = inner_area.y + inner_area.height - 1;
        if help_y < area.y + area.height {
            buf.set_string(help_x, help_y, help_text, Style::new().dim());
        }
    }
}

impl Default for TimePickerPopup {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_popup_is_not_visible() {
        let popup = TimePickerPopup::new();
        assert!(!popup.is_visible());
    }

    #[test]
    fn test_open_sets_visible() {
        let mut popup = TimePickerPopup::new();
        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        popup.open(1, date, 30);
        assert!(popup.is_visible());
    }

    #[test]
    fn test_close_clears_visibility() {
        let mut popup = TimePickerPopup::new();
        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        popup.open(1, date, 30);
        popup.close();
        assert!(!popup.is_visible());
    }

    #[test]
    fn test_selected_time_initializes_correctly() {
        let mut popup = TimePickerPopup::new();
        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        popup.open(1, date, 30);
        assert_eq!(popup.selected_time().hour(), 8);
        assert_eq!(popup.selected_time().minute(), 0);
    }

    #[test]
    fn test_booked_slots_tracking() {
        let mut popup = TimePickerPopup::new();
        let booked = vec![
            NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(10, 30, 0).unwrap(),
        ];
        popup.set_booked_slots(booked.clone());
        assert!(popup.is_slot_booked(NaiveTime::from_hms_opt(9, 0, 0).unwrap()));
        assert!(popup.is_slot_booked(NaiveTime::from_hms_opt(10, 30, 0).unwrap()));
        assert!(!popup.is_slot_booked(NaiveTime::from_hms_opt(11, 0, 0).unwrap()));
    }

    #[test]
    fn test_handle_key_enter_selected_time() {
        let mut popup = TimePickerPopup::new();
        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        popup.open(1, date, 30);

        let key = KeyEvent::new(KeyCode::Enter, crossterm::event::KeyModifiers::NONE);
        let action = popup.handle_key(key);

        assert!(matches!(action, Some(TimePickerAction::Selected(_))));
        assert!(!popup.is_visible());
    }

    #[test]
    fn test_handle_key_esc_dismisses() {
        let mut popup = TimePickerPopup::new();
        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        popup.open(1, date, 30);

        let key = KeyEvent::new(KeyCode::Esc, crossterm::event::KeyModifiers::NONE);
        let action = popup.handle_key(key);

        assert!(matches!(action, Some(TimePickerAction::Dismissed)));
        assert!(!popup.is_visible());
    }

    #[test]
    fn test_handle_key_navigation() {
        let mut popup = TimePickerPopup::new();
        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        popup.open(1, date, 30);

        let key = KeyEvent::new(KeyCode::Down, crossterm::event::KeyModifiers::NONE);
        popup.handle_key(key);

        assert_eq!(popup.selected_time().hour(), 9);
        assert_eq!(popup.selected_time().minute(), 0);
    }

    #[test]
    fn test_cannot_select_booked_slot() {
        let mut popup = TimePickerPopup::new();
        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        popup.open(1, date, 30);
        popup.set_booked_slots(vec![NaiveTime::from_hms_opt(8, 0, 0).unwrap()]);

        let key = KeyEvent::new(KeyCode::Enter, crossterm::event::KeyModifiers::NONE);
        let action = popup.handle_key(key);

        assert!(action.is_none());
        assert!(popup.is_visible());
    }
}

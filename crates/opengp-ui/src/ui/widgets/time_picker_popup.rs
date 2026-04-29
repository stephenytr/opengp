//! Time Picker Popup Widget
//!
//! Reusable time slot picker popup for selecting appointment times.
//! Provides keyboard-driven time selection with centered overlay rendering.

use chrono::{NaiveDate, NaiveTime, Timelike};
use crossterm::event::{Event, KeyEvent, MouseEvent, MouseEventKind};
use opengp_config::CalendarConfig;
use rat_event::ct_event;
use rat_focus::{FocusBuilder, FocusFlag, HasFocus};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::symbols::border;
use ratatui::widgets::{Block, Clear, Widget};

use crate::ui::shared::hover_style;
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
    hovered_index: Option<usize>,
    popup_area: Option<Rect>,
    pub focus: FocusFlag,
}

const GRID_COLS: u8 = 4;
const GRID_ROWS: u8 = 10; // 8 AM to 6 PM = 10 hours

impl TimePickerPopup {
    #[allow(clippy::unwrap_used)]
    pub fn new() -> Self {
        let config = CalendarConfig::default();
        Self {
            is_visible: false,
            // SAFETY: 9:00:00 is a valid time
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
            hovered_index: None,
            popup_area: None,
            focus: FocusFlag::default(),
        }
    }

    #[allow(clippy::unwrap_used)]
    pub fn with_theme(theme: Theme) -> Self {
        let config = CalendarConfig::default();
        Self {
            is_visible: false,
            // SAFETY: 9:00:00 is a valid time
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
            hovered_index: None,
            popup_area: None,
            focus: FocusFlag::default(),
        }
    }

    #[allow(clippy::unwrap_used)]
    pub fn open(&mut self, practitioner_id: i64, date: NaiveDate, duration: u32) {
        self.practitioner_id = Some(practitioner_id);
        self.date = Some(date);
        self.duration = duration;
        // SAFETY: viewport_start_hour is 0-23 (valid hour), 0,0 are valid minutes/seconds
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
        self.hovered_index = None;
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
        self.booked_slots.contains(&time)
    }

    fn get_available_slots(&self) -> Vec<NaiveTime> {
        let mut available = Vec::new();
        let max_row = self.grid_max_row();
        for row in 0..=max_row {
            let hour = self.config.viewport_start_hour as u32 + row as u32;
            for col in 0..GRID_COLS {
                let minute = col as u32 * 15;
                if hour >= self.config.max_hour as u32 {
                    break;
                }
                if let Some(time) = NaiveTime::from_hms_opt(hour, minute, 0) {
                    if !self.is_slot_booked(time) {
                        available.push(time);
                    }
                }
            }
        }
        available
    }

    fn find_available_slot_index(&self, target_time: NaiveTime) -> usize {
        let available = self.get_available_slots();
        available
            .iter()
            .position(|&t| t == target_time)
            .unwrap_or(0)
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Option<TimePickerAction> {
        if !self.is_visible {
            return None;
        }

        let event = Event::Key(key);

        // If no available slots, only allow Esc
        if self.get_available_slots().is_empty() {
            if matches!(&event, ct_event!(keycode press Esc)) {
                self.is_visible = false;
                return Some(TimePickerAction::Dismissed);
            }
            return None;
        }

        match &event {
            ct_event!(keycode press Enter) | ct_event!(key press ' ') => {
                if !self.is_slot_booked(self.selected_time) {
                    let selected = self.selected_time;
                    self.is_visible = false;
                    return Some(TimePickerAction::Selected(selected));
                }
                None
            }
            ct_event!(keycode press Esc) => {
                self.is_visible = false;
                Some(TimePickerAction::Dismissed)
            }
            ct_event!(keycode press Up) | ct_event!(key press 'k') => {
                self.move_selection_up();
                None
            }
            ct_event!(keycode press Down) | ct_event!(key press 'j') => {
                self.move_selection_down();
                None
            }
            ct_event!(keycode press Left) | ct_event!(key press 'h') => {
                self.move_selection_left();
                None
            }
            ct_event!(keycode press Right) | ct_event!(key press 'l') => {
                self.move_selection_right();
                None
            }
            _ => None,
        }
    }

    /// Handle mouse events while the popup is open.
    ///
    /// Returns `Some(TimePickerAction)` if user clicked a time slot.
    /// Returns `None` for movement or other interactions.
    pub fn handle_mouse(&mut self, mouse: MouseEvent) -> Option<TimePickerAction> {
        if !self.is_visible {
            self.hovered_index = None;
            return None;
        }

        match mouse.kind {
            MouseEventKind::Moved => {
                if let Some(area) = self.popup_area {
                    let available = self.get_available_slots();
                    self.hovered_index = self.get_time_index_at(mouse.column, mouse.row, area);
                    if let Some(idx) = self.hovered_index {
                        if idx < available.len() {
                            self.selected_time = available[idx];
                        }
                    }
                }
                None
            }
            MouseEventKind::Up(crossterm::event::MouseButton::Left) => {
                if let Some(area) = self.popup_area {
                    if let Some(idx) = self.get_time_index_at(mouse.column, mouse.row, area) {
                        let available = self.get_available_slots();
                        if idx < available.len() && !self.is_slot_booked(available[idx]) {
                            let selected = available[idx];
                            self.selected_time = selected;
                            self.is_visible = false;
                            self.hovered_index = None;
                            return Some(TimePickerAction::Selected(selected));
                        }
                    }
                }
                None
            }
            _ => {
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

    fn get_time_index_at(&self, column: u16, row: u16, area: Rect) -> Option<usize> {
        let block = Block::bordered();
        let inner = block.inner(area);

        if inner.is_empty() {
            return None;
        }

        let cell_width = 10u16;
        let start_x = inner.x + 1;
        let start_y = inner.y + 1;
        let cols = GRID_COLS as u16;

        if column < start_x || row < start_y || column >= inner.right() {
            return None;
        }

        let col = (column - start_x) / cell_width;
        let row = (row - start_y) / 2;

        if col >= cols {
            return None;
        }

        Some((row as usize * GRID_COLS as usize) + col as usize)
    }

    fn move_selection_up(&mut self) {
        let available = self.get_available_slots();
        if available.is_empty() {
            return;
        }
        let current_index = self.find_available_slot_index(self.selected_time);
        if current_index >= GRID_COLS as usize {
            self.selected_time = available[current_index - GRID_COLS as usize];
        }
    }

    fn move_selection_down(&mut self) {
        let available = self.get_available_slots();
        if available.is_empty() {
            return;
        }
        let current_index = self.find_available_slot_index(self.selected_time);
        let next_index = current_index + GRID_COLS as usize;
        if next_index < available.len() {
            self.selected_time = available[next_index];
        }
    }

    fn move_selection_left(&mut self) {
        let available = self.get_available_slots();
        if available.is_empty() {
            return;
        }
        let current_index = self.find_available_slot_index(self.selected_time);
        if current_index > 0 {
            self.selected_time = available[current_index - 1];
        }
    }

    fn move_selection_right(&mut self) {
        let available = self.get_available_slots();
        if available.is_empty() {
            return;
        }
        let current_index = self.find_available_slot_index(self.selected_time);
        if current_index + 1 < available.len() {
            self.selected_time = available[current_index + 1];
        }
    }

    fn grid_max_row(&self) -> u8 {
        let hours_range = self.config.max_hour - self.config.viewport_start_hour;
        hours_range.min(GRID_ROWS - 1)
    }

    #[allow(dead_code)]
    fn update_time_from_grid(&mut self) {
        let hour = self.config.viewport_start_hour as u32 + self.selected_row as u32;
        let minute = self.selected_col as u32 * 15;
        if let Some(time) = NaiveTime::from_hms_opt(hour, minute, 0) {
            self.selected_time = time;
        }
    }

    pub fn render(&mut self, area: Rect, buf: &mut Buffer) {
        if !self.is_visible || area.is_empty() {
            return;
        }

        let popup_width = 44.min(area.width.saturating_sub(4));
        let popup_height = 24.min(area.height.saturating_sub(2));

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
        self.popup_area = Some(popup_area);

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

        let available_slots = self.get_available_slots();

        // No available slots - show message
        if available_slots.is_empty() {
            let message = "No Appointments Available";
            let msg_width = message.len() as u16;

            let msg_x = inner_area.x + (inner_area.width.saturating_sub(msg_width)) / 2;
            let msg_y = inner_area.y + inner_area.height / 2;

            buf.set_string(
                msg_x,
                msg_y,
                message,
                Style::new().fg(self.theme.colors.text_dim).dim(),
            );

            let help_text = "Esc to dismiss";
            let help_x = inner_area.x + 1;
            let help_y = inner_area.y + inner_area.height - 1;
            if help_y < area.y + area.height {
                buf.set_string(help_x, help_y, help_text, Style::new().dim());
            }
            return;
        }

        // Render available slots in a list format
        let cell_width = 10u16;
        let start_x = inner_area.x + 1;
        let start_y = inner_area.y + 1;
        let cols = GRID_COLS;

        let current_index = self.find_available_slot_index(self.selected_time);

        for (index, time) in available_slots.iter().enumerate() {
            let row = index / cols as usize;
            let col = index % cols as usize;

            let is_selected = index == current_index;
            let is_hovered = self.hovered_index == Some(index);
            let time_str = format!("{:02}:{:02}", time.hour(), time.minute());

            let style = if is_hovered {
                hover_style(&self.theme)
            } else if is_selected {
                Style::new().fg(self.theme.colors.primary).bold().reversed()
            } else {
                Style::new().fg(self.theme.colors.foreground)
            };

            let display = format!("{}[ ]", time_str);

            let x = start_x + (col as u16 * cell_width);
            let y = start_y + (row as u16 * 2);

            if x < inner_area.x + inner_area.width && y < inner_area.y + inner_area.height {
                buf.set_string(x, y, display, style);
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
    use crossterm::event::KeyCode;

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

impl HasFocus for TimePickerPopup {
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

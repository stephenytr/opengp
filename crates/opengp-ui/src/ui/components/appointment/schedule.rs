//! Schedule view component for appointment management.
//!
//! Displays a day view with practitioner columns and time slots.

use chrono::{NaiveDate, Timelike};
use rat_focus::{FocusBuilder, FocusFlag, HasFocus};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::{Block, Borders, StatefulWidget, Widget};
use std::collections::HashMap;
use uuid::Uuid;

use crate::ui::layout::TIME_COLUMN_WIDTH;
use crate::ui::shared::invert_color;
use crate::ui::theme::Theme;
use opengp_config::CalendarConfig;
use opengp_domain::domain::appointment::{AppointmentStatus, AppointmentType, CalendarAppointment};

/// Actions that can be triggered from the schedule view
#[derive(Debug, Clone)]
pub enum ScheduleAction {
    /// Select a practitioner column
    SelectPractitioner(Uuid),
    /// Select an appointment block
    SelectAppointment(Uuid),
    /// Navigate to a time slot (positive = down, negative = up)
    NavigateTimeSlot(i32),
    /// Navigate between practitioners (positive = right, negative = left)
    NavigatePractitioner(i32),
    /// Toggle visibility of the selected practitioner column
    ToggleColumn,
    /// Create a new appointment at the selected empty slot
    CreateAtSlot {
        /// The practitioner for the new appointment
        practitioner_id: Uuid,
        /// The date for the new appointment
        date: NaiveDate,
        /// The start time for the new appointment (e.g., "09:00")
        time: String,
    },
}

/// Schedule view widget displaying practitioner columns and time slots.
///
/// Shows a day's appointments across multiple practitioners with:
/// - Time column on the left (configurable time range in 15-minute slots)
/// - One column per practitioner
/// - Appointment blocks sized by duration
/// - Selection highlighting for time slots and practitioner columns
#[derive(Debug, Clone)]
pub struct Schedule {
    /// Theme for styling
    theme: Theme,
    /// Calendar configuration
    config: CalendarConfig,
    appointment_abbreviations: HashMap<String, String>,
    pub focus: FocusFlag,
}

impl Schedule {
    /// Create a new schedule component with the given theme and calendar configuration.
    pub fn new(theme: Theme, config: CalendarConfig) -> Self {
        Self {
            theme,
            config,
            appointment_abbreviations: HashMap::new(),
            focus: FocusFlag::default(),
        }
    }

    /// Set appointment abbreviations via builder pattern.
    pub fn with_abbreviations(mut self, abbrevs: HashMap<String, String>) -> Self {
        self.appointment_abbreviations = abbrevs;
        self
    }

    #[cfg(debug_assertions)]
    fn render_debug_overlay(
        &self,
        area: Rect,
        buf: &mut Buffer,
        state: &crate::ui::components::appointment::state::AppointmentState,
    ) {
        use ratatui::style::Color;
        use ratatui::widgets::{Block, Borders};

        let overlay_width = 32u16;
        let overlay_height = 12u16;
        if area.width < overlay_width || area.height < overlay_height {
            return;
        }
        let overlay_x = area.x + area.width - overlay_width;
        let overlay_y = area.y;
        let overlay_area = Rect::new(overlay_x, overlay_y, overlay_width, overlay_height);

        for y in overlay_area.y..overlay_area.y + overlay_area.height {
            for x in overlay_area.x..overlay_area.x + overlay_area.width {
                if let Some(cell) = buf.cell_mut((x, y)) {
                    cell.set_symbol(" ");
                    cell.set_style(
                        Style::default()
                            .bg(self.theme.colors.background_dark)
                            .fg(self.theme.colors.foreground),
                    );
                }
            }
        }

        let block = Block::default()
            .title(" DEBUG ")
            .borders(Borders::ALL)
            .style(
                Style::default()
                    .bg(self.theme.colors.background_dark)
                    .fg(self.theme.colors.warning),
            );
        block.render(overlay_area, buf);

        let inner = overlay_area.inner(ratatui::layout::Margin {
            vertical: 1,
            horizontal: 1,
        });

        let lines = vec![
            format!(
                "slot: {} ({})",
                state.selected_time_slot,
                state.slot_to_time(state.selected_time_slot)
            ),
            format!(
                "prac: {}/{}",
                state.selected_practitioner_index,
                state.practitioners_view.len()
            ),
            format!(
                "viewport: {}h-{}h",
                state.viewport_start_hour, state.viewport_end_hour
            ),
            format!("inner_h: {}", state.last_inner_height),
            format!(
                "appts: {}",
                state
                    .schedule_data
                    .as_ref()
                    .map(|d| d
                        .practitioners
                        .iter()
                        .map(|p| p.appointments.len())
                        .sum::<usize>())
                    .unwrap_or(0)
            ),
        ];

        for (i, line) in lines.iter().enumerate() {
            let y = inner.y + i as u16;
            if y < inner.y + inner.height {
                buf.set_string(
                    inner.x,
                    y,
                    line,
                    Style::default()
                        .bg(self.theme.colors.background_dark)
                        .fg(self.theme.colors.foreground),
                );
            }
        }
    }

    /// Render the time column on the left side.
    fn render_time_column(
        &self,
        area: Rect,
        buf: &mut Buffer,
        state: &crate::ui::components::appointment::state::AppointmentState,
    ) {
        let max_slot = state.max_time_slot();
        for slot in 0..=max_slot {
            let y = (area.y + 1) + slot as u16 * 2;
            if y < area.y + area.height {
                let time_str = state.slot_to_time(slot);
                let style = if slot == state.selected_time_slot {
                    Style::default()
                        .fg(self.theme.colors.primary)
                        .bold()
                        .bg(self.theme.colors.selected)
                } else {
                    Style::default().fg(self.theme.colors.foreground)
                };
                buf.set_string(area.x, y, &time_str, style);
            }
        }
    }

    /// Render practitioner columns with their appointments.
    fn render_practitioner_columns(
        &self,
        area: Rect,
        buf: &mut Buffer,
        state: &crate::ui::components::appointment::state::AppointmentState,
    ) {
        if state.practitioners_view.is_empty() {
            return;
        }

        let col_width = area.width / state.practitioners_view.len() as u16;
        let max_slot = state.max_time_slot();

        for (idx, practitioner) in state.practitioners_view.iter().enumerate() {
            let col_x = area.x + (idx as u16 * col_width);
            let is_selected = idx == state.selected_practitioner_index;

            let header_style = if is_selected {
                Style::default()
                    .bg(practitioner.colour)
                    .fg(invert_color(practitioner.colour))
                    .bold()
            } else {
                Style::default().fg(practitioner.colour)
            };

            let header_text = &practitioner.display_name;
            let header_len = header_text.len().min(col_width as usize - 2);
            buf.set_string(col_x, area.y, &header_text[..header_len], header_style);

            // for y in area.y..area.y + area.height {
            //     buf.set_string(col_x, y, "│", sep_style);
            // }

            if let Some(schedule) = &state.schedule_data {
                if let Some(practitioner_schedule) = schedule
                    .practitioners
                    .iter()
                    .find(|ps| ps.practitioner_id == practitioner.id)
                {
                    if let Some(working_hours) = &practitioner_schedule.working_hours {
                        let start_hour = working_hours.start_time.hour() as u8;
                        let start_minute = working_hours.start_time.minute() as u8;
                        let end_hour = working_hours.end_time.hour() as u8;
                        let end_minute = working_hours.end_time.minute() as u8;

                        let start_slot = if start_hour >= state.viewport_start_hour {
                            Some(
                                ((start_hour - state.viewport_start_hour) * 4)
                                    + (start_minute / 15),
                            )
                        } else {
                            None
                        };

                        let end_slot = if end_hour >= state.viewport_start_hour {
                            Some(((end_hour - state.viewport_start_hour) * 4) + (end_minute / 15))
                        } else {
                            None
                        };

                        if let (Some(start), Some(end)) = (start_slot, end_slot) {
                            let viewport_max_slot = max_slot;

                            for slot in 0..start.min(viewport_max_slot + 1) {
                                let y = area.y + 1 + slot as u16 * 2;
                                if y < area.y + area.height {
                                    for x in col_x + 1..col_x + col_width {
                                        if let Some(cell) = buf.cell_mut((x, y)) {
                                            cell.set_bg(self.theme.colors.disabled);
                                        }
                                    }
                                }
                            }

                            for slot in (end + 1)..=viewport_max_slot {
                                let y = area.y + 1 + slot as u16 * 2;
                                if y < area.y + area.height {
                                    for x in col_x + 1..col_x + col_width {
                                        if let Some(cell) = buf.cell_mut((x, y)) {
                                            cell.set_bg(self.theme.colors.disabled);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if let Some(schedule) = &state.schedule_data {
                if let Some(practitioner_schedule) = schedule
                    .practitioners
                    .iter()
                    .find(|ps| ps.practitioner_id == practitioner.id)
                {
                    let mut overlap_groups: Vec<Vec<usize>> = Vec::new();
                    let mut processed = std::collections::HashSet::new();

                    for (idx, apt) in practitioner_schedule.appointments.iter().enumerate() {
                        if processed.contains(&idx) {
                            continue;
                        }

                        let mut group = vec![idx];
                        processed.insert(idx);

                        if apt.is_overlapping {
                            for (other_idx, other_apt) in
                                practitioner_schedule.appointments.iter().enumerate()
                            {
                                if other_idx != idx
                                    && !processed.contains(&other_idx)
                                    && other_apt.is_overlapping
                                {
                                    if !(other_apt.end_time <= apt.start_time
                                        || other_apt.start_time >= apt.end_time)
                                    {
                                        group.push(other_idx);
                                        processed.insert(other_idx);
                                    }
                                }
                            }
                        }

                        overlap_groups.push(group);
                    }

                    for group in overlap_groups {
                        if group.len() <= 2 {
                            for (group_pos, &apt_idx) in group.iter().enumerate() {
                                let apt = &practitioner_schedule.appointments[apt_idx];
                                let half_width = col_width / 2;
                                let apt_x = if apt.is_overlapping {
                                    col_x + (group_pos as u16 * half_width)
                                } else {
                                    col_x
                                };
                                let apt_width = if apt.is_overlapping {
                                    half_width
                                } else {
                                    col_width
                                };

                                self.render_appointment_block(
                                    buf, apt_x, apt_width, apt, max_slot, area, state,
                                );
                            }
                        } else {
                            for (group_pos, &apt_idx) in group.iter().enumerate() {
                                if group_pos >= 2 {
                                    break;
                                }
                                let apt = &practitioner_schedule.appointments[apt_idx];
                                let half_width = col_width / 2;
                                let apt_x = col_x + (group_pos as u16 * half_width);

                                self.render_appointment_block(
                                    buf, apt_x, half_width, apt, max_slot, area, state,
                                );
                            }

                            if let Some(&second_idx) = group.get(1) {
                                let second_apt = &practitioner_schedule.appointments[second_idx];
                                if let Some(start_slot) = state.time_to_slot(second_apt.start_time)
                                {
                                    if start_slot <= max_slot {
                                        let y = area.y + 1 + start_slot as u16 * 2;
                                        if y < area.y + area.height {
                                            let half_width = col_width / 2;
                                            let badge_x = col_x + half_width;
                                            let remaining = group.len() - 2;
                                            let badge_text = format!("+{}", remaining);
                                            buf.set_string(
                                                badge_x,
                                                y,
                                                &badge_text,
                                                Style::default()
                                                    .fg(self.theme.colors.warning)
                                                    .bold(),
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            let hovered_style = crate::ui::shared::hover_style(&self.theme);
            if let Some((hovered_idx, hovered_slot)) = state.hovered_slot.element_id {
                if hovered_idx == idx {
                    let hover_y = area.y + 1 + (hovered_slot as u16 * 2);
                    if hover_y < area.y + area.height {
                        for x in col_x..col_x + col_width {
                            if let Some(cell) = buf.cell_mut((x, hover_y)) {
                                cell.set_fg(
                                    hovered_style.fg.unwrap_or(ratatui::style::Color::Reset),
                                );
                                cell.set_bg(
                                    hovered_style.bg.unwrap_or(ratatui::style::Color::Reset),
                                );
                            }
                        }
                    }
                }
            }

            let slot_y = area.y + 1 + (state.selected_time_slot as u16 * 2);
            if slot_y < area.y + area.height {
                let highlight_style = if is_selected {
                    Style::default().bg(self.theme.colors.highlight)
                } else {
                    Style::default()
                        .bg(self.theme.colors.selected)
                        .fg(self.theme.colors.border)
                };
                for x in col_x..col_x + col_width {
                    if let Some(cell) = buf.cell_mut((x, slot_y)) {
                        cell.set_bg(highlight_style.bg.unwrap_or(ratatui::style::Color::Reset));
                    }
                }
            }
        }
    }

    /// Render a single appointment block.
    fn render_appointment_block(
        &self,
        buf: &mut Buffer,
        area_x: u16,
        area_width: u16,
        apt: &CalendarAppointment,
        max_slot: u8,
        area: Rect,
        state: &crate::ui::components::appointment::state::AppointmentState,
    ) {
        let Some(start_slot) = state.time_to_slot(apt.start_time) else {
            return;
        };
        if start_slot > max_slot {
            return;
        }

        let slot_span = apt.slot_span as u16;
        let height = slot_span * 2;
        let mut y = area.y + 1 + start_slot as u16 * 2;

        if y >= area.y + area.height {
            return;
        }

        let clipped_top = area.y.saturating_sub(y);
        y = y.max(area.y);
        let height = height.saturating_sub(clipped_top);

        let max_height = area.y + area.height - y;
        let actual_height = height.min(max_height).max(1);

        let color = if apt.is_urgent {
            self.theme.colors.warning
        } else {
            self.get_appointment_color(apt.status)
        };

        let (content_x, content_y, content_width, content_height) =
            (area_x, y, area_width, actual_height);

        for row in 0..content_height {
            let row_y = content_y + row;
            if row_y >= area.y + area.height {
                break;
            }
            for col in 0..content_width {
                let col_x = content_x + col;
                if col_x >= area.x + area.width {
                    break;
                }
                if let Some(cell) = buf.cell_mut((col_x, row_y)) {
                    cell.set_bg(color);
                }
            }
        }

        let name_width = (content_width as usize).saturating_sub(2);
        let name_x = content_x + 1;
        if name_width > 0 && content_y < area.y + area.height && name_x < area.x + area.width {
            let name = if apt.patient_name.len() > name_width {
                format!("{}...", &apt.patient_name[..name_width.saturating_sub(3)])
            } else {
                apt.patient_name.clone()
            };
            let text_color = invert_color(color);
            buf.set_string(
                name_x,
                content_y,
                &name,
                Style::default().fg(text_color).bold(),
            );

            let line2_y = content_y + 1;
            if content_height >= 2 && line2_y < area.y + area.height {
                let abbreviation = self.get_abbreviation(&apt.appointment_type);
                let line2 = if apt.is_urgent {
                    format!("⚡ [{}]", abbreviation)
                } else {
                    format!("[{}]", abbreviation)
                };
                buf.set_string(
                    name_x,
                    line2_y,
                    &line2,
                    Style::default().fg(text_color).bold(),
                );
            }
        }
    }

    fn get_appointment_color(&self, status: AppointmentStatus) -> ratatui::style::Color {
        match status {
            AppointmentStatus::Scheduled => self.theme.colors.appointment_scheduled,
            AppointmentStatus::Confirmed => self.theme.colors.appointment_confirmed,
            AppointmentStatus::Arrived => self.theme.colors.appointment_arrived,
            AppointmentStatus::InProgress => self.theme.colors.appointment_in_progress,
            AppointmentStatus::Billing => self.theme.colors.appointment_completed,
            AppointmentStatus::Completed => self.theme.colors.appointment_completed,
            AppointmentStatus::Cancelled => self.theme.colors.appointment_cancelled,
            AppointmentStatus::NoShow => self.theme.colors.appointment_dna,
            AppointmentStatus::Rescheduled => self.theme.colors.disabled,
        }
    }

    fn get_abbreviation(&self, apt_type: &AppointmentType) -> String {
        self.appointment_abbreviations
            .get(&apt_type.to_string())
            .cloned()
            .unwrap_or_else(|| self.config.get_abbreviation(&apt_type.to_string()))
    }
}

impl StatefulWidget for Schedule {
    type State = crate::ui::components::appointment::state::AppointmentState;

    fn render(
        self,
        area: Rect,
        buf: &mut Buffer,
        state: &mut crate::ui::components::appointment::state::AppointmentState,
    ) {
        if area.is_empty() {
            return;
        }

        let title = state
            .schedule_data
            .as_ref()
            .map(|d| d.date.format("%A %d %B %Y").to_string())
            .unwrap_or_else(|| "Schedule".to_string());

        let border_style = if state.is_focused() {
            Style::default().fg(self.theme.colors.primary)
        } else {
            Style::default().fg(self.theme.colors.border)
        };

        let block = Block::default()
            .title(format!(" {} ", title))
            .borders(Borders::ALL)
            .border_style(border_style);

        block.clone().render(area, buf);

        let inner = block.inner(area);

        if state.practitioners_view.is_empty() {
            let msg = "No practitioners available";
            let x = inner.x + (inner.width.saturating_sub(msg.len() as u16) / 2);
            let y = inner.y + inner.height / 2;
            buf.set_string(x, y, msg, Style::default().fg(self.theme.colors.disabled));
            return;
        }

        let time_column_width = TIME_COLUMN_WIDTH;
        let practitioner_area = Rect {
            x: inner.x + time_column_width,
            y: inner.y,
            width: inner.width.saturating_sub(time_column_width),
            height: inner.height,
        };

        let time_column = Rect {
            x: inner.x,
            y: inner.y,
            width: time_column_width,
            height: inner.height,
        };
        self.render_time_column(time_column, buf, state);

        let today = chrono::Local::now().date_naive();
        if state.schedule_data.as_ref().map(|d| d.date) == Some(today) {
            let now_local = chrono::Local::now();
            let now_hour = now_local.hour() as u8;
            let now_minute = now_local.minute() as u8;

            if now_hour >= state.viewport_start_hour && now_hour < state.viewport_end_hour {
                let now_slot = (now_hour - state.viewport_start_hour) * 4 + (now_minute / 15);
                let viewport_max_slot =
                    (state.viewport_end_hour - state.viewport_start_hour) as u8 * 4;

                if now_slot <= viewport_max_slot {
                    let line_y = 1 + now_slot as u16 * 2;
                    let style = Style::default()
                        .fg(self.theme.colors.primary)
                        .add_modifier(ratatui::style::Modifier::UNDERLINED);

                    for col in 0..practitioner_area.width {
                        let x = practitioner_area.x + col;
                        let y = practitioner_area.y + line_y;
                        if y < inner.y + inner.height {
                            if let Some(cell) = buf.cell_mut((x, y)) {
                                cell.set_style(style);
                                cell.set_char('─');
                            }
                        }
                    }

                    let indicator_x = inner.x;
                    let indicator_y = inner.y + line_y;
                    if indicator_y < inner.y + inner.height {
                        buf.set_string(
                            indicator_x,
                            indicator_y,
                            "●",
                            Style::default().fg(self.theme.colors.primary).bold(),
                        );
                    }
                }
            }
        }

        self.render_practitioner_columns(practitioner_area, buf, state);

        #[cfg(debug_assertions)]
        if state.debug_overlay_visible {
            self.render_debug_overlay(area, buf, state);
        }
    }
}

impl HasFocus for Schedule {
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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use opengp_domain::domain::appointment::{
        AppointmentStatus, AppointmentType, CalendarAppointment, CalendarDayView,
        PractitionerSchedule,
    };
    use opengp_infrastructure::infrastructure::fixtures::schedule_scenarios::ScheduleScenario;
    use ratatui::backend::TestBackend;
    use ratatui::widgets::StatefulWidget;
    use ratatui::Terminal;

    #[test]
    fn test_slot_to_time() {
        let theme = Theme::default();
        let config = CalendarConfig::default();
        let mut state =
            crate::ui::components::appointment::state::AppointmentState::new(theme, config);

        assert_eq!(state.slot_to_time(0), "08:00");
        assert_eq!(state.slot_to_time(4), "09:00");
        assert_eq!(state.slot_to_time(8), "10:00");
        assert_eq!(state.slot_to_time(39), "17:45");
    }

    #[test]
    fn test_max_time_slot() {
        let theme = Theme::default();
        let config = CalendarConfig::default();
        let state = crate::ui::components::appointment::state::AppointmentState::new(theme, config);

        assert_eq!(state.max_time_slot(), 55);
    }

    #[test]
    fn test_scroll_viewport_preserves_selection() {
        let theme = Theme::default();
        let config = CalendarConfig::default();
        let mut state =
            crate::ui::components::appointment::state::AppointmentState::new(theme, config);

        state.selected_time_slot = 16;

        let time_before = state.slot_to_time(state.selected_time_slot);
        assert_eq!(time_before, "12:00");

        let key = crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::PageDown,
            crossterm::event::KeyModifiers::NONE,
        );
        state.handle_key(key);

        let time_after = state.slot_to_time(state.selected_time_slot);
        assert_eq!(
            time_before, time_after,
            "Selection time should be preserved after viewport scroll"
        );
    }

    #[test]
    fn test_viewport_clamps_at_boundaries() {
        let theme = Theme::default();
        let config = CalendarConfig::default();
        let mut state =
            crate::ui::components::appointment::state::AppointmentState::new(theme, config);

        state.viewport_start_hour = 8;
        state.viewport_end_hour = 18;

        let key = crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::PageUp,
            crossterm::event::KeyModifiers::NONE,
        );
        state.handle_key(key);

        assert!(
            state.viewport_start_hour >= 6,
            "viewport should not go below 6"
        );

        for _ in 0..20 {
            let key = crossterm::event::KeyEvent::new(
                crossterm::event::KeyCode::PageDown,
                crossterm::event::KeyModifiers::NONE,
            );
            state.handle_key(key);
        }

        assert!(
            state.viewport_end_hour <= 22,
            "viewport should not exceed 22"
        );
    }

    #[test]
    fn test_max_slot_reaches_max_hour() {
        let theme = Theme::default();

        let config = CalendarConfig {
            min_hour: 6,
            max_hour: 22,
            viewport_start_hour: 8,
            viewport_end_hour: 18,
            appointment_type_abbreviations: CalendarConfig::default_appointment_type_abbreviations(
            ),
        };
        let state =
            crate::ui::components::appointment::state::AppointmentState::new(theme.clone(), config);
        let max_slot = state.max_time_slot();

        assert_eq!(
            max_slot, 55,
            "max_slot should reach max_hour=22 from start=8: (22-8)*4-1=55"
        );
        assert_eq!(state.slot_to_time(max_slot), "21:45");

        let config2 = CalendarConfig {
            min_hour: 6,
            max_hour: 22,
            viewport_start_hour: 6,
            viewport_end_hour: 18,
            appointment_type_abbreviations: CalendarConfig::default_appointment_type_abbreviations(
            ),
        };
        let state2 =
            crate::ui::components::appointment::state::AppointmentState::new(theme, config2);
        assert_eq!(
            state2.max_time_slot(),
            63,
            "max_slot from start=6: (22-6)*4-1=63"
        );
        assert_eq!(state2.slot_to_time(state2.max_time_slot()), "21:45");
    }

    #[test]
    fn test_render_empty_schedule() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        let theme = Theme::default();
        let config = CalendarConfig::default();
        let schedule = Schedule::new(theme.clone(), config.clone());
        let mut state =
            crate::ui::components::appointment::state::AppointmentState::new(theme, config);

        let result = terminal.draw(|f| {
            StatefulWidget::render(schedule.clone(), f.area(), f.buffer_mut(), &mut state);
        });

        assert!(result.is_ok(), "render should complete without panic");
    }

    #[test]
    fn test_render_appointment_name_visible() {
        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).unwrap();

        let theme = Theme::default();
        let config = CalendarConfig::default();
        let schedule = Schedule::new(theme.clone(), config.clone());

        let date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
        let practitioner_id = Uuid::from_u128(1);
        let view = ScheduleScenario::single_appointment(date, practitioner_id);

        let mut state =
            crate::ui::components::appointment::state::AppointmentState::new(theme, config);
        state.load_schedule_data(view.clone());
        state.set_inner_height(38);

        terminal
            .draw(|f| {
                StatefulWidget::render(schedule.clone(), f.area(), f.buffer_mut(), &mut state);
            })
            .unwrap();

        let buffer = terminal.backend().buffer().clone();

        let mut found_j = false;
        for y in 0..buffer.content.len() as u16 {
            for x in 0..120u16 {
                if let Some(cell) = buffer.cell((x, y)) {
                    if cell.symbol() == "J" {
                        found_j = true;
                        break;
                    }
                }
            }
            if found_j {
                break;
            }
        }
        assert!(found_j, "Should find patient name 'John Doe' in buffer");
    }

    #[test]
    fn test_render_selected_slot_highlighted() {
        let backend = TestBackend::new(80, 30);
        let mut terminal = Terminal::new(backend).unwrap();

        let theme = Theme::default();
        let config = CalendarConfig::default();
        let schedule = Schedule::new(theme.clone(), config.clone());

        let date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
        let practitioner_id = Uuid::from_u128(1);
        let view = ScheduleScenario::single_appointment(date, practitioner_id);

        let mut state =
            crate::ui::components::appointment::state::AppointmentState::new(theme, config);
        state.load_schedule_data(view.clone());
        state.selected_time_slot = 4;
        state.set_inner_height(28);

        terminal
            .draw(|f| {
                StatefulWidget::render(schedule.clone(), f.area(), f.buffer_mut(), &mut state);
            })
            .unwrap();

        let buffer = terminal.backend().buffer().clone();

        let mut found_highlighted = false;
        for y in 0..buffer.content.len() as u16 {
            for x in 0..10u16 {
                if let Some(cell) = buffer.cell((x, y)) {
                    if cell.symbol() == "9" {
                        let style = cell.style();
                        if style.bg.is_some() || style.fg.is_some() {
                            found_highlighted = true;
                            break;
                        }
                    }
                }
            }
            if found_highlighted {
                break;
            }
        }
        assert!(
            found_highlighted,
            "Selected time slot should have highlighting"
        );
    }

    #[test]
    fn test_render_overlap_side_by_side() {
        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).unwrap();

        let theme = Theme::default();
        let config = CalendarConfig::default();
        let schedule = Schedule::new(theme.clone(), config.clone());

        let date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
        let practitioner_id = Uuid::from_u128(1);
        let view = ScheduleScenario::two_overlapping(date, practitioner_id);

        let mut state =
            crate::ui::components::appointment::state::AppointmentState::new(theme, config);
        state.load_schedule_data(view.clone());
        state.set_inner_height(38);

        let result = terminal.draw(|f| {
            StatefulWidget::render(schedule.clone(), f.area(), f.buffer_mut(), &mut state);
        });

        assert!(
            result.is_ok(),
            "render should handle overlapping appointments without panic"
        );
    }

    #[test]
    fn test_render_working_hours_shading() {
        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).unwrap();

        let theme = Theme::default();
        let config = CalendarConfig::default();
        let schedule = Schedule::new(theme.clone(), config.clone());

        let date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
        let practitioner_id = Uuid::from_u128(1);
        let view = ScheduleScenario::with_working_hours(date, practitioner_id, 9, 17);

        let mut state =
            crate::ui::components::appointment::state::AppointmentState::new(theme, config);
        state.load_schedule_data(view.clone());
        state.set_inner_height(38);

        let result = terminal.draw(|f| {
            StatefulWidget::render(schedule.clone(), f.area(), f.buffer_mut(), &mut state);
        });

        assert!(
            result.is_ok(),
            "render should handle working hours shading without panic"
        );
    }

    #[test]
    fn test_render_name_truncated() {
        let backend = TestBackend::new(30, 20);
        let mut terminal = Terminal::new(backend).unwrap();

        let theme = Theme::default();
        let config = CalendarConfig::default();
        let schedule = Schedule::new(theme.clone(), config.clone());

        let date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
        let practitioner_id = Uuid::from_u128(1);

        let long_name = "A".repeat(60);
        let start_time = date_time_at(date, 9, 0);
        let end_time = start_time + chrono::Duration::minutes(15);

        let appointment = CalendarAppointment {
            id: Uuid::from_u128(100),
            patient_id: Uuid::from_u128(1000),
            patient_name: long_name,
            practitioner_id,
            start_time,
            end_time,
            appointment_type: AppointmentType::Standard,
            status: AppointmentStatus::Scheduled,
            is_urgent: false,
            slot_span: 1,
            is_overlapping: false,
            reason: None,
            notes: None,
        };

        let view = CalendarDayView {
            date,
            practitioners: vec![PractitionerSchedule {
                practitioner_id,
                practitioner_name: "Dr. Test".to_string(),
                appointments: vec![appointment],
                working_hours: None,
            }],
        };

        let mut state =
            crate::ui::components::appointment::state::AppointmentState::new(theme, config);
        state.load_schedule_data(view.clone());
        state.set_inner_height(18);

        let result = terminal.draw(|f| {
            StatefulWidget::render(schedule.clone(), f.area(), f.buffer_mut(), &mut state);
        });

        assert!(
            result.is_ok(),
            "render should handle long names without panic"
        );
    }

    #[test]
    fn test_render_time_column_present() {
        let backend = TestBackend::new(80, 30);
        let mut terminal = Terminal::new(backend).unwrap();

        let theme = Theme::default();
        let config = CalendarConfig::default();
        let schedule = Schedule::new(theme.clone(), config.clone());

        let date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
        let practitioner_id = Uuid::from_u128(1);
        let view = ScheduleScenario::single_appointment(date, practitioner_id);

        let mut state =
            crate::ui::components::appointment::state::AppointmentState::new(theme, config);
        state.load_schedule_data(view.clone());
        state.set_inner_height(28);

        terminal
            .draw(|f| {
                StatefulWidget::render(schedule.clone(), f.area(), f.buffer_mut(), &mut state);
            })
            .unwrap();

        let buffer = terminal.backend().buffer().clone();

        let mut found_time = false;
        for y in 0..buffer.content.len() as u16 {
            for x in 0..10u16 {
                if let Some(cell) = buffer.cell((x, y)) {
                    if cell.symbol() == "0" {
                        let mut time_str = String::new();
                        for check_x in x.saturating_sub(2)..=(x + 4).min(9) {
                            if let Some(check_cell) = buffer.cell((check_x, y)) {
                                time_str.push_str(check_cell.symbol());
                            }
                        }
                        if time_str.contains("08:00") {
                            found_time = true;
                            break;
                        }
                    }
                }
            }
            if found_time {
                break;
            }
        }
        assert!(found_time, "Should find time '08:00' in time column");
    }

    /// Helper function to create DateTime<Utc> from date and time components
    fn date_time_at(date: NaiveDate, hour: u8, minute: u8) -> chrono::DateTime<chrono::Utc> {
        let naive_datetime = date
            .and_hms_opt(hour as u32, minute as u32, 0)
            .expect("Invalid datetime");
        chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(naive_datetime, chrono::Utc)
    }
}

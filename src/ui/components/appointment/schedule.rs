//! Schedule view component for appointment management.
//!
//! Displays a day view with practitioner columns and time slots.

use chrono::Timelike;
use crossterm::event::{KeyEvent, MouseEvent, MouseEventKind};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::Stylize;
use ratatui::style::Style;
use ratatui::widgets::{Block, Borders, Widget};
use uuid::Uuid;

use crate::domain::appointment::{AppointmentStatus, CalendarAppointment, CalendarDayView};
use crate::domain::user::Practitioner;
use crate::ui::keybinds::{Action, KeyContext, KeybindRegistry};
use crate::ui::theme::Theme;

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
}

/// Schedule view widget displaying practitioner columns and time slots.
///
/// Shows a day's appointments across multiple practitioners with:
/// - Time column on the left (8:00 - 18:00 in 15-minute slots)
/// - One column per practitioner
/// - Appointment blocks sized by duration
/// - Selection highlighting for time slots and practitioner columns
#[derive(Debug, Clone)]
pub struct Schedule {
    /// List of practitioners to display
    practitioners: Vec<Practitioner>,
    /// Schedule data for the current day
    schedule_data: Option<CalendarDayView>,
    /// Currently selected practitioner column index
    selected_practitioner_index: usize,
    /// Currently selected time slot (0-39 for 8am-6pm in 15-min increments)
    selected_time_slot: u8,
    /// First hour to display in viewport
    viewport_start_hour: u8,
    /// Last hour to display in viewport
    viewport_end_hour: u8,
    /// Theme for styling
    theme: Theme,
}

impl Schedule {
    /// Create a new schedule component with default values.
    pub fn new(theme: Theme) -> Self {
        Self {
            practitioners: Vec::new(),
            schedule_data: None,
            selected_practitioner_index: 0,
            selected_time_slot: 0, // 8:00 AM
            viewport_start_hour: 8,
            viewport_end_hour: 18,
            theme,
        }
    }

    /// Load schedule data from a CalendarDayView.
    ///
    /// Extracts practitioner information from the schedule data
    /// and populates the internal practitioner list.
    pub fn load_schedule(&mut self, data: CalendarDayView) {
        self.schedule_data = Some(data.clone());
        self.practitioners.clear();

        // Extract unique practitioners from schedule data
        for practitioner_schedule in &data.practitioners {
            let practitioner = Practitioner {
                id: practitioner_schedule.practitioner_id,
                user_id: None,
                first_name: practitioner_schedule
                    .practitioner_name
                    .split_whitespace()
                    .next()
                    .unwrap_or("")
                    .to_string(),
                middle_name: None,
                last_name: practitioner_schedule
                    .practitioner_name
                    .split_whitespace()
                    .last()
                    .unwrap_or("")
                    .to_string(),
                title: "Dr".to_string(),
                hpi_i: None,
                ahpra_registration: None,
                prescriber_number: None,
                provider_number: String::new(),
                speciality: None,
                qualifications: Vec::new(),
                phone: None,
                email: None,
                is_active: true,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            };
            self.practitioners.push(practitioner);
        }

        // Reset selection if out of bounds
        if self.selected_practitioner_index >= self.practitioners.len() {
            self.selected_practitioner_index = self.practitioners.len().saturating_sub(1);
        }
    }

    /// Handle keyboard input and return an action if triggered.
    pub fn handle_key(&mut self, key: KeyEvent) -> Option<ScheduleAction> {
        let registry = KeybindRegistry::new();

        if let Some(keybind) = registry.lookup(key, KeyContext::Schedule) {
            return match keybind.action {
                Action::PrevPractitioner => {
                    if self.selected_practitioner_index > 0 {
                        self.selected_practitioner_index -= 1;
                    }
                    // Return action with practitioner ID if available
                    self.practitioners
                        .get(self.selected_practitioner_index)
                        .map(|p| ScheduleAction::SelectPractitioner(p.id))
                }
                Action::NextPractitioner => {
                    if self.selected_practitioner_index < self.practitioners.len().saturating_sub(1)
                    {
                        self.selected_practitioner_index += 1;
                    }
                    self.practitioners
                        .get(self.selected_practitioner_index)
                        .map(|p| ScheduleAction::SelectPractitioner(p.id))
                }
                Action::PrevTimeSlot => {
                    if self.selected_time_slot > 0 {
                        self.selected_time_slot -= 1;
                    }
                    Some(ScheduleAction::NavigateTimeSlot(-1))
                }
                Action::NextTimeSlot => {
                    let max_slot = self.max_time_slot();
                    if self.selected_time_slot < max_slot {
                        self.selected_time_slot += 1;
                    }
                    Some(ScheduleAction::NavigateTimeSlot(1))
                }
                Action::Enter => {
                    // Try to select appointment at current position
                    self.get_appointment_at_selection()
                        .map(|apt| ScheduleAction::SelectAppointment(apt.id))
                }
                _ => None,
            };
        }
        None
    }

    /// Handle mouse input and return an action if triggered.
    pub fn handle_mouse(&mut self, mouse: MouseEvent, area: Rect) -> Option<ScheduleAction> {
        match mouse.kind {
            MouseEventKind::Up(crossterm::event::MouseButton::Left) => {
                // Calculate layout
                let time_column_width = 7; // "08:00 " width
                let inner = area.inner(ratatui::layout::Margin {
                    horizontal: 1,
                    vertical: 1,
                });

                if mouse.column < inner.x + time_column_width {
                    // Clicked on time column - select time slot
                    let y = mouse.row.saturating_sub(inner.y);
                    let slot = (y as u8 / 2).min(self.max_time_slot());
                    self.selected_time_slot = slot;
                    return Some(ScheduleAction::NavigateTimeSlot(0));
                }

                // Clicked on practitioner area
                let col = mouse.column.saturating_sub(inner.x + time_column_width);
                let practitioner_cols = inner.width.saturating_sub(time_column_width);

                if practitioner_cols > 0 && self.practitioners.len() > 0 {
                    let col_width = practitioner_cols / self.practitioners.len() as u16;
                    if col_width > 0 {
                        let practitioner_index = (col / col_width) as usize;
                        if practitioner_index < self.practitioners.len() {
                            self.selected_practitioner_index = practitioner_index;

                            // Check if clicked on an appointment
                            let y = mouse.row.saturating_sub(inner.y);
                            let slot = (y as u8 / 2).min(self.max_time_slot());

                            // Get appointment at this position for the selected practitioner
                            if let Some(apt) = self.get_appointment_at_slot_for_practitioner(
                                slot,
                                self.selected_practitioner_index,
                            ) {
                                return Some(ScheduleAction::SelectAppointment(apt.id));
                            }

                            return Some(ScheduleAction::SelectPractitioner(
                                self.practitioners[practitioner_index].id,
                            ));
                        }
                    }
                }
                None
            }
            MouseEventKind::ScrollUp => {
                // Scroll time view up (earlier times)
                if self.selected_time_slot > 0 {
                    self.selected_time_slot = self.selected_time_slot.saturating_sub(4);
                }
                Some(ScheduleAction::NavigateTimeSlot(-4))
            }
            MouseEventKind::ScrollDown => {
                // Scroll time view down (later times)
                let max_slot = self.max_time_slot();
                self.selected_time_slot = (self.selected_time_slot + 4).min(max_slot);
                Some(ScheduleAction::NavigateTimeSlot(4))
            }
            _ => None,
        }
    }

    /// Get the maximum valid time slot index.
    fn max_time_slot(&self) -> u8 {
        ((self.viewport_end_hour - self.viewport_start_hour) * 4 - 1) as u8
    }

    /// Get appointment at the currently selected position.
    fn get_appointment_at_selection(&self) -> Option<&CalendarAppointment> {
        self.get_appointment_at_slot_for_practitioner(
            self.selected_time_slot,
            self.selected_practitioner_index,
        )
    }

    /// Get appointment at a specific slot for a practitioner.
    fn get_appointment_at_slot_for_practitioner(
        &self,
        slot: u8,
        practitioner_index: usize,
    ) -> Option<&CalendarAppointment> {
        let schedule = self.schedule_data.as_ref()?;
        let practitioner = self.practitioners.get(practitioner_index)?;

        schedule
            .practitioners
            .iter()
            .find(|ps| ps.practitioner_id == practitioner.id)
            .and_then(|ps| {
                ps.appointments
                    .iter()
                    .find(|apt| self.is_appointment_at_slot(apt, slot))
            })
    }

    /// Check if an appointment occupies a specific time slot.
    fn is_appointment_at_slot(&self, apt: &CalendarAppointment, slot: u8) -> bool {
        let start_slot = self.time_to_slot(apt.start_time);
        let end_slot = start_slot + apt.slot_span as u8 - 1;
        slot >= start_slot && slot <= end_slot
    }

    /// Convert a DateTime to a time slot index (0-39).
    fn time_to_slot(&self, time: chrono::DateTime<chrono::Utc>) -> u8 {
        let hour = time.hour() as u8;
        let minute = time.minute() as u8;
        let hour_offset = hour.saturating_sub(self.viewport_start_hour);
        (hour_offset * 4 + minute / 15) as u8
    }

    /// Get the time string for a given slot.
    fn slot_to_time(&self, slot: u8) -> String {
        let total_minutes = (self.viewport_start_hour as u16 * 60) + (slot as u16 * 15);
        let hour = total_minutes / 60;
        let minute = total_minutes % 60;
        format!("{:02}:{:02}", hour, minute)
    }

    /// Get color for an appointment based on its status.
    fn get_appointment_color(&self, status: AppointmentStatus) -> ratatui::style::Color {
        match status {
            AppointmentStatus::Scheduled => self.theme.colors.appointment_scheduled,
            AppointmentStatus::Confirmed => self.theme.colors.appointment_confirmed,
            AppointmentStatus::Arrived => self.theme.colors.appointment_arrived,
            AppointmentStatus::InProgress => self.theme.colors.appointment_in_progress,
            AppointmentStatus::Completed => self.theme.colors.appointment_completed,
            AppointmentStatus::Cancelled => self.theme.colors.appointment_cancelled,
            AppointmentStatus::NoShow => self.theme.colors.appointment_dna,
            AppointmentStatus::Rescheduled => self.theme.colors.disabled,
        }
    }

    /// Render the time column on the left side.
    fn render_time_column(&self, area: Rect, buf: &mut Buffer) {
        let max_slot = self.max_time_slot();
        for slot in 0..=max_slot {
            let y = area.y + slot as u16 * 2;
            if y < area.y + area.height {
                let time_str = self.slot_to_time(slot);
                let style = if slot == self.selected_time_slot {
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
    fn render_practitioner_columns(&self, area: Rect, buf: &mut Buffer) {
        if self.practitioners.is_empty() {
            return;
        }

        let col_width = area.width / self.practitioners.len() as u16;
        let max_slot = self.max_time_slot();

        for (idx, practitioner) in self.practitioners.iter().enumerate() {
            let col_x = area.x + (idx as u16 * col_width);
            let is_selected = idx == self.selected_practitioner_index;

            // Render column header
            let header_style = if is_selected {
                Style::default()
                    .fg(self.theme.colors.primary)
                    .bold()
                    .bg(self.theme.colors.selected)
            } else {
                Style::default().fg(self.theme.colors.foreground)
            };

            let header_text = practitioner.display_name();
            let header_len = header_text.len().min(col_width as usize - 2);
            buf.set_string(col_x + 1, area.y, &header_text[..header_len], header_style);

            // Render column separator
            let sep_style = if is_selected {
                Style::default().fg(self.theme.colors.primary)
            } else {
                Style::default().fg(self.theme.colors.border)
            };
            for y in area.y..area.y + area.height {
                buf.set_string(col_x, y, "│", sep_style);
            }

            // Render appointments for this practitioner
            if let Some(schedule) = &self.schedule_data {
                if let Some(practitioner_schedule) = schedule
                    .practitioners
                    .iter()
                    .find(|ps| ps.practitioner_id == practitioner.id)
                {
                    for apt in &practitioner_schedule.appointments {
                        self.render_appointment_block(
                            buf,
                            col_x + 1,
                            col_width.saturating_sub(2),
                            apt,
                            max_slot,
                            area,
                        );
                    }
                }
            }

            // Highlight selected time slot in this column
            let slot_y = area.y + 1 + (self.selected_time_slot as u16 * 2);
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
    ) {
        let start_slot = self.time_to_slot(apt.start_time);
        if start_slot > max_slot {
            return;
        }

        let slot_span = apt.slot_span as u16;
        let height = slot_span * 2;
        let mut y = 1 + start_slot as u16 * 2;

        // Clamp y to stay within buffer bounds
        if y >= area.y + area.height {
            return;
        }
        if y < area.y {
            y = area.y;
        }

        // Clamp height to fit within buffer
        let max_height = (area.y + area.height - y) as u16;
        let actual_height = height.min(max_height).max(1);

        let color = self.get_appointment_color(apt.status);

        // Render appointment block
        for row in 0..actual_height {
            let row_y = y + row;
            if row_y >= area.y + area.height {
                break;
            }
            for col in 0..area_width {
                let col_x = area_x + col;
                if col_x >= area.x + area.width {
                    break;
                }
                if let Some(cell) = buf.cell_mut((col_x, row_y)) {
                    cell.set_bg(color);
                    if row == 0 {
                        cell.set_fg(ratatui::style::Color::Black);
                    }
                }
            }
        }

        // Render patient name in first line (with bounds checking)
        let name_width = (area_width as usize).saturating_sub(2);
        if name_width > 0 && y < area.y + area.height {
            let name = if apt.patient_name.len() > name_width {
                format!("{}...", &apt.patient_name[..name_width.saturating_sub(3)])
            } else {
                apt.patient_name.clone()
            };
            let name_x = area_x + 1;
            if name_x < area.x + area.width {
                let _ = buf.set_string(
                    name_x,
                    y,
                    name,
                    Style::default().fg(ratatui::style::Color::Black).bold(),
                );
            }
        }
    }
}

impl Widget for Schedule {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.is_empty() {
            return;
        }

        // Render block container
        let title = self
            .schedule_data
            .as_ref()
            .map(|d| d.date.format("%A %d %B %Y").to_string())
            .unwrap_or_else(|| "Schedule".to_string());

        let block = Block::default()
            .title(format!(" {} ", title))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.colors.border));

        block.clone().render(area, buf);

        let inner = block.inner(area);

        // If no practitioners, show empty state
        if self.practitioners.is_empty() {
            let msg = "No practitioners available";
            let x = inner.x + (inner.width.saturating_sub(msg.len() as u16) / 2);
            let y = inner.y + inner.height / 2;
            buf.set_string(x, y, msg, Style::default().fg(self.theme.colors.disabled));
            return;
        }

        // Calculate layout
        let time_column_width = 7;
        let practitioner_area = Rect {
            x: inner.x + time_column_width,
            y: inner.y,
            width: inner.width.saturating_sub(time_column_width),
            height: inner.height,
        };

        // Render time column
        let time_column = Rect {
            x: inner.x,
            y: inner.y,
            width: time_column_width,
            height: inner.height,
        };
        self.render_time_column(time_column, buf);

        // Render practitioner columns
        self.render_practitioner_columns(practitioner_area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slot_to_time() {
        let theme = Theme::default();
        let schedule = Schedule::new(theme);

        assert_eq!(schedule.slot_to_time(0), "08:00");
        assert_eq!(schedule.slot_to_time(4), "09:00");
        assert_eq!(schedule.slot_to_time(8), "10:00");
        assert_eq!(schedule.slot_to_time(39), "17:45");
    }

    #[test]
    fn test_max_time_slot() {
        let theme = Theme::default();
        let schedule = Schedule::new(theme);

        // 8am to 6pm = 10 hours = 40 slots (0-39)
        assert_eq!(schedule.max_time_slot(), 39);
    }
}

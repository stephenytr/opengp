//! Schedule view component for appointment management.
//!
//! Displays a day view with practitioner columns and time slots.

use chrono::Timelike;
use crossterm::event::{KeyEvent, MouseEvent, MouseEventKind};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::{Block, Borders, Widget};
use uuid::Uuid;

use crate::ui::keybinds::{Action, KeyContext, KeybindRegistry};
use crate::ui::layout::TIME_COLUMN_WIDTH;
use crate::ui::theme::Theme;
use crate::ui::view_models::PractitionerViewItem;
use chrono::NaiveDate;
use opengp_config::CalendarConfig;
use opengp_domain::domain::appointment::{AppointmentStatus, CalendarAppointment, CalendarDayView};

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
    practitioners: Vec<PractitionerViewItem>,
    /// Schedule data for the current day
    schedule_data: Option<CalendarDayView>,
    /// Currently selected practitioner column index
    selected_practitioner_index: usize,
    /// Currently selected time slot (in 15-min increments, relative to viewport_start_hour)
    pub selected_time_slot: u8,
    /// First hour to display in viewport
    viewport_start_hour: u8,
    /// Last hour to display in viewport
    viewport_end_hour: u8,
    /// Theme for styling
    theme: Theme,
    /// Whether this widget has keyboard focus
    pub focused: bool,
    /// Calendar configuration
    config: CalendarConfig,
    /// Height of the inner render area (excluding borders), updated each frame.
    /// Used to compute how many slots are visible so the viewport can auto-scroll.
    last_inner_height: u16,
}

impl Schedule {
    /// Create a new schedule component with the given theme and calendar configuration.
    pub fn new(theme: Theme, config: CalendarConfig) -> Self {
        Self {
            practitioners: Vec::new(),
            schedule_data: None,
            selected_practitioner_index: 0,
            selected_time_slot: 0,
            viewport_start_hour: config.viewport_start_hour,
            viewport_end_hour: config.viewport_end_hour,
            theme,
            focused: false,
            config,
            last_inner_height: 0,
        }
    }

    pub fn set_inner_height(&mut self, inner_height: u16) {
        self.last_inner_height = inner_height;
        self.fit_viewport_to_height();
    }

    fn visible_slots(&self) -> u8 {
        if self.last_inner_height < 2 {
            return 1;
        }
        // Row 0 of inner area = practitioner header. Each slot = 2 rows.
        // render: y = area.y + 1 + slot * 2, visible when y < area.y + area.height
        // => slot < (inner_height - 1) / 2
        ((self.last_inner_height.saturating_sub(1)) / 2).min(255) as u8
    }

    fn fit_viewport_to_height(&mut self) {
        let visible = self.visible_slots();
        // Convert visible slots to whole hours (ceiling), minimum 1 hour
        let hours_needed = ((visible as u16).div_ceil(4) as u8).max(1);
        let new_end = (self.viewport_start_hour + hours_needed).min(self.config.max_hour);
        self.viewport_end_hour = new_end.max(self.viewport_start_hour + 1);
    }

    fn ensure_slot_visible(&mut self) {
        let visible = self.visible_slots();
        if visible == 0 {
            return;
        }
        let slot = self.selected_time_slot;
        let max_hour = self.config.max_hour;
        let min_hour = self.config.min_hour;
        let window = self.viewport_end_hour - self.viewport_start_hour;

        if slot >= visible {
            let abs_slot = self.viewport_start_hour as u16 * 4 + slot as u16;
            let new_start_slot = abs_slot.saturating_sub(visible as u16 - 1);
            let new_start_hour = ((new_start_slot / 4) as u8).min(max_hour - window);
            let new_start_hour = new_start_hour.max(min_hour);
            self.viewport_start_hour = new_start_hour;
            self.viewport_end_hour = (self.viewport_start_hour + window).min(max_hour);
            self.selected_time_slot = (abs_slot as u8).saturating_sub(self.viewport_start_hour * 4);
        }
    }

    fn scroll_viewport_up_if_needed(&mut self) {
        let visible = self.visible_slots();
        if visible == 0 {
            return;
        }
        let slot = self.selected_time_slot;
        let min_hour = self.config.min_hour;
        let window = self.viewport_end_hour - self.viewport_start_hour;

        if slot == 0 && self.viewport_start_hour > min_hour {
            let new_start = self.viewport_start_hour.saturating_sub(1).max(min_hour);
            self.viewport_start_hour = new_start;
            self.viewport_end_hour = (self.viewport_start_hour + window).min(self.config.max_hour);
            self.selected_time_slot = 4;
        }
    }

    /// Load schedule data from a CalendarDayView.
    ///
    /// Extracts practitioner information from the schedule data
    /// and populates the internal practitioner list.
    pub fn load_schedule(&mut self, data: CalendarDayView) {
        self.schedule_data = Some(data.clone());
        self.practitioners.clear();

        for practitioner_schedule in &data.practitioners {
            let practitioner = PractitionerViewItem {
                id: practitioner_schedule.practitioner_id,
                display_name: practitioner_schedule.practitioner_name.clone(),
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
        use crossterm::event::KeyEventKind;

        if key.kind != KeyEventKind::Press {
            return None;
        }

        let registry = KeybindRegistry::global();

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
                        self.scroll_viewport_up_if_needed();
                    }
                    Some(ScheduleAction::NavigateTimeSlot(-1))
                }
                Action::NextTimeSlot => {
                    let max_slot = self.max_time_slot();
                    if self.selected_time_slot < max_slot {
                        self.selected_time_slot += 1;
                        self.ensure_slot_visible();
                    }
                    Some(ScheduleAction::NavigateTimeSlot(1))
                }
                Action::Enter => {
                    // Try to select appointment at current position, or create new if empty
                    if let Some(apt) = self.get_appointment_at_selection() {
                        Some(ScheduleAction::SelectAppointment(apt.id))
                    } else if let (Some(practitioner), Some(schedule_data)) = (
                        self.practitioners.get(self.selected_practitioner_index),
                        &self.schedule_data,
                    ) {
                        Some(ScheduleAction::CreateAtSlot {
                            practitioner_id: practitioner.id,
                            date: schedule_data.date,
                            time: self.slot_to_time(self.selected_time_slot),
                        })
                    } else {
                        None
                    }
                }
                Action::ScrollViewportUp => {
                    let min_hour = self.config.min_hour;
                    if self.viewport_start_hour > min_hour {
                        let abs_hour = self.viewport_start_hour + (self.selected_time_slot / 4);
                        let abs_min_slot = self.selected_time_slot % 4;

                        let window = self.viewport_end_hour - self.viewport_start_hour;
                        self.viewport_start_hour =
                            self.viewport_start_hour.saturating_sub(2).max(min_hour);
                        self.viewport_end_hour = self.viewport_start_hour + window;

                        if abs_hour >= self.viewport_start_hour && abs_hour < self.viewport_end_hour
                        {
                            self.selected_time_slot =
                                (abs_hour - self.viewport_start_hour) * 4 + abs_min_slot;
                        } else {
                            self.selected_time_slot = 0;
                        }
                    }
                    None
                }
                Action::ScrollViewportDown => {
                    let max_hour = self.config.max_hour;
                    if self.viewport_end_hour < max_hour {
                        let abs_hour = self.viewport_start_hour + (self.selected_time_slot / 4);
                        let abs_min_slot = self.selected_time_slot % 4;

                        let window = self.viewport_end_hour - self.viewport_start_hour;
                        self.viewport_end_hour = (self.viewport_end_hour + 2).min(max_hour);
                        self.viewport_start_hour = self.viewport_end_hour - window;

                        if abs_hour >= self.viewport_start_hour && abs_hour < self.viewport_end_hour
                        {
                            self.selected_time_slot =
                                (abs_hour - self.viewport_start_hour) * 4 + abs_min_slot;
                        } else {
                            self.selected_time_slot = self.max_time_slot();
                        }
                    }
                    None
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
                let time_column_width = TIME_COLUMN_WIDTH;
                let inner = area.inner(ratatui::layout::Margin {
                    horizontal: 1,
                    vertical: 1,
                });

                // Always calculate time slot from row, regardless of which column was clicked
                let y = mouse.row.saturating_sub(inner.y);
                let slot = (y as u8 / 2).min(self.max_time_slot());
                self.selected_time_slot = slot;

                if mouse.column < inner.x + time_column_width {
                    // Clicked on time column - select time slot only
                    return Some(ScheduleAction::NavigateTimeSlot(0));
                }

                // Clicked on practitioner area
                let col = mouse.column.saturating_sub(inner.x + time_column_width);
                let practitioner_cols = inner.width.saturating_sub(time_column_width);

                if practitioner_cols > 0 && !self.practitioners.is_empty() {
                    let col_width = practitioner_cols / self.practitioners.len() as u16;
                    if col_width > 0 {
                        let practitioner_index = (col / col_width) as usize;
                        if practitioner_index < self.practitioners.len() {
                            self.selected_practitioner_index = practitioner_index;

                            // Check if clicked on an appointment
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
                let min_hour = self.config.min_hour;
                if self.viewport_start_hour > min_hour {
                    self.viewport_start_hour = self.viewport_start_hour.saturating_sub(1);
                    self.viewport_end_hour = self.viewport_end_hour.saturating_sub(1);
                    self.scroll_viewport_to_show_selection();
                }
                None
            }
            MouseEventKind::ScrollDown => {
                let max_hour = self.config.max_hour;
                let window_hours = self.viewport_end_hour - self.viewport_start_hour;
                if self.viewport_end_hour < max_hour {
                    self.viewport_start_hour =
                        (self.viewport_start_hour + 1).min(max_hour - window_hours);
                    self.viewport_end_hour = (self.viewport_end_hour + 1).min(max_hour);
                    self.scroll_viewport_to_show_selection();
                }
                None
            }
            _ => None,
        }
    }

    fn max_time_slot(&self) -> u8 {
        (self.config.max_hour - self.viewport_start_hour) * 4 - 1
    }

    /// Ensure the viewport includes the currently selected time slot.
    /// Scrolls viewport_start_hour/viewport_end_hour as needed.
    /// This is the equivalent of patient/list.rs's adjust_scroll().
    fn scroll_viewport_to_show_selection(&mut self) {
        let window_hours = self.viewport_end_hour - self.viewport_start_hour;
        // Convert selected slot back to absolute hour
        let selected_hour = self.viewport_start_hour + (self.selected_time_slot / 4);

        if selected_hour < self.viewport_start_hour {
            // Selection is above viewport - scroll up
            self.viewport_start_hour = selected_hour;
            self.viewport_end_hour = selected_hour + window_hours;
        } else if selected_hour >= self.viewport_end_hour {
            // Selection is below viewport - scroll down
            self.viewport_end_hour = selected_hour + 1;
            self.viewport_start_hour = self.viewport_end_hour.saturating_sub(window_hours);
        }
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
        let Some(start_slot) = self.time_to_slot(apt.start_time) else {
            return false;
        };
        let end_slot = start_slot
            .saturating_add(apt.slot_span)
            .saturating_sub(1);
        slot >= start_slot && slot <= end_slot
    }

    /// Convert a DateTime to a time slot index relative to viewport.
    /// Returns None if the appointment time is before the visible viewport.
    fn time_to_slot(&self, time: chrono::DateTime<chrono::Utc>) -> Option<u8> {
        let hour = time.hour() as u8;
        let minute = time.minute() as u8;
        // Don't use saturating_sub - we need to know if hour is before viewport
        if hour < self.viewport_start_hour {
            return None;
        }
        let hour_offset = hour - self.viewport_start_hour;
        let slot = hour_offset * 4 + minute / 15;
        Some(slot)
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

    fn get_appointment_type_abbreviation(
        apt_type: &opengp_domain::domain::appointment::AppointmentType,
    ) -> &'static str {
        match apt_type {
            opengp_domain::domain::appointment::AppointmentType::Standard => "STD",
            opengp_domain::domain::appointment::AppointmentType::Long => "LNG",
            opengp_domain::domain::appointment::AppointmentType::Brief => "BRF",
            opengp_domain::domain::appointment::AppointmentType::NewPatient => "NEW",
            opengp_domain::domain::appointment::AppointmentType::HealthAssessment => "HLT",
            opengp_domain::domain::appointment::AppointmentType::ChronicDiseaseReview => "CHR",
            opengp_domain::domain::appointment::AppointmentType::MentalHealthPlan => "MHP",
            opengp_domain::domain::appointment::AppointmentType::Immunisation => "IMM",
            opengp_domain::domain::appointment::AppointmentType::Procedure => "PRC",
            opengp_domain::domain::appointment::AppointmentType::Telephone => "TEL",
            opengp_domain::domain::appointment::AppointmentType::Telehealth => "TEL",
            opengp_domain::domain::appointment::AppointmentType::HomeVisit => "HOM",
            opengp_domain::domain::appointment::AppointmentType::Emergency => "EMG",
        }
    }

    /// Render the time column on the left side.
    fn render_time_column(&self, area: Rect, buf: &mut Buffer) {
        let max_slot = self.max_time_slot();
        for slot in 0..=max_slot {
            let y = (area.y + 1) + slot as u16 * 2;
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

            let header_text = &practitioner.display_name;
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
        let Some(start_slot) = self.time_to_slot(apt.start_time) else {
            return;
        };
        if start_slot > max_slot {
            return;
        }

        let slot_span = apt.slot_span as u16;
        let height = slot_span * 2;
        let mut y = area.y + 1 + start_slot as u16 * 2;

        // Clamp y to stay within buffer bounds
        if y >= area.y + area.height {
            return;
        }

        // Handle appointments that start before visible area
        let clipped_top = area.y.saturating_sub(y);
        y = y.max(area.y);
        let height = height.saturating_sub(clipped_top);

        // Clamp height to fit within buffer
        let max_height = area.y + area.height - y;
        let actual_height = height.min(max_height).max(1);

        let color = self.get_appointment_color(apt.status);

        if slot_span >= 2 {
            // Render with border for multi-slot appointments
            let block_area = Rect::new(area_x, y, area_width, actual_height);

            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(self.theme.colors.border));

            block.clone().render(block_area, buf);

            let inner = block.inner(block_area);

            if inner.is_empty() {
                return;
            }

            let (content_x, content_y, content_width, content_height) =
                (inner.x, inner.y, inner.width, inner.height);

            // Render appointment block background
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

            let abbreviation = Self::get_appointment_type_abbreviation(&apt.appointment_type);
            let abbrev_len = abbreviation.len();
            let name_width = (content_width as usize)
                .saturating_sub(2)
                .saturating_sub(abbrev_len + 1);
            if name_width > 0 && content_y < area.y + area.height {
                let name = if apt.patient_name.len() > name_width {
                    format!("{}...", &apt.patient_name[..name_width.saturating_sub(3)])
                } else {
                    apt.patient_name.clone()
                };
                let name_x = content_x + 1;
                if name_x < area.x + area.width {
                    let full_text = format!("{} {}", name, abbreviation);
                    buf.set_string(
                        name_x,
                        content_y,
                        full_text,
                        Style::default().fg(self.theme.colors.success).bold(),
                    );
                }
            }
        } else {
            // Original behavior for 1-slot appointments (no border)
            let _block_area = Rect::new(area_x, y, area_width, actual_height);

            let (content_x, content_y, content_width, content_height) =
                (area_x, y, area_width, actual_height);

            // Render appointment block background
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
                        if row == 0 {
                            cell.set_fg(self.theme.colors.foreground);
                        }
                    }
                }
            }

            let abbreviation = Self::get_appointment_type_abbreviation(&apt.appointment_type);
            let abbrev_len = abbreviation.len();
            let name_width = (content_width as usize)
                .saturating_sub(2)
                .saturating_sub(abbrev_len + 1);
            if name_width > 0 && content_y < area.y + area.height {
                let name = if apt.patient_name.len() > name_width {
                    format!("{}...", &apt.patient_name[..name_width.saturating_sub(3)])
                } else {
                    apt.patient_name.clone()
                };
                let name_x = content_x + 1;
                if name_x < area.x + area.width {
                    let full_text = format!("{} {}", name, abbreviation);
                    buf.set_string(
                        name_x,
                        content_y,
                        full_text,
                        Style::default().fg(self.theme.colors.background).bold(),
                    );
                }
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

        let border_style = if self.focused {
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

        // If no practitioners, show empty state
        if self.practitioners.is_empty() {
            let msg = "No practitioners available";
            let x = inner.x + (inner.width.saturating_sub(msg.len() as u16) / 2);
            let y = inner.y + inner.height / 2;
            buf.set_string(x, y, msg, Style::default().fg(self.theme.colors.disabled));
            return;
        }

        // Calculate layout
        let time_column_width = TIME_COLUMN_WIDTH;
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
        let config = CalendarConfig::default();
        let schedule = Schedule::new(theme, config);

        assert_eq!(schedule.slot_to_time(0), "08:00");
        assert_eq!(schedule.slot_to_time(4), "09:00");
        assert_eq!(schedule.slot_to_time(8), "10:00");
        assert_eq!(schedule.slot_to_time(39), "17:45");
    }

    #[test]
    fn test_max_time_slot() {
        let theme = Theme::default();
        let config = CalendarConfig::default();
        let schedule = Schedule::new(theme, config);

        assert_eq!(schedule.max_time_slot(), 55);
    }

    #[test]
    fn test_scroll_viewport_preserves_selection() {
        let theme = Theme::default();
        let config = CalendarConfig::default();
        let mut schedule = Schedule::new(theme, config);

        schedule.selected_time_slot = 16;

        let time_before = schedule.slot_to_time(schedule.selected_time_slot);
        assert_eq!(time_before, "12:00");

        let key = crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::PageDown,
            crossterm::event::KeyModifiers::NONE,
        );
        schedule.handle_key(key);

        let time_after = schedule.slot_to_time(schedule.selected_time_slot);
        assert_eq!(
            time_before, time_after,
            "Selection time should be preserved after viewport scroll"
        );
    }

    #[test]
    fn test_viewport_clamps_at_boundaries() {
        let theme = Theme::default();
        let config = CalendarConfig::default();
        let mut schedule = Schedule::new(theme, config);

        schedule.viewport_start_hour = 8;
        schedule.viewport_end_hour = 18;

        let key = crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::PageUp,
            crossterm::event::KeyModifiers::NONE,
        );
        schedule.handle_key(key);

        assert!(
            schedule.viewport_start_hour >= 6,
            "viewport should not go below 6"
        );

        for _ in 0..20 {
            let key = crossterm::event::KeyEvent::new(
                crossterm::event::KeyCode::PageDown,
                crossterm::event::KeyModifiers::NONE,
            );
            schedule.handle_key(key);
        }

        assert!(
            schedule.viewport_end_hour <= 22,
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
        };
        let schedule = Schedule::new(theme.clone(), config);
        let max_slot = schedule.max_time_slot();

        assert_eq!(
            max_slot, 55,
            "max_slot should reach max_hour=22 from start=8: (22-8)*4-1=55"
        );
        assert_eq!(schedule.slot_to_time(max_slot), "21:45");

        let config2 = CalendarConfig {
            min_hour: 6,
            max_hour: 22,
            viewport_start_hour: 6,
            viewport_end_hour: 18,
        };
        let schedule2 = Schedule::new(theme, config2);
        assert_eq!(
            schedule2.max_time_slot(),
            63,
            "max_slot from start=6: (22-6)*4-1=63"
        );
        assert_eq!(schedule2.slot_to_time(schedule2.max_time_slot()), "21:45");
    }
}

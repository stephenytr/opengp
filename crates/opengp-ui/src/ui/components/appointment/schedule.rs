//! Schedule view component for appointment management.
//!
//! Displays a day view with practitioner columns and time slots.

use chrono::Timelike;
use crossterm::event::{KeyEvent, MouseEvent, MouseEventKind};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::{Block, Borders, Widget};
use std::collections::HashMap;
use uuid::Uuid;

use crate::ui::keybinds::{Action, KeyContext, KeybindRegistry};
use crate::ui::layout::TIME_COLUMN_WIDTH;
use crate::ui::theme::Theme;
use crate::ui::view_models::PractitionerViewItem;
use chrono::NaiveDate;
use opengp_config::CalendarConfig;
use opengp_domain::domain::appointment::{
    AppointmentStatus, AppointmentType, CalendarAppointment, CalendarDayView,
};

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
    appointment_abbreviations: HashMap<String, String>,
    /// Height of the inner render area (excluding borders), updated each frame.
    /// Used to compute how many slots are visible so the viewport can auto-scroll.
    last_inner_height: u16,
}

impl Schedule {
    fn appointment_type_config_key(apt_type: AppointmentType) -> &'static str {
        match apt_type {
            AppointmentType::Standard => "standard",
            AppointmentType::Long => "long",
            AppointmentType::Brief => "brief",
            AppointmentType::NewPatient => "new_patient",
            AppointmentType::HealthAssessment => "health_assessment",
            AppointmentType::ChronicDiseaseReview => "chronic_disease_review",
            AppointmentType::MentalHealthPlan => "mental_health_plan",
            AppointmentType::Immunisation => "immunisation",
            AppointmentType::Procedure => "procedure",
            AppointmentType::Telephone => "telephone",
            AppointmentType::Telehealth => "telehealth",
            AppointmentType::HomeVisit => "home_visit",
            AppointmentType::Emergency => "emergency",
        }
    }

    fn load_appointment_abbreviations() -> HashMap<String, String> {
        let mut abbreviations = HashMap::new();

        if let Ok(appointment_config) = opengp_config::load_appointment_config() {
            for apt_type in [
                AppointmentType::Standard,
                AppointmentType::Long,
                AppointmentType::Brief,
                AppointmentType::NewPatient,
                AppointmentType::HealthAssessment,
                AppointmentType::ChronicDiseaseReview,
                AppointmentType::MentalHealthPlan,
                AppointmentType::Immunisation,
                AppointmentType::Procedure,
                AppointmentType::Telephone,
                AppointmentType::Telehealth,
                AppointmentType::HomeVisit,
                AppointmentType::Emergency,
            ] {
                if let Some(option) = appointment_config
                    .types
                    .get(Self::appointment_type_config_key(apt_type))
                    .filter(|option| option.enabled)
                {
                    abbreviations.insert(apt_type.to_string(), option.abbreviation.clone());
                }
            }
        }

        abbreviations
    }

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
            appointment_abbreviations: Self::load_appointment_abbreviations(),
            last_inner_height: 0,
        }
    }

    pub fn set_inner_height(&mut self, inner_height: u16) {
        self.last_inner_height = inner_height;
        self.fit_viewport_to_height();
    }

    // ── VIEWPORT MATH ──

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

    // ── INPUT HANDLING ──

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
                Action::TogglePractitionerColumn => Some(ScheduleAction::ToggleColumn),
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
        let end_slot = start_slot.saturating_add(apt.slot_span).saturating_sub(1);
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

    // ── RENDERING ──

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

            // Render working hours background
            if let Some(schedule) = &self.schedule_data {
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

                        let start_slot = if start_hour >= self.viewport_start_hour {
                            Some(
                                ((start_hour - self.viewport_start_hour) * 4) + (start_minute / 15),
                            )
                        } else {
                            None
                        };

                        let end_slot = if end_hour >= self.viewport_start_hour {
                            Some(((end_hour - self.viewport_start_hour) * 4) + (end_minute / 15))
                        } else {
                            None
                        };

                        if let (Some(start), Some(end)) = (start_slot, end_slot) {
                            let viewport_max_slot = max_slot;

                            // Grey out slots before working hours
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

                            // Grey out slots after working hours
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

            // Render appointments for this practitioner
            if let Some(schedule) = &self.schedule_data {
                if let Some(practitioner_schedule) = schedule
                    .practitioners
                    .iter()
                    .find(|ps| ps.practitioner_id == practitioner.id)
                {
                    // Pre-compute overlap groups for side-by-side rendering
                    let mut overlap_groups: Vec<Vec<usize>> = Vec::new();
                    let mut processed = std::collections::HashSet::new();

                    for (idx, apt) in practitioner_schedule.appointments.iter().enumerate() {
                        if processed.contains(&idx) {
                            continue;
                        }

                        let mut group = vec![idx];
                        processed.insert(idx);

                        if apt.is_overlapping {
                            // Find all appointments that overlap with this one
                            for (other_idx, other_apt) in
                                practitioner_schedule.appointments.iter().enumerate()
                            {
                                if other_idx != idx
                                    && !processed.contains(&other_idx)
                                    && other_apt.is_overlapping
                                {
                                    // Check if time ranges overlap
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

                    // Render each group
                    for group in overlap_groups {
                        if group.len() <= 2 {
                            // Render up to 2 appointments side-by-side
                            for (group_pos, &apt_idx) in group.iter().enumerate() {
                                let apt = &practitioner_schedule.appointments[apt_idx];
                                let half_width = col_width.saturating_sub(2) / 2;
                                let apt_x = if apt.is_overlapping {
                                    col_x + 1 + (group_pos as u16 * half_width)
                                } else {
                                    col_x + 1
                                };
                                let apt_width = if apt.is_overlapping {
                                    half_width
                                } else {
                                    col_width.saturating_sub(2)
                                };

                                self.render_appointment_block(
                                    buf, apt_x, apt_width, apt, max_slot, area,
                                );
                            }
                        } else {
                            // 3+ overlapping appointments: show first 2, +N badge on 2nd
                            for (group_pos, &apt_idx) in group.iter().enumerate() {
                                if group_pos >= 2 {
                                    break; // Only render first 2
                                }
                                let apt = &practitioner_schedule.appointments[apt_idx];
                                let half_width = col_width.saturating_sub(2) / 2;
                                let apt_x = col_x + 1 + (group_pos as u16 * half_width);

                                self.render_appointment_block(
                                    buf, apt_x, half_width, apt, max_slot, area,
                                );
                            }

                            // Render +N badge on the 2nd appointment slot
                            if let Some(&second_idx) = group.get(1) {
                                let second_apt = &practitioner_schedule.appointments[second_idx];
                                if let Some(start_slot) = self.time_to_slot(second_apt.start_time) {
                                    if start_slot <= max_slot {
                                        let y = area.y + 1 + start_slot as u16 * 2;
                                        if y < area.y + area.height {
                                            let half_width = col_width.saturating_sub(2) / 2;
                                            let badge_x = col_x + 1 + half_width;
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

        let color = if apt.is_urgent {
            self.theme.colors.warning
        } else {
            self.get_appointment_color(apt.status)
        };

        // Unified rendering for all appointments (no borders regardless of slot_span)
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
                }
            }
        }

        let name_width = (content_width as usize).saturating_sub(2);
        let name_x = content_x + 1;
        if name_width > 0 && content_y < area.y + area.height && name_x < area.x + area.width {
            // Line 1: patient name only
            let name = if apt.patient_name.len() > name_width {
                format!("{}...", &apt.patient_name[..name_width.saturating_sub(3)])
            } else {
                apt.patient_name.clone()
            };
            buf.set_string(
                name_x,
                content_y,
                &name,
                Style::default().fg(self.theme.colors.foreground).bold(),
            );

            // Line 2: urgent symbol + type abbreviation (only when there is room)
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
                    Style::default().fg(self.theme.colors.foreground).bold(),
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

        // Draw "now" indicator line if viewing today
        let today = chrono::Local::now().date_naive();
        if self.schedule_data.as_ref().map(|d| d.date) == Some(today) {
            let now_local = chrono::Local::now();
            let now_hour = now_local.hour() as u8;
            let now_minute = now_local.minute() as u8;

            // Convert now to slot relative to viewport
            if now_hour >= self.viewport_start_hour && now_hour < self.viewport_end_hour {
                let now_slot = (now_hour - self.viewport_start_hour) * 4 + (now_minute / 15);
                let viewport_max_slot =
                    (self.viewport_end_hour - self.viewport_start_hour) as u8 * 4;

                if now_slot <= viewport_max_slot {
                    let line_y = 1 + now_slot as u16 * 2;
                    let style = Style::default()
                        .fg(self.theme.colors.primary)
                        .add_modifier(ratatui::style::Modifier::UNDERLINED);

                    // Draw across all practitioner columns
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

                    // Draw ● at leftmost edge (time column)
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

        // Render practitioner columns
        self.render_practitioner_columns(practitioner_area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use opengp_domain::domain::appointment::PractitionerSchedule;
    use opengp_infrastructure::infrastructure::fixtures::schedule_scenarios::ScheduleScenario;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

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
            appointment_type_abbreviations: CalendarConfig::default_appointment_type_abbreviations(
            ),
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
            appointment_type_abbreviations: CalendarConfig::default_appointment_type_abbreviations(
            ),
        };
        let schedule2 = Schedule::new(theme, config2);
        assert_eq!(
            schedule2.max_time_slot(),
            63,
            "max_slot from start=6: (22-6)*4-1=63"
        );
        assert_eq!(schedule2.slot_to_time(schedule2.max_time_slot()), "21:45");
    }

    #[test]
    fn test_render_empty_schedule() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        let theme = Theme::default();
        let config = CalendarConfig::default();
        let schedule = Schedule::new(theme, config);

        // Render should not panic
        let result = terminal.draw(|f| {
            schedule.render(f.area(), f.buffer_mut());
        });

        assert!(result.is_ok(), "render should complete without panic");
    }

    #[test]
    fn test_render_appointment_name_visible() {
        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).unwrap();

        let theme = Theme::default();
        let config = CalendarConfig::default();
        let mut schedule = Schedule::new(theme, config);

        let date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
        let practitioner_id = Uuid::from_u128(1);
        let view = ScheduleScenario::single_appointment(date, practitioner_id);
        schedule.load_schedule(view);
        schedule.set_inner_height(38);

        terminal
            .draw(|f| {
                schedule.render(f.area(), f.buffer_mut());
            })
            .unwrap();

        let buffer = terminal.backend().buffer().clone();

        // Search for 'J' from "John Doe" in the buffer
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
        let mut schedule = Schedule::new(theme, config);

        let date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
        let practitioner_id = Uuid::from_u128(1);
        let view = ScheduleScenario::single_appointment(date, practitioner_id);
        schedule.load_schedule(view);
        schedule.selected_time_slot = 4; // 09:00
        schedule.set_inner_height(28);

        terminal
            .draw(|f| {
                schedule.render(f.area(), f.buffer_mut());
            })
            .unwrap();

        let buffer = terminal.backend().buffer().clone();

        // Search for "09:00" and verify it has a non-default style
        let mut found_highlighted = false;
        for y in 0..buffer.content.len() as u16 {
            for x in 0..10u16 {
                if let Some(cell) = buffer.cell((x, y)) {
                    if cell.symbol() == "9" {
                        // Check if this cell has a styled background or foreground
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
        let mut schedule = Schedule::new(theme, config);

        let date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
        let practitioner_id = Uuid::from_u128(1);
        let view = ScheduleScenario::two_overlapping(date, practitioner_id);
        schedule.load_schedule(view);
        schedule.set_inner_height(38);

        // Render should not panic even with overlapping appointments
        let result = terminal.draw(|f| {
            schedule.render(f.area(), f.buffer_mut());
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
        let mut schedule = Schedule::new(theme, config);

        let date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
        let practitioner_id = Uuid::from_u128(1);
        let view = ScheduleScenario::with_working_hours(date, practitioner_id, 9, 17);
        schedule.load_schedule(view);
        schedule.set_inner_height(38);

        let result = terminal.draw(|f| {
            schedule.render(f.area(), f.buffer_mut());
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
        let mut schedule = Schedule::new(theme, config);

        let date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
        let practitioner_id = Uuid::from_u128(1);

        // Create appointment with very long name
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

        schedule.load_schedule(view);
        schedule.set_inner_height(18);

        let result = terminal.draw(|f| {
            schedule.render(f.area(), f.buffer_mut());
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
        let mut schedule = Schedule::new(theme, config);

        let date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
        let practitioner_id = Uuid::from_u128(1);
        let view = ScheduleScenario::single_appointment(date, practitioner_id);
        schedule.load_schedule(view);
        schedule.set_inner_height(28);

        terminal
            .draw(|f| {
                schedule.render(f.area(), f.buffer_mut());
            })
            .unwrap();

        let buffer = terminal.backend().buffer().clone();

        // Search for "08:00" in the buffer (viewport_start_hour is 8 by default)
        let mut found_time = false;
        for y in 0..buffer.content.len() as u16 {
            for x in 0..10u16 {
                if let Some(cell) = buffer.cell((x, y)) {
                    if cell.symbol() == "0" {
                        // Check if we can find "08:00" nearby
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

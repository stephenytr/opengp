//! Appointment Detail Modal Component
//!
//! Read-only modal displaying appointment details with options to view clinical notes.

use crossterm::event::{KeyEvent, KeyModifiers};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Borders, Clear, Widget};
use uuid::Uuid;

use crate::ui::theme::Theme;
use crate::ui::widgets::{DropdownOption, DropdownWidget};
use opengp_domain::domain::appointment::{AppointmentStatus, CalendarAppointment};

/// Actions returned by the appointment detail modal's key handler.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppointmentDetailModalAction {
    /// Close the modal
    Close,
    /// Navigate to Clinical tab to view patient notes
    ViewClinicalNotes,
    /// Mark appointment as arrived
    MarkArrived,
    /// Mark appointment as in progress
    MarkInProgress,
    /// Mark appointment as completed
    MarkCompleted,
    /// Mark appointment as no show
    MarkNoShow,
}

/// Appointment detail modal widget.
///
/// Displays read-only appointment information with options to view clinical notes.
/// Follows the modal pattern: centered, with clear background, Escape to close.
pub struct AppointmentDetailModal {
    /// The appointment data to display
    appointment: CalendarAppointment,
    /// Theme for styling
    theme: Theme,
    /// Which button is focused (0 = Close, 1 = View Clinical Notes)
    focused_button: usize,
    /// Patient ID for clinical navigation
    patient_id: Uuid,
    /// Status dropdown for selecting valid transitions
    status_dropdown: DropdownWidget,
}

impl Clone for AppointmentDetailModal {
    fn clone(&self) -> Self {
        Self {
            appointment: self.appointment.clone(),
            theme: self.theme.clone(),
            focused_button: self.focused_button,
            patient_id: self.patient_id,
            status_dropdown: self.status_dropdown.clone(),
        }
    }
}

impl AppointmentDetailModal {
    /// Create a new appointment detail modal.
    pub fn new(appointment: CalendarAppointment, theme: Theme) -> Self {
        // Create all possible status options
        let all_statuses = vec![
            DropdownOption::new("scheduled", "Scheduled"),
            DropdownOption::new("confirmed", "Confirmed"),
            DropdownOption::new("arrived", "Arrived"),
            DropdownOption::new("in_progress", "In Progress"),
            DropdownOption::new("completed", "Completed"),
            DropdownOption::new("cancelled", "Cancelled"),
            DropdownOption::new("no_show", "No Show"),
            DropdownOption::new("rescheduled", "Rescheduled"),
        ];

        // Filter to only valid transitions from current status
        let valid_options: Vec<DropdownOption> = all_statuses
            .into_iter()
            .filter(|opt| {
                let target_status = match opt.value.as_str() {
                    "scheduled" => AppointmentStatus::Scheduled,
                    "confirmed" => AppointmentStatus::Confirmed,
                    "arrived" => AppointmentStatus::Arrived,
                    "in_progress" => AppointmentStatus::InProgress,
                    "completed" => AppointmentStatus::Completed,
                    "cancelled" => AppointmentStatus::Cancelled,
                    "no_show" => AppointmentStatus::NoShow,
                    "rescheduled" => AppointmentStatus::Rescheduled,
                    _ => return false,
                };
                Self::can_transition(appointment.status, target_status)
            })
            .collect();

        let mut status_dropdown = DropdownWidget::new("Status", valid_options, theme.clone());

        let status_value = match appointment.status {
            AppointmentStatus::Scheduled => "scheduled",
            AppointmentStatus::Confirmed => "confirmed",
            AppointmentStatus::Arrived => "arrived",
            AppointmentStatus::InProgress => "in_progress",
            AppointmentStatus::Completed => "completed",
            AppointmentStatus::Cancelled => "cancelled",
            AppointmentStatus::NoShow => "no_show",
            AppointmentStatus::Rescheduled => "rescheduled",
        };
        status_dropdown.set_value(status_value);

        Self {
            appointment: appointment.clone(),
            theme,
            focused_button: 0,
            patient_id: appointment.patient_id,
            status_dropdown,
        }
    }

    /// Check if a transition from one status to another is valid (mirrors domain logic)
    fn can_transition(from: AppointmentStatus, to: AppointmentStatus) -> bool {
        use AppointmentStatus::*;

        if from == to {
            return true;
        }

        matches!(
            (from, to),
            (Scheduled, Confirmed | Arrived | Cancelled | Rescheduled)
                | (Confirmed, Arrived | Cancelled | Rescheduled)
                | (Arrived, InProgress | NoShow)
                | (InProgress, Completed)
        )
    }

    /// Get the patient ID for clinical navigation.
    pub fn patient_id(&self) -> Uuid {
        self.patient_id
    }

    /// Get the appointment ID.
    pub fn appointment_id(&self) -> Uuid {
        self.appointment.id
    }

    /// Format the appointment time for display.
    fn format_time(&self) -> String {
        let start = self.appointment.start_time.format("%H:%M").to_string();
        let end = self.appointment.end_time.format("%H:%M").to_string();
        format!("{} - {}", start, end)
    }

    /// Format the appointment date for display.
    fn format_date(&self) -> String {
        self.appointment
            .start_time
            .format("%A %d %B %Y")
            .to_string()
    }

    /// Format the duration for display.
    fn format_duration(&self) -> String {
        let mins = self.appointment.duration_minutes();
        if mins >= 60 {
            let hours = mins / 60;
            let remaining_mins = mins % 60;
            if remaining_mins == 0 {
                format!("{} hour{}", hours, if hours > 1 { "s" } else { "" })
            } else {
                format!("{}h {}m", hours, remaining_mins)
            }
        } else {
            format!("{} minutes", mins)
        }
    }

    /// Format the appointment type for display.
    fn format_type(&self) -> String {
        use opengp_domain::domain::appointment::AppointmentType;
        match self.appointment.appointment_type {
            AppointmentType::Standard => "Standard".to_string(),
            AppointmentType::Long => "Long Consultation".to_string(),
            AppointmentType::Brief => "Brief".to_string(),
            AppointmentType::NewPatient => "New Patient".to_string(),
            AppointmentType::HealthAssessment => "Health Assessment".to_string(),
            AppointmentType::ChronicDiseaseReview => "Chronic Disease Review".to_string(),
            AppointmentType::MentalHealthPlan => "Mental Health Plan".to_string(),
            AppointmentType::Immunisation => "Immunisation".to_string(),
            AppointmentType::Procedure => "Procedure".to_string(),
            AppointmentType::Telephone => "Telephone".to_string(),
            AppointmentType::Telehealth => "Telehealth".to_string(),
            AppointmentType::HomeVisit => "Home Visit".to_string(),
            AppointmentType::Emergency => "Emergency".to_string(),
        }
    }

    /// Format the appointment status for display.
    fn format_status(&self) -> String {
        match self.appointment.status {
            AppointmentStatus::Scheduled => "Scheduled".to_string(),
            AppointmentStatus::Confirmed => "Confirmed".to_string(),
            AppointmentStatus::Arrived => "Arrived".to_string(),
            AppointmentStatus::InProgress => "In Progress".to_string(),
            AppointmentStatus::Completed => "Completed".to_string(),
            AppointmentStatus::Cancelled => "Cancelled".to_string(),
            AppointmentStatus::NoShow => "No Show".to_string(),
            AppointmentStatus::Rescheduled => "Rescheduled".to_string(),
        }
    }

    /// Get the status color for the status display.
    fn get_status_color(&self) -> ratatui::style::Color {
        match self.appointment.status {
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

    // ── Navigation ───────────────────────────────────────────────────────────

    /// Move focus to the next button.
    pub fn next_button(&mut self) {
        let count = self.button_count();
        self.focused_button = (self.focused_button + 1) % count;
    }

    /// Move focus to the previous button.
    pub fn prev_button(&mut self) {
        let count = self.button_count();
        self.focused_button = if self.focused_button == 0 {
            count - 1
        } else {
            self.focused_button - 1
        };
    }

    /// Check if the Close button is focused.
    pub fn is_close_focused(&self) -> bool {
        self.focused_button == 0
    }

    /// Check if the View Clinical Notes button is focused.
    pub fn is_clinical_focused(&self) -> bool {
        self.focused_button == self.button_count() - 1
    }

    /// Get the number of visible buttons (Close, Status Dropdown, View Clinical Notes)
    fn button_count(&self) -> usize {
        3 // Close, Status Dropdown, View Clinical Notes
    }

    /// Get the button index for each action
    fn get_button_index(&self) -> std::collections::HashMap<usize, AppointmentDetailModalAction> {
        let mut map = std::collections::HashMap::new();
        map.insert(0, AppointmentDetailModalAction::Close);
        // Button 1 is the dropdown - handled separately in handle_key
        map.insert(2, AppointmentDetailModalAction::ViewClinicalNotes);
        map
    }

    /// Get action based on dropdown selection
    fn get_dropdown_action(&self) -> Option<AppointmentDetailModalAction> {
        let value = self.status_dropdown.selected_value()?;
        match value {
            "arrived" => Some(AppointmentDetailModalAction::MarkArrived),
            "in_progress" => Some(AppointmentDetailModalAction::MarkInProgress),
            "completed" => Some(AppointmentDetailModalAction::MarkCompleted),
            "no_show" => Some(AppointmentDetailModalAction::MarkNoShow),
            _ => None,
        }
    }

    // ── Key handling ───────────────────────────────────────────────────────

    /// Handle keyboard input and return an action if triggered.
    pub fn handle_key(&mut self, key: KeyEvent) -> Option<AppointmentDetailModalAction> {
        use crate::ui::widgets::DropdownAction;
        use crossterm::event::{KeyCode, KeyEventKind};

        if key.kind != KeyEventKind::Press {
            return None;
        }

        if self.focused_button == 1 {
            match key.code {
                KeyCode::Enter
                | KeyCode::Esc
                | KeyCode::Up
                | KeyCode::Down
                | KeyCode::Char('j')
                | KeyCode::Char('k')
                | KeyCode::Tab
                | KeyCode::BackTab => {
                    if let Some(action) = self.status_dropdown.handle_key(key) {
                        return match action {
                            DropdownAction::Selected(_) => self.get_dropdown_action(),
                            DropdownAction::Closed => {
                                if key.code == KeyCode::Tab {
                                    self.next_button();
                                } else if key.code == KeyCode::BackTab {
                                    self.prev_button();
                                }
                                None
                            }
                            DropdownAction::Opened | DropdownAction::FocusChanged => None,
                        };
                    }
                }
                _ => {}
            }
        }

        match key.code {
            KeyCode::Esc => Some(AppointmentDetailModalAction::Close),
            KeyCode::Tab => {
                if key.modifiers.contains(KeyModifiers::SHIFT) {
                    self.prev_button();
                } else {
                    self.next_button();
                }
                None
            }
            KeyCode::BackTab => {
                self.prev_button();
                None
            }
            KeyCode::Left | KeyCode::Up => {
                self.prev_button();
                None
            }
            KeyCode::Right | KeyCode::Down => {
                self.next_button();
                None
            }
            KeyCode::Enter => {
                if self.focused_button == 1 {
                    if let Some(action) = self.status_dropdown.handle_key(key) {
                        match action {
                            DropdownAction::Selected(_) => return self.get_dropdown_action(),
                            DropdownAction::Closed
                            | DropdownAction::Opened
                            | DropdownAction::FocusChanged => return None,
                        }
                    }
                    return None;
                }

                let button_map = self.get_button_index();
                button_map.get(&self.focused_button).copied()
            }
            _ => None,
        }
    }
}

// ── Widget ───────────────────────────────────────────────────────────────────

impl Widget for AppointmentDetailModal {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.is_empty() {
            return;
        }

        // Calculate modal dimensions (centered, 60% width, auto height)
        let modal_width = (area.width as f32 * 0.6) as u16;
        let modal_width = modal_width.clamp(50, 80);

        // Calculate content height based on fields
        let mut content_lines = 9; // Base fields
        if self.appointment.reason.is_some() {
            content_lines += 1;
        }
        if self.appointment.notes.is_some() {
            content_lines += 1;
        }
        content_lines += 2; // Buttons

        let modal_height = (content_lines as u16).min(area.height.saturating_sub(4));

        // Center the modal
        let x = area.x + (area.width.saturating_sub(modal_width)) / 2;
        let y = area.y + (area.height.saturating_sub(modal_height)) / 2;

        let modal_area = Rect::new(x, y, modal_width, modal_height);

        // Clear the modal area with background color
        let bg_style = Style::default().bg(self.theme.colors.background);
        Clear.render(modal_area, buf);

        // Fill the modal area with background color
        for row in modal_area.top()..modal_area.bottom() {
            for col in modal_area.left()..modal_area.right() {
                if let Some(cell) = buf.cell_mut((col, row)) {
                    cell.set_style(bg_style);
                }
            }
        }

        // Render modal block with border
        let block = Block::default()
            .title(" Appointment Details ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.colors.primary));

        block.clone().render(modal_area, buf);

        let inner = block.inner(modal_area);
        if inner.is_empty() {
            return;
        }

        let label_width = 18u16;
        let value_x = inner.x + label_width;
        let value_width = inner.width.saturating_sub(label_width + 2);

        let mut y = inner.y + 1;

        // Helper to render a label-value pair
        let mut render_field = |label: &str, value: &str, style: Option<Style>| {
            if y >= inner.y + inner.height - 3 {
                return;
            }
            let label_style = Style::default()
                .fg(self.theme.colors.foreground)
                .add_modifier(Modifier::BOLD);
            buf.set_string(inner.x + 1, y, label, label_style);

            let value_style =
                style.unwrap_or_else(|| Style::default().fg(self.theme.colors.foreground));
            let display_value = if value.len() > value_width as usize {
                format!("{}...", &value[..value_width as usize - 3])
            } else {
                value.to_string()
            };
            buf.set_string(value_x, y, display_value, value_style);
            y += 1;
        };

        // Patient Name
        render_field("Patient:", &self.appointment.patient_name, None);

        // Date
        render_field("Date:", &self.format_date(), None);

        // Time
        render_field("Time:", &self.format_time(), None);

        // Duration
        render_field("Duration:", &self.format_duration(), None);

        // Type
        render_field("Type:", &self.format_type(), None);

        // Status (with color)
        let status_color = self.get_status_color();
        let status_style = Style::default().fg(status_color);
        render_field("Status:", &self.format_status(), Some(status_style));

        // Reason (optional)
        if let Some(ref reason) = self.appointment.reason {
            if !reason.is_empty() {
                render_field("Reason:", reason, None);
            }
        }

        // Notes (optional)
        if let Some(ref notes) = self.appointment.notes {
            if !notes.is_empty() {
                render_field("Notes:", notes, None);
            }
        }

        // Render buttons at the bottom
        y += 1;

        // Build button list: Close, Status Dropdown, View Clinical Notes
        let buttons: Vec<(&str, bool)> = vec![
            (" Close ", self.focused_button == 0),
            (" Change Status ", self.focused_button == 1),
            (" View Clinical Notes ", self.focused_button == 2),
        ];

        // Calculate button layout
        let button_width = 17u16;
        let spacing = 2u16;
        let total_buttons_width = button_width * buttons.len() as u16
            + spacing * (buttons.len().saturating_sub(1)) as u16;
        let button_start_x = inner.x + (inner.width.saturating_sub(total_buttons_width)) / 2;
        let change_status_button_x = button_start_x + button_width + spacing;

        // Render each button
        let mut current_x = button_start_x;
        let button_y = y;
        for (label, is_focused) in &buttons {
            let style = if *is_focused {
                Style::default()
                    .fg(self.theme.colors.primary)
                    .add_modifier(Modifier::REVERSED)
            } else {
                Style::default().fg(self.theme.colors.foreground)
            };
            buf.set_string(current_x, y, label, style);
            current_x += button_width + spacing;
        }

        // Help text
        y += 1;
        let help_text = "Tab: Navigate | Enter: Select | Esc: Close";
        let help_x = inner.x + (inner.width.saturating_sub(help_text.len() as u16)) / 2;
        buf.set_string(
            help_x,
            y,
            help_text,
            Style::default().fg(self.theme.colors.disabled),
        );

        if self.focused_button == 1 {
            let change_status_label_width = " Change Status ".len() as u16;
            let dropdown_width = button_width.max(change_status_label_width.saturating_add(4));
            let centered_x = change_status_button_x
                .saturating_add(button_width / 2)
                .saturating_sub(dropdown_width / 2);
            let max_x = inner.right().saturating_sub(dropdown_width);
            let dropdown_x = centered_x.max(inner.x).min(max_x);
            let dropdown_y = button_y + 1;
            let dropdown_area = Rect::new(dropdown_x, dropdown_y, dropdown_width, 3);
            let mut dropdown = self.status_dropdown.clone();
            dropdown.focused = true;
            dropdown.render(dropdown_area, buf);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};
    use opengp_domain::domain::appointment::AppointmentType;

    fn make_appointment() -> CalendarAppointment {
        let start = Utc.with_ymd_and_hms(2026, 3, 15, 9, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2026, 3, 15, 9, 30, 0).unwrap();

        CalendarAppointment {
            id: Uuid::new_v4(),
            patient_id: Uuid::new_v4(),
            patient_name: "John Smith".to_string(),
            practitioner_id: Uuid::new_v4(),
            start_time: start,
            end_time: end,
            appointment_type: AppointmentType::Long,
            status: AppointmentStatus::Confirmed,
            is_urgent: false,
            slot_span: 2,
            reason: Some("Follow-up consultation".to_string()),
            notes: Some("Patient requested morning appointment".to_string()),
            is_overlapping: false,
        }
    }

    fn make_modal() -> AppointmentDetailModal {
        AppointmentDetailModal::new(make_appointment(), Theme::dark())
    }

    #[test]
    fn test_modal_creation() {
        let modal = make_modal();
        // Initial focus is on Close (index 0)
        assert!(modal.is_close_focused());
        assert!(!modal.is_clinical_focused());
    }

    #[test]
    fn test_button_navigation() {
        let mut modal = make_modal();
        // make_appointment has Confirmed status
        // Buttons: Close (0), Mark Arrived (1), View Clinical Notes (2)
        // NoShow is NOT available for Confirmed (domain rule: only from Arrived)

        // Initial focus is on Close (index 0)
        assert!(modal.is_close_focused());
        assert!(!modal.is_clinical_focused());

        modal.next_button();
        // Next button is Mark Arrived (index 1) for Confirmed status
        assert!(!modal.is_close_focused());
        assert!(!modal.is_clinical_focused());

        modal.next_button();
        // Next is View Clinical Notes (index 2) - NoShow not available for Confirmed
        assert!(!modal.is_close_focused());
        assert!(modal.is_clinical_focused());

        modal.next_button();
        // Wraps to Close (index 0)
        assert!(modal.is_close_focused());

        modal.prev_button();
        // Previous is View Clinical Notes (index 2)
        assert!(!modal.is_close_focused());
        assert!(modal.is_clinical_focused());
    }

    #[test]
    fn test_format_time() {
        let modal = make_modal();
        assert_eq!(modal.format_time(), "09:00 - 09:30");
    }

    #[test]
    fn test_format_duration() {
        let modal = make_modal();
        assert_eq!(modal.format_duration(), "30 minutes");
    }

    #[test]
    fn test_format_type() {
        let modal = make_modal();
        assert_eq!(modal.format_type(), "Long Consultation");
    }

    #[test]
    fn test_format_status() {
        let modal = make_modal();
        assert_eq!(modal.format_status(), "Confirmed");
    }

    #[test]
    fn test_mark_no_show_returns_correct_action() {
        let start = Utc.with_ymd_and_hms(2026, 3, 15, 9, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2026, 3, 15, 9, 30, 0).unwrap();
        let appointment = CalendarAppointment {
            id: Uuid::new_v4(),
            patient_id: Uuid::new_v4(),
            patient_name: "John Smith".to_string(),
            practitioner_id: Uuid::new_v4(),
            start_time: start,
            end_time: end,
            appointment_type: AppointmentType::Long,
            status: AppointmentStatus::Arrived,
            is_urgent: false,
            slot_span: 2,
            reason: Some("Follow-up consultation".to_string()),
            notes: Some("Patient requested morning appointment".to_string()),
            is_overlapping: false,
        };

        let mut modal = AppointmentDetailModal::new(appointment, Theme::dark());
        modal.next_button();
        assert_eq!(modal.focused_button, 1);

        let enter_key = KeyEvent::new(crossterm::event::KeyCode::Enter, KeyModifiers::empty());
        modal.handle_key(enter_key);

        modal.status_dropdown.select_next();
        modal.status_dropdown.select_next();

        let confirm_key = KeyEvent::new(crossterm::event::KeyCode::Enter, KeyModifiers::empty());
        let action = modal.handle_key(confirm_key);
        assert_eq!(action, Some(AppointmentDetailModalAction::MarkNoShow));
    }

    #[test]
    fn test_tab_moves_focus_away_from_status_when_dropdown_closed() {
        let mut modal = make_modal();
        modal.next_button();
        assert_eq!(modal.focused_button, 1);
        assert!(!modal.status_dropdown.is_open());

        let tab_key = KeyEvent::new(crossterm::event::KeyCode::Tab, KeyModifiers::empty());
        let action = modal.handle_key(tab_key);

        assert_eq!(action, None);
        assert_eq!(modal.focused_button, 2);
    }

    #[test]
    fn test_arrived_can_select_in_progress_action() {
        let start = Utc.with_ymd_and_hms(2026, 3, 15, 9, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2026, 3, 15, 9, 30, 0).unwrap();
        let appointment = CalendarAppointment {
            id: Uuid::new_v4(),
            patient_id: Uuid::new_v4(),
            patient_name: "John Smith".to_string(),
            practitioner_id: Uuid::new_v4(),
            start_time: start,
            end_time: end,
            appointment_type: AppointmentType::Long,
            status: AppointmentStatus::Arrived,
            is_urgent: false,
            slot_span: 2,
            reason: Some("Follow-up consultation".to_string()),
            notes: Some("Patient requested morning appointment".to_string()),
            is_overlapping: false,
        };

        let mut modal = AppointmentDetailModal::new(appointment, Theme::dark());
        modal.next_button();
        assert_eq!(modal.focused_button, 1);

        let open_key = KeyEvent::new(crossterm::event::KeyCode::Enter, KeyModifiers::empty());
        assert_eq!(modal.handle_key(open_key), None);
        assert!(modal.status_dropdown.is_open());

        modal.status_dropdown.select_next();

        let confirm_key = KeyEvent::new(crossterm::event::KeyCode::Enter, KeyModifiers::empty());
        let action = modal.handle_key(confirm_key);
        assert_eq!(action, Some(AppointmentDetailModalAction::MarkInProgress));
    }

    #[test]
    fn test_tab_closes_open_dropdown_and_moves_focus() {
        let mut modal = make_modal();
        modal.next_button();
        assert_eq!(modal.focused_button, 1);

        let open_key = KeyEvent::new(crossterm::event::KeyCode::Enter, KeyModifiers::empty());
        assert_eq!(modal.handle_key(open_key), None);
        assert!(modal.status_dropdown.is_open());

        let tab_key = KeyEvent::new(crossterm::event::KeyCode::Tab, KeyModifiers::empty());
        assert_eq!(modal.handle_key(tab_key), None);
        assert!(!modal.status_dropdown.is_open());
        assert_eq!(modal.focused_button, 2);
    }

    #[test]
    fn test_can_mark_no_show_only_for_arrived() {
        assert!(!AppointmentDetailModal::can_transition(
            AppointmentStatus::Scheduled,
            AppointmentStatus::NoShow
        ));

        assert!(!AppointmentDetailModal::can_transition(
            AppointmentStatus::Confirmed,
            AppointmentStatus::NoShow
        ));

        assert!(AppointmentDetailModal::can_transition(
            AppointmentStatus::Arrived,
            AppointmentStatus::NoShow
        ));

        assert!(!AppointmentDetailModal::can_transition(
            AppointmentStatus::InProgress,
            AppointmentStatus::NoShow
        ));

        assert!(!AppointmentDetailModal::can_transition(
            AppointmentStatus::Completed,
            AppointmentStatus::NoShow
        ));
    }

    #[test]
    fn test_dropdown_filtered_by_valid_transitions() {
        let start = Utc.with_ymd_and_hms(2026, 3, 15, 9, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2026, 3, 15, 9, 30, 0).unwrap();
        let scheduled_appt = CalendarAppointment {
            id: Uuid::new_v4(),
            patient_id: Uuid::new_v4(),
            patient_name: "John Smith".to_string(),
            practitioner_id: Uuid::new_v4(),
            start_time: start,
            end_time: end,
            appointment_type: AppointmentType::Long,
            status: AppointmentStatus::Scheduled,
            is_urgent: false,
            slot_span: 2,
            reason: None,
            notes: None,
            is_overlapping: false,
        };

        let modal = AppointmentDetailModal::new(scheduled_appt, Theme::dark());
        let options = &modal.status_dropdown.options;
        let option_values: Vec<&str> = options.iter().map(|o| o.value.as_str()).collect();

        assert!(option_values.contains(&"scheduled"));
        assert!(option_values.contains(&"confirmed"));
        assert!(option_values.contains(&"arrived"));
        assert!(option_values.contains(&"cancelled"));
        assert!(option_values.contains(&"rescheduled"));
        assert!(!option_values.contains(&"no_show"));
        assert!(!option_values.contains(&"in_progress"));

        let arrived_appt = CalendarAppointment {
            id: Uuid::new_v4(),
            patient_id: Uuid::new_v4(),
            patient_name: "Jane Doe".to_string(),
            practitioner_id: Uuid::new_v4(),
            start_time: start,
            end_time: end,
            appointment_type: AppointmentType::Standard,
            status: AppointmentStatus::Arrived,
            is_urgent: false,
            slot_span: 1,
            reason: None,
            notes: None,
            is_overlapping: false,
        };

        let modal = AppointmentDetailModal::new(arrived_appt, Theme::dark());
        let options = &modal.status_dropdown.options;
        let option_values: Vec<&str> = options.iter().map(|o| o.value.as_str()).collect();

        assert!(option_values.contains(&"arrived"));
        assert!(option_values.contains(&"in_progress"));
        assert!(option_values.contains(&"no_show"));
        assert!(!option_values.contains(&"scheduled"));
        assert!(!option_values.contains(&"confirmed"));
    }
}

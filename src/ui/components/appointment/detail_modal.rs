//! Appointment Detail Modal Component
//!
//! Read-only modal displaying appointment details with options to view clinical notes.

use crossterm::event::{KeyEvent, KeyModifiers};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Borders, Widget};
use uuid::Uuid;

use crate::domain::appointment::{AppointmentStatus, CalendarAppointment};
use crate::ui::theme::Theme;

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
}

impl Clone for AppointmentDetailModal {
    fn clone(&self) -> Self {
        Self {
            appointment: self.appointment.clone(),
            theme: self.theme.clone(),
            focused_button: self.focused_button,
            patient_id: self.patient_id,
        }
    }
}

impl AppointmentDetailModal {
    /// Create a new appointment detail modal.
    pub fn new(appointment: CalendarAppointment, theme: Theme) -> Self {
        Self {
            appointment: appointment.clone(),
            theme,
            focused_button: 0,
            patient_id: appointment.patient_id,
        }
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
        use crate::domain::appointment::AppointmentType;
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

    /// Get the number of visible buttons (depends on valid status transitions)
    fn button_count(&self) -> usize {
        let mut count = 2; // Close and View Clinical Notes
        if self.can_mark_arrived() {
            count += 1;
        }
        if self.can_mark_in_progress() {
            count += 1;
        }
        if self.can_mark_completed() {
            count += 1;
        }
        count
    }

    /// Check if can transition to Arrived
    fn can_mark_arrived(&self) -> bool {
        use AppointmentStatus::*;
        matches!(self.appointment.status, Scheduled | Confirmed)
    }

    /// Check if can transition to InProgress
    fn can_mark_in_progress(&self) -> bool {
        use AppointmentStatus::*;
        matches!(self.appointment.status, Arrived)
    }

    /// Check if can transition to Completed
    fn can_mark_completed(&self) -> bool {
        use AppointmentStatus::*;
        matches!(self.appointment.status, InProgress)
    }

    /// Get the button index for each action
    fn get_button_index(&self) -> std::collections::HashMap<usize, AppointmentDetailModalAction> {
        let mut map = std::collections::HashMap::new();
        map.insert(0, AppointmentDetailModalAction::Close);

        let mut idx = 1;
        if self.can_mark_arrived() {
            map.insert(idx, AppointmentDetailModalAction::MarkArrived);
            idx += 1;
        }
        if self.can_mark_in_progress() {
            map.insert(idx, AppointmentDetailModalAction::MarkInProgress);
            idx += 1;
        }
        if self.can_mark_completed() {
            map.insert(idx, AppointmentDetailModalAction::MarkCompleted);
            idx += 1;
        }
        map.insert(idx, AppointmentDetailModalAction::ViewClinicalNotes);
        map
    }

    // ── Key handling ───────────────────────────────────────────────────────

    /// Handle keyboard input and return an action if triggered.
    pub fn handle_key(&mut self, key: KeyEvent) -> Option<AppointmentDetailModalAction> {
        use crossterm::event::KeyCode;

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
            KeyCode::Left | KeyCode::Up => {
                self.prev_button();
                None
            }
            KeyCode::Right | KeyCode::Down => {
                self.next_button();
                None
            }
            KeyCode::Enter => {
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
        let modal_width = modal_width.max(50).min(80);

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

        // Build button list dynamically based on valid transitions
        let mut buttons: Vec<(&str, bool)> = vec![(" Close ", self.focused_button == 0)];

        let mut idx = 1;
        if self.can_mark_arrived() {
            buttons.push((" Mark Arrived ", self.focused_button == idx));
            idx += 1;
        }
        if self.can_mark_in_progress() {
            buttons.push((" Start Consultation ", self.focused_button == idx));
            idx += 1;
        }
        if self.can_mark_completed() {
            buttons.push((" Complete ", self.focused_button == idx));
            idx += 1;
        }
        buttons.push((" View Clinical Notes ", self.focused_button == idx));

        // Calculate button layout
        let button_width = 17u16;
        let spacing = 2u16;
        let total_buttons_width = button_width * buttons.len() as u16
            + spacing * (buttons.len().saturating_sub(1)) as u16;
        let button_start_x = inner.x + (inner.width.saturating_sub(total_buttons_width)) / 2;

        // Render each button
        let mut current_x = button_start_x;
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::appointment::AppointmentType;
    use chrono::{TimeZone, Utc};

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
        // Initial focus is on Close (index 0)
        assert!(modal.is_close_focused());
        assert!(!modal.is_clinical_focused());

        modal.next_button();
        // Next button is Mark Arrived (index 1) for Confirmed status
        assert!(!modal.is_close_focused());
        assert!(!modal.is_clinical_focused());

        modal.next_button();
        // Next is View Clinical Notes (index 2)
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
}

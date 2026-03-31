//! Appointment block widget for calendar display.
//!
//! Renders individual appointment blocks with status-based colors.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::Widget;

use crate::ui::theme::Theme;
use opengp_domain::domain::appointment::{AppointmentStatus, AppointmentType, CalendarAppointment};

/// Widget for rendering an appointment block in the calendar.
///
/// Displays appointment information with status-based background colors
/// and supports selection highlighting.
#[derive(Debug, Clone)]
pub struct AppointmentBlock {
    /// The appointment data to render
    appointment: CalendarAppointment,
    /// Whether this appointment is currently selected
    is_selected: bool,
    /// Theme for colors
    theme: Theme,
}

impl AppointmentBlock {
    /// Create a new appointment block widget.
    pub fn new(appointment: CalendarAppointment, theme: Theme) -> Self {
        Self {
            appointment,
            is_selected: false,
            theme,
        }
    }

    /// Set the selected state of the appointment block.
    #[must_use]
    pub fn selected(mut self, selected: bool) -> Self {
        self.is_selected = selected;
        self
    }

    /// Get the background color based on appointment status.
    fn get_status_color(&self) -> ratatui::style::Color {
        match self.appointment.status {
            AppointmentStatus::Scheduled => self.theme.colors.appointment_scheduled,
            AppointmentStatus::Confirmed => self.theme.colors.appointment_confirmed,
            AppointmentStatus::Arrived => self.theme.colors.appointment_arrived,
            AppointmentStatus::InProgress => self.theme.colors.appointment_in_progress,
            AppointmentStatus::Completed => self.theme.colors.appointment_completed,
            AppointmentStatus::Cancelled => self.theme.colors.appointment_cancelled,
            AppointmentStatus::NoShow => self.theme.colors.appointment_dna,
            AppointmentStatus::Rescheduled => self.theme.colors.appointment_scheduled,
        }
    }

    /// Get the abbreviation for the appointment type.
    fn get_type_abbreviation(&self) -> &str {
        match self.appointment.appointment_type {
            AppointmentType::Standard => "STD",
            AppointmentType::Long => "LNG",
            AppointmentType::Brief => "BRF",
            AppointmentType::NewPatient => "NEW",
            AppointmentType::HealthAssessment => "HA",
            AppointmentType::ChronicDiseaseReview => "CDR",
            AppointmentType::MentalHealthPlan => "MHP",
            AppointmentType::Immunisation => "IMM",
            AppointmentType::Procedure => "PROC",
            AppointmentType::Telephone => "TEL",
            AppointmentType::Telehealth => "TH",
            AppointmentType::HomeVisit => "HV",
            AppointmentType::Emergency => "EMG",
        }
    }
}

impl Widget for AppointmentBlock {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Calculate height based on duration: duration_minutes / 15 * slot_height
        // Each slot is 15 minutes; we'll use a base slot height of 10 pixels
        let duration_minutes = self.appointment.duration_minutes();
        let slot_height: u16 = 10;
        let calculated_height = ((duration_minutes / 15) as u16) * slot_height;

        // Ensure minimum height of 1 and don't exceed available area
        let height = calculated_height.max(1).min(area.height);

        // Calculate the actual area to render (top-aligned within the given area)
        let render_area = Rect::new(area.x, area.y, area.width, height);

        // Get status color for background
        let status_color = self.get_status_color();

        // Build the style based on status
        let mut style = Style::default().bg(status_color);

        // Apply strikethrough for cancelled appointments
        if self.appointment.status == AppointmentStatus::Cancelled {
            style = style.add_modifier(ratatui::style::Modifier::CROSSED_OUT);
        }

        // If selected, add a bright border
        if self.is_selected {
            style = style.bold();
        }

        for y in render_area.y..(render_area.y + render_area.height) {
            for x in render_area.x..(render_area.x + render_area.width) {
                buf[(x, y)].set_style(style);
            }
        }

        let patient_name = &self.appointment.patient_name;
        let type_abbrev = self.get_type_abbreviation();

        let content = if self.is_selected {
            format!("* {} [{}]", patient_name, type_abbrev)
        } else {
            format!("{} [{}]", patient_name, type_abbrev)
        };

        let max_width = render_area.width.saturating_sub(2) as usize;
        let display_content = if content.len() > max_width {
            format!("{}...", &content[..max_width.saturating_sub(3)])
        } else {
            content
        };

        let text_style = if self.is_selected {
            Style::default().fg(self.theme.colors.background).bold()
        } else {
            Style::default().fg(self.theme.colors.background)
        };

        buf.set_string(
            render_area.x + 1,
            render_area.y,
            display_content,
            text_style,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};

    fn create_test_appointment(status: AppointmentStatus) -> CalendarAppointment {
        let start = Utc.with_ymd_and_hms(2024, 1, 15, 9, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2024, 1, 15, 9, 30, 0).unwrap();

        CalendarAppointment {
            id: uuid::Uuid::new_v4(),
            patient_id: uuid::Uuid::new_v4(),
            patient_name: "John Smith".to_string(),
            practitioner_id: uuid::Uuid::new_v4(),
            start_time: start,
            end_time: end,
            appointment_type: AppointmentType::Standard,
            status,
            is_urgent: false,
            slot_span: 2,
            reason: None,
            notes: None,
            is_overlapping: false,
        }
    }

    #[test]
    fn test_status_colors() {
        let theme = Theme::dark();

        let statuses = [
            AppointmentStatus::Scheduled,
            AppointmentStatus::Confirmed,
            AppointmentStatus::Arrived,
            AppointmentStatus::InProgress,
            AppointmentStatus::Completed,
            AppointmentStatus::Cancelled,
            AppointmentStatus::NoShow,
            AppointmentStatus::Rescheduled,
        ];

        for status in statuses {
            let appointment = create_test_appointment(status);
            let block = AppointmentBlock::new(appointment, theme.clone());
            let _color = block.get_status_color();
        }
    }

    #[test]
    fn test_type_abbreviation() {
        let theme = Theme::dark();

        let appointment = create_test_appointment(AppointmentStatus::Scheduled);
        let block = AppointmentBlock::new(appointment, theme);

        assert_eq!(block.get_type_abbreviation(), "STD");
    }
}

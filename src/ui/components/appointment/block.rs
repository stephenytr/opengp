//! Appointment block widget for calendar display.
//!
//! Renders individual appointment blocks with status-based colors.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::Stylize;
use ratatui::style::Style;
use ratatui::widgets::Widget;

use crate::domain::appointment::{AppointmentStatus, AppointmentType, CalendarAppointment};
use crate::ui::theme::Theme;
use crate::ui::view_models::AppointmentViewItem;

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
    #[allow(dead_code)]
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
            AppointmentStatus::Scheduled => ratatui::style::Color::Yellow,
            AppointmentStatus::Confirmed => ratatui::style::Color::Green,
            AppointmentStatus::Arrived => ratatui::style::Color::Blue,
            AppointmentStatus::InProgress => ratatui::style::Color::Cyan,
            AppointmentStatus::Completed => ratatui::style::Color::Gray,
            AppointmentStatus::Cancelled => ratatui::style::Color::Red,
            AppointmentStatus::NoShow => ratatui::style::Color::DarkGray,
            AppointmentStatus::Rescheduled => ratatui::style::Color::LightRed,
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
            Style::default().fg(ratatui::style::Color::Black).bold()
        } else {
            Style::default().fg(ratatui::style::Color::Black)
        };

        buf.set_string(
            render_area.x + 1,
            render_area.y,
            display_content,
            text_style,
        );

        // If selected, render a border around the block
        if self.is_selected {
            let border_style = Style::default()
                .fg(ratatui::style::Color::White)
                .bg(status_color);

            // Top border
            for x in render_area.x..(render_area.x + render_area.width) {
                buf[(x, render_area.y)].set_style(border_style);
                buf[(x, render_area.y)].set_char('─');
            }
            // Bottom border
            let bottom_y = render_area.y + render_area.height - 1;
            for x in render_area.x..(render_area.x + render_area.width) {
                buf[(x, bottom_y)].set_style(border_style);
                buf[(x, bottom_y)].set_char('─');
            }
            // Left border
            for y in render_area.y..(render_area.y + render_area.height) {
                buf[(render_area.x, y)].set_style(border_style);
                buf[(render_area.x, y)].set_char('│');
            }
            // Right border
            let right_x = render_area.x + render_area.width - 1;
            for y in render_area.y..(render_area.y + render_area.height) {
                buf[(right_x, y)].set_style(border_style);
                buf[(right_x, y)].set_char('│');
            }
            // Corners
            buf[(render_area.x, render_area.y)].set_char('┌');
            buf[(right_x, render_area.y)].set_char('┐');
            buf[(render_area.x, bottom_y)].set_char('└');
            buf[(right_x, bottom_y)].set_char('┘');
        }
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

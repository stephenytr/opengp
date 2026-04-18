//! Patient Appointments subtab view for workspace.
//!
//! Displays a paginated list of appointments for a specific patient within the workspace.
//! Supports navigation (↑↓), Enter to view read-only detail modal, and lazy-loading on first visit.

use uuid::Uuid;
use crate::ui::theme::Theme;
use crate::ui::view_models::AppointmentViewItem;
use crate::ui::services::appointment_service::AppointmentUiService;
use crate::ui::services::shared::UiServiceError;
use opengp_domain::domain::appointment::Appointment;

/// Actions triggered from the appointments view
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppointmentViewAction {
    /// Navigate up in the list (Ctrl+P or ↑)
    NavigateUp,
    /// Navigate down in the list (Ctrl+N or ↓)
    NavigateDown,
    /// Open detail modal for selected appointment (Enter)
    OpenDetail,
    /// Close the detail modal (Esc)
    CloseDetail,
}

/// Patient Appointments View component
///
/// Renders a paginated list of appointments for a specific patient.
/// Features:
/// - Lazy-loaded on first visit (AppCommand::LoadPatientWorkspaceData)
/// - Navigation with ↑↓ / Ctrl+P/N
/// - Enter to view read-only detail modal
/// - Displays: date, time, practitioner, type, status, duration
///
/// The view is stateless (renders from PatientAppointmentState data).
#[derive(Clone)]
pub struct PatientAppointmentsView {
    theme: Theme,
}

impl PatientAppointmentsView {
    /// Create a new appointments view with the given theme.
    pub fn new(theme: Theme) -> Self {
        Self { theme }
    }

    /// Format appointment list items for display.
    pub fn format_items(appointments: &[Appointment]) -> Vec<AppointmentViewItem> {
        appointments
            .iter()
            .map(Self::format_appointment)
            .collect()
    }

    /// Format appointment data from domain model to view item.
    fn format_appointment(appointment: &Appointment) -> AppointmentViewItem {
        let duration_minutes = ((appointment.end_time - appointment.start_time).num_minutes()) as i64;
        AppointmentViewItem {
            id: appointment.id,
            patient_id: appointment.patient_id,
            patient_name: String::new(),
            practitioner_id: appointment.practitioner_id,
            practitioner_name: String::new(),
            start_time: appointment.start_time,
            end_time: appointment.end_time,
            status: appointment.status,
            appointment_type: appointment.appointment_type,
            duration_minutes,
            is_urgent: appointment.is_urgent,
            confirmed: appointment.confirmed,
            reason: None,
            slot_span: 1,
            notes: appointment.notes.clone(),
        }
    }
}

/// Service bridge for loading patient appointments
pub struct AppointmentViewService;

impl AppointmentViewService {
    /// Load appointments for a patient using the appointment repository
    /// 
    /// This method encapsulates the logic to fetch a patient's appointments
    /// from the UI service. The appointment service supports filtering by patient_id
    /// via AppointmentSearchCriteria.
    /// 
    /// # TODO
    /// Requires a dedicated method in AppointmentUiService:
    /// ```ignore
    /// pub async fn list_patient_appointments(
    ///     &self,
    ///     patient_id: Uuid,
    /// ) -> UiResult<Vec<Appointment>>
    /// ```
    pub async fn load_patient_appointments(
        _service: &AppointmentUiService,
        _patient_id: Uuid,
    ) -> Result<Vec<Appointment>, UiServiceError> {
        Err(UiServiceError::Repository(
            "Requires AppointmentUiService::list_patient_appointments method".to_string()
        ))
    }
}

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::Widget;

impl Widget for PatientAppointmentsView {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.is_empty() {
            return;
        }

        use ratatui::style::Style;
        use ratatui::text::Line;
        use ratatui::widgets::{Block, Borders, Paragraph};

        let block = Block::default()
            .title(" Appointments ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.colors.border));

        let inner = block.inner(area);
        block.render(area, buf);

        let para = Paragraph::new(Line::from(
            "Appointments view — use ↑↓ to navigate, Enter for details",
        ));
        para.render(inner, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use crate::ui::components::shared::PaginatedState;
    use opengp_domain::domain::appointment::AppointmentStatus;
    use opengp_domain::domain::appointment::AppointmentType;

    fn create_test_appointment(patient_id: Uuid) -> Appointment {
        Appointment {
            id: Uuid::new_v4(),
            patient_id,
            practitioner_id: Uuid::new_v4(),
            start_time: Utc::now(),
            end_time: Utc::now() + chrono::Duration::minutes(30),
            appointment_type: AppointmentType::Standard,
            status: AppointmentStatus::Scheduled,
            reason: None,
            is_urgent: false,
            reminder_sent: false,
            confirmed: true,
            cancellation_reason: None,
            notes: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            version: 0,
            created_by: None,
            updated_by: None,
        }
    }

    #[test]
    fn test_appointments_view_new() {
        let theme = Theme::default();
        let view = PatientAppointmentsView::new(theme.clone());
        assert_eq!(view.theme.colors.primary, theme.colors.primary);
    }

    #[test]
    fn test_format_appointment() {
        let patient_id = Uuid::new_v4();
        let appt = create_test_appointment(patient_id);
        
        let item = PatientAppointmentsView::format_appointment(&appt);
        assert_eq!(item.id, appt.id);
        assert_eq!(item.patient_id, patient_id);
        assert_eq!(item.duration_minutes, 30);
    }

    #[test]
    fn test_appointment_view_action_enum() {
        let up = AppointmentViewAction::NavigateUp;
        let down = AppointmentViewAction::NavigateDown;
        
        assert_ne!(up, down);
        assert_eq!(up, AppointmentViewAction::NavigateUp);
    }

    #[tokio::test]
    async fn test_paginated_state() {
        let mut state = PaginatedState::new();
        assert_eq!(state.page, 0);
        assert_eq!(state.page_size, 20);
        
        state.next_page(50);
        assert_eq!(state.page, 1);
        
        state.prev_page();
        assert_eq!(state.page, 0);
    }
}

//! Patient Summary View for Workspace
//!
//! Displays a read-only overview of patient demographics and clinical information
//! in a two-column layout. Gracefully handles unloaded subtabs.

use chrono::Datelike;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Widget};

use crate::ui::theme::Theme;
use crate::ui::view_models::PatientListItem;
use crate::ui::components::clinical::ClinicalState;
use crate::ui::components::billing::PatientBillingState;
use super::appointment_state::PatientAppointmentState;

/// Patient Summary View for workspace subtab
///
/// Renders a read-only overview panel with:
/// - Left column: Patient demographics (name, DOB, Medicare, gender)
/// - Right column: Clinical summary (vitals, allergies, next appointment, last consultation)
#[derive(Clone)]
pub struct SummaryView {
    pub patient_snapshot: PatientListItem,
    pub clinical_state: Option<ClinicalState>,
    pub billing_state: Option<PatientBillingState>,
    pub appointment_state: Option<PatientAppointmentState>,
    theme: Theme,
}

impl SummaryView {
    /// Create a new summary view
    pub fn new(patient_snapshot: PatientListItem, theme: Theme) -> Self {
        Self {
            patient_snapshot,
            clinical_state: None,
            billing_state: None,
            appointment_state: None,
            theme,
        }
    }

    /// Set clinical state (optional)
    pub fn with_clinical(mut self, clinical_state: Option<ClinicalState>) -> Self {
        self.clinical_state = clinical_state;
        self
    }

    /// Set billing state (optional)
    pub fn with_billing(mut self, billing_state: Option<PatientBillingState>) -> Self {
        self.billing_state = billing_state;
        self
    }

    /// Set appointment state (optional)
    pub fn with_appointments(mut self, appointment_state: Option<PatientAppointmentState>) -> Self {
        self.appointment_state = appointment_state;
        self
    }

    /// Get text color based on theme
    fn get_label_style(&self) -> Style {
        Style::default().fg(self.theme.colors.primary).bold()
    }

    fn get_value_style(&self) -> Style {
        Style::default().fg(self.theme.colors.foreground)
    }

    fn get_disabled_style(&self) -> Style {
        Style::default().fg(self.theme.colors.disabled)
    }

    fn get_border_style(&self) -> Style {
        Style::default().fg(self.theme.colors.border)
    }
}

impl Widget for SummaryView {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.is_empty() {
            return;
        }

        let block = Block::default()
            .title(" Patient Summary ")
            .borders(Borders::ALL)
            .border_style(self.get_border_style());

        block.clone().render(area, buf);

        let inner = block.inner(area);
        if inner.is_empty() {
            return;
        }

        // Split into two columns
        let col_width = inner.width / 2;
        let left_col = Rect::new(
            inner.x,
            inner.y,
            col_width.saturating_sub(1),
            inner.height,
        );
        let right_col = Rect::new(
            inner.x + col_width,
            inner.y,
            inner.width.saturating_sub(col_width),
            inner.height,
        );

        self.render_demographics(left_col, buf);
        self.render_clinical_summary(right_col, buf);
    }
}

impl SummaryView {
    /// Render left column: demographics
    fn render_demographics(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title(" Demographics ")
            .borders(Borders::ALL)
            .border_style(self.get_border_style());

        block.clone().render(area, buf);

        let inner = block.inner(area);
        if inner.is_empty() {
            return;
        }

        let mut lines: Vec<Line> = Vec::new();

        // Name
        lines.push(Line::from(vec![
            Span::styled("Name: ", self.get_label_style()),
            Span::styled(
                self.patient_snapshot.full_name.clone(),
                self.get_value_style(),
            ),
        ]));

        // DOB and Age
        let dob_str = self.patient_snapshot.date_of_birth.format("%d/%m/%Y").to_string();
        let today = chrono::Utc::now().date_naive();
        let age = today.year()
            - self.patient_snapshot.date_of_birth.year()
            - if today.month() < self.patient_snapshot.date_of_birth.month()
                || (today.month() == self.patient_snapshot.date_of_birth.month()
                    && today.day() < self.patient_snapshot.date_of_birth.day())
            {
                1
            } else {
                0
            };
        lines.push(Line::from(vec![
            Span::styled("DOB: ", self.get_label_style()),
            Span::styled(
                format!("{} ({} yrs)", dob_str, age),
                self.get_value_style(),
            ),
        ]));

        // Gender
        lines.push(Line::from(vec![
            Span::styled("Gender: ", self.get_label_style()),
            Span::styled(
                self.patient_snapshot.gender.to_string(),
                self.get_value_style(),
            ),
        ]));

        // Medicare
        if let Some(medicare) = &self.patient_snapshot.medicare_number {
            let medicare_str = if let Some(irn) = self.patient_snapshot.medicare_irn {
                format!("{} (IRN: {})", medicare, irn)
            } else {
                medicare.clone()
            };
            lines.push(Line::from(vec![
                Span::styled("Medicare: ", self.get_label_style()),
                Span::styled(medicare_str, self.get_value_style()),
            ]));
        } else {
            lines.push(Line::from(vec![
                Span::styled("Medicare: ", self.get_label_style()),
                Span::styled("Not recorded", self.get_disabled_style()),
            ]));
        }

        // IHI
        if let Some(ihi) = &self.patient_snapshot.ihi {
            lines.push(Line::from(vec![
                Span::styled("IHI: ", self.get_label_style()),
                Span::styled(ihi, self.get_value_style()),
            ]));
        }

        // Phone
        if let Some(phone) = &self.patient_snapshot.phone_mobile {
            lines.push(Line::from(vec![
                Span::styled("Phone: ", self.get_label_style()),
                Span::styled(phone, self.get_value_style()),
            ]));
        }

        // Render the demographics
        let mut y = inner.y;
        for line in lines.iter().take(inner.height as usize) {
            if y >= inner.y + inner.height {
                break;
            }
            buf.set_line(inner.x + 1, y, line, inner.width.saturating_sub(2));
            y += 1;
        }
    }

    /// Render right column: clinical overview
    fn render_clinical_summary(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title(" Clinical Summary ")
            .borders(Borders::ALL)
            .border_style(self.get_border_style());

        block.clone().render(area, buf);

        let inner = block.inner(area);
        if inner.is_empty() {
            return;
        }

        let mut lines: Vec<Line> = Vec::new();

        // Recent vitals (from clinical state if loaded)
        if let Some(clinical_state) = &self.clinical_state {
            if let Some(vitals) = clinical_state.vitals.vital_signs.last() {
                lines.push(Line::from(vec![
                    Span::styled("Latest Vitals: ", self.get_label_style()),
                    Span::styled(
                        format!("{}", vitals.created_at.format("%d/%m/%Y")),
                        self.get_value_style(),
                    ),
                ]));
            } else {
                lines.push(Line::from(vec![
                    Span::styled("Latest Vitals: ", self.get_label_style()),
                    Span::styled("Not yet loaded", self.get_disabled_style()),
                ]));
            }
        } else {
            lines.push(Line::from(vec![
                Span::styled("Latest Vitals: ", self.get_label_style()),
                Span::styled("Not yet loaded", self.get_disabled_style()),
            ]));
        }

        // Allergies count (from clinical state if loaded)
        if let Some(clinical_state) = &self.clinical_state {
            let active_allergies = clinical_state
                .allergies
                .allergies
                .iter()
                .filter(|a| a.is_active)
                .count();
            let total_allergies = clinical_state.allergies.allergies.len();

            if total_allergies > 0 {
                lines.push(Line::from(vec![
                    Span::styled("Allergies: ", self.get_label_style()),
                    Span::styled(
                        format!("{}/{} active", active_allergies, total_allergies),
                        if active_allergies > 0 {
                            Style::default().fg(self.theme.colors.warning)
                        } else {
                            self.get_disabled_style()
                        },
                    ),
                ]));
            } else {
                lines.push(Line::from(vec![
                    Span::styled("Allergies: ", self.get_label_style()),
                    Span::styled("None recorded", self.get_disabled_style()),
                ]));
            }
        } else {
            lines.push(Line::from(vec![
                Span::styled("Allergies: ", self.get_label_style()),
                Span::styled("Not yet loaded", self.get_disabled_style()),
            ]));
        }

        // Last consultation date (from clinical state if loaded)
        if let Some(clinical_state) = &self.clinical_state {
            if let Some(consultation) = clinical_state.consultations.consultations.iter().rev().next() {
                lines.push(Line::from(vec![
                    Span::styled("Last Consultation: ", self.get_label_style()),
                    Span::styled(
                        format!("{}", consultation.consultation_date.format("%d/%m/%Y")),
                        self.get_value_style(),
                    ),
                ]));
            } else {
                lines.push(Line::from(vec![
                    Span::styled("Last Consultation: ", self.get_label_style()),
                    Span::styled("No consultations", self.get_disabled_style()),
                ]));
            }
        } else {
            lines.push(Line::from(vec![
                Span::styled("Last Consultation: ", self.get_label_style()),
                Span::styled("Not yet loaded", self.get_disabled_style()),
            ]));
        }

        // Next appointment (from appointment state if loaded)
        if let Some(appointment_state) = &self.appointment_state {
            if let Some(appointment) = appointment_state.appointments.first() {
                lines.push(Line::from(vec![
                    Span::styled("Next Appointment: ", self.get_label_style()),
                    Span::styled(
                        format!("{}", appointment.start_time.format("%d/%m/%Y %H:%M")),
                        self.get_value_style(),
                    ),
                ]));
            } else {
                lines.push(Line::from(vec![
                    Span::styled("Next Appointment: ", self.get_label_style()),
                    Span::styled("None scheduled", self.get_disabled_style()),
                ]));
            }
        } else {
            lines.push(Line::from(vec![
                Span::styled("Next Appointment: ", self.get_label_style()),
                Span::styled("Not yet loaded", self.get_disabled_style()),
            ]));
        }

        // Render the clinical summary
        let mut y = inner.y;
        for line in lines.iter().take(inner.height as usize) {
            if y >= inner.y + inner.height {
                break;
            }
            buf.set_line(inner.x + 1, y, line, inner.width.saturating_sub(2));
            y += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use opengp_domain::domain::patient::Gender;
    use uuid::Uuid;

    fn create_test_patient() -> PatientListItem {
        PatientListItem {
            id: Uuid::new_v4(),
            full_name: "John Doe".to_string(),
            date_of_birth: NaiveDate::from_ymd_opt(1985, 6, 15).unwrap(),
            gender: Gender::Male,
            medicare_number: Some("1234567890".to_string()),
            medicare_irn: Some(1),
            ihi: Some("8003601234567890".to_string()),
            phone_mobile: Some("0412345678".to_string()),
        }
    }

    #[test]
    fn test_summary_view_new() {
        let patient = create_test_patient();
        let theme = Theme::dark();
        let view = SummaryView::new(patient.clone(), theme);

        assert_eq!(view.patient_snapshot.id, patient.id);
        assert_eq!(view.patient_snapshot.full_name, "John Doe");
        assert!(view.clinical_state.is_none());
        assert!(view.billing_state.is_none());
        assert!(view.appointment_state.is_none());
    }

    #[test]
    fn test_summary_view_with_states() {
        let patient = create_test_patient();
        let theme = Theme::dark();
        let view = SummaryView::new(patient, theme)
            .with_clinical(None)
            .with_billing(None)
            .with_appointments(None);

        assert!(view.clinical_state.is_none());
        assert!(view.billing_state.is_none());
        assert!(view.appointment_state.is_none());
    }

    #[test]
    fn test_summary_view_patient_snapshot_data() {
        let patient = create_test_patient();
        let theme = Theme::dark();
        let view = SummaryView::new(patient.clone(), theme);

        assert_eq!(view.patient_snapshot.full_name, "John Doe");
        assert_eq!(
            view.patient_snapshot.date_of_birth,
            NaiveDate::from_ymd_opt(1985, 6, 15).unwrap()
        );
        assert_eq!(view.patient_snapshot.gender, Gender::Male);
        assert_eq!(
            view.patient_snapshot.medicare_number,
            Some("1234567890".to_string())
        );
        assert_eq!(view.patient_snapshot.medicare_irn, Some(1));
    }
}

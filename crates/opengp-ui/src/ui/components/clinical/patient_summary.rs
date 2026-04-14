//! Patient Summary Component
//!
//! Displays patient demographics, allergies, and clinical overview in a three-column layout.

use chrono::Datelike;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Row, Table, Widget};
use uuid::Uuid;

use crate::ui::theme::Theme;
use crate::ui::view_models::PatientListItem;
use opengp_domain::domain::clinical::{Allergy, Consultation, MedicalHistory, VitalSigns};

pub struct PatientSummaryComponent {
    pub patient: Option<PatientListItem>,
    pub allergies: Vec<Allergy>,
    pub conditions: Vec<MedicalHistory>,
    pub consultations: Vec<Consultation>,
    pub vitals: Option<VitalSigns>,
    theme: Theme,
    /// Optional practitioner names map (practitioner_id -> name)
    practitioner_names: std::collections::HashMap<Uuid, String>,
}

impl Clone for PatientSummaryComponent {
    fn clone(&self) -> Self {
        Self {
            patient: self.patient.clone(),
            allergies: self.allergies.clone(),
            conditions: self.conditions.clone(),
            consultations: self.consultations.clone(),
            vitals: self.vitals.clone(),
            theme: self.theme.clone(),
            practitioner_names: self.practitioner_names.clone(),
        }
    }
}

impl PatientSummaryComponent {
    pub fn new(theme: Theme) -> Self {
        Self {
            patient: None,
            allergies: Vec::new(),
            conditions: Vec::new(),
            consultations: Vec::new(),
            vitals: None,
            theme,
            practitioner_names: std::collections::HashMap::new(),
        }
    }

    pub fn with_patient(patient: Option<PatientListItem>, theme: Theme) -> Self {
        Self {
            patient,
            allergies: Vec::new(),
            conditions: Vec::new(),
            consultations: Vec::new(),
            vitals: None,
            theme,
            practitioner_names: std::collections::HashMap::new(),
        }
    }

    pub fn with_allergies(mut self, allergies: Vec<Allergy>) -> Self {
        self.allergies = allergies;
        self
    }

    pub fn with_conditions(mut self, conditions: Vec<MedicalHistory>) -> Self {
        self.conditions = conditions;
        self
    }

    pub fn with_consultations(mut self, consultations: Vec<Consultation>) -> Self {
        self.consultations = consultations;
        self
    }

    pub fn with_vitals(mut self, vitals: Option<VitalSigns>) -> Self {
        self.vitals = vitals;
        self
    }

    pub fn set_practitioner_name(&mut self, practitioner_id: Uuid, name: String) {
        self.practitioner_names.insert(practitioner_id, name);
    }

    /// Get count of active allergies
    pub fn active_allergy_count(&self) -> usize {
        self.allergies.iter().filter(|a| a.is_active).count()
    }

    /// Get total allergy count
    pub fn total_allergy_count(&self) -> usize {
        self.allergies.len()
    }

    /// Get count of active conditions
    pub fn active_condition_count(&self) -> usize {
        self.conditions.iter().filter(|c| c.is_active).count()
    }

    /// Get total condition count
    pub fn total_condition_count(&self) -> usize {
        self.conditions.len()
    }

    /// Get recent consultations (last 3)
    pub fn recent_consultations(&self, count: usize) -> Vec<&Consultation> {
        let mut sorted: Vec<_> = self.consultations.iter().collect();
        sorted.sort_by(|a, b| b.consultation_date.cmp(&a.consultation_date));
        sorted.into_iter().take(count).collect()
    }

    /// Get last visit date
    pub fn last_visit_date(&self) -> Option<chrono::DateTime<chrono::Utc>> {
        self.consultations.iter().map(|c| c.consultation_date).max()
    }
}

impl Widget for PatientSummaryComponent {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.is_empty() {
            return;
        }

        let block = Block::default()
            .title(" Patient Summary ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.colors.border));

        block.clone().render(area, buf);

        let inner = block.inner(area);
        if inner.is_empty() {
            return;
        }

        // Check if we have patient data
        if self.patient.is_none() {
            let message = "No patient selected.";
            let text = Line::from(vec![Span::styled(
                message,
                Style::default().fg(self.theme.colors.disabled),
            )]);
            let x = inner.x + (inner.width.saturating_sub(message.len() as u16)) / 2;
            let y = inner.y + inner.height / 2;
            buf.set_line(x, y, &text, inner.width);
            return;
        }

        // SAFETY: We just checked is_none() and returned early if true
        #[allow(clippy::unwrap_used)]
        let patient = self.patient.as_ref().unwrap();

        let top_height = if inner.height > 10 {
            8
        } else {
            inner.height.saturating_sub(2)
        };
        let bottom_height = inner.height.saturating_sub(top_height);

        let top_area = Rect::new(inner.x, inner.y, inner.width, top_height);
        let bottom_area = Rect::new(inner.x, inner.y + top_height, inner.width, bottom_height);

        let col_width = inner.width / 3;
        let remaining = inner.width.saturating_sub(col_width * 2);
        let col1 = Rect::new(
            top_area.x,
            top_area.y,
            col_width.saturating_sub(1),
            top_height,
        );
        let col2 = Rect::new(
            top_area.x + col_width,
            top_area.y,
            col_width.saturating_sub(1),
            top_height,
        );
        let col3 = Rect::new(
            top_area.x + col_width * 2,
            top_area.y,
            remaining,
            top_height,
        );

        self.render_demographics(col1, buf, patient);
        self.render_allergies_box(col2, buf);
        self.render_clinical_overview(col3, buf);

        if bottom_height > 2 {
            self.render_recent_consultations(bottom_area, buf);
        }
    }
}

impl PatientSummaryComponent {
    fn render_demographics(&self, area: Rect, buf: &mut Buffer, patient: &PatientListItem) {
        let block = Block::default()
            .title(" Demographics ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.colors.border));

        block.clone().render(area, buf);

        let inner = block.inner(area);
        if inner.is_empty() {
            return;
        }

        // Build demographics lines
        let mut lines: Vec<Line> = Vec::new();

        // Name
        lines.push(Line::from(vec![
            Span::styled(
                "Name: ",
                Style::default().fg(self.theme.colors.primary).bold(),
            ),
            Span::styled(
                patient.full_name.clone(),
                Style::default().fg(self.theme.colors.foreground),
            ),
        ]));

        // DOB and Age
        let dob_str = patient.date_of_birth.format("%d/%m/%Y").to_string();
        let today = chrono::Utc::now().date_naive();
        let age = today.year()
            - patient.date_of_birth.year()
            - if today.month() < patient.date_of_birth.month()
                || (today.month() == patient.date_of_birth.month()
                    && today.day() < patient.date_of_birth.day())
            {
                1
            } else {
                0
            };
        lines.push(Line::from(vec![
            Span::styled(
                "DOB: ",
                Style::default().fg(self.theme.colors.primary).bold(),
            ),
            Span::styled(
                format!("{} ({} yrs)", dob_str, age),
                Style::default().fg(self.theme.colors.foreground),
            ),
        ]));

        // Gender
        lines.push(Line::from(vec![
            Span::styled(
                "Gender: ",
                Style::default().fg(self.theme.colors.primary).bold(),
            ),
            Span::styled(
                patient.gender.to_string(),
                Style::default().fg(self.theme.colors.foreground),
            ),
        ]));

        // Medicare
        if let Some(medicare) = &patient.medicare_number {
            let medicare_str = if let Some(irn) = patient.medicare_irn {
                format!("{} (IRN: {})", medicare, irn)
            } else {
                medicare.clone()
            };
            lines.push(Line::from(vec![
                Span::styled(
                    "Medicare: ",
                    Style::default().fg(self.theme.colors.primary).bold(),
                ),
                Span::styled(
                    medicare_str,
                    Style::default().fg(self.theme.colors.foreground),
                ),
            ]));
        }

        // IHI
        if let Some(ihi) = &patient.ihi {
            lines.push(Line::from(vec![
                Span::styled(
                    "IHI: ",
                    Style::default().fg(self.theme.colors.primary).bold(),
                ),
                Span::styled(ihi, Style::default().fg(self.theme.colors.foreground)),
            ]));
        }

        // Phone
        if let Some(phone) = &patient.phone_mobile {
            lines.push(Line::from(vec![
                Span::styled(
                    "Phone: ",
                    Style::default().fg(self.theme.colors.primary).bold(),
                ),
                Span::styled(phone, Style::default().fg(self.theme.colors.foreground)),
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

    fn render_allergies_box(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title(" Allergies ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.colors.border));

        block.clone().render(area, buf);

        let inner = block.inner(area);
        if inner.is_empty() {
            return;
        }

        let active: Vec<&Allergy> = self.allergies.iter().filter(|a| a.is_active).collect();

        if active.is_empty() {
            let message = if self.allergies.is_empty() {
                "None recorded"
            } else {
                "No active allergies"
            };
            let text = Line::from(vec![Span::styled(
                message,
                Style::default().fg(self.theme.colors.disabled),
            )]);
            buf.set_line(inner.x + 1, inner.y, &text, inner.width.saturating_sub(2));
            return;
        }

        let col_widths = [
            Constraint::Length(14),
            Constraint::Length(10),
            Constraint::Min(8),
        ];

        let header = Row::new(vec!["Allergen", "Severity", "Reaction"])
            .style(Style::default().fg(self.theme.colors.primary).bold());

        let rows: Vec<Row> = active
            .iter()
            .take(inner.height.saturating_sub(2) as usize)
            .map(|allergy| {
                let severity_style = match allergy.severity {
                    opengp_domain::domain::clinical::Severity::Severe => {
                        Style::default().fg(self.theme.colors.error)
                    }
                    opengp_domain::domain::clinical::Severity::Moderate => {
                        Style::default().fg(self.theme.colors.warning)
                    }
                    opengp_domain::domain::clinical::Severity::Mild => {
                        Style::default().fg(self.theme.colors.foreground)
                    }
                };

                let reaction = allergy.reaction.as_deref().unwrap_or("-").to_string();

                Row::new(vec![
                    allergy.allergen.clone(),
                    allergy.severity.to_string(),
                    reaction,
                ])
                .style(severity_style)
                .height(1)
            })
            .collect();

        let table = Table::new(rows, col_widths)
            .header(header)
            .block(Block::default().borders(Borders::NONE))
            .widths(col_widths);

        table.render(inner, buf);
    }

    fn render_clinical_overview(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title(" Clinical Overview ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.colors.border));

        block.clone().render(area, buf);

        let inner = block.inner(area);
        if inner.is_empty() {
            return;
        }

        // Build clinical overview lines
        let mut lines: Vec<Line> = Vec::new();

        // Allergies
        let active_allergies = self.active_allergy_count();
        let total_allergies = self.total_allergy_count();
        let allergy_text = if total_allergies > 0 {
            format!("{}/{} active", active_allergies, total_allergies)
        } else {
            "None recorded".to_string()
        };
        let allergy_style = if active_allergies > 0 {
            Style::default().fg(self.theme.colors.warning)
        } else {
            Style::default().fg(self.theme.colors.disabled)
        };
        lines.push(Line::from(vec![
            Span::styled(
                "Allergies: ",
                Style::default().fg(self.theme.colors.primary).bold(),
            ),
            Span::styled(allergy_text, allergy_style),
        ]));

        // Conditions
        let active_conditions = self.active_condition_count();
        let total_conditions = self.total_condition_count();
        let condition_text = if total_conditions > 0 {
            format!("{}/{} active", active_conditions, total_conditions)
        } else {
            "None recorded".to_string()
        };
        lines.push(Line::from(vec![
            Span::styled(
                "Conditions: ",
                Style::default().fg(self.theme.colors.primary).bold(),
            ),
            Span::styled(
                condition_text,
                Style::default().fg(self.theme.colors.foreground),
            ),
        ]));

        // Last visit
        if let Some(last_visit) = self.last_visit_date() {
            let last_visit_str = last_visit.format("%d/%m/%Y").to_string();
            lines.push(Line::from(vec![
                Span::styled(
                    "Last Visit: ",
                    Style::default().fg(self.theme.colors.primary).bold(),
                ),
                Span::styled(
                    last_visit_str,
                    Style::default().fg(self.theme.colors.foreground),
                ),
            ]));
        } else {
            lines.push(Line::from(vec![
                Span::styled(
                    "Last Visit: ",
                    Style::default().fg(self.theme.colors.primary).bold(),
                ),
                Span::styled("No visits", Style::default().fg(self.theme.colors.disabled)),
            ]));
        }

        // Vitals (if available)
        if let Some(vitals) = &self.vitals {
            let mut vitals_parts: Vec<String> = Vec::new();

            if let Some(bp) = vitals.blood_pressure_string() {
                vitals_parts.push(format!("BP: {}", bp));
            }
            if let Some(hr) = vitals.heart_rate {
                vitals_parts.push(format!("HR: {} bpm", hr));
            }
            if let Some(rr) = vitals.respiratory_rate {
                vitals_parts.push(format!("RR: {}", rr));
            }
            if let Some(o2) = vitals.oxygen_saturation {
                vitals_parts.push(format!("SpO2: {}%", o2));
            }
            if let Some(temp) = vitals.temperature {
                vitals_parts.push(format!("Temp: {:.1}°C", temp));
            }
            if let Some(bmi) = vitals.bmi {
                vitals_parts.push(format!("BMI: {:.1}", bmi));
            }

            if !vitals_parts.is_empty() {
                lines.push(Line::from(vec![
                    Span::styled(
                        "Vitals: ",
                        Style::default().fg(self.theme.colors.primary).bold(),
                    ),
                    Span::styled(
                        vitals_parts.join(" | "),
                        Style::default().fg(self.theme.colors.foreground),
                    ),
                ]));
            }
        }

        // Render the clinical overview
        let mut y = inner.y;
        for line in lines.iter().take(inner.height as usize) {
            if y >= inner.y + inner.height {
                break;
            }
            buf.set_line(inner.x + 1, y, line, inner.width.saturating_sub(2));
            y += 1;
        }
    }

    fn render_recent_consultations(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title(" Recent Consultations ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.colors.border));

        block.clone().render(area, buf);

        let inner = block.inner(area);
        if inner.is_empty() {
            return;
        }

        let recent = self.recent_consultations(3);

        if recent.is_empty() {
            let message = "No consultations recorded.";
            let text = Line::from(vec![Span::styled(
                message,
                Style::default().fg(self.theme.colors.disabled),
            )]);
            let x = inner.x + (inner.width.saturating_sub(message.len() as u16)) / 2;
            let y = inner.y + inner.height / 2;
            buf.set_line(x, y, &text, inner.width);
            return;
        }

        // Table columns
        let col_widths = [
            Constraint::Length(12),
            Constraint::Length(20),
            Constraint::Length(30),
        ];

        let header = Row::new(vec!["Date", "Practitioner", "Reason"])
            .style(Style::default().fg(self.theme.colors.primary).bold());

        let rows: Vec<Row> = recent
            .iter()
            .take(inner.height.saturating_sub(2) as usize)
            .map(|consultation| {
                let date = consultation
                    .consultation_date
                    .format("%d/%m/%Y")
                    .to_string();
                let practitioner = self
                    .practitioner_names
                    .get(&consultation.practitioner_id)
                    .cloned()
                    .unwrap_or_else(|| "Unknown".to_string());
                let reason = consultation
                    .clinical_notes
                    .as_ref()
                    .map(|s| {
                        if s.len() > 28 {
                            format!("{}...", &s[..28])
                        } else {
                            s.clone()
                        }
                    })
                    .unwrap_or_else(|| "-".to_string());

                Row::new(vec![date, practitioner, reason])
                    .style(Style::default().fg(self.theme.colors.foreground))
                    .height(1)
            })
            .collect();

        let table = Table::new(rows, col_widths)
            .header(header)
            .block(Block::default().borders(Borders::NONE))
            .widths(col_widths);

        table.render(inner, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    #[test]
    fn test_patient_summary_empty() {
        let theme = Theme::dark();
        let component = PatientSummaryComponent::new(theme);
        assert!(component.patient.is_none());
        assert!(component.allergies.is_empty());
        assert!(component.conditions.is_empty());
    }

    #[test]
    fn test_allergy_counts() {
        let theme = Theme::dark();
        let mut component = PatientSummaryComponent::new(theme);

        // Add some allergies
        let active_allergy = Allergy {
            id: Uuid::new_v4(),
            patient_id: Uuid::new_v4(),
            allergen: "Penicillin".to_string(),
            allergy_type: opengp_domain::domain::clinical::AllergyType::Drug,
            severity: opengp_domain::domain::clinical::Severity::Severe,
            reaction: Some("Anaphylaxis".to_string()),
            onset_date: None,
            notes: None,
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_by: Uuid::new_v4(),
            updated_by: None,
        };

        let inactive_allergy = Allergy {
            id: Uuid::new_v4(),
            patient_id: Uuid::new_v4(),
            allergen: "Peanuts".to_string(),
            allergy_type: opengp_domain::domain::clinical::AllergyType::Food,
            severity: opengp_domain::domain::clinical::Severity::Moderate,
            reaction: Some("Hives".to_string()),
            onset_date: None,
            notes: None,
            is_active: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_by: Uuid::new_v4(),
            updated_by: None,
        };

        component.allergies = vec![active_allergy, inactive_allergy];

        assert_eq!(component.active_allergy_count(), 1);
        assert_eq!(component.total_allergy_count(), 2);
    }

    #[test]
    fn test_condition_counts() {
        let theme = Theme::dark();
        let mut component = PatientSummaryComponent::new(theme);

        let active_condition = MedicalHistory {
            id: Uuid::new_v4(),
            patient_id: Uuid::new_v4(),
            condition: "Type 2 Diabetes".to_string(),
            diagnosis_date: None,
            status: opengp_domain::domain::clinical::ConditionStatus::Active,
            severity: Some(opengp_domain::domain::clinical::Severity::Moderate),
            notes: None,
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_by: Uuid::new_v4(),
            updated_by: None,
        };

        let resolved_condition = MedicalHistory {
            id: Uuid::new_v4(),
            patient_id: Uuid::new_v4(),
            condition: "Appendicitis".to_string(),
            diagnosis_date: None,
            status: opengp_domain::domain::clinical::ConditionStatus::Resolved,
            severity: None,
            notes: None,
            is_active: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_by: Uuid::new_v4(),
            updated_by: None,
        };

        component.conditions = vec![active_condition, resolved_condition];

        assert_eq!(component.active_condition_count(), 1);
        assert_eq!(component.total_condition_count(), 2);
    }
}

use crate::ui::theme::Theme;
use opengp_domain::domain::clinical::Consultation;
#[cfg(feature = "prescription")]
use opengp_domain::domain::prescription::Prescription;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Borders, Widget};

pub struct ConsultationDetail {
    pub consultation: Option<Consultation>,
    #[cfg(feature = "prescription")]
    pub prescriptions: Vec<Prescription>,
    #[cfg(not(feature = "prescription"))]
    pub prescriptions: Vec<()>,
    pub is_editing: bool,
    pub signed: bool,
    pub theme: Theme,
}

#[derive(Debug, Clone)]
pub enum ConsultationDetailAction {
    Edit,
    Save,
    Sign,
    Cancel,
}

impl ConsultationDetail {
    pub fn new(theme: Theme) -> Self {
        Self {
            consultation: None,
            prescriptions: Vec::new(),
            is_editing: false,
            signed: false,
            theme,
        }
    }

    pub fn with_consultation(consultation: Consultation, theme: Theme) -> Self {
        let signed = consultation.signed_at.is_some();
        Self {
            consultation: Some(consultation),
            prescriptions: Vec::new(),
            is_editing: false,
            signed,
            theme,
        }
    }

    #[cfg(feature = "prescription")]
    pub fn set_prescriptions(&mut self, prescriptions: Vec<Prescription>) {
        self.prescriptions = prescriptions;
    }

    #[cfg(not(feature = "prescription"))]
    pub fn set_prescriptions(&mut self, prescriptions: Vec<()>) {
        self.prescriptions = prescriptions;
    }

    pub fn can_edit(&self) -> bool {
        !self.signed
    }

    pub fn start_editing(&mut self) {
        if self.can_edit() {
            self.is_editing = true;
        }
    }

    pub fn stop_editing(&mut self) {
        self.is_editing = false;
    }

    fn format_date(&self) -> String {
        if let Some(ref consultation) = self.consultation {
            consultation
                .consultation_date
                .format("%A %d %B %Y at %H:%M")
                .to_string()
        } else {
            "New Consultation".to_string()
        }
    }

    fn format_status(&self) -> String {
        if self.signed {
            if let Some(ref consultation) = self.consultation {
                if let Some(signed_at) = consultation.signed_at {
                    return format!("Signed at {}", signed_at.format("%H:%M on %d/%m/%Y"));
                }
            }
            "Signed".to_string()
        } else {
            "Draft".to_string()
        }
    }

    fn get_status_color(&self) -> ratatui::style::Color {
        if self.signed {
            self.theme.colors.success
        } else {
            self.theme.colors.warning
        }
    }
}

impl Widget for ConsultationDetail {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.is_empty() {
            return;
        }

        let block = Block::default()
            .title(if self.consultation.is_some() {
                " Consultation Detail "
            } else {
                " New Consultation "
            })
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.colors.border));

        block.clone().render(area, buf);

        let inner = block.inner(area);
        if inner.is_empty() {
            return;
        }

        let label_width = 18u16;
        let value_x = inner.x + label_width;
        let value_width = inner.width.saturating_sub(label_width + 2);

        let mut y = inner.y + 1;

        // Date
        if y < inner.y + inner.height - 4 {
            buf.set_string(
                inner.x + 1,
                y,
                "Date:",
                Style::default()
                    .fg(self.theme.colors.foreground)
                    .add_modifier(Modifier::BOLD),
            );
            let date_val = self.format_date();
            buf.set_string(
                value_x,
                y,
                date_val,
                Style::default().fg(self.theme.colors.foreground),
            );
            y += 1;
        }

        // Reason (optional)
        if let Some(ref consultation) = self.consultation {
            if let Some(ref reason) = consultation.reason {
                if !reason.is_empty() && y < inner.y + inner.height - 4 {
                    buf.set_string(
                        inner.x + 1,
                        y,
                        "Reason:",
                        Style::default()
                            .fg(self.theme.colors.foreground)
                            .add_modifier(Modifier::BOLD),
                    );
                    let display_reason = if reason.len() > value_width as usize {
                        format!("{}...", &reason[..value_width as usize - 3])
                    } else {
                        reason.clone()
                    };
                    buf.set_string(
                        value_x,
                        y,
                        display_reason,
                        Style::default().fg(self.theme.colors.foreground),
                    );
                    y += 1;
                }
            }

            // Practitioner ID (placeholder for now)
            if y < inner.y + inner.height - 4 {
                buf.set_string(
                    inner.x + 1,
                    y,
                    "Practitioner:",
                    Style::default()
                        .fg(self.theme.colors.foreground)
                        .add_modifier(Modifier::BOLD),
                );
                let practitioner_short = &consultation.practitioner_id.to_string()[..8];
                buf.set_string(
                    value_x,
                    y,
                    practitioner_short,
                    Style::default().fg(self.theme.colors.foreground),
                );
                y += 1;
            }
        }

        // Status with color
        if y < inner.y + inner.height - 4 {
            let status_color = self.get_status_color();
            buf.set_string(
                inner.x + 1,
                y,
                "Status:",
                Style::default()
                    .fg(self.theme.colors.foreground)
                    .add_modifier(Modifier::BOLD),
            );
            buf.set_string(
                value_x,
                y,
                self.format_status(),
                Style::default().fg(status_color),
            );
            y += 1;
        }

        // SOAP Notes Section
        y += 1;
        buf.set_string(
            inner.x + 1,
            y,
            " SOAP Notes ",
            Style::default()
                .fg(self.theme.colors.primary)
                .add_modifier(Modifier::BOLD),
        );
        y += 1;

        if let Some(ref consultation) = self.consultation {
            if let Some(ref notes) = consultation.clinical_notes {
                if y < inner.y + inner.height - 4 {
                    buf.set_string(
                        inner.x + 1,
                        y,
                        "  Clinical Notes:",
                        Style::default()
                            .fg(self.theme.colors.foreground)
                            .add_modifier(Modifier::BOLD),
                    );
                    y += 1;
                    let lines: Vec<&str> = notes.lines().collect();
                    for line in lines.iter().take(10) {
                        if y >= inner.y + inner.height - 4 {
                            break;
                        }
                        buf.set_string(
                            value_x,
                            y,
                            line,
                            Style::default().fg(self.theme.colors.foreground),
                        );
                        y += 1;
                    }
                }
            }
        } else {
            // Empty state for new consultation
            if y < inner.y + inner.height - 4 {
                buf.set_string(
                    inner.x + 1,
                    y,
                    "  Clinical Notes:",
                    Style::default()
                        .fg(self.theme.colors.foreground)
                        .add_modifier(Modifier::BOLD),
                );
                buf.set_string(
                    value_x,
                    y,
                    "-",
                    Style::default().fg(self.theme.colors.foreground),
                );
                y += 1;
            }
        }

        // Prescriptions Section
        #[cfg(feature = "prescription")]
        {
            y += 1;
            buf.set_string(
                inner.x + 1,
                y,
                " Prescriptions ",
                Style::default()
                    .fg(self.theme.colors.primary)
                    .add_modifier(Modifier::BOLD),
            );
            y += 1;

            if self.prescriptions.is_empty() {
                if y < inner.y + inner.height - 4 {
                    buf.set_string(
                        inner.x + 1,
                        y,
                        "  No prescriptions",
                        Style::default().fg(self.theme.colors.disabled),
                    );
                    y += 1;
                }
            } else {
                for prescription in &self.prescriptions {
                    if y >= inner.y + inner.height - 4 {
                        break;
                    }

                    // Medication name (prefer brand name if available)
                    let med_name = prescription
                        .medication
                        .brand_name
                        .as_ref()
                        .unwrap_or(&prescription.medication.generic_name);
                    buf.set_string(
                        inner.x + 1,
                        y,
                        "  ",
                        Style::default().fg(self.theme.colors.foreground),
                    );
                    buf.set_string(
                        inner.x + 3,
                        y,
                        med_name,
                        Style::default()
                            .fg(self.theme.colors.foreground)
                            .add_modifier(Modifier::BOLD),
                    );
                    y += 1;

                    // Dosage
                    if y < inner.y + inner.height - 4 {
                        let dosage_str = format!("    Dosage: {}", prescription.dosage);
                        buf.set_string(
                            inner.x + 1,
                            y,
                            dosage_str,
                            Style::default().fg(self.theme.colors.foreground),
                        );
                        y += 1;
                    }

                    // Frequency (from directions)
                    if y < inner.y + inner.height - 4 {
                        let freq_str = format!("    {}", prescription.directions);
                        let display_freq = if freq_str.len() > value_width as usize {
                            format!("{}...", &freq_str[..value_width as usize - 3])
                        } else {
                            freq_str
                        };
                        buf.set_string(
                            inner.x + 1,
                            y,
                            display_freq,
                            Style::default().fg(self.theme.colors.foreground),
                        );
                        y += 1;
                    }

                    // Quantity
                    if y < inner.y + inner.height - 4 {
                        let qty_str = format!("    Quantity: {}", prescription.quantity);
                        buf.set_string(
                            inner.x + 1,
                            y,
                            qty_str,
                            Style::default().fg(self.theme.colors.foreground),
                        );
                        y += 1;
                    }

                    // Add spacing between prescriptions
                    if y < inner.y + inner.height - 4 {
                        y += 1;
                    }
                }
            }
        }

        // Action buttons at bottom
        y += 1;

        let mut buttons: Vec<(&str, bool)> = Vec::new();

        // Edit button - only if not signed
        if !self.signed {
            buttons.push(("[E]dit", false));
        }

        // Sign button - only if not signed
        if !self.signed {
            buttons.push(("[S]ign", false));
        }

        // Back button
        buttons.push(("[Esc] Back", false));

        if !buttons.is_empty() {
            let button_width = 12u16;
            let spacing = 2u16;
            let total_buttons_width = button_width * buttons.len() as u16
                + spacing * (buttons.len().saturating_sub(1)) as u16;
            let button_start_x = inner.x + (inner.width.saturating_sub(total_buttons_width)) / 2;

            let mut current_x = button_start_x;
            for (label, _is_focused) in &buttons {
                let style = Style::default().fg(self.theme.colors.foreground);
                buf.set_string(current_x, y, label, style);
                current_x += button_width + spacing;
            }
        }

        // Help text
        y += 1;
        let help_text = if self.signed {
            "Esc: Back"
        } else {
            "E: Edit | S: Sign | Esc: Back"
        };
        buf.set_string(
            inner.x + 1,
            y,
            help_text,
            Style::default().fg(self.theme.colors.disabled),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    fn create_test_consultation() -> Consultation {
        Consultation {
            id: Uuid::new_v4(),
            patient_id: Uuid::new_v4(),
            practitioner_id: Uuid::new_v4(),
            appointment_id: Some(Uuid::new_v4()),
            consultation_date: Utc::now(),
            reason: Some("Annual checkup".to_string()),
            clinical_notes: Some("Patient in good health".to_string()),
            is_signed: false,
            signed_at: None,
            signed_by: None,
            consultation_started_at: None,
            consultation_ended_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            version: 1,
            created_by: Uuid::new_v4(),
            updated_by: None,
        }
    }

    #[test]
    fn test_new_creates_empty_detail() {
        let theme = Theme::dark();
        let detail = ConsultationDetail::new(theme.clone());

        assert!(detail.consultation.is_none());
        assert!(detail.prescriptions.is_empty());
        assert!(!detail.is_editing);
        assert!(!detail.signed);
    }

    #[test]
    fn test_with_consultation_sets_signed_from_domain() {
        let theme = Theme::dark();
        let mut consultation = create_test_consultation();
        consultation.signed_at = Some(Utc::now());

        let detail = ConsultationDetail::with_consultation(consultation.clone(), theme);

        assert!(detail.consultation.is_some());
        assert!(detail.signed);
        assert!(!detail.is_editing);
    }

    #[test]
    fn test_with_consultation_unsigned_sets_signed_false() {
        let theme = Theme::dark();
        let consultation = create_test_consultation();

        let detail = ConsultationDetail::with_consultation(consultation, theme);

        assert!(!detail.signed);
    }

    #[test]
    fn test_can_edit_returns_false_when_signed() {
        let theme = Theme::dark();
        let mut consultation = create_test_consultation();
        consultation.signed_at = Some(Utc::now());
        let detail = ConsultationDetail::with_consultation(consultation, theme);

        assert!(!detail.can_edit());
    }

    #[test]
    fn test_can_edit_returns_true_when_not_signed() {
        let theme = Theme::dark();
        let consultation = create_test_consultation();
        let detail = ConsultationDetail::with_consultation(consultation, theme);

        assert!(detail.can_edit());
    }

    #[test]
    fn test_start_editing_sets_flag_when_can_edit() {
        let theme = Theme::dark();
        let consultation = create_test_consultation();
        let mut detail = ConsultationDetail::with_consultation(consultation, theme);

        detail.start_editing();

        assert!(detail.is_editing);
    }

    #[test]
    fn test_start_editing_does_not_set_flag_when_signed() {
        let theme = Theme::dark();
        let mut consultation = create_test_consultation();
        consultation.signed_at = Some(Utc::now());
        let mut detail = ConsultationDetail::with_consultation(consultation, theme);

        detail.start_editing();

        assert!(!detail.is_editing);
    }

    #[test]
    fn test_stop_editing_clears_flag() {
        let theme = Theme::dark();
        let consultation = create_test_consultation();
        let mut detail = ConsultationDetail::with_consultation(consultation, theme);
        detail.is_editing = true;

        detail.stop_editing();

        assert!(!detail.is_editing);
    }

    #[test]
    fn test_format_date_with_consultation() {
        let theme = Theme::dark();
        let consultation = create_test_consultation();
        let detail = ConsultationDetail::with_consultation(consultation, theme);

        let formatted = detail.format_date();

        assert!(!formatted.is_empty());
        assert!(formatted.contains("at"));
    }

    #[test]
    fn test_format_date_without_consultation() {
        let theme = Theme::dark();
        let detail = ConsultationDetail::new(theme);

        let formatted = detail.format_date();

        assert_eq!(formatted, "New Consultation");
    }

    #[test]
    fn test_format_status_draft_when_not_signed() {
        let theme = Theme::dark();
        let consultation = create_test_consultation();
        let detail = ConsultationDetail::with_consultation(consultation, theme);

        let status = detail.format_status();

        assert_eq!(status, "Draft");
    }

    #[test]
    fn test_format_status_signed_when_signed() {
        let theme = Theme::dark();
        let mut consultation = create_test_consultation();
        consultation.signed_at = Some(Utc::now());
        let detail = ConsultationDetail::with_consultation(consultation, theme);

        let status = detail.format_status();

        assert!(status.contains("Signed"));
    }

    #[test]
    fn test_get_status_color_success_when_signed() {
        let theme = Theme::dark();
        let mut consultation = create_test_consultation();
        consultation.signed_at = Some(Utc::now());
        let detail = ConsultationDetail::with_consultation(consultation, theme.clone());

        let color = detail.get_status_color();

        assert_eq!(color, theme.colors.success);
    }

    #[test]
    fn test_get_status_color_warning_when_not_signed() {
        let theme = Theme::dark();
        let consultation = create_test_consultation();
        let detail = ConsultationDetail::with_consultation(consultation, theme.clone());

        let color = detail.get_status_color();

        assert_eq!(color, theme.colors.warning);
    }

    #[test]
    fn test_consultation_detail_action_enum_variants() {
        let _edit = ConsultationDetailAction::Edit;
        let _save = ConsultationDetailAction::Save;
        let _sign = ConsultationDetailAction::Sign;
        let _cancel = ConsultationDetailAction::Cancel;
    }

    #[test]
    fn test_set_prescriptions_stores_empty_vec() {
        let theme = Theme::dark();
        let consultation = create_test_consultation();
        let mut detail = ConsultationDetail::with_consultation(consultation, theme);

        detail.set_prescriptions(Vec::new());

        assert!(detail.prescriptions.is_empty());
    }

    #[test]
    fn test_state_transitions_edit_flow() {
        let theme = Theme::dark();
        let consultation = create_test_consultation();
        let mut detail = ConsultationDetail::with_consultation(consultation, theme);

        assert!(!detail.is_editing);
        detail.start_editing();
        assert!(detail.is_editing);
        detail.stop_editing();
        assert!(!detail.is_editing);
    }

    #[test]
    fn test_signed_consultation_prevents_editing() {
        let theme = Theme::dark();
        let mut consultation = create_test_consultation();
        consultation.signed_at = Some(Utc::now());
        let mut detail = ConsultationDetail::with_consultation(consultation, theme);

        assert!(!detail.can_edit());
        detail.start_editing();
        assert!(!detail.is_editing);
    }
}

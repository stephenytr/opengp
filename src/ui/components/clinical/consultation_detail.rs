use crate::domain::clinical::Consultation;
use crate::domain::prescription::Prescription;
use crate::ui::theme::Theme;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Borders, Widget};

pub struct ConsultationDetail {
    pub consultation: Option<Consultation>,
    pub prescriptions: Vec<Prescription>,
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

    pub fn set_prescriptions(&mut self, prescriptions: Vec<Prescription>) {
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

    fn format_soap_field(value: &Option<String>) -> String {
        value
            .as_ref()
            .filter(|s| !s.is_empty())
            .cloned()
            .unwrap_or_else(|| "-".to_string())
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

        // Render SOAP notes if we have a consultation
        if let Some(ref consultation) = self.consultation {
            let soap = &consultation.soap_notes;

            // Subjective
            if y < inner.y + inner.height - 4 {
                let subjective = Self::format_soap_field(&soap.subjective);
                buf.set_string(
                    inner.x + 1,
                    y,
                    "  Subjective:",
                    Style::default()
                        .fg(self.theme.colors.foreground)
                        .add_modifier(Modifier::BOLD),
                );
                buf.set_string(
                    value_x,
                    y,
                    subjective,
                    Style::default().fg(self.theme.colors.foreground),
                );
                y += 1;
            }

            // Objective
            if y < inner.y + inner.height - 4 {
                let objective = Self::format_soap_field(&soap.objective);
                buf.set_string(
                    inner.x + 1,
                    y,
                    "  Objective:",
                    Style::default()
                        .fg(self.theme.colors.foreground)
                        .add_modifier(Modifier::BOLD),
                );
                buf.set_string(
                    value_x,
                    y,
                    objective,
                    Style::default().fg(self.theme.colors.foreground),
                );
                y += 1;
            }

            // Assessment
            if y < inner.y + inner.height - 4 {
                let assessment = Self::format_soap_field(&soap.assessment);
                buf.set_string(
                    inner.x + 1,
                    y,
                    "  Assessment:",
                    Style::default()
                        .fg(self.theme.colors.foreground)
                        .add_modifier(Modifier::BOLD),
                );
                buf.set_string(
                    value_x,
                    y,
                    assessment,
                    Style::default().fg(self.theme.colors.foreground),
                );
                y += 1;
            }

            // Plan
            if y < inner.y + inner.height - 4 {
                let plan = Self::format_soap_field(&soap.plan);
                buf.set_string(
                    inner.x + 1,
                    y,
                    "  Plan:",
                    Style::default()
                        .fg(self.theme.colors.foreground)
                        .add_modifier(Modifier::BOLD),
                );
                buf.set_string(
                    value_x,
                    y,
                    plan,
                    Style::default().fg(self.theme.colors.foreground),
                );
                y += 1;
            }
        } else {
            // Empty state for new consultation
            if y < inner.y + inner.height - 4 {
                buf.set_string(
                    inner.x + 1,
                    y,
                    "  Subjective:",
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
            if y < inner.y + inner.height - 4 {
                buf.set_string(
                    inner.x + 1,
                    y,
                    "  Objective:",
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
            if y < inner.y + inner.height - 4 {
                buf.set_string(
                    inner.x + 1,
                    y,
                    "  Assessment:",
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
            if y < inner.y + inner.height - 4 {
                buf.set_string(
                    inner.x + 1,
                    y,
                    "  Plan:",
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

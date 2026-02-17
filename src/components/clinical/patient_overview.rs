use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::components::clinical::component::ClinicalComponent;
use crate::domain::clinical::Severity;

pub fn render_patient_overview(component: &mut ClinicalComponent, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Clinical - Patient Overview ");

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let Some(ref _patient) = component.current_patient else {
        let msg = Paragraph::new(Span::raw("No patient selected. Press '/' to search."))
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(msg, inner);
        return;
    };

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(inner);

    let left_chunk = chunks[0];
    let right_chunk = chunks[1];

    render_patient_summary(component, frame, left_chunk);
    render_clinical_options(component, frame, right_chunk);
}

fn render_patient_summary(component: &mut ClinicalComponent, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Patient Summary ");

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let Some(ref patient) = component.current_patient else {
        return;
    };

    let today = chrono::Utc::now().date_naive();
    let age = today
        .signed_duration_since(patient.date_of_birth)
        .num_days()
        / 365;

    let mut lines = Vec::new();

    lines.push(Line::from(vec![
        Span::raw("Name: "),
        Span::raw(&patient.first_name),
        Span::raw(" "),
        Span::raw(&patient.last_name),
    ]));

    lines.push(Line::from(vec![
        Span::raw("DOB: "),
        Span::raw(patient.date_of_birth.format("%d/%m/%Y").to_string()),
        Span::raw(" ("),
        Span::raw(format!("{}y", age)),
        Span::raw(")"),
    ]));

    if let Some(ref medicare) = patient.medicare_number {
        lines.push(Line::from(vec![
            Span::raw("Medicare: "),
            Span::raw(medicare),
        ]));
    }

    if let Some(ref ihi) = patient.ihi {
        lines.push(Line::from(vec![Span::raw("IHI: "), Span::raw(ihi)]));
    }

    lines.push(Line::from(Span::raw("")));
    lines.push(Line::from(vec![
        Span::raw("⚠️ ALLERGIES: "),
        Span::styled("Press F4", Style::default().fg(Color::DarkGray)),
    ]));

    for allergy in &component.allergies {
        let severity_color = match allergy.severity {
            Severity::Mild => Color::Green,
            Severity::Moderate => Color::Yellow,
            Severity::Severe => Color::Red,
        };
        lines.push(Line::from(vec![
            Span::raw("  • "),
            Span::raw(&allergy.allergen),
            Span::raw(" ("),
            Span::styled(
                format!("{:?}", allergy.severity),
                Style::default().fg(severity_color),
            ),
            Span::raw(")"),
        ]));
    }

    if component.allergies.is_empty() {
        lines.push(Line::from(vec![Span::raw("  No known allergies")]));
    }

    lines.push(Line::from(Span::raw("")));
    lines.push(Line::from(vec![
        Span::raw("ACTIVE CONDITIONS: "),
        Span::styled("Press F5", Style::default().fg(Color::DarkGray)),
    ]));

    let active_conditions: Vec<_> = component
        .medical_history
        .iter()
        .filter(|h| h.is_active)
        .collect();
    for history in &active_conditions {
        lines.push(Line::from(vec![
            Span::raw("  • "),
            Span::raw(&history.condition),
            Span::raw(" ("),
            Span::raw(format!("{:?}", history.status)),
            Span::raw(")"),
        ]));
    }

    if active_conditions.is_empty() {
        lines.push(Line::from(vec![Span::raw("  No active conditions")]));
    }

    let paragraph = Paragraph::new(lines).block(Block::default());
    frame.render_widget(paragraph, inner);
}

fn render_clinical_options(component: &mut ClinicalComponent, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Clinical Options ");

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines = Vec::new();

    lines.push(Line::from(Span::styled(
        " QUICK ACTIONS ",
        Style::default().add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(Span::raw("")));
    lines.push(Line::from(vec![
        Span::styled(
            "  [F2] ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("Consultations - View/Edit clinical notes"),
    ]));
    lines.push(Line::from(vec![
        Span::styled(
            "  [F2] ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("Vital Signs - BP, HR, temperature, weight"),
    ]));
    lines.push(Line::from(vec![
        Span::styled(
            "  [F3] ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("Consultations - View/Edit clinical notes"),
    ]));
    lines.push(Line::from(vec![
        Span::styled(
            "  [F4] ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("Allergies - Manage patient allergies"),
    ]));
    lines.push(Line::from(vec![
        Span::styled(
            "  [F5] ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("Medical History - Conditions & diagnoses"),
    ]));
    lines.push(Line::from(vec![
        Span::styled(
            "  [f]  ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("Family History - Hereditary conditions"),
    ]));
    lines.push(Line::from(vec![
        Span::styled(
            "  [s]  ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("Social History - Smoking, alcohol, lifestyle"),
    ]));

    lines.push(Line::from(Span::raw("")));
    lines.push(Line::from(Span::styled(
        " RECENT CONSULTATIONS ",
        Style::default().add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(Span::raw("")));

    if component.consultations.is_empty() {
        lines.push(Line::from(vec![Span::raw("  No consultations yet")]));
    } else {
        lines.push(Line::from(vec![
            Span::styled(
                " Date       ",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::styled(" Status   ", Style::default().add_modifier(Modifier::BOLD)),
        ]));

        for consultation in component.consultations.iter().take(5) {
            let status = if consultation.is_signed {
                Span::styled("Signed", Style::default().fg(Color::Green))
            } else {
                Span::styled("Draft", Style::default().fg(Color::Yellow))
            };

            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::raw(
                    consultation
                        .consultation_date
                        .format("%d/%m/%Y")
                        .to_string(),
                ),
                Span::raw("  "),
                status,
            ]));
        }
    }

    lines.push(Line::from(Span::raw("")));
    lines.push(Line::from(Span::styled(
        " VITAL SIGNS ",
        Style::default().add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(Span::raw("")));

    if let Some(ref vitals) = component.latest_vitals {
        lines.push(Line::from(vec![
            Span::raw("  Recorded: "),
            Span::raw(vitals.measured_at.format("%d/%m/%Y %H:%M").to_string()),
        ]));

        let mut vitals_str = Vec::new();
        if let Some(bp) = vitals.blood_pressure_string() {
            vitals_str.push(format!("BP: {}", bp));
        }
        if let Some(hr) = vitals.heart_rate {
            vitals_str.push(format!("HR: {}bpm", hr));
        }
        if let Some(temp) = vitals.temperature {
            vitals_str.push(format!("Temp: {:.1}°C", temp));
        }
        if let Some(rr) = vitals.respiratory_rate {
            vitals_str.push(format!("RR: {}/min", rr));
        }
        if let Some(bmi) = vitals.bmi {
            vitals_str.push(format!("BMI: {:.1}", bmi));
        }

        if !vitals_str.is_empty() {
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::raw(vitals_str.join("  |  ")),
            ]));
        }
    } else {
        lines.push(Line::from(vec![
            Span::raw("  No vital signs recorded"),
            Span::raw(" "),
            Span::styled("[v] to add", Style::default().fg(Color::DarkGray)),
        ]));
    }

    lines.push(Line::from(Span::raw("")));
    lines.push(Line::from(Span::styled(
        " FAMILY HISTORY ",
        Style::default().add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(Span::raw("")));

    if component.family_history.is_empty() {
        lines.push(Line::from(vec![
            Span::raw("  No family history recorded"),
            Span::raw(" "),
            Span::styled("[f] to add", Style::default().fg(Color::DarkGray)),
        ]));
    } else {
        for fh in component.family_history.iter().take(3) {
            lines.push(Line::from(vec![
                Span::raw("  • "),
                Span::raw(&fh.condition),
                Span::raw(" ("),
                Span::raw(format!("{:?}", fh.relative_relationship)),
                Span::raw(")"),
            ]));
        }
    }

    lines.push(Line::from(Span::raw("")));
    lines.push(Line::from(Span::styled(
        " SOCIAL HISTORY ",
        Style::default().add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(Span::raw("")));

    if let Some(ref sh) = component.social_history {
        lines.push(Line::from(vec![
            Span::raw("  Smoking: "),
            Span::raw(format!("{:?}", sh.smoking_status)),
        ]));
        lines.push(Line::from(vec![
            Span::raw("  Alcohol: "),
            Span::raw(format!("{:?}", sh.alcohol_status)),
        ]));
        lines.push(Line::from(vec![
            Span::raw("  No social history recorded"),
            Span::raw(" "),
            Span::styled("[s] to add", Style::default().fg(Color::DarkGray)),
        ]));
    }

    let paragraph = Paragraph::new(lines).block(Block::default());
    frame.render_widget(paragraph, inner);
}

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
    render_recent_consultations(component, frame, right_chunk);
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
        Span::styled("Press F4 to manage", Style::default().fg(Color::DarkGray)),
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

    lines.push(Line::from(Span::raw("")));
    lines.push(Line::from(vec![
        Span::raw("ACTIVE CONDITIONS: "),
        Span::styled("Press F5 to manage", Style::default().fg(Color::DarkGray)),
    ]));

    for history in &component.medical_history {
        if history.is_active {
            lines.push(Line::from(vec![
                Span::raw("  • "),
                Span::raw(&history.condition),
                Span::raw(" ("),
                Span::raw(format!("{:?}", history.status)),
                Span::raw(")"),
            ]));
        }
    }

    if let Some(ref vitals) = component.latest_vitals {
        lines.push(Line::from(Span::raw("")));
        lines.push(Line::from(vec![
            Span::raw("LATEST VITALS: "),
            Span::raw(vitals.measured_at.format("%d/%m/%Y").to_string()),
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
        if let Some(bmi) = vitals.bmi {
            vitals_str.push(format!("BMI: {:.1}", bmi));
        }

        if !vitals_str.is_empty() {
            lines.push(Line::from(Span::raw(vitals_str.join("  "))));
        }
    }

    let paragraph = Paragraph::new(lines).block(Block::default());
    frame.render_widget(paragraph, inner);
}

fn render_recent_consultations(component: &mut ClinicalComponent, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Recent Consultations ");

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if component.consultations.is_empty() {
        let msg = Paragraph::new(Span::raw("No consultations yet."))
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(msg, inner);
        return;
    }

    let mut lines = Vec::new();
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

    let paragraph = Paragraph::new(lines).block(Block::default());
    frame.render_widget(paragraph, inner);
}

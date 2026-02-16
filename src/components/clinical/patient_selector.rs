use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::components::clinical::component::ClinicalComponent;

pub fn render_patient_selector(component: &mut ClinicalComponent, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Clinical - Select Patient ");

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let search_text = if component.patient_search.query.is_empty() {
        "Press '/' to search patients..."
    } else {
        &component.patient_search.query
    };

    let search_paragraph = Paragraph::new(Span::raw(search_text))
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL).title(" Search "));

    frame.render_widget(search_paragraph, inner);

    if component.patient_search.is_open && !component.patient_search.results.is_empty() {
        let results_area = Rect::new(
            inner.x,
            inner.y + 3,
            inner.width,
            inner.height.saturating_sub(3),
        );

        let mut lines = Vec::new();
        for (i, patient) in component.patient_search.results.iter().enumerate() {
            let prefix = if i == component.patient_search.selected_index {
                " > "
            } else {
                "   "
            };
            let line = format!(
                "{}{} {} (DOB: {})",
                prefix,
                patient.first_name,
                patient.last_name,
                patient.date_of_birth.format("%d/%m/%Y")
            );
            lines.push(Line::from(Span::raw(line)));
        }

        let results_list =
            Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title(" Results "));

        frame.render_widget(results_list, results_area);
    }

    let help_text = Paragraph::new(Span::raw(
        "/ - Search  ↑↓ - Navigate  Enter - Select  Esc - Close  ? - Help",
    ))
    .style(Style::default().fg(Color::DarkGray));

    let help_area = Rect::new(
        area.x,
        area.y + area.height.saturating_sub(1),
        area.width,
        1,
    );
    frame.render_widget(help_text, help_area);
}

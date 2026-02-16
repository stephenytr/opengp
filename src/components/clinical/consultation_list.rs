use ratatui::layout::Rect;
use ratatui::text::Span;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::components::clinical::component::ClinicalComponent;

pub fn render_consultation_list(component: &mut ClinicalComponent, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Clinical - Consultations ");

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if component.consultations.is_empty() {
        let msg = Paragraph::new(Span::raw("No consultations yet. Press F1 to create one."))
            .block(Block::default());
        frame.render_widget(msg, inner);
        return;
    }

    let text = Paragraph::new(Span::raw(
        "Consultation list - use arrow keys to navigate, Enter to view",
    ))
    .block(Block::default());
    frame.render_widget(text, inner);
}

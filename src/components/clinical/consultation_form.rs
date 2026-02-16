use ratatui::layout::Rect;
use ratatui::text::Span;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;
use uuid::Uuid;

use crate::components::clinical::component::ClinicalComponent;

pub fn render_consultation_form(
    _component: &mut ClinicalComponent,
    frame: &mut Frame,
    area: Rect,
    _consultation_id: Uuid,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Clinical - SOAP Notes ");

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let text = Paragraph::new(Span::raw(
        "SOAP Editor - Enter key to edit, Tab to navigate sections",
    ))
    .block(Block::default());
    frame.render_widget(text, inner);
}

use ratatui::layout::Rect;
use ratatui::text::Span;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::components::clinical::component::ClinicalComponent;

pub fn render_vital_signs_form(_component: &mut ClinicalComponent, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Clinical - Record Vital Signs ");

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let text = Paragraph::new(Span::raw("Vital signs form - fill in the measurements"))
        .block(Block::default());
    frame.render_widget(text, inner);
}

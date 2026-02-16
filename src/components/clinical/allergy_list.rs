use ratatui::layout::Rect;
use ratatui::text::Span;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::components::clinical::component::ClinicalComponent;

pub fn render_allergy_list(component: &mut ClinicalComponent, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Clinical - Allergies ");

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if component.allergies.is_empty() {
        let msg = Paragraph::new(Span::raw("No allergies recorded. Press 'a' to add one."))
            .block(Block::default());
        frame.render_widget(msg, inner);
        return;
    }

    let text = Paragraph::new(Span::raw("Allergy list - use arrow keys to navigate"))
        .block(Block::default());
    frame.render_widget(text, inner);
}

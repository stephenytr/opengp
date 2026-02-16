use ratatui::layout::Rect;
use ratatui::text::Span;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;
use uuid::Uuid;

use crate::components::clinical::component::ClinicalComponent;

pub fn render_allergy_form(
    _component: &mut ClinicalComponent,
    frame: &mut Frame,
    area: Rect,
    _allergy_id: Option<Uuid>,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Clinical - Add/Edit Allergy ");

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let text =
        Paragraph::new(Span::raw("Allergy form - fill in the details")).block(Block::default());
    frame.render_widget(text, inner);
}

use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;
use uuid::Uuid;

use crate::components::clinical::component::ClinicalComponent;
use crate::components::clinical::state::AllergyFormField;

pub fn render_allergy_form(
    component: &mut ClinicalComponent,
    frame: &mut Frame,
    area: Rect,
    _allergy_id: Option<Uuid>,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Clinical - Add/Edit Allergy ");

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.width < 40 || inner.height < 15 {
        let text = Paragraph::new(Span::raw("Terminal too small for allergy form"))
            .block(Block::default());
        frame.render_widget(text, inner);
        return;
    }

    if component.allergy_form.patient_id.is_none() {
        let text = Paragraph::new(Span::raw("No patient selected")).block(Block::default());
        frame.render_widget(text, inner);
        return;
    }

    // Form layout - split into two columns
    let left_width = inner.width.saturating_sub(4) / 2;
    let right_width = inner.width.saturating_sub(4) - left_width;

    // Row 1: Allergen (left) | Allergy Type (right)
    let allergen_area = Rect::new(inner.x + 1, inner.y + 1, left_width, 3);
    let allergy_type_area = Rect::new(inner.x + 1 + left_width + 2, inner.y + 1, right_width, 3);

    // Row 2: Severity (left) | Reaction (right)
    let severity_area = Rect::new(inner.x + 1, inner.y + 5, left_width, 3);
    let reaction_area = Rect::new(inner.x + 1 + left_width + 2, inner.y + 5, right_width, 3);

    // Row 3: Onset Date (left) | Notes (right spans full width)
    let onset_date_area = Rect::new(inner.x + 1, inner.y + 9, left_width, 3);
    let notes_area = Rect::new(inner.x + 1, inner.y + 13, inner.width.saturating_sub(2), 5);

    let active_field = component.allergy_form.active_field;

    // Render field helpers
    render_form_field(
        frame,
        allergen_area,
        "Allergen",
        &component.allergy_form.allergen,
        "Required - e.g., Penicillin, Peanuts",
        active_field == AllergyFormField::Allergen,
    );

    render_form_field(
        frame,
        allergy_type_area,
        "Allergy Type",
        &component.allergy_form.allergy_type,
        "drug, food, environmental, other",
        active_field == AllergyFormField::AllergyType,
    );

    render_form_field(
        frame,
        severity_area,
        "Severity",
        &component.allergy_form.severity,
        "mild, moderate, severe",
        active_field == AllergyFormField::Severity,
    );

    render_form_field(
        frame,
        reaction_area,
        "Reaction",
        &component.allergy_form.reaction,
        "e.g., rash, anaphylaxis",
        active_field == AllergyFormField::Reaction,
    );

    render_form_field(
        frame,
        onset_date_area,
        "Onset Date",
        &component.allergy_form.onset_date,
        "dd/mm/yyyy (optional)",
        active_field == AllergyFormField::OnsetDate,
    );

    render_text_area(
        frame,
        notes_area,
        "Notes",
        &component.allergy_form.notes,
        active_field == AllergyFormField::Notes,
    );

    if let Some(ref error) = component.error_message {
        let error_area = Rect::new(
            inner.x + 1,
            inner.y + inner.height.saturating_sub(4),
            inner.width.saturating_sub(2),
            2,
        );
        let error_text = Paragraph::new(Span::raw(error)).style(Style::default().fg(Color::Red));
        frame.render_widget(error_text, error_area);
    }

    let help_text = Paragraph::new(Span::raw(
        "Tab/Arrow Keys - Navigate  Enter - Save  Esc - Cancel",
    ))
    .style(Style::default().fg(Color::DarkGray));

    let help_area = Rect::new(
        inner.x,
        inner.y + inner.height.saturating_sub(1),
        inner.width,
        1,
    );
    frame.render_widget(help_text, help_area);
}

fn render_form_field(
    frame: &mut Frame,
    area: Rect,
    label: &str,
    value: &str,
    hint: &str,
    is_selected: bool,
) {
    let (border_color, label_style) = if is_selected {
        (
            Color::Green,
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        (Color::Blue, Style::default().fg(Color::Cyan))
    };

    let label_block = Block::default()
        .borders(Borders::NONE)
        .title(Line::from(Span::styled(label, label_style)));

    let input_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    let input_area = Rect::new(
        area.x,
        area.y + 1,
        area.width,
        area.height.saturating_sub(1),
    );
    frame.render_widget(label_block, area);
    frame.render_widget(input_block, input_area);

    let display_text = if value.is_empty() {
        Span::raw(hint)
    } else {
        Span::raw(value)
    };
    let input_text = Paragraph::new(display_text).style(if value.is_empty() {
        Style::default().fg(Color::DarkGray)
    } else {
        Style::default()
    });
    frame.render_widget(input_text, input_area);
}

fn render_text_area(frame: &mut Frame, area: Rect, label: &str, value: &str, is_selected: bool) {
    let (border_color, label_style) = if is_selected {
        (
            Color::Green,
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        (Color::Blue, Style::default().fg(Color::Cyan))
    };

    let label_block = Block::default()
        .borders(Borders::NONE)
        .title(Line::from(Span::styled(label, label_style)));

    let input_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    let input_area = Rect::new(
        area.x,
        area.y + 1,
        area.width,
        area.height.saturating_sub(1),
    );
    frame.render_widget(label_block, area);
    frame.render_widget(input_block, input_area);

    let display_text = if value.is_empty() {
        Span::raw("Enter notes...")
    } else {
        Span::raw(value)
    };
    let input_text = Paragraph::new(display_text)
        .wrap(Wrap { trim: true })
        .style(if value.is_empty() {
            Style::default().fg(Color::DarkGray)
        } else {
            Style::default()
        });
    frame.render_widget(input_text, input_area);
}

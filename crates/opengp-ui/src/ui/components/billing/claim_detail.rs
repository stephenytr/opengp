use opengp_domain::domain::billing::{ClaimStatus, MedicareClaim};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Row, Table, Widget, Wrap};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClaimDetailAction {
    ExportJson,
    MarkSubmitted,
    MarkPaid,
    Back,
}

#[derive(Debug, Clone, Default)]
pub struct ClaimDetail {
    pub claim: Option<MedicareClaim>,
    pub scroll_offset: usize,
}

impl ClaimDetail {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_claim(&mut self, claim: MedicareClaim) {
        self.claim = Some(claim);
    }

    pub fn available_actions(&self) -> Vec<ClaimDetailAction> {
        let mut actions = vec![ClaimDetailAction::ExportJson];

        if let Some(claim) = &self.claim {
            if claim.status == ClaimStatus::Draft {
                actions.push(ClaimDetailAction::MarkSubmitted);
            }

            if claim.status == ClaimStatus::Submitted {
                actions.push(ClaimDetailAction::MarkPaid);
            }
        }

        actions.push(ClaimDetailAction::Back);
        actions
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        if area.is_empty() {
            return;
        }

        let root = Block::default()
            .title(" Claim Detail ")
            .borders(Borders::ALL);
        root.clone().render(area, buf);
        let inner = root.inner(area);
        if inner.is_empty() {
            return;
        }

        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(9),
                Constraint::Min(5),
                Constraint::Length(5),
                Constraint::Length(3),
            ])
            .split(inner);

        self.render_header(rows[0], buf);
        self.render_items(rows[1], buf);
        self.render_amounts(rows[2], buf);
        self.render_actions(rows[3], buf);
    }

    fn render_header(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::default().title(" Summary ").borders(Borders::ALL);
        block.clone().render(area, buf);
        let inner = block.inner(area);
        if inner.is_empty() {
            return;
        }

        let lines = if let Some(claim) = &self.claim {
            vec![
                Line::from(vec![
                    Span::styled("Reference: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(
                        claim
                            .claim_reference
                            .clone()
                            .unwrap_or_else(|| "(not assigned)".to_string()),
                    ),
                    Span::raw("   "),
                    Span::styled("Status: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::styled(
                        claim.status.to_string(),
                        Style::default().fg(status_color(claim.status)),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("Patient: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(claim.patient_id.to_string()),
                ]),
                Line::from(vec![
                    Span::styled(
                        "Practitioner: ",
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(claim.practitioner_id.to_string()),
                ]),
                Line::from(vec![
                    Span::styled(
                        "Service Date: ",
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(claim.service_date.format("%d/%m/%Y").to_string()),
                    Span::raw("   "),
                    Span::styled(
                        "Claim Type: ",
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(claim.claim_type.to_string()),
                ]),
            ]
        } else {
            vec![Line::from("No claim selected")]
        };

        Paragraph::new(lines)
            .wrap(Wrap { trim: true })
            .render(inner, buf);
    }

    fn render_items(&self, area: Rect, buf: &mut Buffer) {
        let header = Row::new(vec!["MBS Item", "Description", "Qty", "Fee", "Benefit"])
            .style(Style::default().add_modifier(Modifier::BOLD));

        let widths = [
            Constraint::Length(10),
            Constraint::Percentage(50),
            Constraint::Length(6),
            Constraint::Length(12),
            Constraint::Length(12),
        ];

        let rows: Vec<Row> = if let Some(claim) = &self.claim {
            let visible_height = area.height.saturating_sub(2) as usize;
            let start = self
                .scroll_offset
                .min(claim.items.len().saturating_sub(visible_height.max(1)));

            claim
                .items
                .iter()
                .skip(start)
                .take(visible_height)
                .map(|item| {
                    Row::new(vec![
                        item.item_number.clone(),
                        item.description.clone(),
                        item.quantity.to_string(),
                        format!("${:.2}", item.fee),
                        format!("${:.2}", item.benefit),
                    ])
                })
                .collect()
        } else {
            vec![Row::new(vec!["-", "-", "-", "-", "-"])]
        };

        Table::new(rows, widths)
            .header(header)
            .block(Block::default().title(" MBS Items ").borders(Borders::ALL))
            .render(area, buf);
    }

    fn render_amounts(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::default().title(" Amounts ").borders(Borders::ALL);
        block.clone().render(area, buf);
        let inner = block.inner(area);
        if inner.is_empty() {
            return;
        }

        let lines = if let Some(claim) = &self.claim {
            vec![
                Line::from(format!("Total Claimed: ${:.2}", claim.total_claimed)),
                Line::from(format!("Total Benefit: ${:.2}", claim.total_benefit)),
                Line::from(format!(
                    "Patient Contribution: ${:.2}",
                    claim.patient_contribution
                )),
            ]
        } else {
            vec![Line::from("No amount data")]
        };

        Paragraph::new(lines)
            .wrap(Wrap { trim: true })
            .render(inner, buf);
    }

    fn render_actions(&self, area: Rect, buf: &mut Buffer) {
        let mut labels = Vec::new();
        for action in self.available_actions() {
            let label = match action {
                ClaimDetailAction::ExportJson => "[X] Export JSON",
                ClaimDetailAction::MarkSubmitted => "[S] Mark Submitted",
                ClaimDetailAction::MarkPaid => "[P] Mark Paid",
                ClaimDetailAction::Back => "[B] Back",
            };
            labels.push(label);
        }

        let guidance = "JSON export: use 'x' to copy to clipboard";
        let bulk_billing_stub = Span::styled(
            "Bulk Billing (coming soon)",
            Style::default().fg(Color::DarkGray),
        );

        let lines = vec![
            Line::from(labels.join("  ")),
            Line::from(guidance),
            Line::from(vec![bulk_billing_stub]),
        ];

        Paragraph::new(lines)
            .wrap(Wrap { trim: true })
            .render(area, buf);
    }
}

fn status_color(status: ClaimStatus) -> Color {
    match status {
        ClaimStatus::Draft => Color::Yellow,
        ClaimStatus::Submitted => Color::Blue,
        ClaimStatus::Processing => Color::Cyan,
        ClaimStatus::Paid => Color::Green,
        ClaimStatus::Rejected => Color::Red,
        ClaimStatus::PartiallyPaid => Color::Magenta,
    }
}

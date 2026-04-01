#![cfg(feature = "billing")]

use opengp_domain::domain::billing::{ClaimStatus, MedicareClaim};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Cell, Row, Table};

#[derive(Debug, Clone)]
pub struct ClaimList {
    pub claims: Vec<MedicareClaim>,
    pub selected_index: usize,
    pub scroll_state: ratatui::widgets::ListState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClaimListAction {
    Select(usize),
    ViewDetail,
    PrepareNew,
    Back,
}

impl ClaimList {
    pub fn new(claims: Vec<MedicareClaim>) -> Self {
        let mut scroll_state = ratatui::widgets::ListState::default();
        if !claims.is_empty() {
            scroll_state.select(Some(0));
        }

        Self {
            claims,
            selected_index: 0,
            scroll_state,
        }
    }

    pub fn select_next(&mut self) {
        if self.claims.is_empty() {
            self.selected_index = 0;
            self.scroll_state.select(None);
            return;
        }

        self.selected_index = (self.selected_index + 1) % self.claims.len();
        self.scroll_state.select(Some(self.selected_index));
    }

    pub fn select_prev(&mut self) {
        if self.claims.is_empty() {
            self.selected_index = 0;
            self.scroll_state.select(None);
            return;
        }

        self.selected_index = if self.selected_index == 0 {
            self.claims.len().saturating_sub(1)
        } else {
            self.selected_index.saturating_sub(1)
        };
        self.scroll_state.select(Some(self.selected_index));
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        if area.is_empty() {
            return;
        }

        let header = Row::new(vec![
            Cell::from("Reference"),
            Cell::from("Date"),
            Cell::from("Patient"),
            Cell::from("Type"),
            Cell::from("Total Claimed"),
            Cell::from("Status"),
        ])
        .style(Style::default().add_modifier(Modifier::BOLD));

        let rows = self.claims.iter().enumerate().map(|(index, claim)| {
            let row_style = if index == self.selected_index {
                Style::default().bg(Color::Blue).fg(Color::White)
            } else {
                Style::default()
            };

            let reference = claim
                .claim_reference
                .clone()
                .unwrap_or_else(|| "-".to_string());

            Row::new(vec![
                Cell::from(reference),
                Cell::from(claim.service_date.format("%d/%m/%Y").to_string()),
                Cell::from(short_patient(claim)),
                Cell::from(claim.claim_type.to_string()),
                Cell::from(format!("${:.2}", claim.total_claimed)),
                Cell::from(claim.status.to_string())
                    .style(Style::default().fg(claim_status_color(claim.status))),
            ])
            .style(row_style)
        });

        let table = Table::new(
            rows,
            [
                Constraint::Length(18),
                Constraint::Length(12),
                Constraint::Length(12),
                Constraint::Length(14),
                Constraint::Length(14),
                Constraint::Length(12),
            ],
        )
        .header(header)
        .block(Block::default().title(" Claims ").borders(Borders::ALL));

        ratatui::widgets::Widget::render(table, area, buf);
    }
}

fn short_patient(claim: &MedicareClaim) -> String {
    claim.patient_id.to_string().chars().take(8).collect()
}

fn claim_status_color(status: ClaimStatus) -> Color {
    match status {
        ClaimStatus::Draft => Color::Yellow,
        ClaimStatus::Submitted => Color::Blue,
        ClaimStatus::Processing => Color::Cyan,
        ClaimStatus::Paid => Color::Green,
        ClaimStatus::Rejected => Color::Red,
        ClaimStatus::PartiallyPaid => Color::Magenta,
    }
}

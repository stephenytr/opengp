use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
use opengp_domain::domain::billing::{ClaimStatus, MedicareClaim};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Position, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Cell, Row, Table};
use uuid::Uuid;

use crate::ui::input::DoubleClickDetector;
use crate::ui::shared::{hover_style, selected_hover_style};
use crate::ui::theme::Theme;

#[derive(Debug, Clone)]
pub struct ClaimList {
    pub claims: Vec<MedicareClaim>,
    pub selected_index: usize,
    pub scroll_state: ratatui::widgets::ListState,
    pub hovered_index: Option<usize>,
    pub double_click_detector: DoubleClickDetector,
    pub theme: Theme,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClaimListAction {
    Select(usize),
    ViewDetail,
    PrepareNew,
    Back,
    ContextMenu { x: u16, y: u16, claim_id: Uuid },
}

impl ClaimList {
    pub fn new(claims: Vec<MedicareClaim>, theme: Theme) -> Self {
        let mut scroll_state = ratatui::widgets::ListState::default();
        if !claims.is_empty() {
            scroll_state.select(Some(0));
        }

        Self {
            claims,
            selected_index: 0,
            scroll_state,
            hovered_index: None,
            double_click_detector: DoubleClickDetector::default(),
            theme,
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

    pub fn handle_mouse(&mut self, mouse: MouseEvent, area: Rect) -> Option<ClaimListAction> {
        const HEADER_HEIGHT: u16 = 2;

        // Track hover state on mouse movement
        if let MouseEventKind::Moved = mouse.kind {
            if area.contains(Position::new(mouse.column, mouse.row))
                && mouse.row >= area.y + HEADER_HEIGHT
            {
                let row_index = (mouse.row - area.y - HEADER_HEIGHT) as usize;
                if row_index < self.claims.len() {
                    self.hovered_index = Some(row_index);
                } else {
                    self.hovered_index = None;
                }
            } else {
                self.hovered_index = None;
            }
            return None;
        }

        // Handle right-click for context menu
        if let MouseEventKind::Down(MouseButton::Right) = mouse.kind {
            if !area.contains(Position::new(mouse.column, mouse.row)) {
                return None;
            }

            if mouse.row < area.y + HEADER_HEIGHT {
                return None;
            }

            let row_index = (mouse.row - area.y - HEADER_HEIGHT) as usize;
            if row_index < self.claims.len() {
                self.selected_index = row_index;
                self.scroll_state.select(Some(self.selected_index));
                if let Some(claim) = self.claims.get(row_index) {
                    return Some(ClaimListAction::ContextMenu {
                        x: mouse.column,
                        y: mouse.row,
                        claim_id: claim.id,
                    });
                }
            }
            return None;
        }

        // Handle double-click for open action
        if let MouseEventKind::Down(MouseButton::Left) = mouse.kind {
            if !area.contains(Position::new(mouse.column, mouse.row)) {
                return None;
            }

            if mouse.row < area.y + HEADER_HEIGHT {
                return None;
            }

            let row_index = (mouse.row - area.y - HEADER_HEIGHT) as usize;

            if row_index >= self.claims.len() {
                return None;
            }

            // Check for double-click
            if self.double_click_detector.check_double_click_now(&mouse) {
                self.selected_index = row_index;
                self.scroll_state.select(Some(self.selected_index));
                return Some(ClaimListAction::ViewDetail);
            }
            return None;
        }

        // Only process left mouse up for normal selection
        if mouse.kind != MouseEventKind::Up(MouseButton::Left) {
            return None;
        }

        if !area.contains(Position::new(mouse.column, mouse.row)) {
            return None;
        }

        if mouse.row < area.y + HEADER_HEIGHT {
            return None;
        }

        let row_index = (mouse.row - area.y - HEADER_HEIGHT) as usize;
        if row_index < self.claims.len() {
            self.selected_index = row_index;
            self.scroll_state.select(Some(self.selected_index));
            Some(ClaimListAction::Select(self.selected_index))
        } else {
            None
        }
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
            let is_selected = index == self.selected_index;
            let is_hovered = self.hovered_index == Some(index);

            let row_style = match (is_selected, is_hovered) {
                (true, true) => selected_hover_style(&self.theme),
                (true, false) => Style::default().bg(Color::Blue).fg(Color::White),
                (false, true) => hover_style(&self.theme),
                (false, false) => Style::default(),
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

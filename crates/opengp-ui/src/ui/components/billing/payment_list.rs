use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
use opengp_domain::domain::billing::Payment;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Position, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Borders, Cell, Row, Table};
use uuid::Uuid;

use crate::ui::input::DoubleClickDetector;
use crate::ui::shared::{hover_style, selected_hover_style};
use crate::ui::theme::Theme;

#[derive(Debug, Clone)]
pub struct PaymentList {
    pub payments: Vec<Payment>,
    pub selected_index: usize,
    pub scroll_state: ratatui::widgets::ListState,
    pub hovered_index: Option<usize>,
    pub double_click_detector: DoubleClickDetector,
    pub theme: Theme,
}

impl PaymentList {
    pub fn new(payments: Vec<Payment>, theme: Theme) -> Self {
        let mut scroll_state = ratatui::widgets::ListState::default();
        if !payments.is_empty() {
            scroll_state.select(Some(0));
        }

        Self {
            payments,
            selected_index: 0,
            scroll_state,
            hovered_index: None,
            double_click_detector: DoubleClickDetector::default(),
            theme,
        }
    }

    pub fn select_next(&mut self) {
        if self.payments.is_empty() {
            self.selected_index = 0;
            self.scroll_state.select(None);
            return;
        }

        self.selected_index = (self.selected_index + 1) % self.payments.len();
        self.scroll_state.select(Some(self.selected_index));
    }

    pub fn select_prev(&mut self) {
        if self.payments.is_empty() {
            self.selected_index = 0;
            self.scroll_state.select(None);
            return;
        }

        self.selected_index = if self.selected_index == 0 {
            self.payments.len().saturating_sub(1)
        } else {
            self.selected_index.saturating_sub(1)
        };
        self.scroll_state.select(Some(self.selected_index));
    }

    pub fn handle_mouse(&mut self, mouse: MouseEvent, area: Rect) -> Option<PaymentListAction> {
        const HEADER_HEIGHT: u16 = 2;

        // Track hover state on mouse movement
        if let MouseEventKind::Moved = mouse.kind {
            if area.contains(Position::new(mouse.column, mouse.row))
                && mouse.row >= area.y + HEADER_HEIGHT
            {
                let row_index = (mouse.row - area.y - HEADER_HEIGHT) as usize;
                if row_index < self.payments.len() {
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
            if row_index < self.payments.len() {
                self.selected_index = row_index;
                self.scroll_state.select(Some(self.selected_index));
                if let Some(payment) = self.payments.get(row_index) {
                    return Some(PaymentListAction::ContextMenu {
                        x: mouse.column,
                        y: mouse.row,
                        payment_id: payment.id,
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

            if row_index >= self.payments.len() {
                return None;
            }

            // Check for double-click
            if self.double_click_detector.check_double_click_now(&mouse) {
                self.selected_index = row_index;
                self.scroll_state.select(Some(self.selected_index));
                return Some(PaymentListAction::ViewDetail);
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
        if row_index < self.payments.len() {
            self.selected_index = row_index;
            self.scroll_state.select(Some(self.selected_index));
            Some(PaymentListAction::Select(self.selected_index))
        } else {
            None
        }
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        if area.is_empty() {
            return;
        }

        let header = Row::new(vec![
            Cell::from("Date"),
            Cell::from("Invoice #"),
            Cell::from("Patient"),
            Cell::from("Amount"),
            Cell::from("Method"),
            Cell::from("Reference"),
        ])
        .style(Style::default().add_modifier(Modifier::BOLD));

        let rows = self.payments.iter().enumerate().map(|(index, payment)| {
            let is_selected = index == self.selected_index;
            let is_hovered = self.hovered_index == Some(index);

            let style = match (is_selected, is_hovered) {
                (true, true) => selected_hover_style(&self.theme),
                (true, false) => Style::default().add_modifier(Modifier::REVERSED),
                (false, true) => hover_style(&self.theme),
                (false, false) => Style::default(),
            };

            let reference = payment.reference.clone().unwrap_or_else(|| "-".to_string());

            Row::new(vec![
                Cell::from(payment.payment_date.format("%d/%m/%Y").to_string()),
                Cell::from(payment.invoice_id.to_string()),
                Cell::from(payment.patient_id.to_string()),
                Cell::from(format!("${:.2}", payment.amount)),
                Cell::from(payment.payment_method.to_string()),
                Cell::from(reference),
            ])
            .style(style)
        });

        let table = Table::new(
            rows,
            [
                Constraint::Length(12),
                Constraint::Length(12),
                Constraint::Length(36),
                Constraint::Length(12),
                Constraint::Length(12),
                Constraint::Min(10),
            ],
        )
        .header(header)
        .block(Block::default().title(" Payments ").borders(Borders::ALL));

        ratatui::widgets::Widget::render(table, area, buf);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PaymentListAction {
    Select(usize),
    ViewDetail,
    Back,
    ContextMenu { x: u16, y: u16, payment_id: Uuid },
}

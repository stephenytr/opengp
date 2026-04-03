use opengp_domain::domain::billing::Payment;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Borders, Cell, Row, Table};

#[derive(Debug, Clone)]
pub struct PaymentList {
    pub payments: Vec<Payment>,
    pub selected_index: usize,
    pub scroll_state: ratatui::widgets::ListState,
}

impl PaymentList {
    pub fn new(payments: Vec<Payment>) -> Self {
        let mut scroll_state = ratatui::widgets::ListState::default();
        if !payments.is_empty() {
            scroll_state.select(Some(0));
        }

        Self {
            payments,
            selected_index: 0,
            scroll_state,
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
            let style = if index == self.selected_index {
                Style::default().add_modifier(Modifier::REVERSED)
            } else {
                Style::default()
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
}

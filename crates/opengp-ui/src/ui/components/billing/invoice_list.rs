#![cfg(feature = "billing")]

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use opengp_domain::domain::billing::{Invoice, InvoiceStatus};
use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Cell, ListState, Row, Table};
use ratatui::Frame;
use uuid::Uuid;

pub struct InvoiceList {
    invoices: Vec<Invoice>,
    filtered: Vec<Invoice>,
    search_query: String,
    scroll_state: ListState,
    selected_index: usize,
}

#[derive(Debug, Clone)]
pub enum InvoiceListAction {
    Select(usize),
    Open(Uuid),
    SearchChanged(String),
    New,
}

impl Default for InvoiceList {
    fn default() -> Self {
        Self::new()
    }
}

impl InvoiceList {
    pub fn new() -> Self {
        let mut scroll_state = ListState::default();
        scroll_state.select(Some(0));

        Self {
            invoices: Vec::new(),
            filtered: Vec::new(),
            search_query: String::new(),
            scroll_state,
            selected_index: 0,
        }
    }

    pub fn set_invoices(&mut self, invoices: Vec<Invoice>) {
        self.invoices = invoices;
        self.filter();
    }

    pub fn invoices(&self) -> &[Invoice] {
        &self.invoices
    }

    pub fn filtered_invoices(&self) -> &[Invoice] {
        &self.filtered
    }

    pub fn search_query(&self) -> &str {
        &self.search_query
    }

    pub fn set_search_query(&mut self, query: String) {
        self.search_query = query;
        self.filter();
    }

    pub fn selected_invoice(&self) -> Option<&Invoice> {
        self.filtered.get(self.selected_index)
    }

    pub fn selected_invoice_id(&self) -> Option<Uuid> {
        self.selected_invoice().map(|invoice| invoice.id)
    }

    pub fn filter(&mut self) {
        let query = self.search_query.trim().to_lowercase();

        if query.is_empty() {
            self.filtered = self.invoices.clone();
        } else {
            self.filtered = self
                .invoices
                .iter()
                .filter(|invoice| {
                    invoice.invoice_number.to_lowercase().contains(&query)
                        || invoice.status.to_string().to_lowercase().contains(&query)
                })
                .cloned()
                .collect();
        }

        if self.filtered.is_empty() {
            self.selected_index = 0;
            self.scroll_state.select(None);
        } else {
            self.selected_index = self.selected_index.min(self.filtered.len() - 1);
            self.scroll_state.select(Some(self.selected_index));
        }
    }

    pub fn select_next(&mut self) {
        if self.filtered.is_empty() {
            return;
        }

        self.selected_index = (self.selected_index + 1).min(self.filtered.len().saturating_sub(1));
        self.scroll_state.select(Some(self.selected_index));
    }

    pub fn select_prev(&mut self) {
        if self.filtered.is_empty() {
            return;
        }

        self.selected_index = self.selected_index.saturating_sub(1);
        self.scroll_state.select(Some(self.selected_index));
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Option<InvoiceListAction> {
        if key.kind != KeyEventKind::Press {
            return None;
        }

        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.select_prev();
                Some(InvoiceListAction::Select(self.selected_index))
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.select_next();
                Some(InvoiceListAction::Select(self.selected_index))
            }
            KeyCode::Enter => self.selected_invoice_id().map(InvoiceListAction::Open),
            KeyCode::Backspace => {
                self.search_query.pop();
                self.filter();
                Some(InvoiceListAction::SearchChanged(self.search_query.clone()))
            }
            KeyCode::Char('n') => Some(InvoiceListAction::New),
            KeyCode::Char(c) => {
                self.search_query.push(c);
                self.filter();
                Some(InvoiceListAction::SearchChanged(self.search_query.clone()))
            }
            _ => None,
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default().title(" Invoices ").borders(Borders::ALL);
        let inner = block.inner(area);
        frame.render_widget(block, area);

        let header = Row::new(vec!["Invoice #", "Date", "Patient", "Total", "Status"])
            .style(Style::default().fg(Color::Cyan).bold());

        let widths = [
            Constraint::Length(16),
            Constraint::Length(12),
            Constraint::Length(14),
            Constraint::Length(12),
            Constraint::Length(16),
        ];

        let rows: Vec<Row> = self
            .filtered
            .iter()
            .enumerate()
            .map(|(idx, invoice)| {
                let row_style = if idx == self.selected_index {
                    Style::default().bg(Color::Blue).fg(Color::White)
                } else {
                    Style::default().fg(Color::White)
                };

                let status = invoice.status.to_string();
                let status_color = status_color(invoice.status);

                Row::new(vec![
                    Cell::from(invoice.invoice_number.clone()),
                    Cell::from(invoice.invoice_date.format("%d/%m/%Y").to_string()),
                    Cell::from(short_patient(invoice)),
                    Cell::from(format!("${:.2}", invoice.total_amount)),
                    Cell::from(status).style(Style::default().fg(status_color)),
                ])
                .style(row_style)
            })
            .collect();

        let table = Table::new(rows, widths)
            .header(header)
            .block(Block::default().borders(Borders::NONE))
            .widths(widths);

        frame.render_widget(table, inner);
    }
}

fn status_color(status: InvoiceStatus) -> Color {
    match status {
        InvoiceStatus::Draft => Color::Yellow,
        InvoiceStatus::Issued => Color::Blue,
        InvoiceStatus::Paid => Color::Green,
        InvoiceStatus::Overdue => Color::Red,
        InvoiceStatus::PartiallyPaid => Color::Cyan,
        InvoiceStatus::Cancelled => Color::Gray,
        InvoiceStatus::Refunded => Color::Magenta,
    }
}

fn short_patient(invoice: &Invoice) -> String {
    invoice.patient_id.to_string().chars().take(8).collect()
}

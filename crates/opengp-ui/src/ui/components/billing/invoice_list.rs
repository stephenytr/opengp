use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, MouseButton, MouseEvent, MouseEventKind};
use opengp_domain::domain::billing::{Invoice, InvoiceStatus};
use ratatui::layout::{Constraint, Position, Rect};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Cell, ListState, Row, Table};
use ratatui::Frame;
use uuid::Uuid;

use crate::ui::input::DoubleClickDetector;
use crate::ui::shared::{hover_style, selected_hover_style};
use crate::ui::theme::Theme;

pub struct InvoiceList {
    invoices: Vec<Invoice>,
    filtered: Vec<Invoice>,
    search_query: String,
    scroll_state: ListState,
    pub selected_index: usize,
    hovered_index: Option<usize>,
    double_click_detector: DoubleClickDetector,
    theme: Theme,
}

#[derive(Debug, Clone)]
pub enum InvoiceListAction {
    Select(usize),
    Open(Uuid),
    SearchChanged(String),
    New,
    ContextMenu { x: u16, y: u16, invoice_id: Uuid },
}

impl Default for InvoiceList {
    fn default() -> Self {
        Self::new(Theme::dark())
    }
}

impl InvoiceList {
    pub fn new(theme: Theme) -> Self {
        let mut scroll_state = ListState::default();
        scroll_state.select(Some(0));

        Self {
            invoices: Vec::new(),
            filtered: Vec::new(),
            search_query: String::new(),
            scroll_state,
            selected_index: 0,
            hovered_index: None,
            double_click_detector: DoubleClickDetector::default(),
            theme,
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

    pub fn handle_mouse(&mut self, mouse: MouseEvent, area: Rect) -> Option<InvoiceListAction> {
        const HEADER_HEIGHT: u16 = 2;

        // Track hover state on mouse movement
        if let MouseEventKind::Moved = mouse.kind {
            if area.contains(Position::new(mouse.column, mouse.row))
                && mouse.row >= area.y + HEADER_HEIGHT
            {
                let row_index = (mouse.row - area.y - HEADER_HEIGHT) as usize;
                if row_index < self.filtered.len() {
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
            if row_index < self.filtered.len() {
                self.selected_index = row_index;
                self.scroll_state.select(Some(self.selected_index));
                if let Some(invoice) = self.filtered.get(row_index) {
                    return Some(InvoiceListAction::ContextMenu {
                        x: mouse.column,
                        y: mouse.row,
                        invoice_id: invoice.id,
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

            if row_index >= self.filtered.len() {
                return None;
            }

            // Check for double-click
            if self.double_click_detector.check_double_click_now(&mouse) {
                if let Some(invoice) = self.filtered.get(row_index) {
                    self.selected_index = row_index;
                    self.scroll_state.select(Some(self.selected_index));
                    return Some(InvoiceListAction::Open(invoice.id));
                }
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
        if row_index < self.filtered.len() {
            self.selected_index = row_index;
            self.scroll_state.select(Some(self.selected_index));
            Some(InvoiceListAction::Select(self.selected_index))
        } else {
            None
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default().title(" Invoices ").borders(Borders::ALL);
        let inner = block.inner(area);
        frame.render_widget(block, area);

        let header = Row::new(vec!["Invoice #", "Date", "Patient", "Total", "Status"])
            .style(Style::default().fg(self.theme.colors.primary).bold());

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
                let is_selected = idx == self.selected_index;
                let is_hovered = self.hovered_index == Some(idx);

                let row_style = match (is_selected, is_hovered) {
                    (true, true) => selected_hover_style(&self.theme),
                    (true, false) => Style::default().bg(self.theme.colors.selected).fg(self.theme.colors.background_dark),
                    (false, true) => hover_style(&self.theme),
                    (false, false) => Style::default().fg(self.theme.colors.foreground),
                };

                let status = invoice.status.to_string();
                let status_color = status_color(invoice.status, &self.theme);

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

fn status_color(status: InvoiceStatus, theme: &Theme) -> Color {
    match status {
        InvoiceStatus::Draft => theme.colors.warning,
        InvoiceStatus::Issued => theme.colors.info,
        InvoiceStatus::Paid => theme.colors.success,
        InvoiceStatus::Overdue => theme.colors.error,
        InvoiceStatus::PartiallyPaid => theme.colors.highlight,
        InvoiceStatus::Cancelled => theme.colors.disabled,
        InvoiceStatus::Refunded => theme.colors.secondary,
    }
}

fn short_patient(invoice: &Invoice) -> String {
    invoice.patient_id.to_string().chars().take(8).collect()
}

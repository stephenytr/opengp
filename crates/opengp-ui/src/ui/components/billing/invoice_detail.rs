use opengp_domain::domain::billing::{Invoice, InvoiceItem, InvoiceStatus, Payment};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Row, Table, Widget, Wrap};

#[derive(Debug, Clone)]
pub struct InvoiceDetail {
    pub invoice: Option<Invoice>,
    pub items: Vec<InvoiceItem>,
    pub payments: Vec<Payment>,
    pub scroll_offset: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvoiceDetailAction {
    Edit,
    Issue,
    RecordPayment,
    Cancel,
    Back,
}

impl Default for InvoiceDetail {
    fn default() -> Self {
        Self {
            invoice: None,
            items: Vec::new(),
            payments: Vec::new(),
            scroll_offset: 0,
        }
    }
}

impl InvoiceDetail {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_invoice(&mut self, invoice: Invoice) {
        self.items = invoice.items.clone();
        self.invoice = Some(invoice);
    }

    pub fn set_payments(&mut self, payments: Vec<Payment>) {
        self.payments = payments;
    }

    pub fn available_actions(&self) -> Vec<InvoiceDetailAction> {
        let mut actions = Vec::new();

        if let Some(invoice) = &self.invoice {
            if invoice.status == InvoiceStatus::Draft {
                actions.push(InvoiceDetailAction::Edit);
                actions.push(InvoiceDetailAction::Issue);
            }

            if matches!(
                invoice.status,
                InvoiceStatus::Issued | InvoiceStatus::PartiallyPaid
            ) {
                actions.push(InvoiceDetailAction::RecordPayment);
            }

            if !matches!(
                invoice.status,
                InvoiceStatus::Cancelled | InvoiceStatus::Refunded
            ) {
                actions.push(InvoiceDetailAction::Cancel);
            }
        }

        actions.push(InvoiceDetailAction::Back);
        actions
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        if area.is_empty() {
            return;
        }

        let root = Block::default()
            .title(" Invoice Detail ")
            .borders(Borders::ALL);
        root.clone().render(area, buf);

        let inner = root.inner(area);
        if inner.is_empty() {
            return;
        }

        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(7),
                Constraint::Length(9),
                Constraint::Length(7),
                Constraint::Min(4),
                Constraint::Length(2),
            ])
            .split(inner);

        self.render_header(rows[0], buf);
        self.render_items_table(rows[1], buf);
        self.render_totals(rows[2], buf);
        self.render_payment_history(rows[3], buf);
        self.render_actions(rows[4], buf);
    }

    fn render_header(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::default().title(" Header ").borders(Borders::ALL);
        block.clone().render(area, buf);
        let inner = block.inner(area);
        if inner.is_empty() {
            return;
        }

        let content = if let Some(invoice) = &self.invoice {
            vec![
                Line::from(vec![
                    Span::styled("Invoice: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(invoice.invoice_number.clone()),
                    Span::raw("   "),
                    Span::styled("Date: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(invoice.invoice_date.to_string()),
                ]),
                Line::from(vec![
                    Span::styled("Patient: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(invoice.patient_id.to_string()),
                ]),
                Line::from(vec![
                    Span::styled(
                        "Practitioner: ",
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(invoice.practitioner_id.to_string()),
                ]),
                Line::from(vec![
                    Span::styled(
                        "Billing Type: ",
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(invoice.billing_type.to_string()),
                    Span::raw("   "),
                    Span::styled("Status: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(invoice.status.to_string()),
                ]),
            ]
        } else {
            vec![Line::from("No invoice selected")]
        };

        Paragraph::new(content)
            .wrap(Wrap { trim: true })
            .render(inner, buf);
    }

    fn render_items_table(&self, area: Rect, buf: &mut Buffer) {
        let header = Row::new(vec![
            "Description",
            "Item Code",
            "Qty",
            "Unit Price",
            "Amount",
            "GST Free",
        ])
        .style(Style::default().add_modifier(Modifier::BOLD));

        let widths = [
            Constraint::Percentage(32),
            Constraint::Percentage(18),
            Constraint::Length(6),
            Constraint::Length(12),
            Constraint::Length(12),
            Constraint::Length(10),
        ];

        let rows: Vec<Row> = self
            .items
            .iter()
            .map(|item| {
                Row::new(vec![
                    item.description.clone(),
                    item.item_code.clone().unwrap_or_else(|| "-".to_string()),
                    item.quantity.to_string(),
                    format_money(item.unit_price),
                    format_money(item.amount),
                    if item.is_gst_free { "Yes" } else { "No" }.to_string(),
                ])
            })
            .collect();

        Table::new(rows, widths)
            .header(header)
            .block(Block::default().title(" Line Items ").borders(Borders::ALL))
            .render(area, buf);
    }

    fn render_totals(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::default().title(" Totals ").borders(Borders::ALL);
        block.clone().render(area, buf);
        let inner = block.inner(area);
        if inner.is_empty() {
            return;
        }

        let lines = if let Some(invoice) = &self.invoice {
            vec![
                Line::from(format!("Subtotal: {}", format_money(invoice.subtotal))),
                Line::from(format!("GST: {}", format_money(invoice.gst_amount))),
                Line::from(format!("Total: {}", format_money(invoice.total_amount))),
                Line::from(format!("Paid: {}", format_money(invoice.amount_paid))),
                Line::from(format!(
                    "Outstanding: {}",
                    format_money(invoice.amount_outstanding)
                )),
            ]
        } else {
            vec![Line::from("-"), Line::from("-"), Line::from("-")]
        };

        Paragraph::new(lines)
            .wrap(Wrap { trim: true })
            .render(inner, buf);
    }

    fn render_payment_history(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title(" Payment History ")
            .borders(Borders::ALL);
        block.clone().render(area, buf);
        let inner = block.inner(area);
        if inner.is_empty() {
            return;
        }

        if self.payments.is_empty() {
            Paragraph::new("No payments recorded")
                .wrap(Wrap { trim: true })
                .render(inner, buf);
            return;
        }

        let visible_height = inner.height as usize;
        let start = self
            .scroll_offset
            .min(self.payments.len().saturating_sub(visible_height));

        let lines: Vec<Line> = self
            .payments
            .iter()
            .skip(start)
            .take(visible_height)
            .map(|payment| {
                let reference = payment.reference.clone().unwrap_or_else(|| "-".to_string());
                Line::from(format!(
                    "{} | {} | {} | Ref: {}",
                    payment.payment_date.date_naive(),
                    format_money(payment.amount),
                    payment.payment_method,
                    reference
                ))
            })
            .collect();

        Paragraph::new(lines)
            .wrap(Wrap { trim: true })
            .render(inner, buf);
    }

    fn render_actions(&self, area: Rect, buf: &mut Buffer) {
        let mut labels = Vec::new();
        for action in self.available_actions() {
            let label = match action {
                InvoiceDetailAction::Edit => "[E]dit",
                InvoiceDetailAction::Issue => "[I]ssue",
                InvoiceDetailAction::RecordPayment => "[R]ecord Payment",
                InvoiceDetailAction::Cancel => "[C]ancel",
                InvoiceDetailAction::Back => "[B]ack",
            };
            labels.push(label);
        }

        Paragraph::new(labels.join("  ")).render(area, buf);
    }
}

fn format_money(amount: f64) -> String {
    format!("${amount:.2}")
}

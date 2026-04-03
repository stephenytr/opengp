use opengp_config::PracticeConfig;
use opengp_domain::domain::billing::{Invoice, InvoiceItem, Payment};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Row, Table};
use ratatui::Frame;

#[derive(Debug, Clone)]
pub struct ReceiptPopup {
    pub invoice: Invoice,
    pub items: Vec<InvoiceItem>,
    pub payment: Payment,
    pub practice: PracticeConfig,
    pub visible: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReceiptAction {
    Close,
}

impl ReceiptPopup {
    pub fn new(
        invoice: Invoice,
        items: Vec<InvoiceItem>,
        payment: Payment,
        practice: PracticeConfig,
    ) -> Self {
        Self {
            invoice,
            items,
            payment,
            practice,
            visible: true,
        }
    }

    pub fn render(&self, frame: &mut Frame) {
        if !self.visible {
            return;
        }

        let area = frame.area();
        let popup_area = centered_rect(area);

        frame.render_widget(Clear, popup_area);
        let block = Block::default().title(" Receipt ").borders(Borders::ALL);
        let inner = block.inner(popup_area);
        frame.render_widget(block, popup_area);

        let sections = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(7),
                Constraint::Length(2),
                Constraint::Length(8),
                Constraint::Length(7),
                Constraint::Length(2),
            ])
            .split(inner);

        frame.render_widget(self.header_paragraph(), sections[0]);
        frame.render_widget(self.invoice_meta_paragraph(), sections[1]);
        frame.render_widget(self.items_table(), sections[2]);
        frame.render_widget(self.totals_paragraph(), sections[3]);
        frame.render_widget(
            Paragraph::new("Press Esc to close")
                .style(Style::default().add_modifier(Modifier::ITALIC)),
            sections[4],
        );
    }

    pub fn export_to_file(&self) -> Result<(), String> {
        Err("Export not yet implemented".to_string())
    }

    fn header_paragraph(&self) -> Paragraph<'_> {
        let (practitioner_name, provider_number) = self.practitioner_details();
        let patient_name = self.invoice.patient_id.to_string();

        Paragraph::new(format!(
            "{}\nABN: {}\n{}\nPhone: {}\nPractitioner: {} (Provider: {})\nPatient: {}",
            non_empty(&self.practice.profile.name),
            non_empty(&self.practice.profile.abn),
            non_empty(&self.practice.contact.address),
            non_empty(&self.practice.contact.phone),
            practitioner_name,
            provider_number,
            patient_name
        ))
    }

    fn invoice_meta_paragraph(&self) -> Paragraph<'_> {
        Paragraph::new(format!(
            "Invoice #: {}   Date: {}",
            self.invoice.invoice_number, self.invoice.invoice_date
        ))
    }

    fn items_table(&self) -> Table<'_> {
        let header = Row::new(vec!["Description", "Qty", "Unit Price", "Amount"])
            .style(Style::default().add_modifier(Modifier::BOLD));

        let rows: Vec<Row> = self
            .items
            .iter()
            .map(|item| {
                Row::new(vec![
                    item.description.clone(),
                    item.quantity.to_string(),
                    format_money(item.unit_price),
                    format_money(item.amount),
                ])
            })
            .collect();

        Table::new(
            rows,
            [
                Constraint::Percentage(50),
                Constraint::Length(6),
                Constraint::Length(14),
                Constraint::Length(14),
            ],
        )
        .header(header)
        .block(Block::default().title(" Line Items ").borders(Borders::ALL))
    }

    fn totals_paragraph(&self) -> Paragraph<'_> {
        Paragraph::new(format!(
            "Subtotal: {}\nGST: {}\nTotal: {}\nPaid: {} (Cash) on {}\nOutstanding: {}",
            format_money(self.invoice.subtotal),
            format_money(self.invoice.gst_amount),
            format_money(self.invoice.total_amount),
            format_money(self.payment.amount),
            self.payment.payment_date.date_naive(),
            format_money(self.invoice.amount_outstanding)
        ))
    }

    fn practitioner_details(&self) -> (String, String) {
        let practitioner_id = self.invoice.practitioner_id.to_string();

        if let Some((name, provider)) = self
            .practice
            .providers
            .iter()
            .find(|(name, _)| *name == &practitioner_id)
        {
            return (name.clone(), non_empty(&provider.provider_number));
        }

        if let Some((name, provider)) = self.practice.providers.iter().next() {
            return (name.clone(), non_empty(&provider.provider_number));
        }

        ("N/A".to_string(), "N/A".to_string())
    }
}

fn centered_rect(area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(10),
            Constraint::Percentage(80),
            Constraint::Percentage(10),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(12),
            Constraint::Percentage(76),
            Constraint::Percentage(12),
        ])
        .split(vertical[1])[1]
}

fn format_money(amount: f64) -> String {
    format!("${amount:.2}")
}

fn non_empty(value: &str) -> String {
    if value.trim().is_empty() {
        "N/A".to_string()
    } else {
        value.to_string()
    }
}

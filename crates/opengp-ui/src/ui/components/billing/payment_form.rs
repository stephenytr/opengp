use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, Paragraph, Widget};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct PaymentForm {
    pub invoice_id: Uuid,
    pub amount: String,
    pub reference: String,
    pub notes: String,
    pub focused_field: usize,
    pub error: Option<String>,
}

impl PaymentForm {
    pub fn new(invoice_id: Uuid, outstanding_balance: f64) -> Self {
        Self {
            invoice_id,
            amount: format!("{outstanding_balance:.2}"),
            reference: String::new(),
            notes: String::new(),
            focused_field: 0,
            error: None,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        let parsed_amount = self
            .amount
            .trim()
            .parse::<f64>()
            .map_err(|_| "Amount must be a valid number".to_string())?;

        if parsed_amount <= 0.0 {
            return Err("Amount must be greater than 0.00".to_string());
        }

        Ok(())
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        if area.is_empty() {
            return;
        }

        let error_text = self
            .error
            .as_ref()
            .map(|err| format!("\n\nError: {err}"))
            .unwrap_or_default();

        let content = format!(
            "Invoice: {}\nOutstanding (pre-filled): ${}\n\nAmount: {}\nReference: {}\nNotes: {}\nPayment Method: Cash{}",
            self.invoice_id,
            self.amount,
            self.amount,
            if self.reference.is_empty() {
                "-"
            } else {
                self.reference.as_str()
            },
            if self.notes.is_empty() {
                "-"
            } else {
                self.notes.as_str()
            },
            error_text
        );

        let paragraph = Paragraph::new(content).block(
            Block::default()
                .title(" Record Cash Payment ")
                .borders(Borders::ALL),
        );

        paragraph.render(area, buf);
    }

    pub fn set_field_value(&mut self, field_index: usize, value: String) {
        match field_index {
            0 => self.amount = value,
            1 => self.reference = value,
            2 => self.notes = value,
            _ => {}
        }
        self.focused_field = field_index;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PaymentFormAction {
    Submit,
    Cancel,
    FieldChanged(usize, String),
}

use opengp_domain::domain::billing::{BillingType, InvoiceStatus};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Borders, Row, Table, Widget};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct InvoiceItemDraft {
    pub description: String,
    pub item_code: String,
    pub quantity: u32,
    pub unit_price: f64,
    pub is_gst_free: bool,
}

#[derive(Debug, Clone)]
pub struct InvoiceForm {
    pub patient_id: Option<Uuid>,
    pub practitioner_id: Option<Uuid>,
    pub billing_type: BillingType,
    pub due_date: String,
    pub items: Vec<InvoiceItemDraft>,
    pub focused_field: usize,
    pub error: Option<String>,
    pub invoice_status: Option<InvoiceStatus>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InvoiceFormAction {
    Submit,
    Cancel,
    AddItem,
    RemoveItem(usize),
    FieldChanged(usize, String),
}

impl Default for InvoiceForm {
    fn default() -> Self {
        Self {
            patient_id: None,
            practitioner_id: None,
            billing_type: BillingType::PrivateBilling,
            due_date: String::new(),
            items: vec![InvoiceItemDraft::default()],
            focused_field: 0,
            error: None,
            invoice_status: None,
        }
    }
}

impl Default for InvoiceItemDraft {
    fn default() -> Self {
        Self {
            description: String::new(),
            item_code: String::new(),
            quantity: 1,
            unit_price: 0.0,
            is_gst_free: true,
        }
    }
}

impl InvoiceForm {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_editable(&self) -> bool {
        matches!(self.invoice_status, None | Some(InvoiceStatus::Draft))
    }

    pub fn add_item(&mut self) {
        if !self.is_editable() {
            return;
        }
        self.items.push(InvoiceItemDraft::default());
    }

    pub fn remove_item(&mut self, index: usize) {
        if !self.is_editable() {
            return;
        }
        if self.items.len() <= 1 {
            return;
        }

        if index < self.items.len() {
            self.items.remove(index);
            if self.focused_field >= self.items.len() {
                self.focused_field = self.items.len().saturating_sub(1);
            }
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.patient_id.is_none() {
            return Err("Patient is required".to_string());
        }

        if self.practitioner_id.is_none() {
            return Err("Practitioner is required".to_string());
        }

        if self.items.is_empty() {
            return Err("At least one invoice item is required".to_string());
        }

        for (idx, item) in self.items.iter().enumerate() {
            if item.description.trim().is_empty() {
                return Err(format!("Item {} requires description", idx + 1));
            }

            if item.quantity == 0 {
                return Err(format!("Item {} quantity must be greater than 0", idx + 1));
            }

            if item.unit_price <= 0.0 {
                return Err(format!(
                    "Item {} unit price must be greater than 0",
                    idx + 1
                ));
            }
        }

        Ok(())
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        if area.is_empty() {
            return;
        }

        let block = Block::default()
            .title(" Invoice Form ")
            .borders(Borders::ALL);
        block.clone().render(area, buf);
        let inner = block.inner(area);
        if inner.is_empty() {
            return;
        }

        let top_height = 6u16;
        let top_block = Rect {
            x: inner.x,
            y: inner.y,
            width: inner.width,
            height: top_height.min(inner.height),
        };

        let table_block = Rect {
            x: inner.x,
            y: inner.y + top_block.height,
            width: inner.width,
            height: inner.height.saturating_sub(top_block.height),
        };

        self.render_fields(top_block, buf);
        self.render_items_table(table_block, buf);
    }

    fn render_fields(&self, area: Rect, buf: &mut Buffer) {
        let patient = self
            .patient_id
            .map(|id| id.to_string())
            .unwrap_or_else(|| "<select patient>".to_string());
        let practitioner = self
            .practitioner_id
            .map(|id| id.to_string())
            .unwrap_or_else(|| "<select practitioner>".to_string());

        let billing_type_str = self.billing_type.to_string();
        let due_date_str = self.due_date.to_string();
        let header = vec![
            Row::new(vec!["Patient", patient.as_ref()]),
            Row::new(vec!["Practitioner", practitioner.as_ref()]),
            Row::new(vec!["Billing Type", billing_type_str.as_ref()]),
            Row::new(vec!["Due Date", due_date_str.as_ref()]),
        ];

        let table = Table::new(header, [Constraint::Length(16), Constraint::Min(10)])
            .column_spacing(1)
            .block(Block::default().title(" Details ").borders(Borders::ALL));

        table.render(area, buf);
    }

    fn render_items_table(&self, area: Rect, buf: &mut Buffer) {
        let rows: Vec<Row> = self
            .items
            .iter()
            .enumerate()
            .map(|(idx, item)| {
                let amount = item.unit_price * item.quantity as f64;
                let style = if idx == self.focused_field {
                    Style::default().add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                Row::new(vec![
                    (idx + 1).to_string(),
                    item.description.clone(),
                    item.item_code.clone(),
                    item.quantity.to_string(),
                    format!("${:.2}", item.unit_price),
                    format!("${:.2}", amount),
                    if item.is_gst_free { "Yes" } else { "No" }.to_string(),
                ])
                .style(style)
            })
            .collect();

        let header = Row::new(vec![
            "#",
            "Description",
            "Code",
            "Qty",
            "Unit",
            "Amount",
            "GST Free",
        ])
        .style(Style::default().add_modifier(Modifier::BOLD));

        let widths = [
            Constraint::Length(3),
            Constraint::Percentage(34),
            Constraint::Length(12),
            Constraint::Length(6),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
        ];

        Table::new(rows, widths)
            .header(header)
            .block(
                Block::default()
                    .title(" Line Items  [A]dd  [R]emove  [Enter] Submit  [Esc] Cancel ")
                    .borders(Borders::ALL),
            )
            .render(area, buf);
    }

    pub fn supported_billing_types() -> &'static [BillingType] {
        &[
            BillingType::BulkBilling,
            BillingType::PrivateBilling,
            BillingType::MixedBilling,
            BillingType::ThirdParty,
        ]
    }
}

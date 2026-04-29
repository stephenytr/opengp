use std::rc::Rc;

use crate::ui::theme::Theme;
use crate::ui::widgets::{UnifiedColumnDef, UnifiedList, UnifiedListAction, UnifiedListConfig};
use crossterm::event::{Event, KeyEvent, KeyEventKind, MouseEvent};
use opengp_domain::domain::billing::{Invoice, InvoiceStatus};
use rat_event::ct_event;
use rat_focus::{FocusBuilder, FocusFlag, HasFocus};
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct InvoiceList {
    pub invoices: Vec<Invoice>,
    pub selected_index: usize,
    pub scroll_offset: usize,
    pub hovered_index: Option<usize>,
    pub theme: Theme,
    pub search_query: String,
    pub focus: FocusFlag,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
        let mut scroll_state = ratatui::widgets::ListState::default();
        scroll_state.select(Some(0));

        Self {
            invoices: Vec::new(),
            selected_index: 0,
            scroll_offset: 0,
            hovered_index: None,
            theme,
            search_query: String::new(),
            focus: FocusFlag::default(),
        }
    }

    pub fn set_invoices(&mut self, invoices: Vec<Invoice>) {
        self.invoices = invoices;
    }

    pub fn invoices(&self) -> &[Invoice] {
        &self.invoices
    }

    pub fn filtered_invoices(&self) -> Vec<Invoice> {
        let query = self.search_query.trim().to_lowercase();
        if query.is_empty() {
            self.invoices.clone()
        } else {
            self.invoices
                .iter()
                .filter(|invoice| {
                    invoice.invoice_number.to_lowercase().contains(&query)
                        || invoice.status.to_string().to_lowercase().contains(&query)
                })
                .cloned()
                .collect()
        }
    }

    pub fn search_query(&self) -> &str {
        &self.search_query
    }

    pub fn set_search_query(&mut self, query: String) {
        self.search_query = query;
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Option<InvoiceListAction> {
        if key.kind != KeyEventKind::Press {
            return None;
        }

        let event = Event::Key(key);

        // Handle search mode - keys accumulate in search
        match &event {
            ct_event!(keycode press Backspace) => {
                self.search_query.pop();
                Some(InvoiceListAction::SearchChanged(self.search_query.clone()))
            }
            ct_event!(key press 'n') => Some(InvoiceListAction::New),
            ct_event!(keycode press Enter) => self
                .filtered_invoices()
                .get(self.selected_index)
                .map(|inv| InvoiceListAction::Open(inv.id)),
            ct_event!(key press c) => {
                self.search_query.push(*c);
                Some(InvoiceListAction::SearchChanged(self.search_query.clone()))
            }
            _ => {
                // Delegate navigation to UnifiedList
                let mut list = self.as_list();
                let action = list.handle_key(key);
                self.sync_from(&list);
                action.map(|a| match a {
                    UnifiedListAction::Select(i) => InvoiceListAction::Select(i),
                    UnifiedListAction::Open(inv) => InvoiceListAction::Open(inv.id),
                    UnifiedListAction::New => InvoiceListAction::New,
                    _ => InvoiceListAction::Select(self.selected_index),
                })
            }
        }
    }

    pub fn handle_mouse(&mut self, mouse: MouseEvent, area: Rect) -> Option<InvoiceListAction> {
        let mut list = self.as_list();
        let action = list.handle_mouse(mouse, area);
        self.sync_from(&list);
        action.map(|a| match a {
            UnifiedListAction::Select(i) => InvoiceListAction::Select(i),
            UnifiedListAction::Open(inv) => InvoiceListAction::Open(inv.id),
            UnifiedListAction::ContextMenu { index, x, y } => {
                if let Some(inv) = self.filtered_invoices().get(index) {
                    InvoiceListAction::ContextMenu {
                        x,
                        y,
                        invoice_id: inv.id,
                    }
                } else {
                    InvoiceListAction::Select(index)
                }
            }
            _ => InvoiceListAction::Select(self.selected_index),
        })
    }

    fn as_list(&self) -> UnifiedList<Invoice> {
        let filtered = self.filtered_invoices();
        let mut list = UnifiedList::new(
            filtered,
            columns(),
            self.theme.clone(),
            UnifiedListConfig::new("Invoices", 2, "No invoices found."),
        );
        list.selected_index = self.selected_index;
        list.scroll_offset = self.scroll_offset;
        list.hovered_index = self.hovered_index;
        list
    }

    fn sync_from(&mut self, list: &UnifiedList<Invoice>) {
        self.selected_index = list.selected_index;
        self.scroll_offset = list.scroll_offset;
        self.hovered_index = list.hovered_index;
    }
}

impl Widget for InvoiceList {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.as_list().render(area, buf);
    }
}

fn col(
    title: &'static str,
    width: u16,
    render: impl Fn(&Invoice) -> String + 'static,
) -> UnifiedColumnDef<Invoice> {
    UnifiedColumnDef::new(title, width, render)
}

fn columns() -> Vec<UnifiedColumnDef<Invoice>> {
    vec![
        col("Invoice #", 16, |inv| inv.invoice_number.clone()),
        col("Date", 12, |inv| {
            inv.invoice_date.format("%d/%m/%Y").to_string()
        }),
        col("Patient", 14, |inv| short_patient(inv)),
        col("Total", 12, |inv| format!("${:.2}", inv.total_amount)),
        col("Status", 16, |inv| inv.status.to_string()),
    ]
}

fn short_patient(invoice: &Invoice) -> String {
    invoice.patient_id.to_string().chars().take(8).collect()
}

impl HasFocus for InvoiceList {
    fn build(&self, builder: &mut FocusBuilder) {
        builder.leaf_widget(self);
    }

    fn focus(&self) -> FocusFlag {
        self.focus.clone()
    }

    fn area(&self) -> Rect {
        Rect::default()
    }
}

use std::rc::Rc;

use crate::ui::theme::Theme;
use crate::ui::widgets::{UnifiedColumnDef, UnifiedList, UnifiedListAction, UnifiedListConfig};
use crossterm::event::MouseEvent;
use opengp_domain::domain::billing::Payment;
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct PaymentList {
    pub payments: Vec<Payment>,
    pub selected_index: usize,
    pub scroll_offset: usize,
    pub hovered_index: Option<usize>,
    pub theme: Theme,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PaymentListAction {
    Select(usize),
    ViewDetail,
    Back,
    ContextMenu { x: u16, y: u16, payment_id: Uuid },
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
            scroll_offset: 0,
            hovered_index: None,
            theme,
        }
    }

    pub fn select_next(&mut self) {
        self.selected_index = (self.selected_index + 1).min(self.payments.len().saturating_sub(1));
    }

    pub fn select_prev(&mut self) {
        self.selected_index = self.selected_index.saturating_sub(1);
    }

    pub fn handle_mouse(&mut self, mouse: MouseEvent, area: Rect) -> Option<PaymentListAction> {
        let mut list = self.as_list();
        let action = list.handle_mouse(mouse, area);
        self.sync_from(&list);
        action.map(|a| match a {
            UnifiedListAction::Select(i) => PaymentListAction::Select(i),
            UnifiedListAction::Open(_) => PaymentListAction::ViewDetail,
            UnifiedListAction::ContextMenu { index, x, y } => {
                if let Some(payment) = self.payments.get(index) {
                    PaymentListAction::ContextMenu { x, y, payment_id: payment.id }
                } else {
                    PaymentListAction::Select(index)
                }
            }
            UnifiedListAction::New | UnifiedListAction::Edit(_) | UnifiedListAction::Delete(_) | UnifiedListAction::ToggleInactive => {
                PaymentListAction::Select(self.selected_index)
            }
        })
    }

    fn as_list(&self) -> UnifiedList<Payment> {
        let mut list = UnifiedList::new(
            self.payments.clone(),
            columns(),
            self.theme.clone(),
            UnifiedListConfig::new("Payments", 2, "No payments found."),
        );
        list.selected_index = self.selected_index;
        list.scroll_offset = self.scroll_offset;
        list.hovered_index = self.hovered_index;
        list
    }

    fn sync_from(&mut self, list: &UnifiedList<Payment>) {
        self.selected_index = list.selected_index;
        self.scroll_offset = list.scroll_offset;
        self.hovered_index = list.hovered_index;
    }
}

impl Widget for PaymentList {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.as_list().render(area, buf);
    }
}

fn col(
    title: &'static str,
    width: u16,
    render: impl Fn(&Payment) -> String + 'static,
) -> UnifiedColumnDef<Payment> {
    UnifiedColumnDef::new(title, width, render)
}

fn columns() -> Vec<UnifiedColumnDef<Payment>> {
    vec![
        col("Date", 12, |p| p.payment_date.format("%d/%m/%Y").to_string()),
        col("Invoice #", 12, |p| p.invoice_id.to_string()),
        col("Patient", 36, |p| p.patient_id.to_string()),
        col("Amount", 12, |p| format!("${:.2}", p.amount)),
        col("Method", 12, |p| p.payment_method.to_string()),
        col("Reference", 10, |p| p.reference.clone().unwrap_or_else(|| "-".to_string())),
    ]
}

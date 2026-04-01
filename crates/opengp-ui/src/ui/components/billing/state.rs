#![cfg(feature = "billing")]

use opengp_domain::domain::billing::{Invoice, MedicareClaim, Payment};
use uuid::Uuid;

use crate::ui::components::shared::PaginatedState;

#[derive(Debug, Clone, Default)]
pub enum BillingView {
    #[default]
    InvoiceList,
    InvoiceDetail(Uuid),
    ClaimList,
    PaymentList,
}

#[derive(Debug, Clone, Default)]
pub struct BillingState {
    pub view: BillingView,
    pub invoices: Vec<Invoice>,
    pub claims: Vec<MedicareClaim>,
    pub payments: Vec<Payment>,
    pub pagination: PaginatedState,
    pub loading: bool,
    pub error: Option<String>,
}

impl BillingState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn show_invoice_list(&mut self) {
        self.view = BillingView::InvoiceList;
    }

    pub fn show_claim_list(&mut self) {
        self.view = BillingView::ClaimList;
    }

    pub fn show_payment_list(&mut self) {
        self.view = BillingView::PaymentList;
    }

    pub fn show_invoice_detail(&mut self, id: Uuid) {
        self.view = BillingView::InvoiceDetail(id);
    }
}

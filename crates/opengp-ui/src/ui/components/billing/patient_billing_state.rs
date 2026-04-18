use opengp_domain::domain::billing::{Invoice, MedicareClaim, Payment};
use uuid::Uuid;

use crate::ui::components::shared::PaginatedState;

use super::BillingView;

#[derive(Debug, Clone)]
pub struct PatientBillingState {
    pub patient_id: Uuid,
    pub view: BillingView,
    pub invoices: Vec<Invoice>,
    pub claims: Vec<MedicareClaim>,
    pub payments: Vec<Payment>,
    pub pagination: PaginatedState,
    pub loading: bool,
    pub error: Option<String>,
}

impl PatientBillingState {
    pub fn new(patient_id: Uuid) -> Self {
        Self {
            patient_id,
            view: BillingView::default(),
            invoices: Vec::new(),
            claims: Vec::new(),
            payments: Vec::new(),
            pagination: PaginatedState::default(),
            loading: false,
            error: None,
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_patient_billing_state_new() {
        let patient_id = Uuid::new_v4();
        let state = PatientBillingState::new(patient_id);

        assert_eq!(state.patient_id, patient_id);
        assert!(matches!(state.view, BillingView::InvoiceList));
        assert!(state.invoices.is_empty());
        assert!(state.claims.is_empty());
        assert!(state.payments.is_empty());
        assert!(!state.loading);
        assert!(state.error.is_none());
    }

    #[test]
    fn test_patient_billing_state_show_invoice_list() {
        let patient_id = Uuid::new_v4();
        let mut state = PatientBillingState::new(patient_id);

        state.show_invoice_list();
        assert!(matches!(state.view, BillingView::InvoiceList));
    }

    #[test]
    fn test_patient_billing_state_show_claim_list() {
        let patient_id = Uuid::new_v4();
        let mut state = PatientBillingState::new(patient_id);

        state.show_claim_list();
        assert!(matches!(state.view, BillingView::ClaimList));
    }

    #[test]
    fn test_patient_billing_state_show_payment_list() {
        let patient_id = Uuid::new_v4();
        let mut state = PatientBillingState::new(patient_id);

        state.show_payment_list();
        assert!(matches!(state.view, BillingView::PaymentList));
    }

    #[test]
    fn test_patient_billing_state_show_invoice_detail() {
        let patient_id = Uuid::new_v4();
        let invoice_id = Uuid::new_v4();
        let mut state = PatientBillingState::new(patient_id);

        state.show_invoice_detail(invoice_id);
        assert!(matches!(state.view, BillingView::InvoiceDetail(id) if id == invoice_id));
    }
}

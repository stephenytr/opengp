use crate::ui::app::{App, PendingBillingSaveData};
use crate::ui::components::tabs::Tab;

impl App {
    pub fn set_pending_billing(&mut self, pending: PendingBillingSaveData) {
        self.pending_billing = Some(pending);
    }

    pub fn take_pending_billing(&mut self) -> Option<PendingBillingSaveData> {
        if !self.authenticated {
            return None;
        }

        self.pending_billing.take()
    }

    pub fn open_billing_invoice_detail(&mut self, invoice_id: uuid::Uuid) {
        self.billing_state.show_invoice_detail(invoice_id);
        self.tab_bar.select(Tab::Billing);
        self.previous_tab = Tab::Billing;
        self.refresh_status_bar();
        self.refresh_context();
    }
}

use crate::ui::app::{App, PendingBillingSaveData};

impl App {
    pub fn set_pending_billing(&mut self, _pending: PendingBillingSaveData) {
        todo!("Moved to workspace subtab in Task 28")
    }

    pub fn take_pending_billing(&mut self) -> Option<PendingBillingSaveData> {
        if !self.authenticated {
            return None;
        }

        todo!("Moved to workspace subtab in Task 28")
    }

    pub fn open_billing_invoice_detail(&mut self, _invoice_id: uuid::Uuid) {
        todo!("Moved to workspace subtab in Task 28")
    }
}

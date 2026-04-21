use crate::ui::app::{App, PendingBillingSaveData};

impl App {
    pub fn set_pending_billing(&mut self, pending: PendingBillingSaveData) {
        if !self.authenticated {
            return;
        }
        if let Some(workspace) = self.workspace_manager_mut().active_mut() {
            workspace.pending_billing = Some(pending);
        }
    }

    pub fn take_pending_billing(&mut self) -> Option<PendingBillingSaveData> {
        if !self.authenticated {
            return None;
        }
        self.workspace_manager_mut().active_mut().and_then(|w| w.pending_billing.take())
    }

    pub fn open_billing_invoice_detail(&mut self, invoice_id: uuid::Uuid) {
        if let Some(workspace) = self.workspace_manager_mut().active_mut() {
            if workspace.billing.is_none() {
                workspace.billing = Some(crate::ui::components::billing::PatientBillingState::new(workspace.patient_id));
            }
            if let Some(billing) = &mut workspace.billing {
                billing.show_invoice_detail(invoice_id);
            }
        }
    }
}

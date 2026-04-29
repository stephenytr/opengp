use crate::ui::app::{App, AppCommand, PendingBillingSaveData};
use crate::ui::components::billing::PatientBillingState;
use crate::ui::components::SubtabKind;

impl App {
    pub fn set_pending_billing(&mut self, pending: PendingBillingSaveData) {
        if !self.authenticated {
            return;
        }
        if let Some(workspace) = self.workspace_manager_mut().active_mut() {
            workspace.pending_billing = Some(pending);
        }
    }

    pub fn open_billing_invoice_detail(&mut self, invoice_id: uuid::Uuid) {
        if let Some(workspace) = self.workspace_manager_mut().active_mut() {
            if workspace.billing.is_none() {
                workspace.billing = Some(PatientBillingState::new(workspace.patient_id));
            }
            if let Some(billing) = &mut workspace.billing {
                billing.show_invoice_detail(invoice_id);
            }
        }
    }

    pub fn billing_state_mut(&mut self) -> Option<&mut PatientBillingState> {
        let workspace_manager = self.workspace_manager_mut();
        let needs_init = workspace_manager
            .active()
            .map(|w| w.billing.is_none())
            .unwrap_or(false);

        if needs_init {
            if let Some(workspace) = workspace_manager.active_mut() {
                workspace.billing = Some(PatientBillingState::new(workspace.patient_id));
            }
        }

        workspace_manager
            .active_mut()
            .and_then(|w| w.billing.as_mut())
    }

    pub fn load_billing_data(&mut self) {
        let patient_id = match self.workspace_manager().active() {
            Some(ws) => ws.patient_id,
            None => return,
        };
        let _ = self
            .command_tx
            .send(AppCommand::LoadBillingData { patient_id });
    }

    pub fn request_load_billing(&mut self) {
        let patient_id = match self.workspace_manager().active() {
            Some(ws) => ws.patient_id,
            None => return,
        };
        if self
            .workspace_manager()
            .is_subtab_loading(SubtabKind::Billing)
        {
            return;
        }
        let _ = self.command_tx.send(AppCommand::LoadPatientWorkspaceData {
            patient_id,
            subtab: SubtabKind::Billing,
        });
    }
}

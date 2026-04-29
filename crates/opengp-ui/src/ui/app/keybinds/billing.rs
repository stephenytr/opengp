use crate::ui::app::App;
use crate::ui::components::billing::BillingView;
use crate::ui::keybinds::{Action, KeyContext, KeybindRegistry};
use crossterm::event::KeyEvent;

impl App {
    pub(crate) fn handle_billing_keys(&mut self, key: KeyEvent) -> Action {
        let Some(workspace) = self.workspace_manager.active_mut() else {
            return Action::Unknown;
        };

        let Some(billing_state) = &mut workspace.billing else {
            return Action::Unknown;
        };

        let context = match billing_state.view {
            BillingView::InvoiceDetail(_) => KeyContext::BillingForm,
            _ => KeyContext::Billing,
        };

        let registry = KeybindRegistry::global();
        let Some(keybind) = registry.lookup(key, context) else {
            return Action::Unknown;
        };

        match keybind.action {
            Action::NextBillingView => match billing_state.view {
                BillingView::InvoiceList => billing_state.show_claim_list(),
                BillingView::ClaimList => billing_state.show_payment_list(),
                BillingView::PaymentList => billing_state.show_invoice_list(),
                BillingView::InvoiceDetail(_) => {}
            },
            Action::PrevBillingView => match billing_state.view {
                BillingView::InvoiceList => billing_state.show_payment_list(),
                BillingView::ClaimList => billing_state.show_invoice_list(),
                BillingView::PaymentList => billing_state.show_claim_list(),
                BillingView::InvoiceDetail(_) => {}
            },
            Action::NavigateUp | Action::NavigateDown => {}
            Action::Enter => {}
            Action::NewInvoice => {}
            Action::EditInvoice => {}
            Action::ProcessPayment => {}
            Action::VoidInvoice => {}
            Action::GenerateReceipt => {}
            Action::Escape => {
                if matches!(billing_state.view, BillingView::InvoiceDetail(_)) {
                    billing_state.show_invoice_list();
                }
            }
            _ => {}
        }

        keybind.action.clone()
    }
}

#![cfg(feature = "billing")]

use crate::ui::app::App;
use crate::ui::components::billing::BillingView;
use crate::ui::keybinds::Action;
use crossterm::event::{KeyCode, KeyEvent};

impl App {
    pub(crate) fn handle_billing_keys(&mut self, key: KeyEvent) -> Action {
        if key.code == KeyCode::Right {
            match self.billing_state.view {
                BillingView::InvoiceList => {
                    self.billing_state.show_claim_list();
                    return Action::Enter;
                }
                BillingView::ClaimList => {
                    self.billing_state.show_payment_list();
                    return Action::Enter;
                }
                BillingView::PaymentList => {
                    self.billing_state.show_invoice_list();
                    return Action::Enter;
                }
                BillingView::InvoiceDetail(_) => {}
            }
        }

        if key.code == KeyCode::Left {
            match self.billing_state.view {
                BillingView::InvoiceList => {
                    self.billing_state.show_payment_list();
                    return Action::Enter;
                }
                BillingView::ClaimList => {
                    self.billing_state.show_invoice_list();
                    return Action::Enter;
                }
                BillingView::PaymentList => {
                    self.billing_state.show_claim_list();
                    return Action::Enter;
                }
                BillingView::InvoiceDetail(_) => {}
            }
        }

        match self.billing_state.view {
            BillingView::InvoiceList => {
                if key.code == KeyCode::Up || key.code == KeyCode::Char('k') {
                    return Action::Enter;
                }
                if key.code == KeyCode::Down || key.code == KeyCode::Char('j') {
                    return Action::Enter;
                }
                if key.code == KeyCode::Enter {
                    return Action::Enter;
                }
                if key.code == KeyCode::Char('n') {
                    return Action::Enter;
                }
            }
            BillingView::ClaimList => {
                if key.code == KeyCode::Up || key.code == KeyCode::Char('k') {
                    return Action::Enter;
                }
                if key.code == KeyCode::Down || key.code == KeyCode::Char('j') {
                    return Action::Enter;
                }
                if key.code == KeyCode::Enter {
                    return Action::Enter;
                }
                if key.code == KeyCode::Char('n') {
                    return Action::Enter;
                }
            }
            BillingView::PaymentList => {
                if key.code == KeyCode::Up || key.code == KeyCode::Char('k') {
                    return Action::Enter;
                }
                if key.code == KeyCode::Down || key.code == KeyCode::Char('j') {
                    return Action::Enter;
                }
                if key.code == KeyCode::Enter {
                    return Action::Enter;
                }
            }
            BillingView::InvoiceDetail(_) => {
                if key.code == KeyCode::Esc {
                    self.billing_state.show_invoice_list();
                    return Action::Enter;
                }
                if key.code == KeyCode::Char('e') {
                    return Action::Enter;
                }
                if key.code == KeyCode::Char('p') {
                    return Action::Enter;
                }
                if key.code == KeyCode::Char('x') {
                    return Action::Enter;
                }
                if key.code == KeyCode::Char('r') {
                    return Action::Enter;
                }
            }
        }

        Action::Unknown
    }
}

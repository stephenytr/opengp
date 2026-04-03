use std::sync::Arc;

use opengp_domain::domain::billing::{
    BillingError, BillingService, BillingType, Invoice, MedicareClaim, Payment, ServiceError,
};
use uuid::Uuid;

use super::shared::{UiResult, UiServiceError};

pub struct BillingUiService {
    service: Arc<BillingService>,
}

impl BillingUiService {
    pub fn new(service: Arc<BillingService>) -> Self {
        Self { service }
    }

    pub async fn list_invoices_for_patient(&self, patient_id: Uuid) -> UiResult<Vec<Invoice>> {
        self.service
            .repository
            .find_invoices_by_patient(patient_id)
            .await
            .map_err(|err| UiServiceError::Repository(err.to_string()))
    }

    pub async fn get_invoice(&self, id: Uuid) -> UiResult<Invoice> {
        self.service
            .find_invoice_by_id(id)
            .await
            .map_err(Self::map_billing_error)?
            .ok_or(UiServiceError::NotFound(format!("Invoice not found: {id}")))
    }

    pub async fn create_invoice(
        &self,
        consultation_id: Uuid,
        mbs_items: Vec<(String, f64, bool)>,
        billing_type: BillingType,
        created_by: Uuid,
    ) -> UiResult<Invoice> {
        self.service
            .create_invoice_from_consultation(consultation_id, mbs_items, billing_type, created_by)
            .await
            .map_err(Self::map_billing_error)
    }

    pub async fn record_payment(
        &self,
        invoice_id: Uuid,
        amount: f64,
        created_by: Uuid,
    ) -> UiResult<(Payment, Invoice)> {
        self.service
            .record_cash_payment(invoice_id, amount, created_by)
            .await
            .map_err(Self::map_billing_error)
    }

    pub async fn list_claims_for_patient(&self, patient_id: Uuid) -> UiResult<Vec<MedicareClaim>> {
        self.service
            .repository
            .find_claims_by_patient(patient_id)
            .await
            .map_err(|err| UiServiceError::Repository(err.to_string()))
    }

    pub async fn prepare_claim(&self, invoice_id: Uuid) -> UiResult<String> {
        self.service
            .prepare_claim_json(invoice_id)
            .await
            .map_err(Self::map_billing_error)
    }

    pub async fn get_patient_balance(&self, patient_id: Uuid) -> UiResult<f64> {
        self.service
            .find_patient_balance(patient_id)
            .await
            .map_err(Self::map_billing_error)
    }

    fn map_billing_error(error: BillingError) -> UiServiceError {
        match error {
            ServiceError::InvoiceNotFound(id) => UiServiceError::NotFound(format!("Invoice not found: {id}")),
            ServiceError::ClaimNotFound(id) => UiServiceError::NotFound(format!("Claim not found: {id}")),
            ServiceError::ConsultationNotFound(id) => {
                UiServiceError::NotFound(format!("Consultation not found: {id}"))
            }
            ServiceError::Validation(err) => UiServiceError::Validation(err.to_string()),
            ServiceError::Repository(err) => UiServiceError::Repository(err.to_string()),
        }
    }
}

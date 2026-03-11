use async_trait::async_trait;
use uuid::Uuid;

use super::error::RepositoryError;
use super::model::{ClaimStatus, Invoice, MedicareClaim, Payment};

#[async_trait]
pub trait BillingRepository: Send + Sync {
    async fn find_invoice_by_id(&self, id: Uuid) -> Result<Option<Invoice>, RepositoryError>;
    async fn find_invoices_by_patient(
        &self,
        patient_id: Uuid,
    ) -> Result<Vec<Invoice>, RepositoryError>;
    async fn create_invoice(&self, invoice: Invoice) -> Result<Invoice, RepositoryError>;
    async fn update_invoice(&self, invoice: Invoice) -> Result<Invoice, RepositoryError>;

    async fn find_claim_by_id(&self, id: Uuid) -> Result<Option<MedicareClaim>, RepositoryError>;
    async fn create_claim(&self, claim: MedicareClaim) -> Result<MedicareClaim, RepositoryError>;
    async fn find_claims_by_status(
        &self,
        status: ClaimStatus,
    ) -> Result<Vec<MedicareClaim>, RepositoryError>;

    async fn record_payment(&self, payment: Payment) -> Result<Payment, RepositoryError>;
}

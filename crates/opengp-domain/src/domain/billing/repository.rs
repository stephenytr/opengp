use async_trait::async_trait;
use uuid::Uuid;

use super::error::RepositoryError;
use super::model::{ClaimStatus, Invoice, MedicareClaim, Payment};

/// Repository abstraction for billing, invoices, claims and payments.
#[async_trait]
pub trait BillingRepository: Send + Sync {
    /// Look up a single invoice by identifier.
    async fn find_invoice_by_id(&self, id: Uuid) -> Result<Option<Invoice>, RepositoryError>;

    /// Fetch all invoices for a given patient.
    async fn find_invoices_by_patient(
        &self,
        patient_id: Uuid,
    ) -> Result<Vec<Invoice>, RepositoryError>;

    /// Persist a newly created invoice.
    async fn create_invoice(&self, invoice: Invoice) -> Result<Invoice, RepositoryError>;

    /// Persist changes to an existing invoice.
    async fn update_invoice(&self, invoice: Invoice) -> Result<Invoice, RepositoryError>;

    /// Look up a single Medicare claim by identifier.
    async fn find_claim_by_id(&self, id: Uuid) -> Result<Option<MedicareClaim>, RepositoryError>;

    /// Persist a newly created Medicare claim.
    async fn create_claim(&self, claim: MedicareClaim) -> Result<MedicareClaim, RepositoryError>;

    /// Fetch claims filtered by processing status.
    async fn find_claims_by_status(
        &self,
        status: ClaimStatus,
    ) -> Result<Vec<MedicareClaim>, RepositoryError>;

    /// Record a payment against an invoice.
    async fn record_payment(&self, payment: Payment) -> Result<Payment, RepositoryError>;
}

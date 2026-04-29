use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::error::RepositoryError;
use super::model::{ClaimStatus, Invoice, InvoiceItem, InvoiceStatus, MedicareClaim, Payment};

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

    async fn find_invoices_by_date_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<Invoice>, RepositoryError>;

    async fn find_invoices_by_status(
        &self,
        status: InvoiceStatus,
    ) -> Result<Vec<Invoice>, RepositoryError>;

    /// Persist a newly created invoice.
    async fn create_invoice(&self, invoice: Invoice) -> Result<Invoice, RepositoryError>;

    /// Persist changes to an existing invoice.
    async fn update_invoice(&self, invoice: Invoice) -> Result<Invoice, RepositoryError>;

    async fn update_invoice_status(
        &self,
        id: Uuid,
        status: InvoiceStatus,
    ) -> Result<(), RepositoryError>;

    /// Look up a single Medicare claim by identifier.
    async fn find_claim_by_id(&self, id: Uuid) -> Result<Option<MedicareClaim>, RepositoryError>;

    /// Persist a newly created Medicare claim.
    async fn create_claim(&self, claim: MedicareClaim) -> Result<MedicareClaim, RepositoryError>;

    /// Fetch claims filtered by processing status.
    async fn find_claims_by_status(
        &self,
        status: ClaimStatus,
    ) -> Result<Vec<MedicareClaim>, RepositoryError>;

    async fn find_claims_by_patient(
        &self,
        patient_id: Uuid,
    ) -> Result<Vec<MedicareClaim>, RepositoryError>;

    async fn find_claims_by_date_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<MedicareClaim>, RepositoryError>;

    async fn update_claim_status(
        &self,
        id: Uuid,
        status: ClaimStatus,
    ) -> Result<(), RepositoryError>;

    /// Record a payment against an invoice.
    async fn record_payment(&self, payment: Payment) -> Result<Payment, RepositoryError>;

    async fn find_payments_by_invoice(
        &self,
        invoice_id: Uuid,
    ) -> Result<Vec<Payment>, RepositoryError>;

    async fn find_payments_by_patient(
        &self,
        patient_id: Uuid,
    ) -> Result<Vec<Payment>, RepositoryError>;

    async fn find_payments_by_date_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<Payment>, RepositoryError>;

    async fn find_invoice_items(
        &self,
        invoice_id: Uuid,
    ) -> Result<Vec<InvoiceItem>, RepositoryError>;

    async fn next_invoice_number(&self, year: i32) -> Result<String, RepositoryError>;
}

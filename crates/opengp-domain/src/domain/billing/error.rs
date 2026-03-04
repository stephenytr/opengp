use thiserror::Error;
use uuid::Uuid;

pub use crate::domain::error::RepositoryError;

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Invoice must contain at least one item")]
    EmptyInvoiceItems,

    #[error("Payment amount must be greater than zero")]
    InvalidPaymentAmount,
}

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("Invoice not found: {0}")]
    InvoiceNotFound(Uuid),

    #[error("Claim not found: {0}")]
    ClaimNotFound(Uuid),

    #[error("Validation error: {0}")]
    Validation(#[from] ValidationError),

    #[error("Repository error: {0}")]
    Repository(#[from] RepositoryError),
}

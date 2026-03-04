use std::sync::Arc;
use uuid::Uuid;

use crate::service;

use super::error::{ServiceError, ValidationError};
use super::model::{ClaimStatus, Invoice, MedicareClaim, Payment};
use super::repository::BillingRepository;

service! {
    BillingService {
        repository: Arc<dyn BillingRepository>,
    }
}

impl BillingService {
    fn validate_invoice(&self, invoice: &Invoice) -> Result<(), ServiceError> {
        if invoice.items.is_empty() {
            return Err(ValidationError::EmptyInvoiceItems.into());
        }

        Ok(())
    }

    fn validate_payment(&self, payment: &Payment) -> Result<(), ServiceError> {
        if payment.amount <= 0.0 {
            return Err(ValidationError::InvalidPaymentAmount.into());
        }

        Ok(())
    }

    pub async fn create_invoice(&self, mut invoice: Invoice) -> Result<Invoice, ServiceError> {
        self.validate_invoice(&invoice)?;
        invoice.calculate_totals();
        Ok(self.repository.create_invoice(invoice).await?)
    }

    pub async fn submit_claim(&self, claim: MedicareClaim) -> Result<MedicareClaim, ServiceError> {
        Ok(self.repository.create_claim(claim).await?)
    }

    pub async fn find_claims_by_status(
        &self,
        status: ClaimStatus,
    ) -> Result<Vec<MedicareClaim>, ServiceError> {
        Ok(self.repository.find_claims_by_status(status).await?)
    }

    pub async fn record_payment(&self, payment: Payment) -> Result<Payment, ServiceError> {
        self.validate_payment(&payment)?;
        Ok(self.repository.record_payment(payment).await?)
    }

    pub async fn find_invoice_by_id(&self, id: Uuid) -> Result<Option<Invoice>, ServiceError> {
        Ok(self.repository.find_invoice_by_id(id).await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::billing::{
        BillingType, ClaimType, InvoiceItem, InvoiceStatus, MBSItem, PaymentMethod, RepositoryError,
    };
    use async_trait::async_trait;
    use chrono::{NaiveDate, Utc};

    struct MockBillingRepository {
        invoices: Vec<Invoice>,
        claims: Vec<MedicareClaim>,
    }

    #[async_trait]
    impl BillingRepository for MockBillingRepository {
        async fn find_invoice_by_id(&self, id: Uuid) -> Result<Option<Invoice>, RepositoryError> {
            Ok(self.invoices.iter().find(|invoice| invoice.id == id).cloned())
        }

        async fn find_invoices_by_patient(
            &self,
            patient_id: Uuid,
        ) -> Result<Vec<Invoice>, RepositoryError> {
            Ok(self
                .invoices
                .iter()
                .filter(|invoice| invoice.patient_id == patient_id)
                .cloned()
                .collect())
        }

        async fn create_invoice(&self, invoice: Invoice) -> Result<Invoice, RepositoryError> {
            Ok(invoice)
        }

        async fn update_invoice(&self, invoice: Invoice) -> Result<Invoice, RepositoryError> {
            Ok(invoice)
        }

        async fn find_claim_by_id(&self, id: Uuid) -> Result<Option<MedicareClaim>, RepositoryError> {
            Ok(self.claims.iter().find(|claim| claim.id == id).cloned())
        }

        async fn create_claim(&self, claim: MedicareClaim) -> Result<MedicareClaim, RepositoryError> {
            Ok(claim)
        }

        async fn find_claims_by_status(
            &self,
            status: ClaimStatus,
        ) -> Result<Vec<MedicareClaim>, RepositoryError> {
            Ok(self
                .claims
                .iter()
                .filter(|claim| claim.status == status)
                .cloned()
                .collect())
        }

        async fn record_payment(&self, payment: Payment) -> Result<Payment, RepositoryError> {
            Ok(payment)
        }
    }

    fn new_service(invoices: Vec<Invoice>, claims: Vec<MedicareClaim>) -> BillingService {
        BillingService::new(Arc::new(MockBillingRepository { invoices, claims }))
    }

    fn test_invoice() -> Invoice {
        let now = Utc::now();
        Invoice {
            id: Uuid::new_v4(),
            patient_id: Uuid::new_v4(),
            practitioner_id: Uuid::new_v4(),
            consultation_id: None,
            invoice_number: "INV-1001".to_string(),
            invoice_date: now.date_naive(),
            due_date: Some(now.date_naive()),
            items: vec![InvoiceItem {
                id: Uuid::new_v4(),
                description: "Standard consult".to_string(),
                item_code: Some("23".to_string()),
                quantity: 1,
                unit_price: 89.0,
                amount: 89.0,
            }],
            subtotal: 0.0,
            gst_amount: 0.0,
            total_amount: 0.0,
            amount_paid: 0.0,
            amount_outstanding: 0.0,
            status: InvoiceStatus::Issued,
            billing_type: BillingType::PrivateBilling,
            notes: None,
            created_at: now,
            updated_at: now,
            created_by: Uuid::new_v4(),
            updated_by: None,
        }
    }

    fn test_claim(status: ClaimStatus) -> MedicareClaim {
        MedicareClaim {
            id: Uuid::new_v4(),
            patient_id: Uuid::new_v4(),
            practitioner_id: Uuid::new_v4(),
            consultation_id: None,
            invoice_id: None,
            claim_reference: Some("MCL-1".to_string()),
            service_date: NaiveDate::from_ymd_opt(2026, 3, 1).expect("valid date"),
            items: vec![MBSItem {
                item_number: "23".to_string(),
                description: "Level B".to_string(),
                fee: 41.2,
                benefit: 41.2,
                quantity: 1,
            }],
            total_claimed: 41.2,
            total_benefit: 41.2,
            patient_contribution: 0.0,
            claim_type: ClaimType::BulkBill,
            status,
            submitted_at: Some(Utc::now()),
            processed_at: None,
            rejection_reason: None,
            created_at: Utc::now(),
            created_by: Uuid::new_v4(),
        }
    }

    fn test_payment(invoice_id: Uuid) -> Payment {
        Payment {
            id: Uuid::new_v4(),
            invoice_id,
            patient_id: Uuid::new_v4(),
            payment_date: Utc::now(),
            amount: 10.0,
            payment_method: PaymentMethod::EFTPOS,
            reference: None,
            notes: None,
            created_at: Utc::now(),
            created_by: Uuid::new_v4(),
        }
    }

    #[tokio::test]
    async fn test_create_invoice_calculates_totals() {
        let service = new_service(vec![], vec![]);
        let result = service.create_invoice(test_invoice()).await;

        assert!(result.is_ok());
        let invoice = result.expect("invoice should be created");
        assert_eq!(invoice.subtotal, 89.0);
        assert_eq!(invoice.gst_amount, 8.9);
        assert_eq!(invoice.total_amount, 97.9);
    }

    #[tokio::test]
    async fn test_record_payment_rejects_zero_amount() {
        let service = new_service(vec![], vec![]);
        let mut payment = test_payment(Uuid::new_v4());
        payment.amount = 0.0;

        let result = service.record_payment(payment).await;

        assert!(matches!(
            result,
            Err(ServiceError::Validation(ValidationError::InvalidPaymentAmount))
        ));
    }

    #[tokio::test]
    async fn test_find_claims_by_status_filters_correctly() {
        let service = new_service(
            vec![],
            vec![test_claim(ClaimStatus::Submitted), test_claim(ClaimStatus::Paid)],
        );

        let result = service.find_claims_by_status(ClaimStatus::Paid).await;

        assert!(result.is_ok());
        let claims = result.expect("claims should be returned");
        assert_eq!(claims.len(), 1);
        assert_eq!(claims[0].status, ClaimStatus::Paid);
    }
}

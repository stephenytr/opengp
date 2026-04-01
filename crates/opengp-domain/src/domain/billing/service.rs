use std::sync::Arc;
use chrono::{Datelike, Utc};
use serde_json::json;
use uuid::Uuid;

use crate::service;
use crate::domain::clinical::ConsultationRepository;

use super::error::{BillingError, RepositoryError, ServiceError, ValidationError};
use super::model::{
    BillingType, ClaimStatus, Invoice, InvoiceItem, InvoiceStatus, MedicareClaim, Payment,
    PaymentMethod,
};
use super::repository::BillingRepository;

// Application service for billing, Medicare claims and payments.
service! {
    BillingService {
        repository: Arc<dyn BillingRepository>,
        clinical_repo: Arc<dyn ConsultationRepository>,
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

    /// Create a new invoice and calculate totals before persistence.
    ///
    /// # Errors
    /// Returns `ServiceError::Validation` if the invoice data is invalid or `ServiceError::Repository`
    /// if the repository fails to store the invoice.
    pub async fn create_invoice(&self, mut invoice: Invoice) -> Result<Invoice, ServiceError> {
        self.validate_invoice(&invoice)?;
        invoice.calculate_totals();
        Ok(self.repository.create_invoice(invoice).await?)
    }

    /// Submit a Medicare claim for processing.
    ///
    /// # Errors
    /// Returns `ServiceError::Repository` if the claim cannot be persisted.
    pub async fn submit_claim(&self, claim: MedicareClaim) -> Result<MedicareClaim, ServiceError> {
        Ok(self.repository.create_claim(claim).await?)
    }

    /// Find Medicare claims by processing status, for example Submitted or Paid.
    ///
    /// # Errors
    /// Returns `ServiceError::Repository` if the repository query fails.
    pub async fn find_claims_by_status(
        &self,
        status: ClaimStatus,
    ) -> Result<Vec<MedicareClaim>, ServiceError> {
        Ok(self.repository.find_claims_by_status(status).await?)
    }

    /// Record a payment against an invoice after validating the amount.
    ///
    /// # Errors
    /// Returns `ServiceError::Validation` if the payment amount is invalid or
    /// `ServiceError::Repository` if persistence fails.
    pub async fn record_payment(&self, payment: Payment) -> Result<Payment, ServiceError> {
        self.validate_payment(&payment)?;
        Ok(self.repository.record_payment(payment).await?)
    }

    /// Look up an invoice by identifier.
    ///
    /// # Errors
    /// Returns `ServiceError::Repository` if the repository lookup fails.
    pub async fn find_invoice_by_id(&self, id: Uuid) -> Result<Option<Invoice>, ServiceError> {
        Ok(self.repository.find_invoice_by_id(id).await?)
    }

    pub async fn create_invoice_from_consultation(
        &self,
        consultation_id: Uuid,
        mbs_items: Vec<(String, f64, bool)>,
        billing_type: BillingType,
        created_by: Uuid,
    ) -> Result<Invoice, BillingError> {
        let consultation = self
            .clinical_repo
            .find_by_id(consultation_id)
            .await
            .map_err(|err| ServiceError::Repository(RepositoryError::Database(err.to_string())))?
            .ok_or(ServiceError::ConsultationNotFound(consultation_id))?;

        if !consultation.is_signed {
            return Err(ValidationError::ConsultationNotSigned.into());
        }

        let now = Utc::now();
        let invoice_items = mbs_items
            .into_iter()
            .map(|(item_code, unit_price, is_gst_free)| InvoiceItem {
                id: Uuid::new_v4(),
                description: format!("MBS Item {}", item_code),
                item_code: Some(item_code),
                quantity: 1,
                unit_price,
                amount: unit_price,
                is_gst_free,
            })
            .collect();

        let year = now.year();
        let invoice_number = self.repository.next_invoice_number(year).await?;
        let mut invoice = Invoice {
            id: Uuid::new_v4(),
            patient_id: consultation.patient_id,
            practitioner_id: consultation.practitioner_id,
            consultation_id: Some(consultation_id),
            invoice_number,
            invoice_date: consultation.consultation_date.date_naive(),
            due_date: Some(now.date_naive()),
            items: invoice_items,
            subtotal: 0.0,
            gst_amount: 0.0,
            total_amount: 0.0,
            amount_paid: 0.0,
            amount_outstanding: 0.0,
            status: InvoiceStatus::Issued,
            billing_type,
            notes: None,
            created_at: now,
            updated_at: now,
            created_by,
            updated_by: None,
        };

        self.validate_invoice(&invoice)?;
        invoice.calculate_totals();
        Ok(self.repository.create_invoice(invoice).await?)
    }

    pub async fn record_cash_payment(
        &self,
        invoice_id: Uuid,
        amount: f64,
        created_by: Uuid,
    ) -> Result<(Payment, Invoice), BillingError> {
        if amount <= 0.0 {
            return Err(ValidationError::InvalidPaymentAmount.into());
        }

        let mut invoice = self
            .repository
            .find_invoice_by_id(invoice_id)
            .await?
            .ok_or(ServiceError::InvoiceNotFound(invoice_id))?;

        if !matches!(
            invoice.status,
            InvoiceStatus::Issued | InvoiceStatus::PartiallyPaid
        ) {
            return Err(ValidationError::InvoiceNotPayable.into());
        }

        let payment = Payment {
            id: Uuid::new_v4(),
            invoice_id,
            patient_id: invoice.patient_id,
            payment_date: Utc::now(),
            amount,
            payment_method: PaymentMethod::Cash,
            reference: None,
            notes: None,
            created_at: Utc::now(),
            created_by,
        };

        let payment = self.repository.record_payment(payment).await?;

        invoice.amount_paid += amount;
        invoice.amount_outstanding -= amount;

        if invoice.amount_outstanding <= 0.0 {
            invoice.amount_outstanding = 0.0;
            invoice.status = InvoiceStatus::Paid;
        } else {
            invoice.status = InvoiceStatus::PartiallyPaid;
        }

        invoice.updated_at = Utc::now();
        invoice.updated_by = Some(created_by);

        let updated_invoice = self.repository.update_invoice(invoice).await?;
        self.repository
            .update_invoice_status(updated_invoice.id, updated_invoice.status)
            .await?;

        Ok((payment, updated_invoice))
    }

    pub async fn prepare_claim_json(&self, invoice_id: Uuid) -> Result<String, BillingError> {
        let invoice = self
            .repository
            .find_invoice_by_id(invoice_id)
            .await?
            .ok_or(ServiceError::InvoiceNotFound(invoice_id))?;

        let items: Vec<_> = invoice
            .items
            .iter()
            .map(|item| {
                json!({
                    "item_code": item.item_code,
                    "description": item.description,
                    "amount": item.amount,
                    "unit_price": item.unit_price,
                    "quantity": item.quantity,
                })
            })
            .collect();

        let payload = json!({
            "invoice_id": invoice.id,
            "patient_id": invoice.patient_id,
            "practitioner_id": invoice.practitioner_id,
            "service_date": invoice.invoice_date,
            "items": items,
            "total_claimed": invoice.total_amount,
        });

        Ok(serde_json::to_string_pretty(&payload)
            .map_err(|err| ServiceError::Validation(ValidationError::ClaimSerializationFailed(err.to_string())))?)
    }

    pub async fn find_patient_balance(&self, patient_id: Uuid) -> Result<f64, BillingError> {
        let invoices = self.repository.find_invoices_by_patient(patient_id).await?;
        Ok(invoices
            .iter()
            .filter(|invoice| invoice.status != InvoiceStatus::Cancelled)
            .map(|invoice| invoice.amount_outstanding)
            .sum())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::billing::{
        BillingType, ClaimType, InvoiceItem, InvoiceStatus, MBSItem, RepositoryError,
    };
    use crate::domain::clinical::{Consultation, RepositoryError as ClinicalRepositoryError};
    use async_trait::async_trait;
    use chrono::{DateTime, NaiveDate, Utc};
    use std::sync::Mutex;

    struct MockBillingRepository {
        invoices: Mutex<Vec<Invoice>>,
        claims: Vec<MedicareClaim>,
    }

    struct MockConsultationRepository {
        consultations: Vec<Consultation>,
    }

    #[async_trait]
    impl BillingRepository for MockBillingRepository {
        async fn find_invoice_by_id(&self, id: Uuid) -> Result<Option<Invoice>, RepositoryError> {
            Ok(self
                .invoices
                .lock()
                .expect("invoices lock should not be poisoned")
                .iter()
                .find(|invoice| invoice.id == id)
                .cloned())
        }

        async fn find_invoices_by_patient(
            &self,
            patient_id: Uuid,
        ) -> Result<Vec<Invoice>, RepositoryError> {
            Ok(self
                .invoices
                .lock()
                .expect("invoices lock should not be poisoned")
                .iter()
                .filter(|invoice| invoice.patient_id == patient_id)
                .cloned()
                .collect())
        }

        async fn find_invoices_by_date_range(
            &self,
            _start: DateTime<Utc>,
            _end: DateTime<Utc>,
        ) -> Result<Vec<Invoice>, RepositoryError> {
            Ok(vec![])
        }

        async fn find_invoices_by_status(
            &self,
            _status: InvoiceStatus,
        ) -> Result<Vec<Invoice>, RepositoryError> {
            Ok(vec![])
        }

        async fn create_invoice(&self, invoice: Invoice) -> Result<Invoice, RepositoryError> {
            self.invoices
                .lock()
                .expect("invoices lock should not be poisoned")
                .push(invoice.clone());
            Ok(invoice)
        }

        async fn update_invoice(&self, invoice: Invoice) -> Result<Invoice, RepositoryError> {
            let mut invoices = self
                .invoices
                .lock()
                .expect("invoices lock should not be poisoned");
            if let Some(existing) = invoices.iter_mut().find(|existing| existing.id == invoice.id) {
                *existing = invoice.clone();
            }
            Ok(invoice)
        }

        async fn update_invoice_status(
            &self,
            id: Uuid,
            status: InvoiceStatus,
        ) -> Result<(), RepositoryError> {
            let mut invoices = self
                .invoices
                .lock()
                .expect("invoices lock should not be poisoned");
            if let Some(existing) = invoices.iter_mut().find(|invoice| invoice.id == id) {
                existing.status = status;
            }
            Ok(())
        }

        async fn find_claim_by_id(
            &self,
            id: Uuid,
        ) -> Result<Option<MedicareClaim>, RepositoryError> {
            Ok(self.claims.iter().find(|claim| claim.id == id).cloned())
        }

        async fn create_claim(
            &self,
            claim: MedicareClaim,
        ) -> Result<MedicareClaim, RepositoryError> {
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

        async fn find_claims_by_patient(
            &self,
            _patient_id: Uuid,
        ) -> Result<Vec<MedicareClaim>, RepositoryError> {
            Ok(vec![])
        }

        async fn find_claims_by_date_range(
            &self,
            _start: DateTime<Utc>,
            _end: DateTime<Utc>,
        ) -> Result<Vec<MedicareClaim>, RepositoryError> {
            Ok(vec![])
        }

        async fn update_claim_status(
            &self,
            _id: Uuid,
            _status: ClaimStatus,
        ) -> Result<(), RepositoryError> {
            Ok(())
        }

        async fn record_payment(&self, payment: Payment) -> Result<Payment, RepositoryError> {
            Ok(payment)
        }

        async fn find_payments_by_invoice(
            &self,
            _invoice_id: Uuid,
        ) -> Result<Vec<Payment>, RepositoryError> {
            Ok(vec![])
        }

        async fn find_payments_by_patient(
            &self,
            _patient_id: Uuid,
        ) -> Result<Vec<Payment>, RepositoryError> {
            Ok(vec![])
        }

        async fn find_payments_by_date_range(
            &self,
            _start: DateTime<Utc>,
            _end: DateTime<Utc>,
        ) -> Result<Vec<Payment>, RepositoryError> {
            Ok(vec![])
        }

        async fn find_invoice_items(
            &self,
            _invoice_id: Uuid,
        ) -> Result<Vec<InvoiceItem>, RepositoryError> {
            Ok(vec![])
        }

        async fn next_invoice_number(&self, year: i32) -> Result<String, RepositoryError> {
            Ok(format!("INV-{}-00001", year))
        }
    }

    #[async_trait]
    impl ConsultationRepository for MockConsultationRepository {
        async fn find_by_id(
            &self,
            id: Uuid,
        ) -> Result<Option<Consultation>, ClinicalRepositoryError> {
            Ok(self
                .consultations
                .iter()
                .find(|consultation| consultation.id == id)
                .cloned())
        }

        async fn find_by_patient(
            &self,
            patient_id: Uuid,
            _limit: Option<i64>,
        ) -> Result<Vec<Consultation>, ClinicalRepositoryError> {
            Ok(self
                .consultations
                .iter()
                .filter(|consultation| consultation.patient_id == patient_id)
                .cloned()
                .collect())
        }

        async fn find_by_date_range(
            &self,
            _patient_id: Uuid,
            _start: DateTime<Utc>,
            _end: DateTime<Utc>,
        ) -> Result<Vec<Consultation>, ClinicalRepositoryError> {
            Ok(vec![])
        }

        async fn create(
            &self,
            consultation: Consultation,
        ) -> Result<Consultation, ClinicalRepositoryError> {
            Ok(consultation)
        }

        async fn update(
            &self,
            consultation: Consultation,
        ) -> Result<Consultation, ClinicalRepositoryError> {
            Ok(consultation)
        }

        async fn sign(&self, _id: Uuid, _user_id: Uuid) -> Result<(), ClinicalRepositoryError> {
            Ok(())
        }
    }

    fn new_service(
        invoices: Vec<Invoice>,
        claims: Vec<MedicareClaim>,
        consultations: Vec<Consultation>,
    ) -> BillingService {
        BillingService::new(
            Arc::new(MockBillingRepository {
                invoices: Mutex::new(invoices),
                claims,
            }),
            Arc::new(MockConsultationRepository { consultations }),
        )
    }

    fn test_consultation(is_signed: bool) -> Consultation {
        let mut consultation = Consultation::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            None,
            Uuid::new_v4(),
        );
        if is_signed {
            consultation.sign(Uuid::new_v4());
        }
        consultation
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
                is_gst_free: true,
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
            payment_method: PaymentMethod::Cash,
            reference: None,
            notes: None,
            created_at: Utc::now(),
            created_by: Uuid::new_v4(),
        }
    }

    #[tokio::test]
    async fn test_create_invoice_calculates_totals() {
        let service = new_service(vec![], vec![], vec![]);
        let result = service.create_invoice(test_invoice()).await;

        assert!(result.is_ok());
        let invoice = result.expect("invoice should be created");
        assert_eq!(invoice.subtotal, 89.0);
        assert_eq!(invoice.gst_amount, 0.0);
        assert_eq!(invoice.total_amount, 89.0);
    }

    #[tokio::test]
    async fn test_mixed_invoice_gst() {
        let service = new_service(vec![], vec![], vec![]);
        let now = Utc::now();
        let invoice = Invoice {
            id: Uuid::new_v4(),
            patient_id: Uuid::new_v4(),
            practitioner_id: Uuid::new_v4(),
            consultation_id: None,
            invoice_number: "INV-1002".to_string(),
            invoice_date: now.date_naive(),
            due_date: Some(now.date_naive()),
            items: vec![
                InvoiceItem {
                    id: Uuid::new_v4(),
                    description: "Medical consultation".to_string(),
                    item_code: Some("23".to_string()),
                    quantity: 1,
                    unit_price: 89.0,
                    amount: 89.0,
                    is_gst_free: true,
                },
                InvoiceItem {
                    id: Uuid::new_v4(),
                    description: "Supplies".to_string(),
                    item_code: Some("SUPP-001".to_string()),
                    quantity: 1,
                    unit_price: 100.0,
                    amount: 100.0,
                    is_gst_free: false,
                },
            ],
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
        };

        let result = service.create_invoice(invoice).await;

        assert!(result.is_ok());
        let invoice = result.expect("invoice should be created");
        assert_eq!(invoice.subtotal, 189.0);
        assert_eq!(invoice.gst_amount, 10.0);
        assert_eq!(invoice.total_amount, 199.0);
    }

    #[tokio::test]
    async fn test_record_payment_rejects_zero_amount() {
        let service = new_service(vec![], vec![], vec![]);
        let mut payment = test_payment(Uuid::new_v4());
        payment.amount = 0.0;

        let result = service.record_payment(payment).await;

        assert!(matches!(
            result,
            Err(ServiceError::Validation(
                ValidationError::InvalidPaymentAmount
            ))
        ));
    }

    #[tokio::test]
    async fn test_find_claims_by_status_filters_correctly() {
        let service = new_service(
            vec![],
            vec![
                test_claim(ClaimStatus::Submitted),
                test_claim(ClaimStatus::Paid),
            ],
            vec![],
        );

        let result = service.find_claims_by_status(ClaimStatus::Paid).await;

        assert!(result.is_ok());
        let claims = result.expect("claims should be returned");
        assert_eq!(claims.len(), 1);
        assert_eq!(claims[0].status, ClaimStatus::Paid);
    }

    #[tokio::test]
    async fn test_create_invoice_from_signed_consultation() {
        let consultation = test_consultation(true);
        let consultation_id = consultation.id;
        let service = new_service(vec![], vec![], vec![consultation.clone()]);

        let invoice = service
            .create_invoice_from_consultation(
                consultation_id,
                vec![("23".to_string(), 89.0, true), ("10990".to_string(), 25.0, false)],
                BillingType::PrivateBilling,
                Uuid::new_v4(),
            )
            .await
            .expect("invoice should be created from signed consultation");

        assert_eq!(invoice.consultation_id, Some(consultation_id));
        assert_eq!(invoice.patient_id, consultation.patient_id);
        assert_eq!(invoice.practitioner_id, consultation.practitioner_id);
        assert_eq!(invoice.items.len(), 2);
        assert_eq!(invoice.subtotal, 114.0);
        assert_eq!(invoice.gst_amount, 2.5);
        assert_eq!(invoice.total_amount, 116.5);
        assert_eq!(invoice.status, InvoiceStatus::Issued);
    }

    #[tokio::test]
    async fn test_create_invoice_from_unsigned_consultation_rejected() {
        let consultation = test_consultation(false);
        let service = new_service(vec![], vec![], vec![consultation.clone()]);

        let result = service
            .create_invoice_from_consultation(
                consultation.id,
                vec![("23".to_string(), 89.0, true)],
                BillingType::PrivateBilling,
                Uuid::new_v4(),
            )
            .await;

        assert!(matches!(
            result,
            Err(ServiceError::Validation(ValidationError::ConsultationNotSigned))
        ));
    }

    #[tokio::test]
    async fn test_record_cash_payment_updates_invoice_partial() {
        let mut invoice = test_invoice();
        invoice.total_amount = 100.0;
        invoice.amount_outstanding = 100.0;
        invoice.amount_paid = 0.0;
        invoice.status = InvoiceStatus::Issued;

        let service = new_service(vec![invoice.clone()], vec![], vec![]);
        let (payment, updated_invoice) = service
            .record_cash_payment(invoice.id, 40.0, Uuid::new_v4())
            .await
            .expect("cash payment should be recorded");

        assert_eq!(payment.payment_method, PaymentMethod::Cash);
        assert_eq!(payment.amount, 40.0);
        assert_eq!(updated_invoice.amount_paid, 40.0);
        assert_eq!(updated_invoice.amount_outstanding, 60.0);
        assert_eq!(updated_invoice.status, InvoiceStatus::PartiallyPaid);
    }

    #[tokio::test]
    async fn test_record_cash_payment_marks_invoice_paid() {
        let mut invoice = test_invoice();
        invoice.total_amount = 50.0;
        invoice.amount_outstanding = 10.0;
        invoice.amount_paid = 40.0;
        invoice.status = InvoiceStatus::PartiallyPaid;

        let service = new_service(vec![invoice.clone()], vec![], vec![]);
        let (_, updated_invoice) = service
            .record_cash_payment(invoice.id, 10.0, Uuid::new_v4())
            .await
            .expect("cash payment should settle invoice");

        assert_eq!(updated_invoice.amount_paid, 50.0);
        assert_eq!(updated_invoice.amount_outstanding, 0.0);
        assert_eq!(updated_invoice.status, InvoiceStatus::Paid);
    }

    #[tokio::test]
    async fn test_prepare_claim_json_contains_required_fields() {
        let mut invoice = test_invoice();
        invoice.calculate_totals();
        let service = new_service(vec![invoice.clone()], vec![], vec![]);

        let json_payload = service
            .prepare_claim_json(invoice.id)
            .await
            .expect("claim json should be prepared");
        let parsed: serde_json::Value =
            serde_json::from_str(&json_payload).expect("payload should be valid JSON");

        assert_eq!(parsed["invoice_id"], invoice.id.to_string());
        assert_eq!(parsed["patient_id"], invoice.patient_id.to_string());
        assert_eq!(parsed["practitioner_id"], invoice.practitioner_id.to_string());
        assert_eq!(parsed["service_date"], invoice.invoice_date.to_string());
        assert_eq!(parsed["items"].as_array().expect("items should be array").len(), 1);
        assert_eq!(parsed["total_claimed"], invoice.total_amount);
    }

    #[tokio::test]
    async fn test_find_patient_balance_excludes_cancelled() {
        let patient_id = Uuid::new_v4();
        let mut issued = test_invoice();
        issued.patient_id = patient_id;
        issued.amount_outstanding = 25.0;
        issued.status = InvoiceStatus::Issued;

        let mut partially_paid = test_invoice();
        partially_paid.patient_id = patient_id;
        partially_paid.amount_outstanding = 15.0;
        partially_paid.status = InvoiceStatus::PartiallyPaid;

        let mut cancelled = test_invoice();
        cancelled.patient_id = patient_id;
        cancelled.amount_outstanding = 100.0;
        cancelled.status = InvoiceStatus::Cancelled;

        let service = new_service(vec![issued, partially_paid, cancelled], vec![], vec![]);
        let balance = service
            .find_patient_balance(patient_id)
            .await
            .expect("balance should be calculated");

        assert_eq!(balance, 40.0);
    }
}

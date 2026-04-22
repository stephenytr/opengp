use opengp_domain::domain::billing::{
    BillingType, ClaimStatus, ClaimType, DVAClaim, Invoice, InvoiceStatus, MedicareClaim,
    Payment, PaymentMethod,
};
use uuid::Uuid;

pub struct BillingGeneratorConfig {
    pub bulk_billing_percentage: f32,
    pub private_billing_percentage: f32,
    pub dva_percentage: f32,
    pub medicare_claim_percentage: f32,
    pub invoice_paid_percentage: f32,
    pub invoice_overdue_percentage: f32,
    pub average_items_per_invoice: usize,
    pub max_items_per_invoice: usize,
}

impl Default for BillingGeneratorConfig {
    fn default() -> Self {
        Self {
            bulk_billing_percentage: 0.60,
            private_billing_percentage: 0.30,
            dva_percentage: 0.10,
            medicare_claim_percentage: 0.90,
            invoice_paid_percentage: 0.70,
            invoice_overdue_percentage: 0.10,
            average_items_per_invoice: 2,
            max_items_per_invoice: 5,
        }
    }
}

pub struct BillingData {
    pub invoices: Vec<Invoice>,
    pub medicare_claims: Vec<MedicareClaim>,
    pub dva_claims: Vec<DVAClaim>,
    pub payments: Vec<Payment>,
}

pub struct BillingGenerator {
    config: BillingGeneratorConfig,
    rng: rand::rngs::ThreadRng,
}

impl BillingGenerator {
    pub fn new(config: BillingGeneratorConfig) -> Self {
        Self {
            config,
            rng: rand::thread_rng(),
        }
    }

    pub fn generate_for_patient(
        &mut self,
        patient_id: Uuid,
        practitioner_id: Uuid,
        consultation_ids: Vec<Uuid>,
    ) -> BillingData {
        let _ = (
            patient_id,
            practitioner_id,
            consultation_ids,
            &self.config,
            &mut self.rng,
            BillingType::BulkBilling,
            InvoiceStatus::Draft,
            ClaimType::BulkBill,
            ClaimStatus::Draft,
            PaymentMethod::Cash,
        );

        BillingData {
            invoices: Vec::new(),
            medicare_claims: Vec::new(),
            dva_claims: Vec::new(),
            payments: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::BillingGeneratorConfig;

    #[test]
    fn test_billing_generator_config_defaults() {
        let config = BillingGeneratorConfig::default();

        assert_eq!(config.bulk_billing_percentage, 0.60);
        assert_eq!(config.private_billing_percentage, 0.30);
        assert_eq!(config.dva_percentage, 0.10);
        assert_eq!(config.medicare_claim_percentage, 0.90);
        assert_eq!(config.invoice_paid_percentage, 0.70);
        assert_eq!(config.invoice_overdue_percentage, 0.10);
        assert_eq!(config.average_items_per_invoice, 2);
        assert_eq!(config.max_items_per_invoice, 5);
    }
}

use chrono::{DateTime, Duration, NaiveDate, Utc};
use opengp_domain::domain::billing::{
    BillingType, ClaimStatus, ClaimType, DVACardType, DVAClaim, DVAItem, Invoice, InvoiceItem,
    InvoiceStatus, MBSItem, MedicareClaim, Payment, PaymentMethod,
};
use rand::seq::SliceRandom;
use rand::Rng;
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
        let invoice_count = self.rng.gen_range(1..=3);

        let mut invoices = Vec::with_capacity(invoice_count);
        let mut medicare_claims = Vec::new();
        let mut dva_claims = Vec::new();
        let mut payments = Vec::new();

        for _ in 0..invoice_count {
            let billing_type = self.determine_billing_type();
            let consultation_id = consultation_ids.choose(&mut self.rng).copied();
            let service_date = self.random_service_date();

            let mut invoice = self.generate_invoice(
                patient_id,
                practitioner_id,
                consultation_id,
                billing_type,
                service_date,
            );

            match billing_type {
                BillingType::DVA => {
                    let dva_file_number = self.generate_dva_file_number();
                    let card_type = self.random_dva_card_type();
                    let claim = self.generate_dva_claim(
                        patient_id,
                        practitioner_id,
                        consultation_id,
                        &dva_file_number,
                        card_type,
                        service_date,
                    );
                    dva_claims.push(claim);
                }
                _ => {
                    let claim_type = if billing_type == BillingType::BulkBilling {
                        ClaimType::BulkBill
                    } else {
                        ClaimType::PatientClaim
                    };

                    let claim = self.generate_medicare_claim(
                        patient_id,
                        practitioner_id,
                        consultation_id,
                        Some(invoice.id),
                        service_date,
                        claim_type,
                    );
                    medicare_claims.push(claim);
                }
            }

            if invoice.status == InvoiceStatus::Paid {
                let payment_date = invoice
                    .created_at
                    .checked_add_signed(Duration::days(self.rng.gen_range(0..=14)))
                    .unwrap_or(invoice.created_at);
                let payment =
                    self.generate_payment(invoice.id, patient_id, invoice.total_amount, payment_date);
                payments.push(payment);
                invoice.amount_paid = invoice.total_amount;
                invoice.amount_outstanding = 0.0;
            }

            invoices.push(invoice);
        }

        BillingData {
            invoices,
            medicare_claims,
            dva_claims,
            payments,
        }
    }

    fn generate_invoice(
        &mut self,
        patient_id: Uuid,
        practitioner_id: Uuid,
        consultation_id: Option<Uuid>,
        billing_type: BillingType,
        service_date: NaiveDate,
    ) -> Invoice {
        let invoice_number = format!(
            "INV-{}-{:06}",
            service_date.format("%Y%m%d"),
            self.rng.gen_range(0..=999_999)
        );
        let due_date = service_date + Duration::days(30);
        let item_count = self.rng.gen_range(1..=self.config.max_items_per_invoice.max(1).min(5));

        let mut items = Vec::with_capacity(item_count);
        for _ in 0..item_count {
            let (item_code, description, unit_price) = self.random_invoice_item();
            let quantity = self.rng.gen_range(1..=3);
            let amount = unit_price * f64::from(quantity);
            items.push(InvoiceItem {
                id: Uuid::new_v4(),
                description: description.to_string(),
                item_code: Some(item_code.to_string()),
                quantity,
                unit_price,
                amount,
                is_gst_free: true,
            });
        }

        let status = self.determine_invoice_status();
        let mut invoice = Invoice {
            id: Uuid::new_v4(),
            patient_id,
            practitioner_id,
            consultation_id,
            invoice_number,
            invoice_date: service_date,
            due_date: Some(due_date),
            items,
            subtotal: 0.0,
            gst_amount: 0.0,
            total_amount: 0.0,
            amount_paid: 0.0,
            amount_outstanding: 0.0,
            status,
            billing_type,
            notes: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_by: practitioner_id,
            updated_by: None,
        };

        invoice.calculate_totals();

        if billing_type == BillingType::BulkBilling {
            invoice.status = InvoiceStatus::Paid;
            invoice.amount_paid = invoice.total_amount;
            invoice.amount_outstanding = 0.0;
        } else {
            invoice.amount_paid = match invoice.status {
                InvoiceStatus::Paid => invoice.total_amount,
                InvoiceStatus::PartiallyPaid => invoice.total_amount * 0.5,
                _ => 0.0,
            };
            invoice.amount_outstanding = invoice.total_amount - invoice.amount_paid;
        }

        invoice
    }

    fn generate_medicare_claim(
        &mut self,
        patient_id: Uuid,
        practitioner_id: Uuid,
        consultation_id: Option<Uuid>,
        invoice_id: Option<Uuid>,
        service_date: NaiveDate,
        claim_type: ClaimType,
    ) -> MedicareClaim {
        let claim_reference = Some(format!(
            "MC-{}-{:06}",
            service_date.format("%Y%m%d"),
            self.rng.gen_range(0..=999_999)
        ));
        let item_count = self.rng.gen_range(1..=3);

        let mut items = Vec::with_capacity(item_count);
        for _ in 0..item_count {
            let (item_number, description, fee, benefit) = self.random_mbs_item();
            let quantity = self.rng.gen_range(1..=2);
            items.push(MBSItem {
                item_number: item_number.to_string(),
                description: description.to_string(),
                fee,
                benefit,
                quantity,
            });
        }

        let total_claimed: f64 = items
            .iter()
            .map(|item| item.fee * f64::from(item.quantity))
            .sum();
        let total_benefit: f64 = items
            .iter()
            .map(|item| item.benefit * f64::from(item.quantity))
            .sum();
        let patient_contribution = match claim_type {
            ClaimType::BulkBill => 0.0,
            ClaimType::PatientClaim | ClaimType::Assignment => total_claimed - total_benefit,
        };

        let status = self.determine_claim_status();
        let submitted_at = if self.is_submitted_status(status) {
            let offset_days = self.rng.gen_range(1..=3);
            Some((service_date + Duration::days(offset_days)).and_hms_opt(9, 0, 0).unwrap_or_else(|| Utc::now().naive_utc()).and_utc())
        } else {
            None
        };

        let processed_at = if self.is_processed_status(status) {
            submitted_at.map(|submitted| {
                submitted
                    .checked_add_signed(Duration::days(self.rng.gen_range(5..=14)))
                    .unwrap_or(submitted)
            })
        } else {
            None
        };

        MedicareClaim {
            id: Uuid::new_v4(),
            patient_id,
            practitioner_id,
            consultation_id,
            invoice_id,
            claim_reference,
            service_date,
            items,
            total_claimed,
            total_benefit,
            patient_contribution,
            claim_type,
            status,
            submitted_at,
            processed_at,
            rejection_reason: None,
            created_at: Utc::now(),
            created_by: practitioner_id,
        }
    }

    fn generate_dva_claim(
        &mut self,
        patient_id: Uuid,
        practitioner_id: Uuid,
        consultation_id: Option<Uuid>,
        dva_file_number: &str,
        card_type: DVACardType,
        service_date: NaiveDate,
    ) -> DVAClaim {
        let item_count = self.rng.gen_range(1..=3);
        let mut items = Vec::with_capacity(item_count);

        for _ in 0..item_count {
            let (item_code, description, fee) = self.random_dva_item();
            let quantity = self.rng.gen_range(1..=2);
            items.push(DVAItem {
                item_code: item_code.to_string(),
                description: description.to_string(),
                fee,
                quantity,
            });
        }

        let total_claimed: f64 = items
            .iter()
            .map(|item| item.fee * f64::from(item.quantity))
            .sum();

        let status = self.determine_claim_status();
        let submitted_at = if self.is_submitted_status(status) {
            let offset_days = self.rng.gen_range(1..=3);
            Some((service_date + Duration::days(offset_days)).and_hms_opt(9, 0, 0).unwrap_or_else(|| Utc::now().naive_utc()).and_utc())
        } else {
            None
        };

        let processed_at = if self.is_processed_status(status) {
            submitted_at.map(|submitted| {
                submitted
                    .checked_add_signed(Duration::days(self.rng.gen_range(5..=14)))
                    .unwrap_or(submitted)
            })
        } else {
            None
        };

        DVAClaim {
            id: Uuid::new_v4(),
            patient_id,
            practitioner_id,
            consultation_id,
            dva_file_number: dva_file_number.to_string(),
            card_type,
            service_date,
            items,
            total_claimed,
            status,
            submitted_at,
            processed_at,
            created_at: Utc::now(),
            created_by: practitioner_id,
        }
    }

    fn generate_payment(
        &mut self,
        invoice_id: Uuid,
        patient_id: Uuid,
        amount: f64,
        payment_date: DateTime<Utc>,
    ) -> Payment {
        let methods = [
            PaymentMethod::Cash,
            PaymentMethod::EFTPOS,
            PaymentMethod::CreditCard,
            PaymentMethod::DebitCard,
            PaymentMethod::BankTransfer,
        ];
        let payment_method = methods
            .choose(&mut self.rng)
            .copied()
            .unwrap_or(PaymentMethod::Cash);

        Payment {
            id: Uuid::new_v4(),
            invoice_id,
            patient_id,
            payment_date,
            amount,
            payment_method,
            reference: Some(format!("PAY-{:06}", self.rng.gen_range(0..=999_999))),
            notes: None,
            created_at: Utc::now(),
            created_by: patient_id,
        }
    }

    fn determine_billing_type(&mut self) -> BillingType {
        let roll = self.rng.gen_range(0.0..1.0);
        let total = (self.config.bulk_billing_percentage
            + self.config.private_billing_percentage
            + self.config.dva_percentage)
            .max(f32::EPSILON);
        let bulk_cutoff = self.config.bulk_billing_percentage / total;
        let private_cutoff = bulk_cutoff + (self.config.private_billing_percentage / total);

        if roll < bulk_cutoff {
            BillingType::BulkBilling
        } else if roll < private_cutoff {
            BillingType::PrivateBilling
        } else {
            BillingType::DVA
        }
    }

    fn determine_invoice_status(&mut self) -> InvoiceStatus {
        let roll = self.rng.gen_range(0.0..1.0);
        let paid_cutoff = self.config.invoice_paid_percentage;
        let overdue_cutoff = paid_cutoff + self.config.invoice_overdue_percentage;
        let remaining = (1.0 - overdue_cutoff).max(0.0);
        let partial_cutoff = overdue_cutoff + (remaining * 0.30);
        let issued_cutoff = partial_cutoff + (remaining * 0.50);

        if roll < paid_cutoff {
            InvoiceStatus::Paid
        } else if roll < overdue_cutoff {
            InvoiceStatus::Overdue
        } else if roll < partial_cutoff {
            InvoiceStatus::PartiallyPaid
        } else if roll < issued_cutoff {
            InvoiceStatus::Issued
        } else {
            InvoiceStatus::Draft
        }
    }

    fn determine_claim_status(&mut self) -> ClaimStatus {
        let roll = self.rng.gen_range(0.0..1.0);
        if roll < 0.60 {
            ClaimStatus::Paid
        } else if roll < 0.80 {
            ClaimStatus::Submitted
        } else if roll < 0.90 {
            ClaimStatus::Processing
        } else if roll < 0.95 {
            ClaimStatus::Rejected
        } else {
            ClaimStatus::PartiallyPaid
        }
    }

    fn is_submitted_status(&self, status: ClaimStatus) -> bool {
        matches!(
            status,
            ClaimStatus::Submitted
                | ClaimStatus::Processing
                | ClaimStatus::Paid
                | ClaimStatus::Rejected
                | ClaimStatus::PartiallyPaid
        )
    }

    fn is_processed_status(&self, status: ClaimStatus) -> bool {
        matches!(
            status,
            ClaimStatus::Paid | ClaimStatus::Rejected | ClaimStatus::PartiallyPaid
        )
    }

    fn random_service_date(&mut self) -> NaiveDate {
        (Utc::now() - Duration::days(self.rng.gen_range(0..=365))).date_naive()
    }

    fn random_invoice_item(&mut self) -> (&'static str, &'static str, f64) {
        let items = [
            ("23", "Level B attendance", 41.55),
            ("36", "Level C attendance", 76.85),
            ("44", "Level D attendance", 116.10),
            ("721", "GP management plan", 156.55),
            ("723", "Team care arrangements", 124.70),
        ];

        items
            .choose(&mut self.rng)
            .copied()
            .unwrap_or(("23", "Level B attendance", 41.55))
    }

    fn random_mbs_item(&mut self) -> (&'static str, &'static str, f64, f64) {
        let items = [
            ("23", "Level B attendance", 41.55, 35.35),
            ("36", "Level C attendance", 76.85, 65.35),
            ("44", "Level D attendance", 116.10, 98.70),
            ("701", "Brief health assessment", 153.40, 130.40),
            ("703", "Standard health assessment", 233.30, 198.30),
        ];

        items
            .choose(&mut self.rng)
            .copied()
            .unwrap_or(("23", "Level B attendance", 41.55, 35.35))
    }

    fn random_dva_item(&mut self) -> (&'static str, &'static str, f64) {
        let items = [
            ("D904", "DVA standard consultation", 43.10),
            ("D905", "DVA long consultation", 78.90),
            ("D906", "DVA prolonged consultation", 118.20),
            ("D907", "DVA health assessment", 160.50),
        ];

        items
            .choose(&mut self.rng)
            .copied()
            .unwrap_or(("D904", "DVA standard consultation", 43.10))
    }

    fn random_dva_card_type(&mut self) -> DVACardType {
        let card_types = [DVACardType::Gold, DVACardType::White, DVACardType::Orange];
        card_types
            .choose(&mut self.rng)
            .copied()
            .unwrap_or(DVACardType::Gold)
    }

    fn generate_dva_file_number(&mut self) -> String {
        let digits: String = (0..8)
            .map(|_| self.rng.gen_range(0..=9).to_string())
            .collect();
        format!("DVA{}", digits)
    }
}

#[cfg(test)]
mod tests {
    use super::{BillingGenerator, BillingGeneratorConfig};
    use chrono::NaiveDate;
    use opengp_domain::domain::billing::{BillingType, ClaimType};
    use uuid::Uuid;

    fn approx_equal(lhs: f64, rhs: f64) {
        assert!((lhs - rhs).abs() < 0.0001, "{lhs} != {rhs}");
    }

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

    #[test]
    fn test_invoice_total_calculation() {
        let mut generator = BillingGenerator::new(BillingGeneratorConfig::default());
        let service_date = NaiveDate::from_ymd_opt(2026, 3, 15).expect("valid date");
        let invoice = generator.generate_invoice(
            Uuid::new_v4(),
            Uuid::new_v4(),
            None,
            BillingType::PrivateBilling,
            service_date,
        );

        approx_equal(invoice.subtotal + invoice.gst_amount, invoice.total_amount);
    }

    #[test]
    fn test_bulk_billing_no_outstanding() {
        let config = BillingGeneratorConfig {
            invoice_paid_percentage: 0.0,
            ..Default::default()
        };
        let mut generator = BillingGenerator::new(config);
        let service_date = NaiveDate::from_ymd_opt(2026, 4, 1).expect("valid date");
        let invoice = generator.generate_invoice(
            Uuid::new_v4(),
            Uuid::new_v4(),
            None,
            BillingType::BulkBilling,
            service_date,
        );

        approx_equal(invoice.amount_outstanding, 0.0);
        approx_equal(invoice.amount_paid, invoice.total_amount);
    }

    #[test]
    fn test_medicare_claim_benefit_calculation() {
        let mut generator = BillingGenerator::new(BillingGeneratorConfig::default());
        let service_date = NaiveDate::from_ymd_opt(2026, 2, 10).expect("valid date");
        let claim = generator.generate_medicare_claim(
            Uuid::new_v4(),
            Uuid::new_v4(),
            None,
            None,
            service_date,
            ClaimType::PatientClaim,
        );

        let expected_total_claimed: f64 = claim
            .items
            .iter()
            .map(|item| item.fee * f64::from(item.quantity))
            .sum();
        let expected_total_benefit: f64 = claim
            .items
            .iter()
            .map(|item| item.benefit * f64::from(item.quantity))
            .sum();

        approx_equal(claim.total_claimed, expected_total_claimed);
        approx_equal(claim.total_benefit, expected_total_benefit);
        approx_equal(
            claim.patient_contribution,
            claim.total_claimed - claim.total_benefit,
        );
    }

    #[test]
    fn test_dva_file_number_format() {
        let mut generator = BillingGenerator::new(BillingGeneratorConfig::default());
        let file_number = generator.generate_dva_file_number();

        assert!(file_number.starts_with("DVA"));
        assert_eq!(file_number.len(), 11);
        assert!(file_number.chars().skip(3).all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn test_generate_for_patient_produces_data() {
        let mut generator = BillingGenerator::new(BillingGeneratorConfig::default());
        let data = generator.generate_for_patient(
            Uuid::new_v4(),
            Uuid::new_v4(),
            vec![Uuid::new_v4(), Uuid::new_v4()],
        );

        assert!(!data.invoices.is_empty());
        assert!(data.medicare_claims.len() + data.dva_claims.len() >= data.invoices.len());

        for invoice in data.invoices {
            if invoice.billing_type == BillingType::BulkBilling {
                approx_equal(invoice.amount_outstanding, 0.0);
            }
        }
    }
}

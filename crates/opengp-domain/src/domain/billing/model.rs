use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};
use uuid::Uuid;

/// Invoice for services provided in an Australian general practice, including Medicare and private billing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invoice {
    pub id: Uuid,
    pub patient_id: Uuid,
    pub practitioner_id: Uuid,
    pub consultation_id: Option<Uuid>,

    pub invoice_number: String,
    pub invoice_date: NaiveDate,
    pub due_date: Option<NaiveDate>,

    pub items: Vec<InvoiceItem>,

    pub subtotal: f64,
    pub gst_amount: f64,
    pub total_amount: f64,
    pub amount_paid: f64,
    pub amount_outstanding: f64,

    pub status: InvoiceStatus,
    pub billing_type: BillingType,

    pub notes: Option<String>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Uuid,
    pub updated_by: Option<Uuid>,
}

impl Invoice {
    /// Recalculate invoice totals including GST, based on the current item list.
    pub fn calculate_totals(&mut self) {
        self.subtotal = self.items.iter().map(|item| item.amount).sum();
        self.gst_amount = self.subtotal * 0.1;
        self.total_amount = self.subtotal + self.gst_amount;
        self.amount_outstanding = self.total_amount - self.amount_paid;
    }

    /// Return true when the invoice has no outstanding balance.
    pub fn is_paid(&self) -> bool {
        self.amount_outstanding <= 0.0
    }

    /// Return true when the due date has passed and the invoice is not fully paid.
    pub fn is_overdue(&self) -> bool {
        if let Some(due) = self.due_date {
            due < Utc::now().date_naive() && !self.is_paid()
        } else {
            false
        }
    }
}

/// Individual line item on an invoice, often mapped to an MBS or practice-specific item code.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceItem {
    pub id: Uuid,
    pub description: String,
    pub item_code: Option<String>,
    pub quantity: u32,
    pub unit_price: f64,
    pub amount: f64,
}

/// Status of an invoice throughout the billing lifecycle.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Display, EnumString)]
pub enum InvoiceStatus {
    /// Invoice is still being prepared and not yet issued.
    Draft,
    /// Invoice has been issued to the payer.
    Issued,
    /// Some payment has been received but a balance remains.
    PartiallyPaid,
    /// Invoice is fully paid.
    Paid,
    /// Invoice is overdue based on the configured due date.
    Overdue,
    /// Invoice has been cancelled and should not be collected.
    Cancelled,
    /// Invoice has been refunded in full or in part.
    Refunded,
}

/// Billing funding source such as Medicare bulk bill or private fee.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Display, EnumString)]
pub enum BillingType {
    /// Direct bulk bill claim to Medicare.
    BulkBilling,
    /// Patient pays privately and may claim a Medicare rebate separately.
    PrivateBilling,
    /// Mix of Medicare and out of pocket fees.
    MixedBilling,
    /// Workers compensation or WorkCover scheme billing.
    WorkCover,
    /// Department of Veterans' Affairs funded billing.
    DVA,
    /// Other third party payer such as insurer or employer.
    ThirdParty,
}

/// Medicare claim for MBS services, including bulk bill and patient claims.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MedicareClaim {
    pub id: Uuid,
    pub patient_id: Uuid,
    pub practitioner_id: Uuid,
    pub consultation_id: Option<Uuid>,
    pub invoice_id: Option<Uuid>,

    pub claim_reference: Option<String>,
    pub service_date: NaiveDate,

    pub items: Vec<MBSItem>,

    pub total_claimed: f64,
    pub total_benefit: f64,
    pub patient_contribution: f64,

    pub claim_type: ClaimType,
    pub status: ClaimStatus,

    pub submitted_at: Option<DateTime<Utc>>,
    pub processed_at: Option<DateTime<Utc>>,
    pub rejection_reason: Option<String>,

    pub created_at: DateTime<Utc>,
    pub created_by: Uuid,
}

/// Medicare Benefits Schedule item that was claimed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MBSItem {
    pub item_number: String,
    pub description: String,
    pub fee: f64,
    pub benefit: f64,
    pub quantity: u32,
}

/// Type of Medicare claim being submitted.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Display, EnumString)]
pub enum ClaimType {
    /// Bulk bill claim paid directly to the practice.
    BulkBill,
    /// Patient claim where benefit is paid to the patient.
    PatientClaim,
    /// Assignment of benefit to another party.
    Assignment,
}

/// Processing status of a Medicare or DVA claim.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Display, EnumString)]
pub enum ClaimStatus {
    /// Claim is being prepared and not yet sent.
    Draft,
    /// Claim has been submitted to the funder.
    Submitted,
    /// Claim is being processed by the funder.
    Processing,
    /// Claim has been fully paid.
    Paid,
    /// Claim was rejected.
    Rejected,
    /// Claim was only partially paid.
    PartiallyPaid,
}

/// Payment applied to an invoice, including Medicare and DVA benefits and patient payments.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payment {
    pub id: Uuid,
    pub invoice_id: Uuid,
    pub patient_id: Uuid,

    pub payment_date: DateTime<Utc>,
    pub amount: f64,
    pub payment_method: PaymentMethod,
    pub reference: Option<String>,
    pub notes: Option<String>,

    pub created_at: DateTime<Utc>,
    pub created_by: Uuid,
}

/// Method used to pay an invoice.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Display, EnumString)]
pub enum PaymentMethod {
    /// Cash payment.
    Cash,
    /// EFTPOS payment.
    EFTPOS,
    /// Credit card payment.
    CreditCard,
    /// Debit card payment.
    DebitCard,
    /// Electronic bank transfer.
    BankTransfer,
    /// Cheque payment.
    Cheque,
    /// Medicare benefit paid to the practice or patient.
    MedicareBenefit,
    /// Department of Veterans' Affairs benefit.
    DVABenefit,
    /// Any other payment method.
    Other,
}

/// Claim to the Department of Veterans' Affairs for eligible services.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DVAClaim {
    pub id: Uuid,
    pub patient_id: Uuid,
    pub practitioner_id: Uuid,
    pub consultation_id: Option<Uuid>,

    pub dva_file_number: String,
    pub card_type: DVACardType,
    pub service_date: NaiveDate,
    pub items: Vec<DVAItem>,

    pub total_claimed: f64,
    pub status: ClaimStatus,

    pub submitted_at: Option<DateTime<Utc>>,
    pub processed_at: Option<DateTime<Utc>>,

    pub created_at: DateTime<Utc>,
    pub created_by: Uuid,
}

/// DVA card type held by the veteran.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Display, EnumString)]
pub enum DVACardType {
    /// Gold card.
    Gold,
    /// White card.
    White,
    /// Orange card.
    Orange,
}

/// Item claimed under a DVA arrangement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DVAItem {
    pub item_code: String,
    pub description: String,
    pub fee: f64,
    pub quantity: u32,
}

/// Workers compensation claim, including WorkCover and similar schemes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkCoverClaim {
    pub id: Uuid,
    pub patient_id: Uuid,
    pub practitioner_id: Uuid,
    pub consultation_id: Option<Uuid>,

    pub claim_number: String,
    pub employer_name: String,
    pub insurer_name: String,
    pub injury_date: NaiveDate,
    pub service_date: NaiveDate,

    pub state: AustralianState,
    pub diagnosis: String,
    pub treatment: String,

    pub total_claimed: f64,
    pub status: ClaimStatus,

    pub submitted_at: Option<DateTime<Utc>>,
    pub processed_at: Option<DateTime<Utc>>,

    pub created_at: DateTime<Utc>,
    pub created_by: Uuid,
}

/// Australian state or territory for WorkCover and address data.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Display, EnumString)]
pub enum AustralianState {
    /// New South Wales.
    NSW,
    /// Victoria.
    VIC,
    /// Queensland.
    QLD,
    /// South Australia.
    SA,
    /// Western Australia.
    WA,
    /// Tasmania.
    TAS,
    /// Northern Territory.
    NT,
    /// Australian Capital Territory.
    ACT,
}

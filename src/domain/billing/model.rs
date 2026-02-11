use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
    pub fn calculate_totals(&mut self) {
        self.subtotal = self.items.iter().map(|item| item.amount).sum();
        self.gst_amount = self.subtotal * 0.1;
        self.total_amount = self.subtotal + self.gst_amount;
        self.amount_outstanding = self.total_amount - self.amount_paid;
    }

    pub fn is_paid(&self) -> bool {
        self.amount_outstanding <= 0.0
    }

    pub fn is_overdue(&self) -> bool {
        if let Some(due) = self.due_date {
            due < Utc::now().date_naive() && !self.is_paid()
        } else {
            false
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceItem {
    pub id: Uuid,
    pub description: String,
    pub item_code: Option<String>,
    pub quantity: u32,
    pub unit_price: f64,
    pub amount: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum InvoiceStatus {
    Draft,
    Issued,
    PartiallyPaid,
    Paid,
    Overdue,
    Cancelled,
    Refunded,
}

impl std::fmt::Display for InvoiceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InvoiceStatus::Draft => write!(f, "Draft"),
            InvoiceStatus::Issued => write!(f, "Issued"),
            InvoiceStatus::PartiallyPaid => write!(f, "Partially Paid"),
            InvoiceStatus::Paid => write!(f, "Paid"),
            InvoiceStatus::Overdue => write!(f, "Overdue"),
            InvoiceStatus::Cancelled => write!(f, "Cancelled"),
            InvoiceStatus::Refunded => write!(f, "Refunded"),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum BillingType {
    BulkBilling,
    PrivateBilling,
    MixedBilling,
    WorkCover,
    DVA,
    ThirdParty,
}

impl std::fmt::Display for BillingType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BillingType::BulkBilling => write!(f, "Bulk Billing"),
            BillingType::PrivateBilling => write!(f, "Private Billing"),
            BillingType::MixedBilling => write!(f, "Mixed Billing"),
            BillingType::WorkCover => write!(f, "WorkCover"),
            BillingType::DVA => write!(f, "DVA"),
            BillingType::ThirdParty => write!(f, "Third Party"),
        }
    }
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MBSItem {
    pub item_number: String,
    pub description: String,
    pub fee: f64,
    pub benefit: f64,
    pub quantity: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ClaimType {
    BulkBill,
    PatientClaim,
    Assignment,
}

impl std::fmt::Display for ClaimType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClaimType::BulkBill => write!(f, "Bulk Bill"),
            ClaimType::PatientClaim => write!(f, "Patient Claim"),
            ClaimType::Assignment => write!(f, "Assignment"),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ClaimStatus {
    Draft,
    Submitted,
    Processing,
    Paid,
    Rejected,
    PartiallyPaid,
}

impl std::fmt::Display for ClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClaimStatus::Draft => write!(f, "Draft"),
            ClaimStatus::Submitted => write!(f, "Submitted"),
            ClaimStatus::Processing => write!(f, "Processing"),
            ClaimStatus::Paid => write!(f, "Paid"),
            ClaimStatus::Rejected => write!(f, "Rejected"),
            ClaimStatus::PartiallyPaid => write!(f, "Partially Paid"),
        }
    }
}

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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PaymentMethod {
    Cash,
    EFTPOS,
    CreditCard,
    DebitCard,
    BankTransfer,
    Cheque,
    MedicareBenefit,
    DVABenefit,
    Other,
}

impl std::fmt::Display for PaymentMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PaymentMethod::Cash => write!(f, "Cash"),
            PaymentMethod::EFTPOS => write!(f, "EFTPOS"),
            PaymentMethod::CreditCard => write!(f, "Credit Card"),
            PaymentMethod::DebitCard => write!(f, "Debit Card"),
            PaymentMethod::BankTransfer => write!(f, "Bank Transfer"),
            PaymentMethod::Cheque => write!(f, "Cheque"),
            PaymentMethod::MedicareBenefit => write!(f, "Medicare Benefit"),
            PaymentMethod::DVABenefit => write!(f, "DVA Benefit"),
            PaymentMethod::Other => write!(f, "Other"),
        }
    }
}

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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum DVACardType {
    Gold,
    White,
    Orange,
}

impl std::fmt::Display for DVACardType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DVACardType::Gold => write!(f, "Gold Card"),
            DVACardType::White => write!(f, "White Card"),
            DVACardType::Orange => write!(f, "Orange Card"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DVAItem {
    pub item_code: String,
    pub description: String,
    pub fee: f64,
    pub quantity: u32,
}

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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AustralianState {
    NSW,
    VIC,
    QLD,
    SA,
    WA,
    TAS,
    NT,
    ACT,
}

impl std::fmt::Display for AustralianState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AustralianState::NSW => write!(f, "NSW"),
            AustralianState::VIC => write!(f, "VIC"),
            AustralianState::QLD => write!(f, "QLD"),
            AustralianState::SA => write!(f, "SA"),
            AustralianState::WA => write!(f, "WA"),
            AustralianState::TAS => write!(f, "TAS"),
            AustralianState::NT => write!(f, "NT"),
            AustralianState::ACT => write!(f, "ACT"),
        }
    }
}

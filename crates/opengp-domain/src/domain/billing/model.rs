use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Display, EnumString)]
pub enum InvoiceStatus {
    Draft,
    Issued,
    PartiallyPaid,
    Paid,
    Overdue,
    Cancelled,
    Refunded,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Display, EnumString)]
pub enum BillingType {
    BulkBilling,
    PrivateBilling,
    MixedBilling,
    WorkCover,
    DVA,
    ThirdParty,
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Display, EnumString)]
pub enum ClaimType {
    BulkBill,
    PatientClaim,
    Assignment,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Display, EnumString)]
pub enum ClaimStatus {
    Draft,
    Submitted,
    Processing,
    Paid,
    Rejected,
    PartiallyPaid,
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Display, EnumString)]
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Display, EnumString)]
pub enum DVACardType {
    Gold,
    White,
    Orange,
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Display, EnumString)]
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

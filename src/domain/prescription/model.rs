use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prescription {
    pub id: Uuid,
    pub patient_id: Uuid,
    pub practitioner_id: Uuid,
    pub consultation_id: Option<Uuid>,

    pub medication: Medication,

    pub dosage: String,
    pub quantity: u32,
    pub repeats: u8,
    pub authority_required: bool,
    pub authority_approval_number: Option<String>,
    pub authority_type: Option<AuthorityType>,

    pub pbs_status: PBSStatus,
    pub pbs_item_code: Option<String>,

    pub indication: Option<String>,
    pub directions: String,
    pub notes: Option<String>,

    pub prescription_type: PrescriptionType,
    pub prescription_date: DateTime<Utc>,
    pub expiry_date: Option<NaiveDate>,

    pub is_active: bool,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub cancellation_reason: Option<String>,

    pub created_at: DateTime<Utc>,
    pub created_by: Uuid,
}

impl Prescription {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        patient_id: Uuid,
        practitioner_id: Uuid,
        consultation_id: Option<Uuid>,
        medication: Medication,
        dosage: String,
        quantity: u32,
        repeats: u8,
        directions: String,
        created_by: Uuid,
    ) -> Self {
        let prescription_date = Utc::now();
        let expiry_date = Some(prescription_date.date_naive() + chrono::Duration::days(365));

        Self {
            id: Uuid::new_v4(),
            patient_id,
            practitioner_id,
            consultation_id,
            medication,
            dosage,
            quantity,
            repeats,
            authority_required: false,
            authority_approval_number: None,
            authority_type: None,
            pbs_status: PBSStatus::Private,
            pbs_item_code: None,
            indication: None,
            directions,
            notes: None,
            prescription_type: PrescriptionType::Paper,
            prescription_date,
            expiry_date,
            is_active: true,
            cancelled_at: None,
            cancellation_reason: None,
            created_at: prescription_date,
            created_by,
        }
    }

    pub fn is_expired(&self) -> bool {
        if let Some(expiry) = self.expiry_date {
            expiry < Utc::now().date_naive()
        } else {
            false
        }
    }

    pub fn cancel(&mut self, reason: String, _user_id: Uuid) {
        self.is_active = false;
        self.cancelled_at = Some(Utc::now());
        self.cancellation_reason = Some(reason);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Medication {
    pub generic_name: String,
    pub brand_name: Option<String>,
    pub strength: String,
    pub form: MedicationForm,
    pub amt_code: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum MedicationForm {
    Tablet,
    Capsule,
    Liquid,
    Syrup,
    Suspension,
    Cream,
    Ointment,
    Gel,
    Patch,
    Inhaler,
    Injection,
    Drops,
    Spray,
    Suppository,
    Other,
}

impl std::fmt::Display for MedicationForm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MedicationForm::Tablet => write!(f, "Tablet"),
            MedicationForm::Capsule => write!(f, "Capsule"),
            MedicationForm::Liquid => write!(f, "Liquid"),
            MedicationForm::Syrup => write!(f, "Syrup"),
            MedicationForm::Suspension => write!(f, "Suspension"),
            MedicationForm::Cream => write!(f, "Cream"),
            MedicationForm::Ointment => write!(f, "Ointment"),
            MedicationForm::Gel => write!(f, "Gel"),
            MedicationForm::Patch => write!(f, "Patch"),
            MedicationForm::Inhaler => write!(f, "Inhaler"),
            MedicationForm::Injection => write!(f, "Injection"),
            MedicationForm::Drops => write!(f, "Drops"),
            MedicationForm::Spray => write!(f, "Spray"),
            MedicationForm::Suppository => write!(f, "Suppository"),
            MedicationForm::Other => write!(f, "Other"),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PBSStatus {
    GeneralSchedule,
    RestrictedBenefit,
    AuthorityRequired,
    Private,
    RPBS,
}

impl std::fmt::Display for PBSStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PBSStatus::GeneralSchedule => write!(f, "PBS General"),
            PBSStatus::RestrictedBenefit => write!(f, "PBS Restricted"),
            PBSStatus::AuthorityRequired => write!(f, "PBS Authority Required"),
            PBSStatus::Private => write!(f, "Private"),
            PBSStatus::RPBS => write!(f, "RPBS"),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthorityType {
    Streamlined,
    Complex,
    Telephone,
    Written,
}

impl std::fmt::Display for AuthorityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthorityType::Streamlined => write!(f, "Streamlined Authority"),
            AuthorityType::Complex => write!(f, "Complex Authority"),
            AuthorityType::Telephone => write!(f, "Telephone Authority"),
            AuthorityType::Written => write!(f, "Written Authority"),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PrescriptionType {
    Paper,
    Electronic,
    Verbal,
    Fax,
}

impl std::fmt::Display for PrescriptionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PrescriptionType::Paper => write!(f, "Paper"),
            PrescriptionType::Electronic => write!(f, "Electronic"),
            PrescriptionType::Verbal => write!(f, "Verbal"),
            PrescriptionType::Fax => write!(f, "Fax"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentMedication {
    pub id: Uuid,
    pub patient_id: Uuid,

    pub medication: Medication,
    pub dosage: String,
    pub frequency: String,
    pub started_date: NaiveDate,
    pub stopped_date: Option<NaiveDate>,
    pub indication: Option<String>,
    pub prescriber: Option<String>,

    pub is_active: bool,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Uuid,
    pub updated_by: Option<Uuid>,
}

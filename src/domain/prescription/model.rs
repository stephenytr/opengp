use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Display, EnumString)]
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Display, EnumString)]
pub enum PBSStatus {
    GeneralSchedule,
    RestrictedBenefit,
    AuthorityRequired,
    Private,
    RPBS,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Display, EnumString)]
pub enum AuthorityType {
    Streamlined,
    Complex,
    Telephone,
    Written,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Display, EnumString)]
pub enum PrescriptionType {
    Paper,
    Electronic,
    Verbal,
    Fax,
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

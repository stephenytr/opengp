use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::model::{AuthorityType, Medication, PBSStatus, PrescriptionType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewPrescriptionData {
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePrescriptionData {
    pub authority_approval_number: Option<String>,
    pub authority_type: Option<AuthorityType>,
    pub pbs_item_code: Option<String>,
    pub notes: Option<String>,
}

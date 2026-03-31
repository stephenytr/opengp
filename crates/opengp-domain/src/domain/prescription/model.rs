use chrono::{DateTime, NaiveDate, Utc};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};
use uuid::Uuid;

/// Prescription issued for a patient under the PBS or privately.
///
/// Captures the prescribed medication, PBS status, directions,
/// authority details and audit information linking back to the
/// consultation and prescriber.
#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
#[builder(setter(into))]
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
    /// Create a new prescription with default PBS settings.
    ///
    /// Initial prescriptions are created as private, active, paper
    /// prescriptions with a one‑year expiry from creation.
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

    /// Return true when the prescription has passed its expiry date.
    pub fn is_expired(&self) -> bool {
        if let Some(expiry) = self.expiry_date {
            expiry < Utc::now().date_naive()
        } else {
            false
        }
    }

    /// Cancel the prescription with a recorded reason.
    ///
    /// This marks the prescription as inactive and records
    /// cancellation metadata.
    pub fn cancel(&mut self, reason: String, _user_id: Uuid) {
        self.is_active = false;
        self.cancelled_at = Some(Utc::now());
        self.cancellation_reason = Some(reason);
    }
}

/// Core medicine details used within a prescription.
///
/// Includes generic and optional brand names plus AMT/SNOMED
/// identifiers for interoperability with Australian medication
/// terminologies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Medication {
    pub generic_name: String,
    pub brand_name: Option<String>,
    pub strength: String,
    pub form: MedicationForm,
    pub amt_code: Option<String>,
}

/// Pharmaceutical form of a medication.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Display, EnumString)]
pub enum MedicationForm {
    /// Solid tablet dosage form.
    Tablet,
    /// Hard or soft capsule dosage form.
    Capsule,
    /// Liquid preparation, for example oral solution.
    Liquid,
    /// Syrup formulation, often for paediatric dosing.
    Syrup,
    /// Suspended particles in liquid.
    Suspension,
    /// Topical cream.
    Cream,
    /// Topical ointment.
    Ointment,
    /// Topical gel.
    Gel,
    /// Transdermal patch.
    Patch,
    /// Metered dose or dry powder inhaler.
    Inhaler,
    /// Injectable preparation.
    Injection,
    /// Eye, ear or nose drops.
    Drops,
    /// Nasal or topical spray.
    Spray,
    /// Rectal or vaginal suppository.
    Suppository,
    /// Any other medication form.
    Other,
}

/// PBS entitlement or funding status for a prescription.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Display, EnumString)]
pub enum PBSStatus {
    /// Standard PBS general schedule.
    GeneralSchedule,
    /// Restricted benefit requiring an indication.
    RestrictedBenefit,
    /// PBS authority required from Services Australia.
    AuthorityRequired,
    /// Non‑PBS private script.
    Private,
    /// Repatriation PBS (RPBS) for eligible veterans.
    RPBS,
}

/// Type of PBS authority used for AuthorityRequired items.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Display, EnumString)]
pub enum AuthorityType {
    /// Streamlined authority item using a predefined code.
    Streamlined,
    /// Complex authority requiring assessment by Services Australia.
    Complex,
    /// Telephone authority provided by a Medicare operator.
    Telephone,
    /// Written authority application.
    Written,
}

/// Channel through which the prescription is issued.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Display, EnumString)]
pub enum PrescriptionType {
    /// Traditional paper prescription.
    Paper,
    /// Electronic prescription (eScript) token or list.
    Electronic,
    /// Verbal order provided to a pharmacist.
    Verbal,
    /// Faxed copy of a prescription.
    Fax,
}

/// Current long‑term medication record for a patient.
///
/// This models the active medication list used in prescribing and
/// medication review workflows.
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

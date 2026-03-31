use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};
use uuid::Uuid;

/// Record of a single vaccination event for a patient.
///
/// Includes vaccine details, dose information, AIR reporting
/// metadata and any adverse events following immunisation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Immunisation {
    pub id: Uuid,
    pub patient_id: Uuid,
    pub practitioner_id: Uuid,
    pub consultation_id: Option<Uuid>,

    pub vaccine: Vaccine,
    pub vaccination_date: NaiveDate,
    pub dose_number: u8,
    pub total_doses: Option<u8>,

    pub batch_number: String,
    pub expiry_date: Option<NaiveDate>,
    pub manufacturer: Option<String>,

    pub route: AdministrationRoute,
    pub site: AnatomicalSite,
    pub dose_quantity: Option<f32>,
    pub dose_unit: Option<String>,

    pub consent_obtained: bool,
    pub consent_type: Option<ConsentType>,

    pub air_reported: bool,
    pub air_report_date: Option<DateTime<Utc>>,
    pub air_transaction_id: Option<String>,

    pub adverse_event: bool,
    pub adverse_event_details: Option<String>,

    pub notes: Option<String>,

    pub created_at: DateTime<Utc>,
    pub created_by: Uuid,
}

impl Immunisation {
    #[allow(clippy::too_many_arguments)]
    /// Create a new immunisation record with default consent and
    /// AIR reporting state.
    pub fn new(
        patient_id: Uuid,
        practitioner_id: Uuid,
        vaccine: Vaccine,
        vaccination_date: NaiveDate,
        dose_number: u8,
        batch_number: String,
        route: AdministrationRoute,
        site: AnatomicalSite,
        created_by: Uuid,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            patient_id,
            practitioner_id,
            consultation_id: None,
            vaccine,
            vaccination_date,
            dose_number,
            total_doses: None,
            batch_number,
            expiry_date: None,
            manufacturer: None,
            route,
            site,
            dose_quantity: None,
            dose_unit: None,
            consent_obtained: true,
            consent_type: Some(ConsentType::Verbal),
            air_reported: false,
            air_report_date: None,
            air_transaction_id: None,
            adverse_event: false,
            adverse_event_details: None,
            notes: None,
            created_at: Utc::now(),
            created_by,
        }
    }

    /// Mark this immunisation as reported to the Australian
    /// Immunisation Register (AIR).
    pub fn mark_air_reported(&mut self, transaction_id: String) {
        self.air_reported = true;
        self.air_report_date = Some(Utc::now());
        self.air_transaction_id = Some(transaction_id);
    }
}

/// Vaccine metadata used for immunisation recording.
///
/// Includes brand and AMT/SNOMED coding to support AIR reporting and
/// decision support.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vaccine {
    pub name: String,
    pub vaccine_type: VaccineType,
    pub brand_name: Option<String>,
    pub snomed_code: Option<String>,
    pub amt_code: Option<String>,
}

/// Type or program category of vaccine.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Display, EnumString)]
pub enum VaccineType {
    COVID19,
    Influenza,
    Pneumococcal,
    Shingles,
    MMR,
    DTPa,
    Polio,
    HepB,
    HepA,
    HepAB,
    Hib,
    MenC,
    MenB,
    MenACWY,
    Rotavirus,
    Varicella,
    HPV,
    BCG,
    Rabies,
    YellowFever,
    JapaneseEncephalitis,
    TyphoidOral,
    TyphoidInjectable,
    Cholera,
    Other,
}

/// Route of vaccine administration.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Display, EnumString)]
pub enum AdministrationRoute {
    Intramuscular,
    Subcutaneous,
    Intradermal,
    Oral,
    Intranasal,
}

/// Anatomical site where the vaccine was administered.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Display, EnumString)]
pub enum AnatomicalSite {
    LeftDeltoid,
    RightDeltoid,
    LeftThigh,
    RightThigh,
    LeftUpperArm,
    RightUpperArm,
    LeftGluteal,
    RightGluteal,
    Oral,
    Intranasal,
    Other,
}

/// Type of consent documented prior to vaccination.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Display, EnumString)]
pub enum ConsentType {
    Written,
    Verbal,
    Implied,
}

/// Planned vaccination schedule entry for a patient.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaccinationSchedule {
    pub patient_id: Uuid,
    pub vaccine_type: VaccineType,
    pub dose_number: u8,
    pub due_date: NaiveDate,
    pub status: ScheduleStatus,
    pub completed_immunisation_id: Option<Uuid>,
}

/// Status of a scheduled vaccination dose.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Display, EnumString)]
pub enum ScheduleStatus {
    Due,
    Overdue,
    Completed,
    Deferred,
    Contraindicated,
}

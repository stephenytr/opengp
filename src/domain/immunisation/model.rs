use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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

    pub fn mark_air_reported(&mut self, transaction_id: String) {
        self.air_reported = true;
        self.air_report_date = Some(Utc::now());
        self.air_transaction_id = Some(transaction_id);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vaccine {
    pub name: String,
    pub vaccine_type: VaccineType,
    pub brand_name: Option<String>,
    pub snomed_code: Option<String>,
    pub amt_code: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
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

impl std::fmt::Display for VaccineType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VaccineType::COVID19 => write!(f, "COVID-19"),
            VaccineType::Influenza => write!(f, "Influenza"),
            VaccineType::Pneumococcal => write!(f, "Pneumococcal"),
            VaccineType::Shingles => write!(f, "Shingles (Zoster)"),
            VaccineType::MMR => write!(f, "MMR"),
            VaccineType::DTPa => write!(f, "DTPa (Diphtheria/Tetanus/Pertussis)"),
            VaccineType::Polio => write!(f, "Polio"),
            VaccineType::HepB => write!(f, "Hepatitis B"),
            VaccineType::HepA => write!(f, "Hepatitis A"),
            VaccineType::HepAB => write!(f, "Hepatitis A/B"),
            VaccineType::Hib => write!(f, "Hib (Haemophilus influenzae type b)"),
            VaccineType::MenC => write!(f, "Meningococcal C"),
            VaccineType::MenB => write!(f, "Meningococcal B"),
            VaccineType::MenACWY => write!(f, "Meningococcal ACWY"),
            VaccineType::Rotavirus => write!(f, "Rotavirus"),
            VaccineType::Varicella => write!(f, "Varicella (Chickenpox)"),
            VaccineType::HPV => write!(f, "HPV"),
            VaccineType::BCG => write!(f, "BCG"),
            VaccineType::Rabies => write!(f, "Rabies"),
            VaccineType::YellowFever => write!(f, "Yellow Fever"),
            VaccineType::JapaneseEncephalitis => write!(f, "Japanese Encephalitis"),
            VaccineType::TyphoidOral => write!(f, "Typhoid (Oral)"),
            VaccineType::TyphoidInjectable => write!(f, "Typhoid (Injectable)"),
            VaccineType::Cholera => write!(f, "Cholera"),
            VaccineType::Other => write!(f, "Other"),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AdministrationRoute {
    Intramuscular,
    Subcutaneous,
    Intradermal,
    Oral,
    Intranasal,
}

impl std::fmt::Display for AdministrationRoute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AdministrationRoute::Intramuscular => write!(f, "Intramuscular (IM)"),
            AdministrationRoute::Subcutaneous => write!(f, "Subcutaneous (SC)"),
            AdministrationRoute::Intradermal => write!(f, "Intradermal (ID)"),
            AdministrationRoute::Oral => write!(f, "Oral"),
            AdministrationRoute::Intranasal => write!(f, "Intranasal"),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
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

impl std::fmt::Display for AnatomicalSite {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AnatomicalSite::LeftDeltoid => write!(f, "Left Deltoid"),
            AnatomicalSite::RightDeltoid => write!(f, "Right Deltoid"),
            AnatomicalSite::LeftThigh => write!(f, "Left Thigh"),
            AnatomicalSite::RightThigh => write!(f, "Right Thigh"),
            AnatomicalSite::LeftUpperArm => write!(f, "Left Upper Arm"),
            AnatomicalSite::RightUpperArm => write!(f, "Right Upper Arm"),
            AnatomicalSite::LeftGluteal => write!(f, "Left Gluteal"),
            AnatomicalSite::RightGluteal => write!(f, "Right Gluteal"),
            AnatomicalSite::Oral => write!(f, "Oral"),
            AnatomicalSite::Intranasal => write!(f, "Intranasal"),
            AnatomicalSite::Other => write!(f, "Other"),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ConsentType {
    Written,
    Verbal,
    Implied,
}

impl std::fmt::Display for ConsentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConsentType::Written => write!(f, "Written"),
            ConsentType::Verbal => write!(f, "Verbal"),
            ConsentType::Implied => write!(f, "Implied"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaccinationSchedule {
    pub patient_id: Uuid,
    pub vaccine_type: VaccineType,
    pub dose_number: u8,
    pub due_date: NaiveDate,
    pub status: ScheduleStatus,
    pub completed_immunisation_id: Option<Uuid>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ScheduleStatus {
    Due,
    Overdue,
    Completed,
    Deferred,
    Contraindicated,
}

impl std::fmt::Display for ScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScheduleStatus::Due => write!(f, "Due"),
            ScheduleStatus::Overdue => write!(f, "Overdue"),
            ScheduleStatus::Completed => write!(f, "Completed"),
            ScheduleStatus::Deferred => write!(f, "Deferred"),
            ScheduleStatus::Contraindicated => write!(f, "Contraindicated"),
        }
    }
}

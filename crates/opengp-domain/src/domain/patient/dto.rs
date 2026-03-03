use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use super::model::{Address, AtsiStatus, ConcessionType, EmergencyContact, Gender};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewPatientData {
    pub ihi: Option<String>,
    pub medicare_number: Option<String>,
    pub medicare_irn: Option<u8>,
    pub medicare_expiry: Option<NaiveDate>,

    pub title: Option<String>,
    pub first_name: String,
    pub middle_name: Option<String>,
    pub last_name: String,
    pub preferred_name: Option<String>,
    pub date_of_birth: NaiveDate,
    pub gender: Gender,

    pub address: Address,
    pub phone_home: Option<String>,
    pub phone_mobile: Option<String>,
    pub email: Option<String>,

    pub emergency_contact: Option<EmergencyContact>,

    pub concession_type: Option<ConcessionType>,
    pub concession_number: Option<String>,
    pub preferred_language: Option<String>,
    pub interpreter_required: Option<bool>,
    pub aboriginal_torres_strait_islander: Option<AtsiStatus>,
}

/// Data for updating an existing patient - all fields optional for partial updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePatientData {
    pub ihi: Option<String>,
    pub medicare_number: Option<String>,
    pub medicare_irn: Option<u8>,
    pub medicare_expiry: Option<NaiveDate>,

    pub title: Option<String>,
    pub first_name: Option<String>,
    pub middle_name: Option<String>,
    pub last_name: Option<String>,
    pub preferred_name: Option<String>,
    pub date_of_birth: Option<NaiveDate>,
    pub gender: Option<Gender>,

    pub address: Option<Address>,
    pub phone_home: Option<String>,
    pub phone_mobile: Option<String>,
    pub email: Option<String>,

    pub emergency_contact: Option<EmergencyContact>,

    pub concession_type: Option<ConcessionType>,
    pub concession_number: Option<String>,
    pub preferred_language: Option<String>,
    pub interpreter_required: Option<bool>,
    pub aboriginal_torres_strait_islander: Option<AtsiStatus>,
}

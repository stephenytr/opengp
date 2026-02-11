use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::dto::NewPatientData;
use super::error::ValidationError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Patient {
    pub id: Uuid,

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

    pub is_active: bool,
    pub is_deceased: bool,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Patient {
    pub fn new(data: NewPatientData) -> Result<Self, ValidationError> {
        Self::validate_names(&data.first_name, &data.last_name)?;
        Self::validate_date_of_birth(data.date_of_birth)?;

        Ok(Self {
            id: Uuid::new_v4(),
            ihi: data.ihi,
            medicare_number: data.medicare_number,
            medicare_irn: data.medicare_irn,
            medicare_expiry: data.medicare_expiry,
            title: data.title,
            first_name: data.first_name,
            middle_name: data.middle_name,
            last_name: data.last_name,
            preferred_name: data.preferred_name,
            date_of_birth: data.date_of_birth,
            gender: data.gender,
            address: data.address,
            phone_home: data.phone_home,
            phone_mobile: data.phone_mobile,
            email: data.email,
            emergency_contact: data.emergency_contact,
            is_active: true,
            is_deceased: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    pub fn age(&self) -> u32 {
        let today = Utc::now().date_naive();
        today.years_since(self.date_of_birth).unwrap_or(0)
    }

    pub fn is_child(&self) -> bool {
        self.age() < 18
    }

    fn validate_names(first: &str, last: &str) -> Result<(), ValidationError> {
        if first.trim().is_empty() {
            return Err(ValidationError::EmptyName("first name".to_string()));
        }
        if last.trim().is_empty() {
            return Err(ValidationError::EmptyName("last name".to_string()));
        }
        Ok(())
    }

    fn validate_date_of_birth(dob: NaiveDate) -> Result<(), ValidationError> {
        let today = Utc::now().date_naive();
        if dob > today {
            return Err(ValidationError::InvalidDateOfBirth);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Address {
    pub line1: Option<String>,
    pub line2: Option<String>,
    pub suburb: Option<String>,
    pub state: Option<String>,
    pub postcode: Option<String>,
    pub country: String,
}

impl Default for Address {
    fn default() -> Self {
        Self {
            line1: None,
            line2: None,
            suburb: None,
            state: None,
            postcode: None,
            country: "Australia".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmergencyContact {
    pub name: String,
    pub phone: String,
    pub relationship: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Gender {
    Male,
    Female,
    Other,
    PreferNotToSay,
}

impl std::fmt::Display for Gender {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Gender::Male => write!(f, "Male"),
            Gender::Female => write!(f, "Female"),
            Gender::Other => write!(f, "Other"),
            Gender::PreferNotToSay => write!(f, "PreferNotToSay"),
        }
    }
}

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};
use uuid::Uuid;

use super::dto::{NewPatientData, UpdatePatientData};
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

    pub concession_type: Option<ConcessionType>,
    pub concession_number: Option<String>,
    pub preferred_language: String,
    pub interpreter_required: bool,
    pub aboriginal_torres_strait_islander: Option<AtsiStatus>,

    pub is_active: bool,
    pub is_deceased: bool,
    pub deceased_date: Option<NaiveDate>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: i32,
}

impl Patient {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        first_name: String,
        last_name: String,
        date_of_birth: NaiveDate,
        gender: Gender,
        ihi: Option<String>,
        medicare_number: Option<String>,
        medicare_irn: Option<u8>,
        medicare_expiry: Option<NaiveDate>,
        title: Option<String>,
        middle_name: Option<String>,
        preferred_name: Option<String>,
        address: Address,
        phone_home: Option<String>,
        phone_mobile: Option<String>,
        email: Option<String>,
        emergency_contact: Option<EmergencyContact>,
        concession_type: Option<ConcessionType>,
        concession_number: Option<String>,
        preferred_language: Option<String>,
        interpreter_required: Option<bool>,
        aboriginal_torres_strait_islander: Option<AtsiStatus>,
    ) -> Result<Self, ValidationError> {
        Self::validate_names(&first_name, &last_name)?;
        Self::validate_date_of_birth(date_of_birth)?;

        Ok(Self {
            id: Uuid::new_v4(),
            ihi,
            medicare_number,
            medicare_irn,
            medicare_expiry,
            title,
            first_name,
            middle_name,
            last_name,
            preferred_name,
            date_of_birth,
            gender,
            address,
            phone_home,
            phone_mobile,
            email,
            emergency_contact,
            concession_type,
            concession_number,
            preferred_language: preferred_language.unwrap_or_else(|| "English".to_string()),
            interpreter_required: interpreter_required.unwrap_or(false),
            aboriginal_torres_strait_islander,
            is_active: true,
            is_deceased: false,
            deceased_date: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            version: 1,
        })
    }

    pub fn from_dto(data: NewPatientData) -> Result<Self, ValidationError> {
        Self::new(
            data.first_name,
            data.last_name,
            data.date_of_birth,
            data.gender,
            data.ihi,
            data.medicare_number,
            data.medicare_irn,
            data.medicare_expiry,
            data.title,
            data.middle_name,
            data.preferred_name,
            data.address,
            data.phone_home,
            data.phone_mobile,
            data.email,
            data.emergency_contact,
            data.concession_type,
            data.concession_number,
            data.preferred_language,
            data.interpreter_required,
            data.aboriginal_torres_strait_islander,
        )
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

    pub fn update(&mut self, data: UpdatePatientData) -> Result<(), ValidationError> {
        if let Some(first_name) = data.first_name {
            Self::validate_names(&first_name, &self.last_name)?;
            self.first_name = first_name;
        }
        if let Some(last_name) = data.last_name {
            Self::validate_names(&self.first_name, &last_name)?;
            self.last_name = last_name;
        }
        if let Some(date_of_birth) = data.date_of_birth {
            Self::validate_date_of_birth(date_of_birth)?;
            self.date_of_birth = date_of_birth;
        }
        if let Some(gender) = data.gender {
            self.gender = gender;
        }

        self.ihi = data.ihi.or(self.ihi.clone());
        self.medicare_number = data.medicare_number.or(self.medicare_number.clone());
        self.medicare_irn = data.medicare_irn.or(self.medicare_irn);
        self.medicare_expiry = data.medicare_expiry.or(self.medicare_expiry);
        self.title = data.title.or(self.title.clone());
        self.middle_name = data.middle_name.or(self.middle_name.clone());
        self.preferred_name = data.preferred_name.or(self.preferred_name.clone());
        self.address = data.address.unwrap_or_else(|| self.address.clone());
        self.phone_home = data.phone_home.or(self.phone_home.clone());
        self.phone_mobile = data.phone_mobile.or(self.phone_mobile.clone());
        self.email = data.email.or(self.email.clone());
        self.emergency_contact = data.emergency_contact.or(self.emergency_contact.clone());
        self.concession_type = data.concession_type.or(self.concession_type);
        self.concession_number = data.concession_number.or(self.concession_number.clone());
        if let Some(lang) = data.preferred_language {
            self.preferred_language = lang;
        }
        if let Some(interpreter) = data.interpreter_required {
            self.interpreter_required = interpreter;
        }
        self.aboriginal_torres_strait_islander = data
            .aboriginal_torres_strait_islander
            .or(self.aboriginal_torres_strait_islander);

        self.updated_at = Utc::now();
        self.version += 1;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumString)]
pub enum Gender {
    Male,
    Female,
    Other,
    PreferNotToSay,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Display, EnumString)]
pub enum ConcessionType {
    DVA,
    Pensioner,
    HealthcareCard,
    SafetyNetCard,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Display, EnumString)]
pub enum AtsiStatus {
    AboriginalNotTorresStrait,
    TorresStraitNotAboriginal,
    BothAboriginalAndTorresStrait,
    NeitherAboriginalNorTorresStrait,
    NotStated,
}

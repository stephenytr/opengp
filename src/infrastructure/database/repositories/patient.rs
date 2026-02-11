use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use sqlx::{FromRow, SqlitePool};
use uuid::Uuid;

use crate::domain::patient::{Address, EmergencyContact, Gender, Patient, PatientRepository, RepositoryError};

#[derive(Debug, FromRow)]
struct PatientRow {
    id: Vec<u8>,
    ihi: Option<String>,
    medicare_number: Option<String>,
    medicare_irn: Option<i64>,
    medicare_expiry: Option<NaiveDate>,
    title: Option<String>,
    first_name: String,
    middle_name: Option<String>,
    last_name: String,
    preferred_name: Option<String>,
    date_of_birth: NaiveDate,
    gender: String,
    address_line1: Option<String>,
    address_line2: Option<String>,
    suburb: Option<String>,
    state: Option<String>,
    postcode: Option<String>,
    country: Option<String>,
    phone_home: Option<String>,
    phone_mobile: Option<String>,
    email: Option<String>,
    emergency_contact_name: Option<String>,
    emergency_contact_phone: Option<String>,
    emergency_contact_relationship: Option<String>,
    is_active: bool,
    is_deceased: bool,
    created_at: String,
    updated_at: String,
}

impl PatientRow {
    fn into_patient(self) -> Result<Patient, RepositoryError> {
        Ok(Patient {
            id: Uuid::from_slice(&self.id).map_err(|e| RepositoryError::ConstraintViolation(format!("Invalid UUID: {}", e)))?,
            ihi: self.ihi,
            medicare_number: self.medicare_number,
            medicare_irn: self.medicare_irn.map(|i| i as u8),
            medicare_expiry: self.medicare_expiry,
            title: self.title,
            first_name: self.first_name,
            middle_name: self.middle_name,
            last_name: self.last_name,
            preferred_name: self.preferred_name,
            date_of_birth: self.date_of_birth,
            gender: match self.gender.as_str() {
                "Male" => Gender::Male,
                "Female" => Gender::Female,
                "Other" => Gender::Other,
                _ => Gender::PreferNotToSay,
            },
            address: Address {
                line1: self.address_line1,
                line2: self.address_line2,
                suburb: self.suburb,
                state: self.state,
                postcode: self.postcode,
                country: self.country.unwrap_or_else(|| "Australia".to_string()),
            },
            phone_home: self.phone_home,
            phone_mobile: self.phone_mobile,
            email: self.email,
            emergency_contact: if self.emergency_contact_name.is_some() {
                Some(EmergencyContact {
                    name: self.emergency_contact_name.unwrap(),
                    phone: self.emergency_contact_phone.unwrap_or_default(),
                    relationship: self.emergency_contact_relationship.unwrap_or_default(),
                })
            } else {
                None
            },
            concession_type: None,
            concession_number: None,
            preferred_language: "English".to_string(),
            interpreter_required: false,
            aboriginal_torres_strait_islander: None,
            is_active: self.is_active,
            is_deceased: self.is_deceased,
            deceased_date: None,
            created_at: DateTime::parse_from_rfc3339(&self.created_at)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            updated_at: DateTime::parse_from_rfc3339(&self.updated_at)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
        })
    }
}

pub struct SqlxPatientRepository {
    pool: SqlitePool,
}

impl SqlxPatientRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PatientRepository for SqlxPatientRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Patient>, RepositoryError> {
        let id_bytes = id.as_bytes().to_vec();
        
        let row = sqlx::query_as::<_, PatientRow>(
            r#"
            SELECT 
                id, ihi, medicare_number, medicare_irn, medicare_expiry,
                title, first_name, middle_name, last_name, preferred_name,
                date_of_birth, gender,
                address_line1, address_line2, suburb, state, postcode, country,
                phone_home, phone_mobile, email,
                emergency_contact_name, emergency_contact_phone, emergency_contact_relationship,
                is_active, is_deceased,
                created_at, updated_at
            FROM patients
            WHERE id = ? AND is_active = TRUE
            "#
        )
        .bind(id_bytes)
        .fetch_optional(&self.pool)
        .await?;
        
        match row {
            Some(r) => Ok(Some(r.into_patient()?)),
            None => Ok(None),
        }
    }

    async fn find_by_medicare(&self, medicare: &str) -> Result<Option<Patient>, RepositoryError> {
        let row = sqlx::query_as::<_, PatientRow>(
            r#"
            SELECT 
                id, ihi, medicare_number, medicare_irn, medicare_expiry,
                title, first_name, middle_name, last_name, preferred_name,
                date_of_birth, gender,
                address_line1, address_line2, suburb, state, postcode, country,
                phone_home, phone_mobile, email,
                emergency_contact_name, emergency_contact_phone, emergency_contact_relationship,
                is_active, is_deceased,
                created_at, updated_at
            FROM patients
            WHERE medicare_number = ? AND is_active = TRUE
            "#
        )
        .bind(medicare)
        .fetch_optional(&self.pool)
        .await?;
        
        match row {
            Some(r) => Ok(Some(r.into_patient()?)),
            None => Ok(None),
        }
    }

    async fn create(&self, patient: Patient) -> Result<Patient, RepositoryError> {
        let id_bytes = patient.id.as_bytes().to_vec();
        let gender_str = patient.gender.to_string();
        let dob = patient.date_of_birth;
        let medicare_expiry = patient.medicare_expiry;
        let created_at_str = patient.created_at.to_rfc3339();
        let updated_at_str = patient.updated_at.to_rfc3339();
        let medicare_irn_i64 = patient.medicare_irn.map(|i| i as i64);
        
        let emergency_contact_name = patient.emergency_contact.as_ref().map(|ec| ec.name.clone());
        let emergency_contact_phone = patient.emergency_contact.as_ref().map(|ec| ec.phone.clone());
        let emergency_contact_relationship = patient.emergency_contact.as_ref().map(|ec| ec.relationship.clone());
        
        sqlx::query(
            r#"
            INSERT INTO patients (
                id, ihi, medicare_number, medicare_irn, medicare_expiry,
                title, first_name, middle_name, last_name, preferred_name,
                date_of_birth, gender,
                address_line1, address_line2, suburb, state, postcode, country,
                phone_home, phone_mobile, email,
                emergency_contact_name, emergency_contact_phone, emergency_contact_relationship,
                is_active, is_deceased,
                created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(id_bytes)
        .bind(&patient.ihi)
        .bind(&patient.medicare_number)
        .bind(medicare_irn_i64)
        .bind(medicare_expiry)
        .bind(&patient.title)
        .bind(&patient.first_name)
        .bind(&patient.middle_name)
        .bind(&patient.last_name)
        .bind(&patient.preferred_name)
        .bind(dob)
        .bind(gender_str)
        .bind(&patient.address.line1)
        .bind(&patient.address.line2)
        .bind(&patient.address.suburb)
        .bind(&patient.address.state)
        .bind(&patient.address.postcode)
        .bind(&patient.address.country)
        .bind(&patient.phone_home)
        .bind(&patient.phone_mobile)
        .bind(&patient.email)
        .bind(emergency_contact_name)
        .bind(emergency_contact_phone)
        .bind(emergency_contact_relationship)
        .bind(patient.is_active)
        .bind(patient.is_deceased)
        .bind(created_at_str)
        .bind(updated_at_str)
        .execute(&self.pool)
        .await?;
        
        Ok(patient)
    }

    async fn update(&self, _patient: Patient) -> Result<Patient, RepositoryError> {
        todo!("Implement update")
    }

    async fn deactivate(&self, _id: Uuid) -> Result<(), RepositoryError> {
        todo!("Implement deactivate")
    }
}

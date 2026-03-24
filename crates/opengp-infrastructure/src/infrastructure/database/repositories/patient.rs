use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use sqlx::postgres::PgPool;
use sqlx::FromRow;
use tracing::{debug, error};
use uuid::Uuid;

use crate::infrastructure::crypto::EncryptionService;
use crate::infrastructure::database::helpers::*;
use crate::infrastructure::database::sqlx_to_patient_error;
use opengp_domain::domain::error::RepositoryError as BaseRepositoryError;
use opengp_domain::domain::patient::{
    Address, EmergencyContact, Gender, Patient, PatientRepository, RepositoryError,
};

#[derive(Debug, FromRow)]
struct PatientRow {
    id: Uuid,
    ihi: Option<Vec<u8>>,
    medicare_number: Option<Vec<u8>>,
    #[sqlx(default)]
    medicare_irn: Option<i32>,
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
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    version: i32,
}

impl PatientRow {
    fn into_patient(self, crypto: &EncryptionService) -> Result<Patient, RepositoryError> {
        let patient_id = self.id;

        // Try to decrypt, but fall back to plain text if decryption fails
        // This handles both encrypted and unencrypted seed data
        let ihi = match self.ihi {
            Some(data) => {
                debug!(patient_id = %patient_id, field = "ihi", "Attempting to decrypt patient field");
                match crypto.decrypt(&data) {
                    Ok(decrypted) => {
                        debug!(patient_id = %patient_id, field = "ihi", "Decryption succeeded");
                        Some(decrypted)
                    }
                    Err(err) => {
                        error!(patient_id = %patient_id, field = "ihi", error = %err, "Decryption failed, attempting plaintext fallback");
                        // Decryption failed - try to interpret as plain text
                        match String::from_utf8(data) {
                            Ok(plain) => {
                                debug!(patient_id = %patient_id, field = "ihi", "Plaintext fallback succeeded");
                                Some(plain)
                            }
                            Err(utf8_err) => {
                                error!(patient_id = %patient_id, field = "ihi", error = %utf8_err, "Plaintext fallback failed");
                                None
                            }
                        }
                    }
                }
            }
            None => None,
        };

        let medicare_number = match self.medicare_number {
            Some(data) => {
                debug!(patient_id = %patient_id, field = "medicare_number", "Attempting to decrypt patient field");
                match crypto.decrypt(&data) {
                    Ok(decrypted) => {
                        debug!(patient_id = %patient_id, field = "medicare_number", "Decryption succeeded");
                        Some(decrypted)
                    }
                    Err(err) => {
                        error!(patient_id = %patient_id, field = "medicare_number", error = %err, "Decryption failed, attempting plaintext fallback");
                        // Decryption failed - try to interpret as plain text
                        match String::from_utf8(data) {
                            Ok(plain) => {
                                debug!(patient_id = %patient_id, field = "medicare_number", "Plaintext fallback succeeded");
                                Some(plain)
                            }
                            Err(utf8_err) => {
                                error!(patient_id = %patient_id, field = "medicare_number", error = %utf8_err, "Plaintext fallback failed");
                                None
                            }
                        }
                    }
                }
            }
            None => None,
        };

        debug!(patient_id = %patient_id, "Patient row converted into domain model");

        Ok(Patient {
            id: self.id,
            ihi,
            medicare_number,
            medicare_irn: self.medicare_irn.map(|i| i as u8),
            medicare_expiry: self.medicare_expiry,
            title: self.title,
            first_name: self.first_name,
            middle_name: self.middle_name,
            last_name: self.last_name,
            preferred_name: self.preferred_name,
            date_of_birth: self.date_of_birth,
            gender: self
                .gender
                .parse::<Gender>()
                .unwrap_or(Gender::PreferNotToSay),
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
            emergency_contact: if let Some(name) = self.emergency_contact_name {
                Some(EmergencyContact {
                    name,
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
            created_at: self.created_at,
            updated_at: self.updated_at,
            version: self.version,
        })
    }
}

const PATIENT_SELECT_QUERY: &str = r#"
SELECT 
    id, ihi, medicare_number, medicare_irn, medicare_expiry,
    title, first_name, middle_name, last_name, preferred_name,
    date_of_birth, gender,
    address_line1, address_line2, suburb, state, postcode, country,
    phone_home, phone_mobile, email,
    emergency_contact_name, emergency_contact_phone, emergency_contact_relationship,
    is_active, is_deceased,
    created_at, updated_at,
    version
FROM patients
"#;

pub struct SqlxPatientRepository {
    pool: PgPool,
    crypto: Arc<EncryptionService>,
}

impl SqlxPatientRepository {
    pub fn new(pool: PgPool, crypto: Arc<EncryptionService>) -> Self {
        Self { pool, crypto }
    }
}

#[async_trait]
impl PatientRepository for SqlxPatientRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Patient>, RepositoryError> {
        let row = sqlx::query_as::<_, PatientRow>(&format!(
            "{} WHERE id = $1 AND is_active = TRUE",
            PATIENT_SELECT_QUERY
        ))
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(sqlx_to_patient_error)?;

        match row {
            Some(r) => Ok(Some(r.into_patient(&self.crypto)?)),
            None => Ok(None),
        }
    }

    async fn find_by_medicare(&self, medicare: &str) -> Result<Option<Patient>, RepositoryError> {
        // With encryption, we cannot search directly on medicare number
        // We must fetch all patients and filter in memory
        let patients = self.list_active().await?;
        Ok(patients.into_iter().find(|p| {
            p.medicare_number
                .as_ref()
                .map(|m| m == medicare)
                .unwrap_or(false)
        }))
    }

    async fn list_active(&self) -> Result<Vec<Patient>, RepositoryError> {
        debug!("Executing patient list_active query");
        let rows = sqlx::query_as::<_, PatientRow>(&format!(
            "{} WHERE is_active = TRUE ORDER BY last_name, first_name",
            PATIENT_SELECT_QUERY
        ))
        .fetch_all(&self.pool)
        .await
        .map_err(|err| {
            error!(error = %err, "Patient list_active query failed");
            sqlx_to_patient_error(err)
        })?;

        debug!(
            row_count = rows.len(),
            "Patient list_active query returned rows"
        );

        let mut patients = Vec::with_capacity(rows.len());
        for row in rows {
            match row.into_patient(&self.crypto) {
                Ok(patient) => patients.push(patient),
                Err(err) => {
                    error!(error = %err, "Failed converting patient row into domain model");
                    return Err(err);
                }
            }
        }

        debug!(
            patient_count = patients.len(),
            "Patient list_active returning converted patients"
        );

        Ok(patients)
    }

    async fn search(&self, query: &str) -> Result<Vec<Patient>, RepositoryError> {
        let all_patients = self.list_active().await?;

        if query.is_empty() {
            return Ok(all_patients);
        }

        let query_lower = query.to_lowercase();
        let filtered: Vec<Patient> = all_patients
            .into_iter()
            .filter(|p| {
                let full_name = format!("{} {}", p.first_name, p.last_name).to_lowercase();
                let preferred = p
                    .preferred_name
                    .as_ref()
                    .map(|n| n.to_lowercase())
                    .unwrap_or_default();
                let medicare = p
                    .medicare_number
                    .as_ref()
                    .map(|m| m.to_lowercase())
                    .unwrap_or_default();
                let email = p
                    .email
                    .as_ref()
                    .map(|e| e.to_lowercase())
                    .unwrap_or_default();
                let phone = p
                    .phone_mobile
                    .as_ref()
                    .or(p.phone_home.as_ref())
                    .map(|p| p.to_lowercase())
                    .unwrap_or_default();

                full_name.contains(&query_lower)
                    || preferred.contains(&query_lower)
                    || medicare.contains(&query_lower)
                    || email.contains(&query_lower)
                    || phone.contains(&query_lower)
            })
            .collect();

        Ok(filtered)
    }

    async fn create(&self, patient: Patient) -> Result<Patient, RepositoryError> {
        let gender_str = patient.gender.to_string();
        let dob = patient.date_of_birth;
        let medicare_expiry = patient.medicare_expiry;
        let medicare_irn_i32 = patient.medicare_irn.map(|i| i as i32);

        // Encrypt sensitive fields
        let ihi_encrypted: Option<Vec<u8>> = match &patient.ihi {
            Some(ihi) => Some(self.crypto.encrypt(ihi).map_err(|e| {
                RepositoryError::Encryption(format!("Failed to encrypt IHI: {}", e))
            })?),
            None => None,
        };

        let medicare_encrypted: Option<Vec<u8>> = match &patient.medicare_number {
            Some(num) => Some(self.crypto.encrypt(num).map_err(|e| {
                RepositoryError::Encryption(format!("Failed to encrypt Medicare number: {}", e))
            })?),
            None => None,
        };

        let emergency_contact_name = patient.emergency_contact.as_ref().map(|ec| ec.name.clone());
        let emergency_contact_phone = patient
            .emergency_contact
            .as_ref()
            .map(|ec| ec.phone.clone());
        let emergency_contact_relationship = patient
            .emergency_contact
            .as_ref()
            .map(|ec| ec.relationship.clone());

        let result = sqlx::query(
            r#"
        INSERT INTO patients (
            id, ihi, medicare_number, medicare_irn, medicare_expiry,
            title, first_name, middle_name, last_name, preferred_name,
            date_of_birth, gender,
            address_line1, address_line2, suburb, state, postcode, country,
            phone_home, phone_mobile, email,
            emergency_contact_name, emergency_contact_phone, emergency_contact_relationship,
            is_active, is_deceased,
            created_at, updated_at,
            version
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26, $27, $28, $29)
        "#,
        )
        .bind(patient.id)
        .bind(ihi_encrypted)
        .bind(medicare_encrypted)
        .bind(medicare_irn_i32)
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
        .bind(patient.created_at)
        .bind(patient.updated_at)
        .bind(patient.version)
        .execute(&self.pool)
        .await;

        match result {
            Ok(_) => Ok(patient),
            Err(sqlx::Error::Database(db_err)) => Err(map_db_error(db_err)),
            Err(e) => Err(RepositoryError::Base(BaseRepositoryError::Database(
                e.to_string(),
            ))),
        }
    }

    async fn update(&self, patient: Patient) -> Result<Patient, RepositoryError> {
        let gender_str = patient.gender.to_string();
        let dob = patient.date_of_birth;
        let medicare_expiry = patient.medicare_expiry;
        let medicare_irn_i32 = patient.medicare_irn.map(|i| i as i32);

        // Encrypt sensitive fields
        let ihi_encrypted: Option<Vec<u8>> = match &patient.ihi {
            Some(ihi) => Some(self.crypto.encrypt(ihi).map_err(|e| {
                RepositoryError::Encryption(format!("Failed to encrypt IHI: {}", e))
            })?),
            None => None,
        };

        let medicare_encrypted: Option<Vec<u8>> = match &patient.medicare_number {
            Some(num) => Some(self.crypto.encrypt(num).map_err(|e| {
                RepositoryError::Encryption(format!("Failed to encrypt Medicare number: {}", e))
            })?),
            None => None,
        };

        let emergency_contact_name = patient.emergency_contact.as_ref().map(|ec| ec.name.clone());
        let emergency_contact_phone = patient
            .emergency_contact
            .as_ref()
            .map(|ec| ec.phone.clone());
        let emergency_contact_relationship = patient
            .emergency_contact
            .as_ref()
            .map(|ec| ec.relationship.clone());

        let current_version =
            sqlx::query_scalar::<_, i32>("SELECT version FROM patients WHERE id = $1")
                .bind(patient.id)
                .fetch_optional(&self.pool)
                .await
                .map_err(sqlx_to_patient_error)?;

        let current_version = match current_version {
            Some(version) => version,
            None => return Err(RepositoryError::Base(BaseRepositoryError::NotFound)),
        };

        if current_version != patient.version {
            return Err(RepositoryError::Base(BaseRepositoryError::Conflict(
                "Patient was modified by another user".to_string(),
            )));
        }

        let new_version = patient.version + 1;

        let result = sqlx::query(
            r#"
        UPDATE patients SET
            ihi = $1,
            medicare_number = $2,
            medicare_irn = $3,
            medicare_expiry = $4,
            title = $5,
            first_name = $6,
            middle_name = $7,
            last_name = $8,
            preferred_name = $9,
            date_of_birth = $10,
            gender = $11,
            address_line1 = $12,
            address_line2 = $13,
            suburb = $14,
            state = $15,
            postcode = $16,
            country = $17,
            phone_home = $18,
            phone_mobile = $19,
            email = $20,
            emergency_contact_name = $21,
            emergency_contact_phone = $22,
            emergency_contact_relationship = $23,
            is_active = $24,
            is_deceased = $25,
            updated_at = $26,
            version = $27
        WHERE id = $28 AND version = $29
        "#,
        )
        .bind(ihi_encrypted)
        .bind(medicare_encrypted)
        .bind(medicare_irn_i32)
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
        .bind(patient.updated_at)
        .bind(new_version)
        .bind(patient.id)
        .bind(patient.version)
        .execute(&self.pool)
        .await;

        match result {
            Ok(query_result) => {
                if query_result.rows_affected() == 0 {
                    return Err(RepositoryError::Base(BaseRepositoryError::Conflict(
                        "Patient was modified by another user".to_string(),
                    )));
                }

                let mut updated_patient = patient;
                updated_patient.version = new_version;
                Ok(updated_patient)
            }
            Err(sqlx::Error::Database(db_err)) => Err(map_db_error(db_err)),
            Err(e) => Err(RepositoryError::Base(BaseRepositoryError::Database(
                e.to_string(),
            ))),
        }
    }

    async fn deactivate(&self, id: Uuid) -> Result<(), RepositoryError> {
        let result = sqlx::query(
            "UPDATE patients SET is_active = FALSE, updated_at = CURRENT_TIMESTAMP, version = version + 1 WHERE id = $1 AND is_active = TRUE",
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(sqlx_to_patient_error)?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::Base(BaseRepositoryError::NotFound));
        }

        Ok(())
    }
}

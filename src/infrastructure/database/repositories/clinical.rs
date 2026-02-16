use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{FromRow, SqlitePool};
use uuid::Uuid;

use crate::domain::clinical::{
    Allergy, AllergyRepository, AllergyType, Consultation, ConsultationRepository,
    ConditionStatus, FamilyHistory, FamilyHistoryRepository, MedicalHistory,
    MedicalHistoryRepository, Severity, SOAPNotes, SocialHistory,
    SocialHistoryRepository, VitalSigns, VitalSignsRepository,
};
use crate::domain::clinical::RepositoryError;
use crate::infrastructure::crypto::EncryptionService;

fn uuid_to_bytes(id: &Uuid) -> Vec<u8> {
    id.as_bytes().to_vec()
}

fn bytes_to_uuid(bytes: &[u8]) -> Result<Uuid, RepositoryError> {
    Uuid::from_slice(bytes).map_err(|e| RepositoryError::Database(sqlx::Error::Decode(Box::new(e))))
}

fn datetime_to_string(dt: &DateTime<Utc>) -> String {
    dt.to_rfc3339()
}

fn string_to_datetime(s: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| {
            tracing::warn!("Failed to parse datetime string: {}, using current time", s);
            Utc::now()
        })
}

#[derive(Debug, FromRow)]
struct ConsultationRow {
    id: Vec<u8>,
    patient_id: Vec<u8>,
    practitioner_id: Vec<u8>,
    appointment_id: Option<Vec<u8>>,
    consultation_date: String,
    soap_subjective: Option<Vec<u8>>,
    soap_objective: Option<Vec<u8>>,
    soap_assessment: Option<Vec<u8>>,
    soap_plan: Option<Vec<u8>>,
    is_signed: bool,
    signed_at: Option<String>,
    signed_by: Option<Vec<u8>>,
    created_at: String,
    updated_at: String,
    created_by: Vec<u8>,
    updated_by: Option<Vec<u8>>,
}

impl ConsultationRow {
    fn into_consultation(
        self,
        crypto: &EncryptionService,
    ) -> Result<Consultation, RepositoryError> {
        let decrypt_soap_field =
            |encrypted: Option<Vec<u8>>| -> Result<Option<String>, RepositoryError> {
                match encrypted {
                    Some(data) => {
                        let decrypted = crypto.decrypt(&data).map_err(|e| {
                            RepositoryError::Decryption(format!(
                                "Failed to decrypt SOAP note: {}",
                                e
                            ))
                        })?;
                        Ok(Some(decrypted))
                    }
                    None => Ok(None),
                }
            };

        Ok(Consultation {
            id: bytes_to_uuid(&self.id)?,
            patient_id: bytes_to_uuid(&self.patient_id)?,
            practitioner_id: bytes_to_uuid(&self.practitioner_id)?,
            appointment_id: self
                .appointment_id
                .as_ref()
                .map(|b| bytes_to_uuid(b))
                .transpose()?,
            consultation_date: string_to_datetime(&self.consultation_date),
            soap_notes: SOAPNotes {
                subjective: decrypt_soap_field(self.soap_subjective)?,
                objective: decrypt_soap_field(self.soap_objective)?,
                assessment: decrypt_soap_field(self.soap_assessment)?,
                plan: decrypt_soap_field(self.soap_plan)?,
            },
            is_signed: self.is_signed,
            signed_at: self.signed_at.as_ref().map(|s| string_to_datetime(s)),
            signed_by: self
                .signed_by
                .as_ref()
                .map(|b| bytes_to_uuid(b))
                .transpose()?,
            created_at: string_to_datetime(&self.created_at),
            updated_at: string_to_datetime(&self.updated_at),
            created_by: bytes_to_uuid(&self.created_by)?,
            updated_by: self
                .updated_by
                .as_ref()
                .map(|b| bytes_to_uuid(b))
                .transpose()?,
        })
    }
}

pub struct SqlxClinicalRepository {
    pool: SqlitePool,
    crypto: Arc<EncryptionService>,
}

impl SqlxClinicalRepository {
    pub fn new(pool: SqlitePool, crypto: Arc<EncryptionService>) -> Self {
        Self { pool, crypto }
    }
}

#[async_trait]
impl ConsultationRepository for SqlxClinicalRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Consultation>, RepositoryError> {
        let id_bytes = uuid_to_bytes(&id);

        let row = sqlx::query_as::<_, ConsultationRow>(
            r#"
            SELECT 
                id, patient_id, practitioner_id, appointment_id,
                consultation_date, soap_subjective, soap_objective, 
                soap_assessment, soap_plan, is_signed, signed_at, signed_by,
                created_at, updated_at, created_by, updated_by
            FROM consultations
            WHERE id = ?
            "#,
        )
        .bind(id_bytes)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(r) => Ok(Some(r.into_consultation(&self.crypto)?)),
            None => Ok(None),
        }
    }

    async fn find_by_patient(
        &self,
        patient_id: Uuid,
    ) -> Result<Vec<Consultation>, RepositoryError> {
        let patient_bytes = uuid_to_bytes(&patient_id);

        let rows = sqlx::query_as::<_, ConsultationRow>(
            r#"
            SELECT 
                id, patient_id, practitioner_id, appointment_id,
                consultation_date, soap_subjective, soap_objective, 
                soap_assessment, soap_plan, is_signed, signed_at, signed_by,
                created_at, updated_at, created_by, updated_by
            FROM consultations
            WHERE patient_id = ?
            ORDER BY consultation_date DESC
            "#,
        )
        .bind(patient_bytes)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|r| r.into_consultation(&self.crypto))
            .collect()
    }

    async fn find_by_date_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<Consultation>, RepositoryError> {
        let start_str = datetime_to_string(&start);
        let end_str = datetime_to_string(&end);

        let rows = sqlx::query_as::<_, ConsultationRow>(
            r#"
            SELECT 
                id, patient_id, practitioner_id, appointment_id,
                consultation_date, soap_subjective, soap_objective, 
                soap_assessment, soap_plan, is_signed, signed_at, signed_by,
                created_at, updated_at, created_by, updated_by
            FROM consultations
            WHERE consultation_date BETWEEN ? AND ?
            ORDER BY consultation_date DESC
            "#,
        )
        .bind(&start_str)
        .bind(&end_str)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|r| r.into_consultation(&self.crypto))
            .collect()
    }

    async fn create(&self, consultation: Consultation) -> Result<Consultation, RepositoryError> {
        let id_bytes = uuid_to_bytes(&consultation.id);
        let patient_bytes = uuid_to_bytes(&consultation.patient_id);
        let practitioner_bytes = uuid_to_bytes(&consultation.practitioner_id);
        let appointment_bytes = consultation.appointment_id.as_ref().map(uuid_to_bytes);
        let created_by_bytes = uuid_to_bytes(&consultation.created_by);
        let consultation_date_str = datetime_to_string(&consultation.consultation_date);
        let created_at_str = datetime_to_string(&consultation.created_at);
        let updated_at_str = datetime_to_string(&consultation.updated_at);

        let encrypt_soap_field =
            |text: &Option<String>| -> Result<Option<Vec<u8>>, RepositoryError> {
                match text {
                    Some(t) => Ok(Some(self.crypto.encrypt(t).map_err(|e| {
                        RepositoryError::Encryption(format!("Failed to encrypt SOAP note: {}", e))
                    })?)),
                    None => Ok(None),
                }
            };

        let soap_subjective = encrypt_soap_field(&consultation.soap_notes.subjective)?;
        let soap_objective = encrypt_soap_field(&consultation.soap_notes.objective)?;
        let soap_assessment = encrypt_soap_field(&consultation.soap_notes.assessment)?;
        let soap_plan = encrypt_soap_field(&consultation.soap_notes.plan)?;

        sqlx::query(
            r#"
            INSERT INTO consultations (
                id, patient_id, practitioner_id, appointment_id,
                consultation_date, soap_subjective, soap_objective, 
                soap_assessment, soap_plan, is_signed, signed_at, signed_by,
                created_at, updated_at, created_by, updated_by
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(id_bytes)
        .bind(patient_bytes)
        .bind(practitioner_bytes)
        .bind(appointment_bytes)
        .bind(&consultation_date_str)
        .bind(soap_subjective)
        .bind(soap_objective)
        .bind(soap_assessment)
        .bind(soap_plan)
        .bind(consultation.is_signed)
        .bind(consultation.signed_at.as_ref().map(datetime_to_string))
        .bind(consultation.signed_by.as_ref().map(uuid_to_bytes))
        .bind(&created_at_str)
        .bind(&updated_at_str)
        .bind(created_by_bytes)
        .bind(consultation.updated_by.as_ref().map(uuid_to_bytes))
        .execute(&self.pool)
        .await?;

        Ok(consultation)
    }

    async fn update(&self, consultation: Consultation) -> Result<Consultation, RepositoryError> {
        let id_bytes = uuid_to_bytes(&consultation.id);
        let updated_at_str = datetime_to_string(&consultation.updated_at);
        let updated_by_bytes = consultation.updated_by.as_ref().map(uuid_to_bytes);

        let encrypt_soap_field =
            |text: &Option<String>| -> Result<Option<Vec<u8>>, RepositoryError> {
                match text {
                    Some(t) => Ok(Some(self.crypto.encrypt(t).map_err(|e| {
                        RepositoryError::Encryption(format!("Failed to encrypt SOAP note: {}", e))
                    })?)),
                    None => Ok(None),
                }
            };

        let soap_subjective = encrypt_soap_field(&consultation.soap_notes.subjective)?;
        let soap_objective = encrypt_soap_field(&consultation.soap_notes.objective)?;
        let soap_assessment = encrypt_soap_field(&consultation.soap_notes.assessment)?;
        let soap_plan = encrypt_soap_field(&consultation.soap_notes.plan)?;

        sqlx::query(
            r#"
            UPDATE consultations
            SET 
                soap_subjective = ?, soap_objective = ?, 
                soap_assessment = ?, soap_plan = ?,
                is_signed = ?, signed_at = ?, signed_by = ?,
                updated_at = ?, updated_by = ?
            WHERE id = ?
            "#,
        )
        .bind(soap_subjective)
        .bind(soap_objective)
        .bind(soap_assessment)
        .bind(soap_plan)
        .bind(consultation.is_signed)
        .bind(consultation.signed_at.as_ref().map(datetime_to_string))
        .bind(consultation.signed_by.as_ref().map(uuid_to_bytes))
        .bind(&updated_at_str)
        .bind(updated_by_bytes)
        .bind(id_bytes)
        .execute(&self.pool)
        .await?;

        Ok(consultation)
    }

    async fn sign(&self, id: Uuid, user_id: Uuid) -> Result<(), RepositoryError> {
        let id_bytes = uuid_to_bytes(&id);
        let user_bytes = uuid_to_bytes(&user_id);
        let signed_at = datetime_to_string(&Utc::now());

        let result = sqlx::query(
            r#"
            UPDATE consultations
            SET is_signed = TRUE, signed_at = ?, signed_by = ?, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(&signed_at)
        .bind(user_bytes)
        .bind(&signed_at)
        .bind(id_bytes)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound(id));
        }

        Ok(())
    }
}

#[derive(Debug, FromRow)]
struct SocialHistoryRow {
    id: Vec<u8>,
    patient_id: Vec<u8>,
    smoking_status: Option<String>,
    cigarettes_per_day: Option<i64>,
    smoking_quit_date: Option<chrono::NaiveDate>,
    alcohol_status: Option<String>,
    standard_drinks_per_week: Option<i64>,
    exercise_frequency: Option<String>,
    occupation: Option<String>,
    living_situation: Option<String>,
    support_network: Option<String>,
    notes: Option<Vec<u8>>,
    updated_at: String,
    updated_by: Vec<u8>,
}

impl SocialHistoryRow {
    fn into_social_history(
        self,
        crypto: &EncryptionService,
    ) -> Result<SocialHistory, RepositoryError> {
        use crate::domain::clinical::{AlcoholStatus, SmokingStatus};

        let notes = match self.notes {
            Some(encrypted) => {
                let decrypted = crypto.decrypt(&encrypted).map_err(|e| {
                    RepositoryError::Decryption(format!(
                        "Failed to decrypt social history notes: {}",
                        e
                    ))
                })?;
                Some(decrypted)
            }
            None => None,
        };

        Ok(SocialHistory {
            id: bytes_to_uuid(&self.id)?,
            patient_id: bytes_to_uuid(&self.patient_id)?,
            smoking_status: self
                .smoking_status
                .as_ref()
                .and_then(|s| s.parse().ok())
                .unwrap_or(SmokingStatus::NeverSmoked),
            cigarettes_per_day: self.cigarettes_per_day.map(|i| i as u8),
            smoking_quit_date: self.smoking_quit_date,
            alcohol_status: self
                .alcohol_status
                .as_ref()
                .and_then(|s| s.parse().ok())
                .unwrap_or(AlcoholStatus::None),
            standard_drinks_per_week: self.standard_drinks_per_week.map(|i| i as u8),
            exercise_frequency: self
                .exercise_frequency
                .as_ref()
                .and_then(|s| s.parse().ok()),
            occupation: self.occupation,
            living_situation: self.living_situation,
            support_network: self.support_network,
            notes,
            updated_at: string_to_datetime(&self.updated_at),
            updated_by: bytes_to_uuid(&self.updated_by)?,
        })
    }
}

pub struct SqlxSocialHistoryRepository {
    pool: SqlitePool,
    crypto: Arc<EncryptionService>,
}

impl SqlxSocialHistoryRepository {
    pub fn new(pool: SqlitePool, crypto: Arc<EncryptionService>) -> Self {
        Self { pool, crypto }
    }
}

#[async_trait]
impl SocialHistoryRepository for SqlxSocialHistoryRepository {
    async fn find_by_patient(
        &self,
        patient_id: Uuid,
    ) -> Result<Option<SocialHistory>, RepositoryError> {
        let patient_bytes = uuid_to_bytes(&patient_id);

        let row = sqlx::query_as::<_, SocialHistoryRow>(
            r#"
            SELECT 
                id, patient_id, smoking_status, cigarettes_per_day, 
                smoking_quit_date, alcohol_status, standard_drinks_per_week,
                exercise_frequency, occupation, living_situation, support_network,
                notes, updated_at, updated_by
            FROM social_history
            WHERE patient_id = ?
            "#,
        )
        .bind(patient_bytes)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(r) => Ok(Some(r.into_social_history(&self.crypto)?)),
            None => Ok(None),
        }
    }

    async fn create(&self, history: SocialHistory) -> Result<SocialHistory, RepositoryError> {
        let id_bytes = uuid_to_bytes(&history.id);
        let patient_bytes = uuid_to_bytes(&history.patient_id);
        let updated_by_bytes = uuid_to_bytes(&history.updated_by);
        let updated_at_str = datetime_to_string(&history.updated_at);

        let notes_encrypted: Option<Vec<u8>> = match &history.notes {
            Some(notes) => Some(self.crypto.encrypt(notes).map_err(|e| {
                RepositoryError::Encryption(format!(
                    "Failed to encrypt social history notes: {}",
                    e
                ))
            })?),
            None => None,
        };

        sqlx::query(
            r#"
            INSERT INTO social_history (
                id, patient_id, smoking_status, cigarettes_per_day, 
                smoking_quit_date, alcohol_status, standard_drinks_per_week,
                exercise_frequency, occupation, living_situation, support_network,
                notes, updated_at, updated_by
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(id_bytes)
        .bind(patient_bytes)
        .bind(history.smoking_status.to_string())
        .bind(history.cigarettes_per_day.map(|i| i as i64))
        .bind(history.smoking_quit_date)
        .bind(history.alcohol_status.to_string())
        .bind(history.standard_drinks_per_week.map(|i| i as i64))
        .bind(history.exercise_frequency.as_ref().map(|e| e.to_string()))
        .bind(&history.occupation)
        .bind(&history.living_situation)
        .bind(&history.support_network)
        .bind(notes_encrypted)
        .bind(&updated_at_str)
        .bind(updated_by_bytes)
        .execute(&self.pool)
        .await?;

        Ok(history)
    }

    async fn update(&self, history: SocialHistory) -> Result<SocialHistory, RepositoryError> {
        let id_bytes = uuid_to_bytes(&history.id);
        let updated_by_bytes = uuid_to_bytes(&history.updated_by);
        let updated_at_str = datetime_to_string(&history.updated_at);

        let notes_encrypted: Option<Vec<u8>> = match &history.notes {
            Some(notes) => Some(self.crypto.encrypt(notes).map_err(|e| {
                RepositoryError::Encryption(format!(
                    "Failed to encrypt social history notes: {}",
                    e
                ))
            })?),
            None => None,
        };

        sqlx::query(
            r#"
            UPDATE social_history
            SET 
                smoking_status = ?, cigarettes_per_day = ?, 
                smoking_quit_date = ?, alcohol_status = ?, standard_drinks_per_week = ?,
                exercise_frequency = ?, occupation = ?, living_situation = ?, support_network = ?,
                notes = ?, updated_at = ?, updated_by = ?
            WHERE id = ?
            "#,
        )
        .bind(history.smoking_status.to_string())
        .bind(history.cigarettes_per_day.map(|i| i as i64))
        .bind(history.smoking_quit_date)
        .bind(history.alcohol_status.to_string())
        .bind(history.standard_drinks_per_week.map(|i| i as i64))
        .bind(history.exercise_frequency.as_ref().map(|e| e.to_string()))
        .bind(&history.occupation)
        .bind(&history.living_situation)
        .bind(&history.support_network)
        .bind(notes_encrypted)
        .bind(&updated_at_str)
        .bind(updated_by_bytes)
        .bind(id_bytes)
        .execute(&self.pool)
        .await?;

        Ok(history)
    }
}

// ============================================================================
// Allergy Repository
// ============================================================================

#[derive(Debug, FromRow)]
struct AllergyRow {
    id: Vec<u8>,
    patient_id: Vec<u8>,
    allergen: String,
    allergy_type: String,
    severity: String,
    reaction: Option<Vec<u8>>,
    onset_date: Option<chrono::NaiveDate>,
    notes: Option<Vec<u8>>,
    is_active: bool,
    created_at: String,
    updated_at: String,
    created_by: Vec<u8>,
    updated_by: Option<Vec<u8>>,
}

impl AllergyRow {
    fn into_allergy(self, crypto: &EncryptionService) -> Result<Allergy, RepositoryError> {
        let reaction = match self.reaction {
            Some(encrypted) => {
                let decrypted = crypto.decrypt(&encrypted).map_err(|e| {
                    RepositoryError::Decryption(format!("Failed to decrypt allergy reaction: {}", e))
                })?;
                Some(decrypted)
            }
            None => None,
        };

        let notes = match self.notes {
            Some(encrypted) => {
                let decrypted = crypto.decrypt(&encrypted).map_err(|e| {
                    RepositoryError::Decryption(format!("Failed to decrypt allergy notes: {}", e))
                })?;
                Some(decrypted)
            }
            None => None,
        };

        Ok(Allergy {
            id: bytes_to_uuid(&self.id)?,
            patient_id: bytes_to_uuid(&self.patient_id)?,
            allergen: self.allergen,
            allergy_type: self.allergy_type.parse().unwrap_or(AllergyType::Other),
            severity: self.severity.parse().unwrap_or(Severity::Mild),
            reaction,
            onset_date: self.onset_date,
            notes,
            is_active: self.is_active,
            created_at: string_to_datetime(&self.created_at),
            updated_at: string_to_datetime(&self.updated_at),
            created_by: bytes_to_uuid(&self.created_by)?,
            updated_by: self.updated_by.as_ref().map(|b| bytes_to_uuid(b)).transpose()?,
        })
    }
}

pub struct SqlxAllergyRepository {
    pool: SqlitePool,
    crypto: Arc<EncryptionService>,
}

impl SqlxAllergyRepository {
    pub fn new(pool: SqlitePool, crypto: Arc<EncryptionService>) -> Self {
        Self { pool, crypto }
    }
}

#[async_trait]
impl AllergyRepository for SqlxAllergyRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Allergy>, RepositoryError> {
        let row = sqlx::query_as::<_, AllergyRow>(
            "SELECT id, patient_id, allergen, allergy_type, severity, reaction, onset_date, notes, is_active, created_at, updated_at, created_by, updated_by FROM allergies WHERE id = ?"
        )
        .bind(uuid_to_bytes(&id))
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(r) => Ok(Some(r.into_allergy(&self.crypto)?)),
            None => Ok(None),
        }
    }

    async fn find_by_patient(&self, patient_id: Uuid) -> Result<Vec<Allergy>, RepositoryError> {
        let rows = sqlx::query_as::<_, AllergyRow>(
            "SELECT id, patient_id, allergen, allergy_type, severity, reaction, onset_date, notes, is_active, created_at, updated_at, created_by, updated_by FROM allergies WHERE patient_id = ? ORDER BY created_at DESC"
        )
        .bind(uuid_to_bytes(&patient_id))
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|r| r.into_allergy(&self.crypto))
            .collect()
    }

    async fn find_active_by_patient(&self, patient_id: Uuid) -> Result<Vec<Allergy>, RepositoryError> {
        let rows = sqlx::query_as::<_, AllergyRow>(
            "SELECT id, patient_id, allergen, allergy_type, severity, reaction, onset_date, notes, is_active, created_at, updated_at, created_by, updated_by FROM allergies WHERE patient_id = ? AND is_active = TRUE ORDER BY created_at DESC"
        )
        .bind(uuid_to_bytes(&patient_id))
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|r| r.into_allergy(&self.crypto))
            .collect()
    }

    async fn create(&self, allergy: Allergy) -> Result<Allergy, RepositoryError> {
        let id_bytes = uuid_to_bytes(&allergy.id);
        let patient_bytes = uuid_to_bytes(&allergy.patient_id);
        let created_by_bytes = uuid_to_bytes(&allergy.created_by);
        let updated_by_bytes = allergy.updated_by.as_ref().map(uuid_to_bytes);
        let created_at_str = datetime_to_string(&allergy.created_at);
        let updated_at_str = datetime_to_string(&allergy.updated_at);

        let reaction_encrypted: Option<Vec<u8>> = allergy.reaction.as_ref()
            .map(|r| self.crypto.encrypt(r))
            .transpose()
            .map_err(|e| RepositoryError::Encryption(format!("Failed to encrypt allergy reaction: {}", e)))?;

        let notes_encrypted: Option<Vec<u8>> = allergy.notes.as_ref()
            .map(|n| self.crypto.encrypt(n))
            .transpose()
            .map_err(|e| RepositoryError::Encryption(format!("Failed to encrypt allergy notes: {}", e)))?;

        sqlx::query(
            "INSERT INTO allergies (id, patient_id, allergen, allergy_type, severity, reaction, onset_date, notes, is_active, created_at, updated_at, created_by, updated_by) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(id_bytes)
        .bind(patient_bytes)
        .bind(&allergy.allergen)
        .bind(allergy.allergy_type.to_string())
        .bind(allergy.severity.to_string())
        .bind(reaction_encrypted)
        .bind(allergy.onset_date)
        .bind(notes_encrypted)
        .bind(allergy.is_active)
        .bind(&created_at_str)
        .bind(&updated_at_str)
        .bind(created_by_bytes)
        .bind(updated_by_bytes)
        .execute(&self.pool)
        .await?;

        Ok(allergy)
    }

    async fn update(&self, allergy: Allergy) -> Result<Allergy, RepositoryError> {
        let id_bytes = uuid_to_bytes(&allergy.id);
        let updated_at_str = datetime_to_string(&allergy.updated_at);
        let updated_by_bytes = allergy.updated_by.as_ref().map(uuid_to_bytes);

        let reaction_encrypted: Option<Vec<u8>> = allergy.reaction.as_ref()
            .map(|r| self.crypto.encrypt(r))
            .transpose()
            .map_err(|e| RepositoryError::Encryption(format!("Failed to encrypt allergy reaction: {}", e)))?;

        let notes_encrypted: Option<Vec<u8>> = allergy.notes.as_ref()
            .map(|n| self.crypto.encrypt(n))
            .transpose()
            .map_err(|e| RepositoryError::Encryption(format!("Failed to encrypt allergy notes: {}", e)))?;

        sqlx::query(
            "UPDATE allergies SET allergen = ?, allergy_type = ?, severity = ?, reaction = ?, onset_date = ?, notes = ?, is_active = ?, updated_at = ?, updated_by = ? WHERE id = ?"
        )
        .bind(&allergy.allergen)
        .bind(allergy.allergy_type.to_string())
        .bind(allergy.severity.to_string())
        .bind(reaction_encrypted)
        .bind(allergy.onset_date)
        .bind(notes_encrypted)
        .bind(allergy.is_active)
        .bind(&updated_at_str)
        .bind(updated_by_bytes)
        .bind(id_bytes)
        .execute(&self.pool)
        .await?;

        Ok(allergy)
    }

    async fn deactivate(&self, id: Uuid) -> Result<(), RepositoryError> {
        let id_bytes = uuid_to_bytes(&id);
        let updated_at_str = datetime_to_string(&Utc::now());

        sqlx::query(
            "UPDATE allergies SET is_active = FALSE, updated_at = ? WHERE id = ?"
        )
        .bind(&updated_at_str)
        .bind(id_bytes)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

// ============================================================================
// Medical History Repository
// ============================================================================

#[derive(Debug, FromRow)]
struct MedicalHistoryRow {
    id: Vec<u8>,
    patient_id: Vec<u8>,
    condition: String,
    diagnosis_date: Option<chrono::NaiveDate>,
    status: String,
    severity: Option<String>,
    notes: Option<Vec<u8>>,
    is_active: bool,
    created_at: String,
    updated_at: String,
    created_by: Vec<u8>,
    updated_by: Option<Vec<u8>>,
}

impl MedicalHistoryRow {
    fn into_medical_history(self, crypto: &EncryptionService) -> Result<MedicalHistory, RepositoryError> {
        let notes = match self.notes {
            Some(encrypted) => {
                let decrypted = crypto.decrypt(&encrypted).map_err(|e| {
                    RepositoryError::Decryption(format!("Failed to decrypt medical history notes: {}", e))
                })?;
                Some(decrypted)
            }
            None => None,
        };

        Ok(MedicalHistory {
            id: bytes_to_uuid(&self.id)?,
            patient_id: bytes_to_uuid(&self.patient_id)?,
            condition: self.condition,
            diagnosis_date: self.diagnosis_date,
            status: self.status.parse().unwrap_or(ConditionStatus::Active),
            severity: self.severity.and_then(|s| s.parse().ok()),
            notes,
            is_active: self.is_active,
            created_at: string_to_datetime(&self.created_at),
            updated_at: string_to_datetime(&self.updated_at),
            created_by: bytes_to_uuid(&self.created_by)?,
            updated_by: self.updated_by.as_ref().map(|b| bytes_to_uuid(b)).transpose()?,
        })
    }
}

pub struct SqlxMedicalHistoryRepository {
    pool: SqlitePool,
    crypto: Arc<EncryptionService>,
}

impl SqlxMedicalHistoryRepository {
    pub fn new(pool: SqlitePool, crypto: Arc<EncryptionService>) -> Self {
        Self { pool, crypto }
    }
}

#[async_trait]
impl MedicalHistoryRepository for SqlxMedicalHistoryRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<MedicalHistory>, RepositoryError> {
        let row = sqlx::query_as::<_, MedicalHistoryRow>(
            "SELECT id, patient_id, condition, diagnosis_date, status, severity, notes, is_active, created_at, updated_at, created_by, updated_by FROM medical_history WHERE id = ?"
        )
        .bind(uuid_to_bytes(&id))
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(r) => Ok(Some(r.into_medical_history(&self.crypto)?)),
            None => Ok(None),
        }
    }

    async fn find_by_patient(&self, patient_id: Uuid) -> Result<Vec<MedicalHistory>, RepositoryError> {
        let rows = sqlx::query_as::<_, MedicalHistoryRow>(
            "SELECT id, patient_id, condition, diagnosis_date, status, severity, notes, is_active, created_at, updated_at, created_by, updated_by FROM medical_history WHERE patient_id = ? ORDER BY created_at DESC"
        )
        .bind(uuid_to_bytes(&patient_id))
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|r| r.into_medical_history(&self.crypto))
            .collect()
    }

    async fn find_active_by_patient(&self, patient_id: Uuid) -> Result<Vec<MedicalHistory>, RepositoryError> {
        let rows = sqlx::query_as::<_, MedicalHistoryRow>(
            "SELECT id, patient_id, condition, diagnosis_date, status, severity, notes, is_active, created_at, updated_at, created_by, updated_by FROM medical_history WHERE patient_id = ? AND is_active = TRUE ORDER BY created_at DESC"
        )
        .bind(uuid_to_bytes(&patient_id))
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|r| r.into_medical_history(&self.crypto))
            .collect()
    }

    async fn create(&self, history: MedicalHistory) -> Result<MedicalHistory, RepositoryError> {
        let id_bytes = uuid_to_bytes(&history.id);
        let patient_bytes = uuid_to_bytes(&history.patient_id);
        let created_by_bytes = uuid_to_bytes(&history.created_by);
        let updated_by_bytes = history.updated_by.as_ref().map(uuid_to_bytes);
        let created_at_str = datetime_to_string(&history.created_at);
        let updated_at_str = datetime_to_string(&history.updated_at);

        let notes_encrypted: Option<Vec<u8>> = history.notes.as_ref()
            .map(|n| self.crypto.encrypt(n))
            .transpose()
            .map_err(|e| RepositoryError::Encryption(format!("Failed to encrypt medical history notes: {}", e)))?;

        sqlx::query(
            "INSERT INTO medical_history (id, patient_id, condition, diagnosis_date, status, severity, notes, is_active, created_at, updated_at, created_by, updated_by) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(id_bytes)
        .bind(patient_bytes)
        .bind(&history.condition)
        .bind(history.diagnosis_date)
        .bind(history.status.to_string())
        .bind(history.severity.as_ref().map(|s| s.to_string()))
        .bind(notes_encrypted)
        .bind(history.is_active)
        .bind(&created_at_str)
        .bind(&updated_at_str)
        .bind(created_by_bytes)
        .bind(updated_by_bytes)
        .execute(&self.pool)
        .await?;

        Ok(history)
    }

    async fn update(&self, history: MedicalHistory) -> Result<MedicalHistory, RepositoryError> {
        let id_bytes = uuid_to_bytes(&history.id);
        let updated_at_str = datetime_to_string(&history.updated_at);
        let updated_by_bytes = history.updated_by.as_ref().map(uuid_to_bytes);

        let notes_encrypted: Option<Vec<u8>> = history.notes.as_ref()
            .map(|n| self.crypto.encrypt(n))
            .transpose()
            .map_err(|e| RepositoryError::Encryption(format!("Failed to encrypt medical history notes: {}", e)))?;

        sqlx::query(
            "UPDATE medical_history SET condition = ?, diagnosis_date = ?, status = ?, severity = ?, notes = ?, is_active = ?, updated_at = ?, updated_by = ? WHERE id = ?"
        )
        .bind(&history.condition)
        .bind(history.diagnosis_date)
        .bind(history.status.to_string())
        .bind(history.severity.as_ref().map(|s| s.to_string()))
        .bind(notes_encrypted)
        .bind(history.is_active)
        .bind(&updated_at_str)
        .bind(updated_by_bytes)
        .bind(id_bytes)
        .execute(&self.pool)
        .await?;

        Ok(history)
    }
}

// ============================================================================
// Vital Signs Repository
// ============================================================================

#[derive(Debug, FromRow)]
struct VitalSignsRow {
    id: Vec<u8>,
    patient_id: Vec<u8>,
    consultation_id: Option<Vec<u8>>,
    measured_at: String,
    systolic_bp: Option<i64>,
    diastolic_bp: Option<i64>,
    heart_rate: Option<i64>,
    respiratory_rate: Option<i64>,
    temperature: Option<f32>,
    oxygen_saturation: Option<i64>,
    height_cm: Option<i64>,
    weight_kg: Option<f32>,
    bmi: Option<f32>,
    notes: Option<String>,
    created_at: String,
    created_by: Vec<u8>,
}

impl VitalSignsRow {
    fn into_vital_signs(self) -> Result<VitalSigns, RepositoryError> {
        Ok(VitalSigns {
            id: bytes_to_uuid(&self.id)?,
            patient_id: bytes_to_uuid(&self.patient_id)?,
            consultation_id: self.consultation_id.as_ref().map(|b| bytes_to_uuid(b)).transpose()?,
            measured_at: string_to_datetime(&self.measured_at),
            systolic_bp: self.systolic_bp.map(|v| v as u16),
            diastolic_bp: self.diastolic_bp.map(|v| v as u16),
            heart_rate: self.heart_rate.map(|v| v as u16),
            respiratory_rate: self.respiratory_rate.map(|v| v as u16),
            temperature: self.temperature,
            oxygen_saturation: self.oxygen_saturation.map(|v| v as u8),
            height_cm: self.height_cm.map(|v| v as u16),
            weight_kg: self.weight_kg,
            bmi: self.bmi,
            notes: self.notes,
            created_at: string_to_datetime(&self.created_at),
            created_by: bytes_to_uuid(&self.created_by)?,
        })
    }
}

pub struct SqlxVitalSignsRepository {
    pool: SqlitePool,
    #[allow(dead_code)]
    crypto: Arc<EncryptionService>,
}

impl SqlxVitalSignsRepository {
    pub fn new(pool: SqlitePool, crypto: Arc<EncryptionService>) -> Self {
        Self { pool, crypto }
    }
}

#[async_trait]
impl VitalSignsRepository for SqlxVitalSignsRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<VitalSigns>, RepositoryError> {
        let row = sqlx::query_as::<_, VitalSignsRow>(
            "SELECT id, patient_id, consultation_id, measured_at, systolic_bp, diastolic_bp, heart_rate, respiratory_rate, temperature, oxygen_saturation, height_cm, weight_kg, bmi, notes, created_at, created_by FROM vital_signs WHERE id = ?"
        )
        .bind(uuid_to_bytes(&id))
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(r) => Ok(Some(r.into_vital_signs()?)),
            None => Ok(None),
        }
    }

    async fn find_by_patient(&self, patient_id: Uuid, limit: usize) -> Result<Vec<VitalSigns>, RepositoryError> {
        let rows = sqlx::query_as::<_, VitalSignsRow>(
            "SELECT id, patient_id, consultation_id, measured_at, systolic_bp, diastolic_bp, heart_rate, respiratory_rate, temperature, oxygen_saturation, height_cm, weight_kg, bmi, notes, created_at, created_by FROM vital_signs WHERE patient_id = ? ORDER BY measured_at DESC LIMIT ?"
        )
        .bind(uuid_to_bytes(&patient_id))
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|r| r.into_vital_signs())
            .collect()
    }

    async fn find_latest_by_patient(&self, patient_id: Uuid) -> Result<Option<VitalSigns>, RepositoryError> {
        let row = sqlx::query_as::<_, VitalSignsRow>(
            "SELECT id, patient_id, consultation_id, measured_at, systolic_bp, diastolic_bp, heart_rate, respiratory_rate, temperature, oxygen_saturation, height_cm, weight_kg, bmi, notes, created_at, created_by FROM vital_signs WHERE patient_id = ? ORDER BY measured_at DESC LIMIT 1"
        )
        .bind(uuid_to_bytes(&patient_id))
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(r) => Ok(Some(r.into_vital_signs()?)),
            None => Ok(None),
        }
    }

    async fn create(&self, vitals: VitalSigns) -> Result<VitalSigns, RepositoryError> {
        let id_bytes = uuid_to_bytes(&vitals.id);
        let patient_bytes = uuid_to_bytes(&vitals.patient_id);
        let consultation_bytes = vitals.consultation_id.as_ref().map(uuid_to_bytes);
        let created_by_bytes = uuid_to_bytes(&vitals.created_by);
        let measured_at_str = datetime_to_string(&vitals.measured_at);
        let created_at_str = datetime_to_string(&vitals.created_at);

        sqlx::query(
            "INSERT INTO vital_signs (id, patient_id, consultation_id, measured_at, systolic_bp, diastolic_bp, heart_rate, respiratory_rate, temperature, oxygen_saturation, height_cm, weight_kg, bmi, notes, created_at, created_by) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(id_bytes)
        .bind(patient_bytes)
        .bind(consultation_bytes)
        .bind(&measured_at_str)
        .bind(vitals.systolic_bp.map(|v| v as i64))
        .bind(vitals.diastolic_bp.map(|v| v as i64))
        .bind(vitals.heart_rate.map(|v| v as i64))
        .bind(vitals.respiratory_rate.map(|v| v as i64))
        .bind(vitals.temperature)
        .bind(vitals.oxygen_saturation.map(|v| v as i64))
        .bind(vitals.height_cm.map(|v| v as i64))
        .bind(vitals.weight_kg)
        .bind(vitals.bmi)
        .bind(&vitals.notes)
        .bind(&created_at_str)
        .bind(created_by_bytes)
        .execute(&self.pool)
        .await?;

        Ok(vitals)
    }
}

// ============================================================================
// Family History Repository
// ============================================================================

#[derive(Debug, FromRow)]
struct FamilyHistoryRow {
    id: Vec<u8>,
    patient_id: Vec<u8>,
    relative_relationship: String,
    condition: String,
    age_at_diagnosis: Option<i64>,
    notes: Option<Vec<u8>>,
    created_at: String,
    created_by: Vec<u8>,
}

impl FamilyHistoryRow {
    fn into_family_history(self, crypto: &EncryptionService) -> Result<FamilyHistory, RepositoryError> {
        let notes = match self.notes {
            Some(encrypted) => {
                let decrypted = crypto.decrypt(&encrypted).map_err(|e| {
                    RepositoryError::Decryption(format!("Failed to decrypt family history notes: {}", e))
                })?;
                Some(decrypted)
            }
            None => None,
        };

        Ok(FamilyHistory {
            id: bytes_to_uuid(&self.id)?,
            patient_id: bytes_to_uuid(&self.patient_id)?,
            relative_relationship: self.relative_relationship,
            condition: self.condition,
            age_at_diagnosis: self.age_at_diagnosis.map(|v| v as u8),
            notes,
            created_at: string_to_datetime(&self.created_at),
            created_by: bytes_to_uuid(&self.created_by)?,
        })
    }
}

pub struct SqlxFamilyHistoryRepository {
    pool: SqlitePool,
    crypto: Arc<EncryptionService>,
}

impl SqlxFamilyHistoryRepository {
    pub fn new(pool: SqlitePool, crypto: Arc<EncryptionService>) -> Self {
        Self { pool, crypto }
    }
}

#[async_trait]
impl FamilyHistoryRepository for SqlxFamilyHistoryRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<FamilyHistory>, RepositoryError> {
        let row = sqlx::query_as::<_, FamilyHistoryRow>(
            "SELECT id, patient_id, relative_relationship, condition, age_at_diagnosis, notes, created_at, created_by FROM family_history WHERE id = ?"
        )
        .bind(uuid_to_bytes(&id))
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(r) => Ok(Some(r.into_family_history(&self.crypto)?)),
            None => Ok(None),
        }
    }

    async fn find_by_patient(&self, patient_id: Uuid) -> Result<Vec<FamilyHistory>, RepositoryError> {
        let rows = sqlx::query_as::<_, FamilyHistoryRow>(
            "SELECT id, patient_id, relative_relationship, condition, age_at_diagnosis, notes, created_at, created_by FROM family_history WHERE patient_id = ? ORDER BY created_at DESC"
        )
        .bind(uuid_to_bytes(&patient_id))
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|r| r.into_family_history(&self.crypto))
            .collect()
    }

    async fn create(&self, history: FamilyHistory) -> Result<FamilyHistory, RepositoryError> {
        let id_bytes = uuid_to_bytes(&history.id);
        let patient_bytes = uuid_to_bytes(&history.patient_id);
        let created_by_bytes = uuid_to_bytes(&history.created_by);
        let created_at_str = datetime_to_string(&history.created_at);

        let notes_encrypted: Option<Vec<u8>> = history.notes.as_ref()
            .map(|n| self.crypto.encrypt(n))
            .transpose()
            .map_err(|e| RepositoryError::Encryption(format!("Failed to encrypt family history notes: {}", e)))?;

        sqlx::query(
            "INSERT INTO family_history (id, patient_id, relative_relationship, condition, age_at_diagnosis, notes, created_at, created_by) VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(id_bytes)
        .bind(patient_bytes)
        .bind(&history.relative_relationship)
        .bind(&history.condition)
        .bind(history.age_at_diagnosis.map(|v| v as i64))
        .bind(notes_encrypted)
        .bind(&created_at_str)
        .bind(created_by_bytes)
        .execute(&self.pool)
        .await?;

        Ok(history)
    }

    async fn update(&self, history: FamilyHistory) -> Result<FamilyHistory, RepositoryError> {
        let id_bytes = uuid_to_bytes(&history.id);

        let notes_encrypted: Option<Vec<u8>> = history.notes.as_ref()
            .map(|n| self.crypto.encrypt(n))
            .transpose()
            .map_err(|e| RepositoryError::Encryption(format!("Failed to encrypt family history notes: {}", e)))?;

        sqlx::query(
            "UPDATE family_history SET relative_relationship = ?, condition = ?, age_at_diagnosis = ?, notes = ? WHERE id = ?"
        )
        .bind(&history.relative_relationship)
        .bind(&history.condition)
        .bind(history.age_at_diagnosis.map(|v| v as i64))
        .bind(notes_encrypted)
        .bind(id_bytes)
        .execute(&self.pool)
        .await?;

        Ok(history)
    }

    async fn delete(&self, id: Uuid) -> Result<(), RepositoryError> {
        let id_bytes = uuid_to_bytes(&id);

        sqlx::query("DELETE FROM family_history WHERE id = ?")
            .bind(id_bytes)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}

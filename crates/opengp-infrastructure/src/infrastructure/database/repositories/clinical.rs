use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::postgres::PgPool;
use sqlx::FromRow;
use uuid::Uuid;

use crate::infrastructure::crypto::EncryptionService;
use crate::infrastructure::database::sqlx_to_clinical_error;
use opengp_domain::domain::clinical::RepositoryError;
use opengp_domain::domain::clinical::{
    Allergy, AllergyRepository, AllergyType, ConditionStatus, Consultation, ConsultationRepository,
    FamilyHistory, FamilyHistoryRepository, MedicalHistory, MedicalHistoryRepository, Severity,
    SocialHistory, SocialHistoryRepository, VitalSigns, VitalSignsRepository,
};
use opengp_domain::domain::error::RepositoryError as BaseRepositoryError;

#[derive(Debug, FromRow)]
struct ConsultationRow {
    id: Uuid,
    patient_id: Uuid,
    practitioner_id: Uuid,
    appointment_id: Option<Uuid>,
    consultation_date: DateTime<Utc>,
    reason: Option<String>,
    clinical_notes: Option<Vec<u8>>,
    is_signed: bool,
    signed_at: Option<DateTime<Utc>>,
    signed_by: Option<Uuid>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    version: i32,
    created_by: Uuid,
    updated_by: Option<Uuid>,
}

impl ConsultationRow {
    fn into_consultation(
        self,
        crypto: &EncryptionService,
    ) -> Result<Consultation, RepositoryError> {
        let clinical_notes = match self.clinical_notes {
            Some(data) => {
                let decrypted = crypto.decrypt(&data).map_err(|e| {
                    RepositoryError::Decryption(format!("Failed to decrypt clinical notes: {}", e))
                })?;
                Some(decrypted)
            }
            None => None,
        };

        Ok(Consultation {
            id: self.id,
            patient_id: self.patient_id,
            practitioner_id: self.practitioner_id,
            appointment_id: self.appointment_id,
            consultation_date: self.consultation_date,
            reason: self.reason.clone(),
            clinical_notes,
            is_signed: self.is_signed,
            signed_at: self.signed_at,
            signed_by: self.signed_by,
            created_at: self.created_at,
            updated_at: self.updated_at,
            version: self.version,
            created_by: self.created_by,
            updated_by: self.updated_by,
        })
    }
}

/// SQLx-backed clinical consultation repository for PostgreSQL
///
/// Stores consultation records and encrypts free-text clinical
/// notes using the shared `EncryptionService`.
pub struct SqlxClinicalRepository {
    pool: PgPool,
    crypto: Arc<EncryptionService>,
}

impl SqlxClinicalRepository {
    /// Create a new clinical repository backed by a PostgreSQL pool
    /// and shared encryption service.
    pub fn new(pool: PgPool, crypto: Arc<EncryptionService>) -> Self {
        Self { pool, crypto }
    }
}

#[async_trait]
impl ConsultationRepository for SqlxClinicalRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Consultation>, RepositoryError> {
        let row = sqlx::query_as::<_, ConsultationRow>(
            r#"
        SELECT 
            id, patient_id, practitioner_id, appointment_id,
            consultation_date, reason, clinical_notes, is_signed, signed_at, signed_by,
            created_at, updated_at, version, created_by, updated_by
        FROM consultations
        WHERE id = $1
        "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(sqlx_to_clinical_error)?;

        match row {
            Some(r) => Ok(Some(r.into_consultation(&self.crypto)?)),
            None => Ok(None),
        }
    }

    async fn find_by_patient(
        &self,
        patient_id: Uuid,
        limit: Option<i64>,
    ) -> Result<Vec<Consultation>, RepositoryError> {
        let query_str = match limit {
            Some(l) => format!(
                r#"
        SELECT 
            id, patient_id, practitioner_id, appointment_id,
            consultation_date, reason, clinical_notes, is_signed, signed_at, signed_by,
            created_at, updated_at, version, created_by, updated_by
        FROM consultations
        WHERE patient_id = $1
        ORDER BY consultation_date DESC
        LIMIT {}
        "#,
                l
            ),
            None => r#"
        SELECT 
            id, patient_id, practitioner_id, appointment_id,
            consultation_date, reason, clinical_notes, is_signed, signed_at, signed_by,
            created_at, updated_at, version, created_by, updated_by
        FROM consultations
        WHERE patient_id = $1
        ORDER BY consultation_date DESC
        "#
            .to_string(),
        };

        let rows = sqlx::query_as::<_, ConsultationRow>(&query_str)
            .bind(patient_id)
            .fetch_all(&self.pool)
            .await
            .map_err(sqlx_to_clinical_error)?;

        rows.into_iter()
            .map(|r| r.into_consultation(&self.crypto))
            .collect()
    }

    async fn find_by_date_range(
        &self,
        patient_id: Uuid,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<Consultation>, RepositoryError> {
        let rows = sqlx::query_as::<_, ConsultationRow>(
            r#"
        SELECT 
            id, patient_id, practitioner_id, appointment_id,
            consultation_date, reason, clinical_notes, is_signed, signed_at, signed_by,
            created_at, updated_at, version, created_by, updated_by
        FROM consultations
        WHERE patient_id = $1 AND consultation_date BETWEEN $2 AND $3
        ORDER BY consultation_date DESC
        "#,
        )
        .bind(patient_id)
        .bind(start)
        .bind(end)
        .fetch_all(&self.pool)
        .await
        .map_err(sqlx_to_clinical_error)?;

        rows.into_iter()
            .map(|r| r.into_consultation(&self.crypto))
            .collect()
    }

    async fn create(&self, consultation: Consultation) -> Result<Consultation, RepositoryError> {
        let clinical_notes_encrypted: Option<Vec<u8>> = consultation
            .clinical_notes
            .as_ref()
            .map(|n| self.crypto.encrypt(n))
            .transpose()
            .map_err(|e| {
                RepositoryError::Encryption(format!("Failed to encrypt clinical notes: {}", e))
            })?;

        sqlx::query(
            r#"
        INSERT INTO consultations (
            id, patient_id, practitioner_id, appointment_id,
            consultation_date, reason, clinical_notes, is_signed, signed_at, signed_by,
            created_at, updated_at, version, created_by, updated_by
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
        "#,
        )
        .bind(consultation.id)
        .bind(consultation.patient_id)
        .bind(consultation.practitioner_id)
        .bind(consultation.appointment_id)
        .bind(consultation.consultation_date)
        .bind(&consultation.reason)
        .bind(clinical_notes_encrypted)
        .bind(consultation.is_signed)
        .bind(consultation.signed_at)
        .bind(consultation.signed_by)
        .bind(consultation.created_at)
        .bind(consultation.updated_at)
        .bind(consultation.version)
        .bind(consultation.created_by)
        .bind(consultation.updated_by)
        .execute(&self.pool)
        .await
        .map_err(sqlx_to_clinical_error)?;

        Ok(consultation)
    }

    async fn update(&self, consultation: Consultation) -> Result<Consultation, RepositoryError> {
        let current_version =
            sqlx::query_scalar::<_, i32>("SELECT version FROM consultations WHERE id = $1")
                .bind(consultation.id)
                .fetch_optional(&self.pool)
                .await
                .map_err(sqlx_to_clinical_error)?;

        let current_version = match current_version {
            Some(version) => version,
            None => return Err(RepositoryError::Base(BaseRepositoryError::NotFound)),
        };

        if current_version != consultation.version {
            return Err(RepositoryError::Base(BaseRepositoryError::Conflict(
                "Consultation was modified by another user".to_string(),
            )));
        }

        let new_version = consultation.version + 1;

        let clinical_notes_encrypted: Option<Vec<u8>> = consultation
            .clinical_notes
            .as_ref()
            .map(|n| self.crypto.encrypt(n))
            .transpose()
            .map_err(|e| {
                RepositoryError::Encryption(format!("Failed to encrypt clinical notes: {}", e))
            })?;

        let result = sqlx::query(
            r#"
        UPDATE consultations
        SET 
            reason = $1, clinical_notes = $2,
            is_signed = $3, signed_at = $4, signed_by = $5,
            updated_at = $6, updated_by = $7, version = $8
        WHERE id = $9 AND version = $10
        "#,
        )
        .bind(&consultation.reason)
        .bind(clinical_notes_encrypted)
        .bind(consultation.is_signed)
        .bind(consultation.signed_at)
        .bind(consultation.signed_by)
        .bind(consultation.updated_at)
        .bind(consultation.updated_by)
        .bind(new_version)
        .bind(consultation.id)
        .bind(consultation.version)
        .execute(&self.pool)
        .await
        .map_err(sqlx_to_clinical_error)?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::Base(BaseRepositoryError::Conflict(
                "Consultation was modified by another user".to_string(),
            )));
        }

        let mut updated_consultation = consultation;
        updated_consultation.version = new_version;

        Ok(updated_consultation)
    }

    async fn sign(&self, id: Uuid, user_id: Uuid) -> Result<(), RepositoryError> {
        let signed_at = Utc::now();

        let result = sqlx::query(
            r#"
        UPDATE consultations
        SET is_signed = TRUE, signed_at = $1, signed_by = $2, updated_at = $3
        WHERE id = $4
        "#,
        )
        .bind(signed_at)
        .bind(user_id)
        .bind(signed_at)
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(sqlx_to_clinical_error)?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::Base(BaseRepositoryError::NotFound));
        }

        Ok(())
    }
}

#[derive(Debug, FromRow)]
struct SocialHistoryRow {
    id: Uuid,
    patient_id: Uuid,
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
    updated_at: DateTime<Utc>,
    updated_by: Uuid,
}

impl SocialHistoryRow {
    fn into_social_history(
        self,
        crypto: &EncryptionService,
    ) -> Result<SocialHistory, RepositoryError> {
        use opengp_domain::domain::clinical::{AlcoholStatus, SmokingStatus};

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
            id: self.id,
            patient_id: self.patient_id,
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
            updated_at: self.updated_at,
            updated_by: self.updated_by,
        })
    }
}

/// SQLx-backed social history repository for PostgreSQL
///
/// Persists lifestyle and social history data, encrypting any
/// free-text notes at rest.
pub struct SqlxSocialHistoryRepository {
    pool: PgPool,
    crypto: Arc<EncryptionService>,
}

impl SqlxSocialHistoryRepository {
    /// Create a new social history repository backed by a PostgreSQL pool
    pub fn new(pool: PgPool, crypto: Arc<EncryptionService>) -> Self {
        Self { pool, crypto }
    }
}

#[async_trait]
impl SocialHistoryRepository for SqlxSocialHistoryRepository {
    async fn find_by_patient(
        &self,
        patient_id: Uuid,
    ) -> Result<Option<SocialHistory>, RepositoryError> {
        let row = sqlx::query_as::<_, SocialHistoryRow>(
            r#"
        SELECT 
            id, patient_id, smoking_status, cigarettes_per_day, 
            smoking_quit_date, alcohol_status, standard_drinks_per_week,
            exercise_frequency, occupation, living_situation, support_network,
            notes, updated_at, updated_by
        FROM social_history
        WHERE patient_id = $1
        "#,
        )
        .bind(patient_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(sqlx_to_clinical_error)?;

        match row {
            Some(r) => Ok(Some(r.into_social_history(&self.crypto)?)),
            None => Ok(None),
        }
    }

    async fn create(&self, history: SocialHistory) -> Result<SocialHistory, RepositoryError> {
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
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
        "#,
        )
        .bind(history.id)
        .bind(history.patient_id)
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
        .bind(history.updated_at)
        .bind(history.updated_by)
        .execute(&self.pool)
        .await
        .map_err(sqlx_to_clinical_error)?;

        Ok(history)
    }

    async fn update(&self, history: SocialHistory) -> Result<SocialHistory, RepositoryError> {
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
            smoking_status = $1, cigarettes_per_day = $2, 
            smoking_quit_date = $3, alcohol_status = $4, standard_drinks_per_week = $5,
            exercise_frequency = $6, occupation = $7, living_situation = $8, support_network = $9,
            notes = $10, updated_at = $11, updated_by = $12
        WHERE id = $13
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
        .bind(history.updated_at)
        .bind(history.updated_by)
        .bind(history.id)
        .execute(&self.pool)
        .await
        .map_err(sqlx_to_clinical_error)?;

        Ok(history)
    }
}

// ============================================================================
// Allergy Repository
// ============================================================================

#[derive(Debug, FromRow)]
struct AllergyRow {
    id: Uuid,
    patient_id: Uuid,
    allergen: String,
    allergy_type: String,
    severity: String,
    reaction: Option<Vec<u8>>,
    onset_date: Option<chrono::NaiveDate>,
    notes: Option<Vec<u8>>,
    is_active: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    created_by: Uuid,
    updated_by: Option<Uuid>,
}

impl AllergyRow {
    fn into_allergy(self, crypto: &EncryptionService) -> Result<Allergy, RepositoryError> {
        let reaction = match self.reaction {
            Some(encrypted) => {
                let decrypted = crypto.decrypt(&encrypted).map_err(|e| {
                    RepositoryError::Decryption(format!(
                        "Failed to decrypt allergy reaction: {}",
                        e
                    ))
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
            id: self.id,
            patient_id: self.patient_id,
            allergen: self.allergen,
            allergy_type: self.allergy_type.parse().unwrap_or(AllergyType::Other),
            severity: self.severity.parse().unwrap_or(Severity::Mild),
            reaction,
            onset_date: self.onset_date,
            notes,
            is_active: self.is_active,
            created_at: self.created_at,
            updated_at: self.updated_at,
            created_by: self.created_by,
            updated_by: self.updated_by,
        })
    }
}

/// SQLx-backed allergy repository for PostgreSQL
///
/// Stores allergy records and encrypts reaction and notes fields
/// for patients in the clinic database.
pub struct SqlxAllergyRepository {
    pool: PgPool,
    crypto: Arc<EncryptionService>,
}

impl SqlxAllergyRepository {
    /// Create a new allergy repository backed by a PostgreSQL pool
    pub fn new(pool: PgPool, crypto: Arc<EncryptionService>) -> Self {
        Self { pool, crypto }
    }
}

#[async_trait]
impl AllergyRepository for SqlxAllergyRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Allergy>, RepositoryError> {
        let row = sqlx::query_as::<_, AllergyRow>("SELECT id, patient_id, allergen, allergy_type, severity, reaction, onset_date, notes, is_active, created_at, updated_at, created_by, updated_by FROM allergies WHERE id = $1")
        .bind(id)
        .fetch_optional(&self.pool)
        .await
            .map_err(sqlx_to_clinical_error)?;

        match row {
            Some(r) => Ok(Some(r.into_allergy(&self.crypto)?)),
            None => Ok(None),
        }
    }

    async fn find_by_patient(
        &self,
        patient_id: Uuid,
        limit: Option<i64>,
    ) -> Result<Vec<Allergy>, RepositoryError> {
        let query_str = match limit {
            Some(l) => format!(
                "SELECT id, patient_id, allergen, allergy_type, severity, reaction, onset_date, notes, is_active, created_at, updated_at, created_by, updated_by FROM allergies WHERE patient_id = $1 ORDER BY created_at DESC LIMIT {}",
                l
            ),
            None => "SELECT id, patient_id, allergen, allergy_type, severity, reaction, onset_date, notes, is_active, created_at, updated_at, created_by, updated_by FROM allergies WHERE patient_id = $1 ORDER BY created_at DESC".to_string(),
        };

        let rows = sqlx::query_as::<_, AllergyRow>(&query_str)
            .bind(patient_id)
            .fetch_all(&self.pool)
            .await
            .map_err(sqlx_to_clinical_error)?;

        rows.into_iter()
            .map(|r| r.into_allergy(&self.crypto))
            .collect()
    }

    async fn find_active_by_patient(
        &self,
        patient_id: Uuid,
        limit: Option<i64>,
    ) -> Result<Vec<Allergy>, RepositoryError> {
        let query_str = match limit {
            Some(l) => format!(
                "SELECT id, patient_id, allergen, allergy_type, severity, reaction, onset_date, notes, is_active, created_at, updated_at, created_by, updated_by FROM allergies WHERE patient_id = $1 AND is_active = TRUE ORDER BY created_at DESC LIMIT {}",
                l
            ),
            None => "SELECT id, patient_id, allergen, allergy_type, severity, reaction, onset_date, notes, is_active, created_at, updated_at, created_by, updated_by FROM allergies WHERE patient_id = $1 AND is_active = TRUE ORDER BY created_at DESC".to_string(),
        };

        let rows = sqlx::query_as::<_, AllergyRow>(&query_str)
            .bind(patient_id)
            .fetch_all(&self.pool)
            .await
            .map_err(sqlx_to_clinical_error)?;

        rows.into_iter()
            .map(|r| r.into_allergy(&self.crypto))
            .collect()
    }

    async fn create(&self, allergy: Allergy) -> Result<Allergy, RepositoryError> {
        let reaction_encrypted: Option<Vec<u8>> = allergy
            .reaction
            .as_ref()
            .map(|r| self.crypto.encrypt(r))
            .transpose()
            .map_err(|e| {
                RepositoryError::Encryption(format!("Failed to encrypt allergy reaction: {}", e))
            })?;

        let notes_encrypted: Option<Vec<u8>> = allergy
            .notes
            .as_ref()
            .map(|n| self.crypto.encrypt(n))
            .transpose()
            .map_err(|e| {
                RepositoryError::Encryption(format!("Failed to encrypt allergy notes: {}", e))
            })?;

        sqlx::query("INSERT INTO allergies (id, patient_id, allergen, allergy_type, severity, reaction, onset_date, notes, is_active, created_at, updated_at, created_by, updated_by) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)")
        .bind(allergy.id)
        .bind(allergy.patient_id)
        .bind(&allergy.allergen)
        .bind(allergy.allergy_type.to_string())
        .bind(allergy.severity.to_string())
        .bind(reaction_encrypted)
        .bind(allergy.onset_date)
        .bind(notes_encrypted)
        .bind(allergy.is_active)
        .bind(allergy.created_at)
        .bind(allergy.updated_at)
        .bind(allergy.created_by)
        .bind(allergy.updated_by)
        .execute(&self.pool)
        .await
            .map_err(sqlx_to_clinical_error)?;

        Ok(allergy)
    }

    async fn update(&self, allergy: Allergy) -> Result<Allergy, RepositoryError> {
        let reaction_encrypted: Option<Vec<u8>> = allergy
            .reaction
            .as_ref()
            .map(|r| self.crypto.encrypt(r))
            .transpose()
            .map_err(|e| {
                RepositoryError::Encryption(format!("Failed to encrypt allergy reaction: {}", e))
            })?;

        let notes_encrypted: Option<Vec<u8>> = allergy
            .notes
            .as_ref()
            .map(|n| self.crypto.encrypt(n))
            .transpose()
            .map_err(|e| {
                RepositoryError::Encryption(format!("Failed to encrypt allergy notes: {}", e))
            })?;

        sqlx::query("UPDATE allergies SET allergen = $1, allergy_type = $2, severity = $3, reaction = $4, onset_date = $5, notes = $6, is_active = $7, updated_at = $8, updated_by = $9 WHERE id = $10")
        .bind(&allergy.allergen)
        .bind(allergy.allergy_type.to_string())
        .bind(allergy.severity.to_string())
        .bind(reaction_encrypted)
        .bind(allergy.onset_date)
        .bind(notes_encrypted)
        .bind(allergy.is_active)
        .bind(allergy.updated_at)
        .bind(allergy.updated_by)
        .bind(allergy.id)
        .execute(&self.pool)
        .await
            .map_err(sqlx_to_clinical_error)?;

        Ok(allergy)
    }

    async fn deactivate(&self, id: Uuid) -> Result<(), RepositoryError> {
        let updated_at = Utc::now();

        sqlx::query("UPDATE allergies SET is_active = FALSE, updated_at = $1 WHERE id = $2")
            .bind(updated_at)
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(sqlx_to_clinical_error)?;

        Ok(())
    }
}

// ============================================================================
// Medical History Repository
// ============================================================================

#[derive(Debug, FromRow)]
struct MedicalHistoryRow {
    id: Uuid,
    patient_id: Uuid,
    condition: String,
    diagnosis_date: Option<chrono::NaiveDate>,
    status: String,
    severity: Option<String>,
    notes: Option<Vec<u8>>,
    is_active: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    created_by: Uuid,
    updated_by: Option<Uuid>,
}

impl MedicalHistoryRow {
    fn into_medical_history(
        self,
        crypto: &EncryptionService,
    ) -> Result<MedicalHistory, RepositoryError> {
        let notes = match self.notes {
            Some(encrypted) => {
                let decrypted = crypto.decrypt(&encrypted).map_err(|e| {
                    RepositoryError::Decryption(format!(
                        "Failed to decrypt medical history notes: {}",
                        e
                    ))
                })?;
                Some(decrypted)
            }
            None => None,
        };

        Ok(MedicalHistory {
            id: self.id,
            patient_id: self.patient_id,
            condition: self.condition,
            diagnosis_date: self.diagnosis_date,
            status: self.status.parse().unwrap_or(ConditionStatus::Active),
            severity: self.severity.and_then(|s| s.parse().ok()),
            notes,
            is_active: self.is_active,
            created_at: self.created_at,
            updated_at: self.updated_at,
            created_by: self.created_by,
            updated_by: self.updated_by,
        })
    }
}

/// SQLx-backed medical history repository for PostgreSQL
///
/// Persists long term condition history and encrypts free-text
/// notes associated with each entry.
pub struct SqlxMedicalHistoryRepository {
    pool: PgPool,
    crypto: Arc<EncryptionService>,
}

impl SqlxMedicalHistoryRepository {
    /// Create a new medical history repository backed by a PostgreSQL pool
    pub fn new(pool: PgPool, crypto: Arc<EncryptionService>) -> Self {
        Self { pool, crypto }
    }
}

#[async_trait]
impl MedicalHistoryRepository for SqlxMedicalHistoryRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<MedicalHistory>, RepositoryError> {
        let row = sqlx::query_as::<_, MedicalHistoryRow>("SELECT id, patient_id, condition, diagnosis_date, status, severity, notes, is_active, created_at, updated_at, created_by, updated_by FROM medical_history WHERE id = $1")
        .bind(id)
        .fetch_optional(&self.pool)
        .await
            .map_err(sqlx_to_clinical_error)?;

        match row {
            Some(r) => Ok(Some(r.into_medical_history(&self.crypto)?)),
            None => Ok(None),
        }
    }

    async fn find_by_patient(
        &self,
        patient_id: Uuid,
        limit: Option<i64>,
    ) -> Result<Vec<MedicalHistory>, RepositoryError> {
        let query_str = match limit {
            Some(l) => format!(
                "SELECT id, patient_id, condition, diagnosis_date, status, severity, notes, is_active, created_at, updated_at, created_by, updated_by FROM medical_history WHERE patient_id = $1 ORDER BY created_at DESC LIMIT {}",
                l
            ),
            None => "SELECT id, patient_id, condition, diagnosis_date, status, severity, notes, is_active, created_at, updated_at, created_by, updated_by FROM medical_history WHERE patient_id = $1 ORDER BY created_at DESC".to_string(),
        };

        let rows = sqlx::query_as::<_, MedicalHistoryRow>(&query_str)
            .bind(patient_id)
            .fetch_all(&self.pool)
            .await
            .map_err(sqlx_to_clinical_error)?;

        rows.into_iter()
            .map(|r| r.into_medical_history(&self.crypto))
            .collect()
    }

    async fn find_active_by_patient(
        &self,
        patient_id: Uuid,
        limit: Option<i64>,
    ) -> Result<Vec<MedicalHistory>, RepositoryError> {
        let query_str = match limit {
            Some(l) => format!(
                "SELECT id, patient_id, condition, diagnosis_date, status, severity, notes, is_active, created_at, updated_at, created_by, updated_by FROM medical_history WHERE patient_id = $1 AND is_active = TRUE ORDER BY created_at DESC LIMIT {}",
                l
            ),
            None => "SELECT id, patient_id, condition, diagnosis_date, status, severity, notes, is_active, created_at, updated_at, created_by, updated_by FROM medical_history WHERE patient_id = $1 AND is_active = TRUE ORDER BY created_at DESC".to_string(),
        };

        let rows = sqlx::query_as::<_, MedicalHistoryRow>(&query_str)
            .bind(patient_id)
            .fetch_all(&self.pool)
            .await
            .map_err(sqlx_to_clinical_error)?;

        rows.into_iter()
            .map(|r| r.into_medical_history(&self.crypto))
            .collect()
    }

    async fn create(&self, history: MedicalHistory) -> Result<MedicalHistory, RepositoryError> {
        let notes_encrypted: Option<Vec<u8>> = history
            .notes
            .as_ref()
            .map(|n| self.crypto.encrypt(n))
            .transpose()
            .map_err(|e| {
                RepositoryError::Encryption(format!(
                    "Failed to encrypt medical history notes: {}",
                    e
                ))
            })?;

        sqlx::query("INSERT INTO medical_history (id, patient_id, condition, diagnosis_date, status, severity, notes, is_active, created_at, updated_at, created_by, updated_by) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)")
        .bind(history.id)
        .bind(history.patient_id)
        .bind(&history.condition)
        .bind(history.diagnosis_date)
        .bind(history.status.to_string())
        .bind(history.severity.as_ref().map(|s| s.to_string()))
        .bind(notes_encrypted)
        .bind(history.is_active)
        .bind(history.created_at)
        .bind(history.updated_at)
        .bind(history.created_by)
        .bind(history.updated_by)
        .execute(&self.pool)
        .await
            .map_err(sqlx_to_clinical_error)?;

        Ok(history)
    }

    async fn update(&self, history: MedicalHistory) -> Result<MedicalHistory, RepositoryError> {
        let notes_encrypted: Option<Vec<u8>> = history
            .notes
            .as_ref()
            .map(|n| self.crypto.encrypt(n))
            .transpose()
            .map_err(|e| {
                RepositoryError::Encryption(format!(
                    "Failed to encrypt medical history notes: {}",
                    e
                ))
            })?;

        sqlx::query("UPDATE medical_history SET condition = $1, diagnosis_date = $2, status = $3, severity = $4, notes = $5, is_active = $6, updated_at = $7, updated_by = $8 WHERE id = $9")
        .bind(&history.condition)
        .bind(history.diagnosis_date)
        .bind(history.status.to_string())
        .bind(history.severity.as_ref().map(|s| s.to_string()))
        .bind(notes_encrypted)
        .bind(history.is_active)
        .bind(history.updated_at)
        .bind(history.updated_by)
        .bind(history.id)
        .execute(&self.pool)
        .await
            .map_err(sqlx_to_clinical_error)?;

        Ok(history)
    }
}

// ============================================================================
// Vital Signs Repository
// ============================================================================

#[derive(Debug, FromRow)]
struct VitalSignsRow {
    id: Uuid,
    patient_id: Uuid,
    consultation_id: Option<Uuid>,
    measured_at: DateTime<Utc>,
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
    created_at: DateTime<Utc>,
    created_by: Uuid,
}

impl VitalSignsRow {
    fn into_vital_signs(self) -> Result<VitalSigns, RepositoryError> {
        Ok(VitalSigns {
            id: self.id,
            patient_id: self.patient_id,
            consultation_id: self.consultation_id,
            measured_at: self.measured_at,
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
            created_at: self.created_at,
            created_by: self.created_by,
        })
    }
}

/// SQLx-backed vital signs repository for PostgreSQL
///
/// Stores structured vital sign measurements for patients and
/// supports fetching recent readings for clinical workflows.
pub struct SqlxVitalSignsRepository {
    pool: PgPool,
    #[allow(dead_code)]
    crypto: Arc<EncryptionService>,
}

impl SqlxVitalSignsRepository {
    /// Create a new vital signs repository backed by a PostgreSQL pool
    pub fn new(pool: PgPool, crypto: Arc<EncryptionService>) -> Self {
        Self { pool, crypto }
    }
}

#[async_trait]
impl VitalSignsRepository for SqlxVitalSignsRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<VitalSigns>, RepositoryError> {
        let row = sqlx::query_as::<_, VitalSignsRow>("SELECT id, patient_id, consultation_id, measured_at, systolic_bp, diastolic_bp, heart_rate, respiratory_rate, temperature, oxygen_saturation, height_cm, weight_kg, bmi, notes, created_at, created_by FROM vital_signs WHERE id = $1")
        .bind(id)
        .fetch_optional(&self.pool)
        .await
            .map_err(sqlx_to_clinical_error)?;

        match row {
            Some(r) => Ok(Some(r.into_vital_signs()?)),
            None => Ok(None),
        }
    }

    async fn find_by_patient(
        &self,
        patient_id: Uuid,
        limit: usize,
    ) -> Result<Vec<VitalSigns>, RepositoryError> {
        let rows = sqlx::query_as::<_, VitalSignsRow>("SELECT id, patient_id, consultation_id, measured_at, systolic_bp, diastolic_bp, heart_rate, respiratory_rate, temperature, oxygen_saturation, height_cm, weight_kg, bmi, notes, created_at, created_by FROM vital_signs WHERE patient_id = $1 ORDER BY measured_at DESC LIMIT $2")
        .bind(patient_id)
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await
            .map_err(sqlx_to_clinical_error)?;

        rows.into_iter().map(|r| r.into_vital_signs()).collect()
    }

    async fn find_latest_by_patient(
        &self,
        patient_id: Uuid,
    ) -> Result<Option<VitalSigns>, RepositoryError> {
        let row = sqlx::query_as::<_, VitalSignsRow>("SELECT id, patient_id, consultation_id, measured_at, systolic_bp, diastolic_bp, heart_rate, respiratory_rate, temperature, oxygen_saturation, height_cm, weight_kg, bmi, notes, created_at, created_by FROM vital_signs WHERE patient_id = $1 ORDER BY measured_at DESC LIMIT 1")
        .bind(patient_id)
        .fetch_optional(&self.pool)
        .await
            .map_err(sqlx_to_clinical_error)?;

        match row {
            Some(r) => Ok(Some(r.into_vital_signs()?)),
            None => Ok(None),
        }
    }

    async fn create(&self, vitals: VitalSigns) -> Result<VitalSigns, RepositoryError> {
        sqlx::query("INSERT INTO vital_signs (id, patient_id, consultation_id, measured_at, systolic_bp, diastolic_bp, heart_rate, respiratory_rate, temperature, oxygen_saturation, height_cm, weight_kg, bmi, notes, created_at, created_by) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)")
        .bind(vitals.id)
        .bind(vitals.patient_id)
        .bind(vitals.consultation_id)
        .bind(vitals.measured_at)
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
        .bind(vitals.created_at)
        .bind(vitals.created_by)
        .execute(&self.pool)
        .await
            .map_err(sqlx_to_clinical_error)?;

        Ok(vitals)
    }
}

// ============================================================================
// Family History Repository
// ============================================================================

#[derive(Debug, FromRow)]
struct FamilyHistoryRow {
    id: Uuid,
    patient_id: Uuid,
    relative_relationship: String,
    condition: String,
    age_at_diagnosis: Option<i64>,
    notes: Option<Vec<u8>>,
    created_at: DateTime<Utc>,
    created_by: Uuid,
}

impl FamilyHistoryRow {
    fn into_family_history(
        self,
        crypto: &EncryptionService,
    ) -> Result<FamilyHistory, RepositoryError> {
        let notes = match self.notes {
            Some(encrypted) => {
                let decrypted = crypto.decrypt(&encrypted).map_err(|e| {
                    RepositoryError::Decryption(format!(
                        "Failed to decrypt family history notes: {}",
                        e
                    ))
                })?;
                Some(decrypted)
            }
            None => None,
        };

        Ok(FamilyHistory {
            id: self.id,
            patient_id: self.patient_id,
            relative_relationship: self.relative_relationship,
            condition: self.condition,
            age_at_diagnosis: self.age_at_diagnosis.map(|v| v as u8),
            notes,
            created_at: self.created_at,
            created_by: self.created_by,
        })
    }
}

/// SQLx-backed family history repository for PostgreSQL
///
/// Stores family history records and encrypts any free-text notes
/// related to hereditary risk.
pub struct SqlxFamilyHistoryRepository {
    pool: PgPool,
    crypto: Arc<EncryptionService>,
}

impl SqlxFamilyHistoryRepository {
    /// Create a new family history repository backed by a PostgreSQL pool
    pub fn new(pool: PgPool, crypto: Arc<EncryptionService>) -> Self {
        Self { pool, crypto }
    }
}

#[async_trait]
impl FamilyHistoryRepository for SqlxFamilyHistoryRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<FamilyHistory>, RepositoryError> {
        let row = sqlx::query_as::<_, FamilyHistoryRow>("SELECT id, patient_id, relative_relationship, condition, age_at_diagnosis, notes, created_at, created_by FROM family_history WHERE id = $1")
        .bind(id)
        .fetch_optional(&self.pool)
        .await
            .map_err(sqlx_to_clinical_error)?;

        match row {
            Some(r) => Ok(Some(r.into_family_history(&self.crypto)?)),
            None => Ok(None),
        }
    }

    async fn find_by_patient(
        &self,
        patient_id: Uuid,
        limit: Option<i64>,
    ) -> Result<Vec<FamilyHistory>, RepositoryError> {
        let query_str = match limit {
            Some(l) => format!(
                "SELECT id, patient_id, relative_relationship, condition, age_at_diagnosis, notes, created_at, created_by FROM family_history WHERE patient_id = $1 ORDER BY created_at DESC LIMIT {}",
                l
            ),
            None => "SELECT id, patient_id, relative_relationship, condition, age_at_diagnosis, notes, created_at, created_by FROM family_history WHERE patient_id = $1 ORDER BY created_at DESC".to_string(),
        };

        let rows = sqlx::query_as::<_, FamilyHistoryRow>(&query_str)
            .bind(patient_id)
            .fetch_all(&self.pool)
            .await
            .map_err(sqlx_to_clinical_error)?;

        rows.into_iter()
            .map(|r| r.into_family_history(&self.crypto))
            .collect()
    }

    async fn create(&self, history: FamilyHistory) -> Result<FamilyHistory, RepositoryError> {
        let notes_encrypted: Option<Vec<u8>> = history
            .notes
            .as_ref()
            .map(|n| self.crypto.encrypt(n))
            .transpose()
            .map_err(|e| {
                RepositoryError::Encryption(format!(
                    "Failed to encrypt family history notes: {}",
                    e
                ))
            })?;

        sqlx::query("INSERT INTO family_history (id, patient_id, relative_relationship, condition, age_at_diagnosis, notes, created_at, created_by) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)")
        .bind(history.id)
        .bind(history.patient_id)
        .bind(&history.relative_relationship)
        .bind(&history.condition)
        .bind(history.age_at_diagnosis.map(|v| v as i64))
        .bind(notes_encrypted)
        .bind(history.created_at)
        .bind(history.created_by)
        .execute(&self.pool)
        .await
            .map_err(sqlx_to_clinical_error)?;

        Ok(history)
    }

    async fn update(&self, history: FamilyHistory) -> Result<FamilyHistory, RepositoryError> {
        let notes_encrypted: Option<Vec<u8>> = history
            .notes
            .as_ref()
            .map(|n| self.crypto.encrypt(n))
            .transpose()
            .map_err(|e| {
                RepositoryError::Encryption(format!(
                    "Failed to encrypt family history notes: {}",
                    e
                ))
            })?;

        sqlx::query("UPDATE family_history SET relative_relationship = $1, condition = $2, age_at_diagnosis = $3, notes = $4 WHERE id = $5")
        .bind(&history.relative_relationship)
        .bind(&history.condition)
        .bind(history.age_at_diagnosis.map(|v| v as i64))
        .bind(notes_encrypted)
        .bind(history.id)
        .execute(&self.pool)
        .await
            .map_err(sqlx_to_clinical_error)?;

        Ok(history)
    }

    async fn delete(&self, id: Uuid) -> Result<(), RepositoryError> {
        sqlx::query("DELETE FROM family_history WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(sqlx_to_clinical_error)?;

        Ok(())
    }
}

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::error::RepositoryError;
use super::model::{
    Allergy, Consultation, FamilyHistory, MedicalHistory, SocialHistory, VitalSigns,
};

/// Repository trait for Consultation entities
#[async_trait]
pub trait ConsultationRepository: Send + Sync {
    /// Find a consultation by ID
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Consultation>, RepositoryError>;

    /// Find all consultations for a patient
    async fn find_by_patient(&self, patient_id: Uuid)
        -> Result<Vec<Consultation>, RepositoryError>;

    /// Find consultations within a date range for a specific patient
    async fn find_by_date_range(
        &self,
        patient_id: Uuid,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<Consultation>, RepositoryError>;

    /// Create a new consultation
    async fn create(&self, consultation: Consultation) -> Result<Consultation, RepositoryError>;

    /// Update an existing consultation
    async fn update(&self, consultation: Consultation) -> Result<Consultation, RepositoryError>;

    /// Sign a consultation (mark as finalized)
    async fn sign(&self, id: Uuid, user_id: Uuid) -> Result<(), RepositoryError>;
}

/// Repository trait for SocialHistory entities
#[async_trait]
pub trait SocialHistoryRepository: Send + Sync {
    /// Find social history for a patient
    async fn find_by_patient(
        &self,
        patient_id: Uuid,
    ) -> Result<Option<SocialHistory>, RepositoryError>;

    /// Create social history for a patient
    async fn create(&self, history: SocialHistory) -> Result<SocialHistory, RepositoryError>;

    /// Update social history
    async fn update(&self, history: SocialHistory) -> Result<SocialHistory, RepositoryError>;
}

/// Repository trait for Allergy entities
#[async_trait]
pub trait AllergyRepository: Send + Sync {
    /// Find an allergy by ID
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Allergy>, RepositoryError>;

    /// Find all allergies for a patient
    async fn find_by_patient(&self, patient_id: Uuid) -> Result<Vec<Allergy>, RepositoryError>;

    /// Find active allergies for a patient
    async fn find_active_by_patient(
        &self,
        patient_id: Uuid,
    ) -> Result<Vec<Allergy>, RepositoryError>;

    /// Create a new allergy
    async fn create(&self, allergy: Allergy) -> Result<Allergy, RepositoryError>;

    /// Update an existing allergy
    async fn update(&self, allergy: Allergy) -> Result<Allergy, RepositoryError>;

    /// Deactivate an allergy (soft delete)
    async fn deactivate(&self, id: Uuid) -> Result<(), RepositoryError>;
}

/// Repository trait for MedicalHistory entities
#[async_trait]
pub trait MedicalHistoryRepository: Send + Sync {
    /// Find a medical history entry by ID
    async fn find_by_id(&self, id: Uuid) -> Result<Option<MedicalHistory>, RepositoryError>;

    /// Find all medical history entries for a patient
    async fn find_by_patient(
        &self,
        patient_id: Uuid,
    ) -> Result<Vec<MedicalHistory>, RepositoryError>;

    /// Find active medical history entries for a patient
    async fn find_active_by_patient(
        &self,
        patient_id: Uuid,
    ) -> Result<Vec<MedicalHistory>, RepositoryError>;

    /// Create a new medical history entry
    async fn create(&self, history: MedicalHistory) -> Result<MedicalHistory, RepositoryError>;

    /// Update an existing medical history entry
    async fn update(&self, history: MedicalHistory) -> Result<MedicalHistory, RepositoryError>;
}

/// Repository trait for VitalSigns entities
#[async_trait]
pub trait VitalSignsRepository: Send + Sync {
    /// Find vital signs by ID
    async fn find_by_id(&self, id: Uuid) -> Result<Option<VitalSigns>, RepositoryError>;

    /// Find vital signs for a patient with limit
    async fn find_by_patient(
        &self,
        patient_id: Uuid,
        limit: usize,
    ) -> Result<Vec<VitalSigns>, RepositoryError>;

    /// Find the most recent vital signs for a patient
    async fn find_latest_by_patient(
        &self,
        patient_id: Uuid,
    ) -> Result<Option<VitalSigns>, RepositoryError>;

    /// Create new vital signs entry
    async fn create(&self, vitals: VitalSigns) -> Result<VitalSigns, RepositoryError>;
}

/// Repository trait for FamilyHistory entities
#[async_trait]
pub trait FamilyHistoryRepository: Send + Sync {
    /// Find a family history entry by ID
    async fn find_by_id(&self, id: Uuid) -> Result<Option<FamilyHistory>, RepositoryError>;

    /// Find all family history entries for a patient
    async fn find_by_patient(
        &self,
        patient_id: Uuid,
    ) -> Result<Vec<FamilyHistory>, RepositoryError>;

    /// Create a new family history entry
    async fn create(&self, history: FamilyHistory) -> Result<FamilyHistory, RepositoryError>;

    /// Update an existing family history entry
    async fn update(&self, history: FamilyHistory) -> Result<FamilyHistory, RepositoryError>;

    /// Delete a family history entry (hard delete allowed for family history)
    async fn delete(&self, id: Uuid) -> Result<(), RepositoryError>;
}

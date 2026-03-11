use std::sync::Arc;
use tracing::{info, instrument, warn};
use uuid::Uuid;

use crate::domain::audit::{AuditEmitter, AuditEntry};
use crate::domain::patient::{PatientService, ServiceError as PatientServiceError};
use crate::service;

use super::dto::{
    NewAllergyData, NewConsultationData, NewFamilyHistoryData, NewMedicalHistoryData,
    NewVitalSignsData, UpdateSocialHistoryData,
};
use super::error::ServiceError;
use super::model::{
    Allergy, Consultation, FamilyHistory, MedicalHistory, SocialHistory, VitalSigns,
};
use super::repository::{
    AllergyRepository, ConsultationRepository, FamilyHistoryRepository, MedicalHistoryRepository,
    SocialHistoryRepository, VitalSignsRepository,
};
use crate::domain::error::RepositoryError as BaseRepositoryError;

pub struct ClinicalRepositories {
    pub consultation: Arc<dyn ConsultationRepository>,
    pub allergy: Arc<dyn AllergyRepository>,
    pub medical_history: Arc<dyn MedicalHistoryRepository>,
    pub vital_signs: Arc<dyn VitalSignsRepository>,
    pub social_history: Arc<dyn SocialHistoryRepository>,
    pub family_history: Arc<dyn FamilyHistoryRepository>,
}

service! {
    ClinicalService {
        repos: ClinicalRepositories,
        patient_service: Arc<PatientService>,
        audit_logger: Arc<dyn AuditEmitter>,
    }
}

impl ClinicalService {
    // ==================== Consultation Management ====================

    #[instrument(skip(self), fields(patient_id = %data.patient_id))]
    pub async fn create_consultation(
        &self,
        data: NewConsultationData,
        user_id: Uuid,
    ) -> Result<Consultation, ServiceError> {
        info!("Creating new consultation for patient: {}", data.patient_id);

        // Verify patient exists
        self.patient_service
            .find_patient(data.patient_id)
            .await
            .map_err(|e| match e {
                PatientServiceError::NotFound(_) => ServiceError::PatientNotFound(data.patient_id),
                _ => ServiceError::Validation(format!("Patient lookup error: {}", e)),
            })?
            .ok_or_else(|| ServiceError::PatientNotFound(data.patient_id))?;

        let mut consultation = Consultation::new(
            data.patient_id,
            data.practitioner_id,
            data.appointment_id,
            user_id,
        );
        consultation.reason = data.reason;
        consultation.clinical_notes = data.clinical_notes;

        let saved = self.repos.consultation.create(consultation).await?;

        self.audit_logger
            .emit(AuditEntry::new_created(
                "consultation",
                saved.id,
                format!("{{\"patient_id\":\"{}\"}}", saved.patient_id),
                user_id,
            ))
            .await
            .ok();

        info!("Consultation created successfully: {}", saved.id);
        Ok(saved)
    }

    #[instrument(skip(self), fields(consultation_id = %id))]
    pub async fn find_consultation(&self, id: Uuid) -> Result<Option<Consultation>, ServiceError> {
        let consultation = self.repos.consultation.find_by_id(id).await?;
        Ok(consultation)
    }

    #[instrument(skip(self), fields(patient_id = %patient_id))]
    pub async fn list_patient_consultations(
        &self,
        patient_id: Uuid,
    ) -> Result<Vec<Consultation>, ServiceError> {
        info!("Listing consultations for patient: {}", patient_id);
        let consultations = self.repos.consultation.find_by_patient(patient_id).await?;
        Ok(consultations)
    }

    #[instrument(skip(self), fields(consultation_id = %consultation_id))]
    pub async fn update_clinical_notes(
        &self,
        consultation_id: Uuid,
        reason: Option<String>,
        clinical_notes: Option<String>,
        user_id: Uuid,
    ) -> Result<Consultation, ServiceError> {
        info!("Updating clinical notes for consultation: {}", consultation_id);

        let mut consultation = self.repos.consultation
            .find_by_id(consultation_id)
            .await?
            .ok_or_else(|| ServiceError::ConsultationNotFound(consultation_id))?;

        if consultation.is_signed {
            warn!("Attempted to edit signed consultation: {}", consultation_id);
            return Err(ServiceError::AlreadySigned);
        }

        if let Some(r) = reason {
            consultation.reason = Some(r);
        }
        if let Some(notes) = clinical_notes {
            consultation.clinical_notes = Some(notes);
        }

        consultation.updated_at = chrono::Utc::now();
        consultation.updated_by = Some(user_id);

        let updated = self
            .repos
            .consultation
            .update(consultation)
            .await
            .map_err(|err| match err {
                super::error::RepositoryError::Base(BaseRepositoryError::Conflict(message)) => {
                    ServiceError::Conflict(message)
                }
                other => ServiceError::Repository(other),
            })?;

        self.audit_logger
                    .emit(AuditEntry::new_updated(
                        "consultation",
                        updated.id,
                        "Clinical notes updated",
                        "",
                        user_id,
                    ))
            .await
            .ok();

        info!("Clinical notes updated for consultation: {}", updated.id);
        Ok(updated)
    }

    #[instrument(skip(self), fields(consultation_id = %consultation_id))]
    pub async fn sign_consultation(
        &self,
        consultation_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), ServiceError> {
        info!("Signing consultation: {}", consultation_id);

        let consultation = self.repos.consultation
            .find_by_id(consultation_id)
            .await?
            .ok_or_else(|| ServiceError::ConsultationNotFound(consultation_id))?;

        if consultation.is_signed {
            return Err(ServiceError::AlreadySigned);
        }

        self.repos.consultation
            .sign(consultation_id, user_id)
            .await?;

        // Audit log
        self.audit_logger
                    .emit(AuditEntry::new_status_changed(
                        "consultation",
                        consultation_id,
                        "Draft",
                        "Signed",
                        user_id,
                    ))
            .await
            .ok();

        info!("Consultation signed: {}", consultation_id);
        Ok(())
    }

    // ==================== Allergy Management ====================

    #[instrument(skip(self), fields(patient_id = %data.patient_id))]
    pub async fn add_allergy(
        &self,
        data: NewAllergyData,
        user_id: Uuid,
    ) -> Result<Allergy, ServiceError> {
        info!("Adding allergy for patient: {}", data.patient_id);

        // Verify patient exists
        self.patient_service
            .find_patient(data.patient_id)
            .await
            .map_err(|e| match e {
                PatientServiceError::NotFound(_) => ServiceError::PatientNotFound(data.patient_id),
                _ => ServiceError::Validation(format!("Patient lookup error: {}", e)),
            })?
            .ok_or_else(|| ServiceError::PatientNotFound(data.patient_id))?;

        let allergy = Allergy {
            id: Uuid::new_v4(),
            patient_id: data.patient_id,
            allergen: data.allergen,
            allergy_type: data.allergy_type,
            severity: data.severity,
            reaction: data.reaction,
            onset_date: data.onset_date,
            notes: data.notes,
            is_active: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            created_by: user_id,
            updated_by: None,
        };

        let saved = self.repos.allergy.create(allergy).await?;

        // Audit log
        self.audit_logger
                    .emit(AuditEntry::new_created(
                        "allergy",
                        saved.id,
                        format!(
                            "{{\"patient_id\":\"{}\",\"allergen\":\"{}\"}}",
                            saved.patient_id, saved.allergen
                        ),
                        user_id,
                    ))
            .await
            .ok();

        info!(
            "Allergy added: {} for patient {}",
            saved.id, saved.patient_id
        );
        Ok(saved)
    }

    #[instrument(skip(self), fields(patient_id = %patient_id))]
    pub async fn list_patient_allergies(
        &self,
        patient_id: Uuid,
        active_only: bool,
    ) -> Result<Vec<Allergy>, ServiceError> {
        info!("Listing allergies for patient: {}", patient_id);

        let allergies = if active_only {
            self.repos.allergy.find_active_by_patient(patient_id).await?
        } else {
            self.repos.allergy.find_by_patient(patient_id).await?
        };

        Ok(allergies)
    }

    #[instrument(skip(self), fields(allergy_id = %allergy_id))]
    pub async fn deactivate_allergy(
        &self,
        allergy_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), ServiceError> {
        info!("Deactivating allergy: {}", allergy_id);

        self.repos.allergy.deactivate(allergy_id).await?;

        // Audit log
        self.audit_logger
                    .emit(AuditEntry::new_status_changed(
                        "allergy", allergy_id, "Active", "Inactive", user_id,
                    ))
            .await
            .ok();

        info!("Allergy deactivated: {}", allergy_id);
        Ok(())
    }

    // ==================== Medical History ====================

    #[instrument(skip(self), fields(patient_id = %data.patient_id))]
    pub async fn add_medical_history(
        &self,
        data: NewMedicalHistoryData,
        user_id: Uuid,
    ) -> Result<MedicalHistory, ServiceError> {
        info!("Adding medical history for patient: {}", data.patient_id);

        // Verify patient exists
        self.patient_service
            .find_patient(data.patient_id)
            .await
            .map_err(|e| match e {
                PatientServiceError::NotFound(_) => ServiceError::PatientNotFound(data.patient_id),
                _ => ServiceError::Validation(format!("Patient lookup error: {}", e)),
            })?
            .ok_or_else(|| ServiceError::PatientNotFound(data.patient_id))?;

        let history = MedicalHistory {
            id: Uuid::new_v4(),
            patient_id: data.patient_id,
            condition: data.condition,
            diagnosis_date: data.diagnosis_date,
            status: data.status,
            severity: data.severity,
            notes: data.notes,
            is_active: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            created_by: user_id,
            updated_by: None,
        };

        let saved = self.repos.medical_history.create(history).await?;

        // Audit log
        self.audit_logger
                    .emit(AuditEntry::new_created(
                        "medical_history",
                        saved.id,
                        format!(
                            "{{\"patient_id\":\"{}\",\"condition\":\"{}\"}}",
                            saved.patient_id, saved.condition
                        ),
                        user_id,
                    ))
            .await
            .ok();

        info!(
            "Medical history added: {} for patient {}",
            saved.id, saved.patient_id
        );
        Ok(saved)
    }

    #[instrument(skip(self), fields(patient_id = %patient_id))]
    pub async fn list_medical_history(
        &self,
        patient_id: Uuid,
        active_only: bool,
    ) -> Result<Vec<MedicalHistory>, ServiceError> {
        info!("Listing medical history for patient: {}", patient_id);

        let history = if active_only {
            self.repos.medical_history
                .find_active_by_patient(patient_id)
                .await?
        } else {
            self.repos.medical_history
                .find_by_patient(patient_id)
                .await?
        };

        Ok(history)
    }

    #[instrument(skip(self), fields(history_id = %history_id))]
    pub async fn update_condition_status(
        &self,
        history_id: Uuid,
        status: super::model::ConditionStatus,
        user_id: Uuid,
    ) -> Result<MedicalHistory, ServiceError> {
        info!("Updating condition status for history: {}", history_id);

        let mut history = self.repos.medical_history
            .find_by_id(history_id)
            .await?
            .ok_or_else(|| ServiceError::MedicalHistoryNotFound(history_id))?;

        let old_status = format!("{:?}", history.status);
        history.status = status;
        history.updated_at = chrono::Utc::now();
        history.updated_by = Some(user_id);

        let updated = self.repos.medical_history.update(history).await?;

        // Audit log
        self.audit_logger
                    .emit(AuditEntry::new_status_changed(
                        "medical_history",
                        history_id,
                        &old_status,
                        format!("{:?}", updated.status),
                        user_id,
                    ))
            .await
            .ok();

        info!("Condition status updated: {}", history_id);
        Ok(updated)
    }

    // ==================== Vital Signs ====================

    #[instrument(skip(self), fields(patient_id = %data.patient_id))]
    pub async fn record_vital_signs(
        &self,
        data: NewVitalSignsData,
        user_id: Uuid,
    ) -> Result<VitalSigns, ServiceError> {
        info!("Recording vital signs for patient: {}", data.patient_id);

        // Verify patient exists
        self.patient_service
            .find_patient(data.patient_id)
            .await
            .map_err(|e| match e {
                PatientServiceError::NotFound(_) => ServiceError::PatientNotFound(data.patient_id),
                _ => ServiceError::Validation(format!("Patient lookup error: {}", e)),
            })?
            .ok_or_else(|| ServiceError::PatientNotFound(data.patient_id))?;

        let mut vitals = VitalSigns {
            id: Uuid::new_v4(),
            patient_id: data.patient_id,
            consultation_id: data.consultation_id,
            measured_at: chrono::Utc::now(),
            systolic_bp: data.systolic_bp,
            diastolic_bp: data.diastolic_bp,
            heart_rate: data.heart_rate,
            respiratory_rate: data.respiratory_rate,
            temperature: data.temperature,
            oxygen_saturation: data.oxygen_saturation,
            height_cm: data.height_cm,
            weight_kg: data.weight_kg,
            bmi: None,
            notes: data.notes,
            created_at: chrono::Utc::now(),
            created_by: user_id,
        };

        // Auto-calculate BMI
        vitals.calculate_bmi();

        let saved = self.repos.vital_signs.create(vitals).await?;

        // Audit log
        self.audit_logger
                    .emit(AuditEntry::new_created(
                        "vital_signs",
                        saved.id,
                        format!("{{\"patient_id\":\"{}\"}}", saved.patient_id),
                        user_id,
                    ))
            .await
            .ok();

        info!(
            "Vital signs recorded: {} for patient {}",
            saved.id, saved.patient_id
        );
        Ok(saved)
    }

    #[instrument(skip(self), fields(patient_id = %patient_id))]
    pub async fn get_latest_vital_signs(
        &self,
        patient_id: Uuid,
    ) -> Result<Option<VitalSigns>, ServiceError> {
        let vitals = self.repos.vital_signs
            .find_latest_by_patient(patient_id)
            .await?;
        Ok(vitals)
    }

    #[instrument(skip(self), fields(patient_id = %patient_id))]
    pub async fn list_vital_signs_history(
        &self,
        patient_id: Uuid,
        limit: usize,
    ) -> Result<Vec<VitalSigns>, ServiceError> {
        info!("Listing vital signs history for patient: {}", patient_id);
        let vitals = self.repos.vital_signs
            .find_by_patient(patient_id, limit)
            .await?;
        Ok(vitals)
    }

    // ==================== Social History ====================

    #[instrument(skip(self), fields(patient_id = %patient_id))]
    pub async fn update_social_history(
        &self,
        patient_id: Uuid,
        data: UpdateSocialHistoryData,
        user_id: Uuid,
    ) -> Result<SocialHistory, ServiceError> {
        info!("Updating social history for patient: {}", patient_id);

        // Verify patient exists
        self.patient_service
            .find_patient(patient_id)
            .await
            .map_err(|e| match e {
                PatientServiceError::NotFound(_) => ServiceError::PatientNotFound(patient_id),
                _ => ServiceError::Validation(format!("Patient lookup error: {}", e)),
            })?
            .ok_or_else(|| ServiceError::PatientNotFound(patient_id))?;

        // Check if social history exists
        let existing = self.repos.social_history.find_by_patient(patient_id).await?;

        let social_history = if let Some(mut existing) = existing {
            // Update existing
            existing.smoking_status = data.smoking_status;
            existing.cigarettes_per_day = data.cigarettes_per_day;
            existing.smoking_quit_date = data.smoking_quit_date;
            existing.alcohol_status = data.alcohol_status;
            existing.standard_drinks_per_week = data.standard_drinks_per_week;
            existing.exercise_frequency = data.exercise_frequency;
            existing.occupation = data.occupation;
            existing.living_situation = data.living_situation;
            existing.support_network = data.support_network;
            existing.notes = data.notes;
            existing.updated_at = chrono::Utc::now();
            existing.updated_by = user_id;

            self.repos.social_history.update(existing).await?
        } else {
            // Create new
            let new = SocialHistory {
                id: Uuid::new_v4(),
                patient_id,
                smoking_status: data.smoking_status,
                cigarettes_per_day: data.cigarettes_per_day,
                smoking_quit_date: data.smoking_quit_date,
                alcohol_status: data.alcohol_status,
                standard_drinks_per_week: data.standard_drinks_per_week,
                exercise_frequency: data.exercise_frequency,
                occupation: data.occupation,
                living_situation: data.living_situation,
                support_network: data.support_network,
                notes: data.notes,
                updated_at: chrono::Utc::now(),
                updated_by: user_id,
            };

            self.repos.social_history.create(new).await?
        };

        // Audit log
        self.audit_logger
                    .emit(AuditEntry::new_updated(
                        "social_history",
                        social_history.id,
                        "Social history updated",
                        "",
                        user_id,
                    ))
            .await
            .ok();

        info!("Social history updated for patient: {}", patient_id);
        Ok(social_history)
    }

    #[instrument(skip(self), fields(patient_id = %patient_id))]
    pub async fn get_social_history(
        &self,
        patient_id: Uuid,
    ) -> Result<Option<SocialHistory>, ServiceError> {
        let history = self.repos.social_history.find_by_patient(patient_id).await?;
        Ok(history)
    }

    // ==================== Family History ====================

    #[instrument(skip(self), fields(patient_id = %data.patient_id))]
    pub async fn add_family_history(
        &self,
        data: NewFamilyHistoryData,
        user_id: Uuid,
    ) -> Result<FamilyHistory, ServiceError> {
        info!("Adding family history for patient: {}", data.patient_id);

        // Verify patient exists
        self.patient_service
            .find_patient(data.patient_id)
            .await
            .map_err(|e| match e {
                PatientServiceError::NotFound(_) => ServiceError::PatientNotFound(data.patient_id),
                _ => ServiceError::Validation(format!("Patient lookup error: {}", e)),
            })?
            .ok_or_else(|| ServiceError::PatientNotFound(data.patient_id))?;

        let history = FamilyHistory {
            id: Uuid::new_v4(),
            patient_id: data.patient_id,
            relative_relationship: data.relative_relationship,
            condition: data.condition,
            age_at_diagnosis: data.age_at_diagnosis,
            notes: data.notes,
            created_at: chrono::Utc::now(),
            created_by: user_id,
        };

        let saved = self.repos.family_history.create(history).await?;

        // Audit log
        self.audit_logger
                    .emit(AuditEntry::new_created(
                        "family_history",
                        saved.id,
                        format!(
                            "{{\"patient_id\":\"{}\",\"condition\":\"{}\"}}",
                            saved.patient_id, saved.condition
                        ),
                        user_id,
                    ))
            .await
            .ok();

        info!(
            "Family history added: {} for patient {}",
            saved.id, saved.patient_id
        );
        Ok(saved)
    }

    #[instrument(skip(self), fields(patient_id = %patient_id))]
    pub async fn list_family_history(
        &self,
        patient_id: Uuid,
    ) -> Result<Vec<FamilyHistory>, ServiceError> {
        info!("Listing family history for patient: {}", patient_id);
        let history = self.repos.family_history.find_by_patient(patient_id).await?;
        Ok(history)
    }

    #[instrument(skip(self), fields(history_id = %history_id))]
    pub async fn delete_family_history(
        &self,
        history_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), ServiceError> {
        info!("Deleting family history: {}", history_id);

        self.repos.family_history.delete(history_id).await?;

        // Audit log
        self.audit_logger
                    .emit(AuditEntry::new_cancelled(
                        "family_history",
                        history_id,
                        "Family history entry deleted",
                        user_id,
                    ))
            .await
            .ok();

        info!("Family history deleted: {}", history_id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::audit::AuditEmitterError;
    use crate::domain::clinical::RepositoryError;
    use crate::domain::patient::{Address, Gender, Patient, PatientRepository};
    use async_trait::async_trait;
    use chrono::NaiveDate;
    use std::sync::Mutex;

    struct NoOpAuditEmitter;

    #[async_trait]
    impl AuditEmitter for NoOpAuditEmitter {
        async fn emit(&self, _entry: AuditEntry) -> Result<(), AuditEmitterError> {
            Ok(())
        }
    }

    struct MockConsultationRepository {
        consultations: Vec<Consultation>,
        update_calls: Mutex<usize>,
        sign_calls: Mutex<Vec<Uuid>>,
        update_error: Mutex<Option<RepositoryError>>,
    }

    #[async_trait]
    impl ConsultationRepository for MockConsultationRepository {
        async fn find_by_id(&self, id: Uuid) -> Result<Option<Consultation>, RepositoryError> {
            Ok(self.consultations.iter().find(|c| c.id == id).cloned())
        }

        async fn find_by_patient(
            &self,
            patient_id: Uuid,
        ) -> Result<Vec<Consultation>, RepositoryError> {
            Ok(self
                .consultations
                .iter()
                .filter(|c| c.patient_id == patient_id)
                .cloned()
                .collect())
        }

        async fn find_by_date_range(
            &self,
            _patient_id: Uuid,
            _start: chrono::DateTime<chrono::Utc>,
            _end: chrono::DateTime<chrono::Utc>,
        ) -> Result<Vec<Consultation>, RepositoryError> {
            Ok(vec![])
        }

        async fn create(
            &self,
            consultation: Consultation,
        ) -> Result<Consultation, RepositoryError> {
            Ok(consultation)
        }

        async fn update(
            &self,
            consultation: Consultation,
        ) -> Result<Consultation, RepositoryError> {
            if let Some(err) = self
                .update_error
                .lock()
                .expect("update_error lock poisoned")
                .take()
            {
                return Err(err);
            }
            *self.update_calls.lock().expect("update_calls lock poisoned") += 1;
            Ok(consultation)
        }

        async fn sign(&self, id: Uuid, _user_id: Uuid) -> Result<(), RepositoryError> {
            self.sign_calls
                .lock()
                .expect("sign_calls lock poisoned")
                .push(id);
            Ok(())
        }
    }

    struct MockAllergyRepository;
    #[async_trait]
    impl AllergyRepository for MockAllergyRepository {
        async fn find_by_id(&self, _id: Uuid) -> Result<Option<Allergy>, RepositoryError> {
            Ok(None)
        }

        async fn find_by_patient(&self, _patient_id: Uuid) -> Result<Vec<Allergy>, RepositoryError> {
            Ok(vec![])
        }

        async fn find_active_by_patient(
            &self,
            _patient_id: Uuid,
        ) -> Result<Vec<Allergy>, RepositoryError> {
            Ok(vec![])
        }

        async fn create(&self, allergy: Allergy) -> Result<Allergy, RepositoryError> {
            Ok(allergy)
        }

        async fn update(&self, allergy: Allergy) -> Result<Allergy, RepositoryError> {
            Ok(allergy)
        }

        async fn deactivate(&self, _id: Uuid) -> Result<(), RepositoryError> {
            Ok(())
        }
    }

    struct MockMedicalHistoryRepository;
    #[async_trait]
    impl MedicalHistoryRepository for MockMedicalHistoryRepository {
        async fn find_by_id(&self, _id: Uuid) -> Result<Option<MedicalHistory>, RepositoryError> {
            Ok(None)
        }

        async fn find_by_patient(
            &self,
            _patient_id: Uuid,
        ) -> Result<Vec<MedicalHistory>, RepositoryError> {
            Ok(vec![])
        }

        async fn find_active_by_patient(
            &self,
            _patient_id: Uuid,
        ) -> Result<Vec<MedicalHistory>, RepositoryError> {
            Ok(vec![])
        }

        async fn create(&self, history: MedicalHistory) -> Result<MedicalHistory, RepositoryError> {
            Ok(history)
        }

        async fn update(&self, history: MedicalHistory) -> Result<MedicalHistory, RepositoryError> {
            Ok(history)
        }
    }

    struct MockVitalSignsRepository;
    #[async_trait]
    impl VitalSignsRepository for MockVitalSignsRepository {
        async fn find_by_id(&self, _id: Uuid) -> Result<Option<VitalSigns>, RepositoryError> {
            Ok(None)
        }

        async fn find_by_patient(
            &self,
            _patient_id: Uuid,
            _limit: usize,
        ) -> Result<Vec<VitalSigns>, RepositoryError> {
            Ok(vec![])
        }

        async fn find_latest_by_patient(
            &self,
            _patient_id: Uuid,
        ) -> Result<Option<VitalSigns>, RepositoryError> {
            Ok(None)
        }

        async fn create(&self, vitals: VitalSigns) -> Result<VitalSigns, RepositoryError> {
            Ok(vitals)
        }
    }

    struct MockSocialHistoryRepository;
    #[async_trait]
    impl SocialHistoryRepository for MockSocialHistoryRepository {
        async fn find_by_patient(
            &self,
            _patient_id: Uuid,
        ) -> Result<Option<SocialHistory>, RepositoryError> {
            Ok(None)
        }

        async fn create(&self, history: SocialHistory) -> Result<SocialHistory, RepositoryError> {
            Ok(history)
        }

        async fn update(&self, history: SocialHistory) -> Result<SocialHistory, RepositoryError> {
            Ok(history)
        }
    }

    struct MockFamilyHistoryRepository;
    #[async_trait]
    impl FamilyHistoryRepository for MockFamilyHistoryRepository {
        async fn find_by_id(&self, _id: Uuid) -> Result<Option<FamilyHistory>, RepositoryError> {
            Ok(None)
        }

        async fn find_by_patient(
            &self,
            _patient_id: Uuid,
        ) -> Result<Vec<FamilyHistory>, RepositoryError> {
            Ok(vec![])
        }

        async fn create(&self, history: FamilyHistory) -> Result<FamilyHistory, RepositoryError> {
            Ok(history)
        }

        async fn update(&self, history: FamilyHistory) -> Result<FamilyHistory, RepositoryError> {
            Ok(history)
        }

        async fn delete(&self, _id: Uuid) -> Result<(), RepositoryError> {
            Ok(())
        }
    }

    struct MockPatientRepository {
        patients: Vec<Patient>,
    }

    #[async_trait]
    impl PatientRepository for MockPatientRepository {
        async fn find_by_id(&self, id: Uuid) -> Result<Option<Patient>, crate::domain::patient::RepositoryError> {
            Ok(self.patients.iter().find(|p| p.id == id).cloned())
        }

        async fn find_by_medicare(
            &self,
            medicare: &str,
        ) -> Result<Option<Patient>, crate::domain::patient::RepositoryError> {
            Ok(self
                .patients
                .iter()
                .find(|p| p.medicare_number.as_deref() == Some(medicare))
                .cloned())
        }

        async fn list_active(&self) -> Result<Vec<Patient>, crate::domain::patient::RepositoryError> {
            Ok(self.patients.clone())
        }

        async fn search(&self, query: &str) -> Result<Vec<Patient>, crate::domain::patient::RepositoryError> {
            Ok(self
                .patients
                .iter()
                .filter(|p| p.first_name.contains(query) || p.last_name.contains(query))
                .cloned()
                .collect())
        }

        async fn create(&self, patient: Patient) -> Result<Patient, crate::domain::patient::RepositoryError> {
            Ok(patient)
        }

        async fn update(&self, patient: Patient) -> Result<Patient, crate::domain::patient::RepositoryError> {
            Ok(patient)
        }

        async fn deactivate(&self, _id: Uuid) -> Result<(), crate::domain::patient::RepositoryError> {
            Ok(())
        }
    }

    fn create_test_patient() -> Patient {
        Patient::new(
            "Jane".to_string(),
            "Citizen".to_string(),
            NaiveDate::from_ymd_opt(1990, 1, 1).expect("valid dob"),
            Gender::Female,
            Some("8003601234567890".to_string()),
            Some("1234567890".to_string()),
            Some(1),
            None,
            None,
            None,
            None,
            Address::default(),
            None,
            None,
            None,
            None,
            None,
            None,
            Some("English".to_string()),
            Some(false),
            None,
        )
        .expect("valid patient")
    }

    fn create_test_service(consultations: Vec<Consultation>, patients: Vec<Patient>) -> ClinicalService {
        let repos = ClinicalRepositories {
            consultation: Arc::new(MockConsultationRepository {
                consultations,
                update_calls: Mutex::new(0),
                sign_calls: Mutex::new(vec![]),
                update_error: Mutex::new(None),
            }),
            allergy: Arc::new(MockAllergyRepository),
            medical_history: Arc::new(MockMedicalHistoryRepository),
            vital_signs: Arc::new(MockVitalSignsRepository),
            social_history: Arc::new(MockSocialHistoryRepository),
            family_history: Arc::new(MockFamilyHistoryRepository),
        };
        let patient_service = Arc::new(PatientService::new(Arc::new(MockPatientRepository { patients })));
        let audit_logger: Arc<dyn AuditEmitter> = Arc::new(NoOpAuditEmitter);

        ClinicalService::new(repos, patient_service, audit_logger)
    }

    #[test]
    fn test_service_error_display() {
        let id = Uuid::new_v4();
        let err = ServiceError::ConsultationNotFound(id);
        assert!(err.to_string().contains("Consultation not found"));
    }

    #[tokio::test]
    async fn test_sign_consultation_rejects_already_signed_consultation() {
        let patient = create_test_patient();
        let user_id = Uuid::new_v4();
        let mut consultation = Consultation::new(patient.id, Uuid::new_v4(), None, user_id);
        consultation.is_signed = true;
        consultation.signed_by = Some(user_id);
        consultation.signed_at = Some(chrono::Utc::now());

        let service = create_test_service(vec![consultation.clone()], vec![patient]);

        let result = service.sign_consultation(consultation.id, user_id).await;

        assert!(matches!(result, Err(ServiceError::AlreadySigned)));
    }

    #[tokio::test]
    async fn test_update_clinical_notes_rejects_signed_consultation() {
        let patient = create_test_patient();
        let user_id = Uuid::new_v4();
        let mut consultation = Consultation::new(patient.id, Uuid::new_v4(), None, user_id);
        consultation.is_signed = true;
        consultation.signed_by = Some(user_id);
        consultation.signed_at = Some(chrono::Utc::now());

        let service = create_test_service(vec![consultation.clone()], vec![patient]);

        let result = service
            .update_clinical_notes(
                consultation.id,
                Some("new reason".to_string()),
                Some("new notes".to_string()),
                user_id,
            )
            .await;

        assert!(matches!(result, Err(ServiceError::AlreadySigned)));
    }

    #[tokio::test]
    async fn test_update_clinical_notes_returns_conflict_for_concurrent_modification() {
        let patient = create_test_patient();
        let user_id = Uuid::new_v4();
        let consultation = Consultation::new(patient.id, Uuid::new_v4(), None, user_id);

        let repos = ClinicalRepositories {
            consultation: Arc::new(MockConsultationRepository {
                consultations: vec![consultation.clone()],
                update_calls: Mutex::new(0),
                sign_calls: Mutex::new(vec![]),
                update_error: Mutex::new(Some(RepositoryError::Base(
                    crate::domain::error::RepositoryError::Conflict(
                        "Consultation was modified by another user".to_string(),
                    ),
                ))),
            }),
            allergy: Arc::new(MockAllergyRepository),
            medical_history: Arc::new(MockMedicalHistoryRepository),
            vital_signs: Arc::new(MockVitalSignsRepository),
            social_history: Arc::new(MockSocialHistoryRepository),
            family_history: Arc::new(MockFamilyHistoryRepository),
        };
        let patient_service = Arc::new(PatientService::new(Arc::new(MockPatientRepository {
            patients: vec![patient],
        })));
        let audit_logger: Arc<dyn AuditEmitter> = Arc::new(NoOpAuditEmitter);
        let service = ClinicalService::new(repos, patient_service, audit_logger);

        let result = service
            .update_clinical_notes(
                consultation.id,
                Some("Updated reason".to_string()),
                Some("Updated notes".to_string()),
                user_id,
            )
            .await;

        assert!(matches!(result, Err(ServiceError::Conflict(_))));
    }
}

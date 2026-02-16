#![allow(clippy::too_many_arguments)]

use std::sync::Arc;
use tracing::{info, instrument, warn};
use uuid::Uuid;

use crate::domain::audit::{AuditEntry, AuditService};
use crate::domain::patient::{PatientService, ServiceError as PatientServiceError};
use crate::infrastructure::crypto::EncryptionService;
use crate::service;

use super::dto::{
    NewAllergyData, NewConsultationData, NewFamilyHistoryData, NewMedicalHistoryData,
    NewVitalSignsData, UpdateSOAPNotesData, UpdateSocialHistoryData,
};
use super::error::ServiceError;
use super::model::{
    Allergy, Consultation, FamilyHistory, MedicalHistory, SocialHistory, VitalSigns,
};
use super::repository::{
    AllergyRepository, ConsultationRepository, FamilyHistoryRepository, MedicalHistoryRepository,
    SocialHistoryRepository, VitalSignsRepository,
};

service! {
    ClinicalService {
        consultation_repo: Arc<dyn ConsultationRepository>,
        allergy_repo: Arc<dyn AllergyRepository>,
        medical_history_repo: Arc<dyn MedicalHistoryRepository>,
        vital_signs_repo: Arc<dyn VitalSignsRepository>,
        social_history_repo: Arc<dyn SocialHistoryRepository>,
        family_history_repo: Arc<dyn FamilyHistoryRepository>,
        patient_service: Arc<PatientService>,
        audit_logger: Arc<AuditService>,
        crypto: Arc<EncryptionService>,
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

        let consultation = Consultation::new(
            data.patient_id,
            data.practitioner_id,
            data.appointment_id,
            user_id,
        );

        let saved = self.consultation_repo.create(consultation).await?;

        // Audit log
        self.audit_logger
            .log(AuditEntry::new_created(
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
        let consultation = self.consultation_repo.find_by_id(id).await?;
        Ok(consultation)
    }

    #[instrument(skip(self), fields(patient_id = %patient_id))]
    pub async fn list_patient_consultations(
        &self,
        patient_id: Uuid,
    ) -> Result<Vec<Consultation>, ServiceError> {
        info!("Listing consultations for patient: {}", patient_id);
        let consultations = self.consultation_repo.find_by_patient(patient_id).await?;
        Ok(consultations)
    }

    #[instrument(skip(self, data), fields(consultation_id = %consultation_id))]
    pub async fn update_soap_notes(
        &self,
        consultation_id: Uuid,
        data: UpdateSOAPNotesData,
        user_id: Uuid,
    ) -> Result<Consultation, ServiceError> {
        info!("Updating SOAP notes for consultation: {}", consultation_id);

        let mut consultation = self
            .consultation_repo
            .find_by_id(consultation_id)
            .await?
            .ok_or_else(|| ServiceError::ConsultationNotFound(consultation_id))?;

        // Business rule: Cannot edit signed consultations
        if consultation.is_signed {
            warn!("Attempted to edit signed consultation: {}", consultation_id);
            return Err(ServiceError::AlreadySigned);
        }

        // Update SOAP notes
        if let Some(subjective) = data.subjective {
            consultation.soap_notes.subjective = Some(subjective);
        }
        if let Some(objective) = data.objective {
            consultation.soap_notes.objective = Some(objective);
        }
        if let Some(assessment) = data.assessment {
            consultation.soap_notes.assessment = Some(assessment);
        }
        if let Some(plan) = data.plan {
            consultation.soap_notes.plan = Some(plan);
        }

        consultation.updated_at = chrono::Utc::now();
        consultation.updated_by = Some(user_id);

        let updated = self.consultation_repo.update(consultation).await?;

        // Audit log
        self.audit_logger
            .log(AuditEntry::new_updated(
                "consultation",
                updated.id,
                "SOAP notes updated",
                "",
                user_id,
            ))
            .await
            .ok();

        info!("SOAP notes updated for consultation: {}", updated.id);
        Ok(updated)
    }

    #[instrument(skip(self), fields(consultation_id = %consultation_id))]
    pub async fn sign_consultation(
        &self,
        consultation_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), ServiceError> {
        info!("Signing consultation: {}", consultation_id);

        let consultation = self
            .consultation_repo
            .find_by_id(consultation_id)
            .await?
            .ok_or_else(|| ServiceError::ConsultationNotFound(consultation_id))?;

        if consultation.is_signed {
            return Err(ServiceError::AlreadySigned);
        }

        self.consultation_repo
            .sign(consultation_id, user_id)
            .await?;

        // Audit log
        self.audit_logger
            .log(AuditEntry::new_status_changed(
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

        let saved = self.allergy_repo.create(allergy).await?;

        // Audit log
        self.audit_logger
            .log(AuditEntry::new_created(
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
            self.allergy_repo.find_active_by_patient(patient_id).await?
        } else {
            self.allergy_repo.find_by_patient(patient_id).await?
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

        self.allergy_repo.deactivate(allergy_id).await?;

        // Audit log
        self.audit_logger
            .log(AuditEntry::new_status_changed(
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

        let saved = self.medical_history_repo.create(history).await?;

        // Audit log
        self.audit_logger
            .log(AuditEntry::new_created(
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
            self.medical_history_repo
                .find_active_by_patient(patient_id)
                .await?
        } else {
            self.medical_history_repo
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

        let mut history = self
            .medical_history_repo
            .find_by_id(history_id)
            .await?
            .ok_or_else(|| ServiceError::MedicalHistoryNotFound(history_id))?;

        let old_status = format!("{:?}", history.status);
        history.status = status;
        history.updated_at = chrono::Utc::now();
        history.updated_by = Some(user_id);

        let updated = self.medical_history_repo.update(history).await?;

        // Audit log
        self.audit_logger
            .log(AuditEntry::new_status_changed(
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

        let saved = self.vital_signs_repo.create(vitals).await?;

        // Audit log
        self.audit_logger
            .log(AuditEntry::new_created(
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
        let vitals = self
            .vital_signs_repo
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
        let vitals = self
            .vital_signs_repo
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
        let existing = self.social_history_repo.find_by_patient(patient_id).await?;

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

            self.social_history_repo.update(existing).await?
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

            self.social_history_repo.create(new).await?
        };

        // Audit log
        self.audit_logger
            .log(AuditEntry::new_updated(
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
        let history = self.social_history_repo.find_by_patient(patient_id).await?;
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

        let saved = self.family_history_repo.create(history).await?;

        // Audit log
        self.audit_logger
            .log(AuditEntry::new_created(
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
        let history = self.family_history_repo.find_by_patient(patient_id).await?;
        Ok(history)
    }

    #[instrument(skip(self), fields(history_id = %history_id))]
    pub async fn delete_family_history(
        &self,
        history_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), ServiceError> {
        info!("Deleting family history: {}", history_id);

        self.family_history_repo.delete(history_id).await?;

        // Audit log
        self.audit_logger
            .log(AuditEntry::new_cancelled(
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

    // Mock implementations would go here for unit tests
    // For now, tests are implemented in the mocks.rs file

    #[test]
    fn test_service_error_display() {
        let id = Uuid::new_v4();
        let err = ServiceError::ConsultationNotFound(id);
        assert!(err.to_string().contains("Consultation not found"));
    }
}

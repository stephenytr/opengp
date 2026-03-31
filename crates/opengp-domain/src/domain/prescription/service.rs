use chrono::Utc;
use std::sync::Arc;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::service;

use super::dto::NewPrescriptionData;
use super::error::ServiceError;
use super::model::{Medication, PBSStatus, Prescription};
use super::repository::PrescriptionRepository;
use crate::domain::audit::{AuditEntry, AuditService};

// Service layer for prescribing and PBS validation.
service! {
    PrescriptionService {
        repository: Arc<dyn PrescriptionRepository>,
        audit_service: Arc<AuditService>,
    }
}

impl PrescriptionService {
    /// Validate PBS status and authority requirements.
    ///
    /// # Arguments
    /// * `prescription` - The prescription to validate
    ///
    /// # Errors
    /// Returns [`ServiceError::PBSAuthorityRequired`] when authority
    /// details are missing for an authority item and
    /// [`ServiceError::Validation`] for other PBS data problems.
    fn validate_pbs_status(&self, prescription: &Prescription) -> Result<(), ServiceError> {
        info!("Validating PBS status for prescription {}", prescription.id);

        match prescription.pbs_status {
            PBSStatus::AuthorityRequired => {
                if prescription.authority_approval_number.is_none() {
                    warn!(
                        "PBS authority required but approval number not provided for prescription {}",
                        prescription.id
                    );
                    return Err(ServiceError::PBSAuthorityRequired(
                        "Authority approval number required for AuthorityRequired PBS status"
                            .to_string(),
                    ));
                }

                if prescription.authority_type.is_none() {
                    warn!(
                        "PBS authority required but authority type not specified for prescription {}",
                        prescription.id
                    );
                    return Err(ServiceError::PBSAuthorityRequired(
                        "Authority type must be specified".to_string(),
                    ));
                }
            }
            PBSStatus::RestrictedBenefit => {
                if prescription.indication.is_none() {
                    warn!(
                        "Restricted benefit requires indication for prescription {}",
                        prescription.id
                    );
                    return Err(ServiceError::Validation(
                        super::error::ValidationError::EmptyField("indication".to_string()),
                    ));
                }
            }
            _ => {}
        }

        Ok(())
    }

    /// Check for potential drug interactions.
    ///
    /// # Arguments
    /// * `patient_id` - Patient ID to check current medications
    /// * `medication` - New medication to check for interactions
    ///
    /// # Errors
    /// Returns [`ServiceError::Repository`] when fetching current
    /// prescriptions fails.
    pub async fn check_drug_interactions(
        &self,
        patient_id: Uuid,
        medication: &Medication,
    ) -> Result<Vec<String>, ServiceError> {
        info!(
            "Checking drug interactions for patient {} with medication {}",
            patient_id, medication.generic_name
        );

        // Get all active prescriptions for patient
        let active_prescriptions = self.repository.find_active_by_patient(patient_id).await?;

        // TODO: Implement actual drug interaction checking
        // This is a placeholder - real implementation would:
        // 1. Query drug interaction database (e.g., using AMT codes)
        // 2. Check for contraindications with current medications
        // 3. Check for duplicate therapeutic classes
        // 4. Return specific interaction warnings

        let warnings: Vec<String> = vec![];

        if active_prescriptions.len() > 10 {
            warn!(
                "Patient {} has {} active prescriptions - high polypharmacy risk",
                patient_id,
                active_prescriptions.len()
            );
        }

        info!(
            "Drug interaction check complete: {} warnings found",
            warnings.len()
        );

        Ok(warnings)
    }

    /// Log an audit entry for prescription operations.
    ///
    /// # Arguments
    /// * `entry` - The audit entry to log
    ///
    /// # Errors
    /// Returns [`ServiceError::Audit`] when logging fails.
    async fn audit_log(&self, entry: AuditEntry) -> Result<(), ServiceError> {
        self.audit_service
            .log(entry)
            .await
            .map_err(|e| ServiceError::Audit(format!("Failed to log audit entry: {}", e)))?;
        Ok(())
    }

    /// Create a new prescription with PBS validation.
    ///
    /// # Errors
    /// * [`ServiceError::PBSAuthorityRequired`] when authority details
    ///   are missing for an AuthorityRequired item.
    /// * [`ServiceError::Validation`] for other PBS data problems.
    /// * [`ServiceError::Repository`] if persistence fails.
    ///
    /// # Examples
    /// ```ignore
    /// let saved = prescription_service
    ///     .create_prescription(data, user_id)
    ///     .await?;
    /// # Ok::<(), opengp_domain::domain::prescription::ServiceError>(())
    /// ```
    pub async fn create_prescription(
        &self,
        data: NewPrescriptionData,
        user_id: Uuid,
    ) -> Result<Prescription, ServiceError> {
        info!(
            "Creating prescription for patient {} with medication {}",
            data.patient_id, data.medication.generic_name
        );

        // Create prescription domain model from data
        let prescription = Prescription {
            id: Uuid::new_v4(),
            patient_id: data.patient_id,
            practitioner_id: data.practitioner_id,
            consultation_id: data.consultation_id,
            medication: data.medication,
            dosage: data.dosage,
            quantity: data.quantity,
            repeats: data.repeats,
            authority_required: data.authority_required,
            authority_approval_number: data.authority_approval_number,
            authority_type: data.authority_type,
            pbs_status: data.pbs_status,
            pbs_item_code: data.pbs_item_code,
            indication: data.indication,
            directions: data.directions,
            notes: data.notes,
            prescription_type: data.prescription_type,
            prescription_date: data.prescription_date,
            expiry_date: data.expiry_date,
            is_active: true,
            cancelled_at: None,
            cancellation_reason: None,
            created_at: Utc::now(),
            created_by: user_id,
        };

        // Validate PBS status
        self.validate_pbs_status(&prescription)?;

        info!(
            "Saving prescription to database with ID: {}",
            prescription.id
        );

        // Save to repository
        let saved = match self.repository.create(prescription.clone()).await {
            Ok(saved) => {
                info!("Prescription saved successfully: {}", saved.id);
                saved
            }
            Err(e) => {
                error!("Failed to save prescription to database: {}", e);
                return Err(e.into());
            }
        };

        // Audit log
        let audit_entry = AuditEntry::new_created(
            "prescription",
            saved.id,
            format!(
                "{{\"medication\":\"{}\",\"patient_id\":\"{}\"}}",
                saved.medication.generic_name, saved.patient_id
            ),
            user_id,
        );
        self.audit_log(audit_entry).await?;

        Ok(saved)
    }

    /// Cancel a prescription using the domain model's cancellation
    /// logic.
    ///
    /// # Errors
    /// * [`ServiceError::NotFound`] if the prescription cannot be
    ///   located.
    /// * [`ServiceError::AlreadyCancelled`] if it is already inactive.
    /// * [`ServiceError::Repository`] if the update fails.
    pub async fn cancel_prescription(
        &self,
        id: Uuid,
        reason: String,
        user_id: Uuid,
    ) -> Result<(), ServiceError> {
        info!("Cancelling prescription: {} with reason: {}", id, reason);

        // Load existing prescription
        let mut prescription = self
            .repository
            .find_by_id(id)
            .await?
            .ok_or(ServiceError::NotFound(id))?;

        // Check if already cancelled
        if !prescription.is_active {
            warn!("Attempt to cancel already cancelled prescription: {}", id);
            return Err(ServiceError::AlreadyCancelled);
        }

        // Use domain method to cancel (enforces business rules)
        prescription.cancel(reason.clone(), user_id);

        // Save changes
        match self.repository.update(prescription.clone()).await {
            Ok(_) => {
                info!("Prescription cancelled successfully: {}", id);
            }
            Err(e) => {
                error!("Failed to cancel prescription in database: {}", e);
                return Err(e.into());
            }
        }

        // Audit log
        let audit_entry = AuditEntry::new_cancelled("prescription", id, reason, user_id);
        self.audit_log(audit_entry).await?;

        Ok(())
    }

    /// Find a prescription by identifier.
    ///
    /// # Errors
    /// Returns [`ServiceError::Repository`] if the repository lookup
    /// fails.
    pub async fn find_prescription(&self, id: Uuid) -> Result<Option<Prescription>, ServiceError> {
        let prescription = self.repository.find_by_id(id).await?;
        Ok(prescription)
    }

    /// List prescriptions for the given patient.
    ///
    /// # Errors
    /// Returns [`ServiceError::Repository`] if the repository query
    /// fails.
    pub async fn find_by_patient(
        &self,
        patient_id: Uuid,
    ) -> Result<Vec<Prescription>, ServiceError> {
        info!("Finding prescriptions for patient: {}", patient_id);

        let prescriptions = self.repository.find_by_patient(patient_id).await?;

        info!(
            "Found {} prescriptions for patient {}",
            prescriptions.len(),
            patient_id
        );

        Ok(prescriptions)
    }

    /// List active prescriptions for the given patient.
    ///
    /// # Errors
    /// Returns [`ServiceError::Repository`] if the repository query
    /// fails.
    pub async fn find_active_by_patient(
        &self,
        patient_id: Uuid,
    ) -> Result<Vec<Prescription>, ServiceError> {
        info!("Finding active prescriptions for patient: {}", patient_id);

        let prescriptions = self.repository.find_active_by_patient(patient_id).await?;

        info!(
            "Found {} active prescriptions for patient {}",
            prescriptions.len(),
            patient_id
        );

        Ok(prescriptions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::prescription::*;
    use async_trait::async_trait;
    use chrono::NaiveDate;

    use crate::domain::audit::{AuditRepository, AuditRepositoryError};
    // Mock repositories for testing
    struct MockPrescriptionRepository {
        prescriptions: Vec<Prescription>,
    }

    #[async_trait]
    impl PrescriptionRepository for MockPrescriptionRepository {
        async fn find_by_id(&self, id: Uuid) -> Result<Option<Prescription>, RepositoryError> {
            Ok(self.prescriptions.iter().find(|p| p.id == id).cloned())
        }

        async fn find_by_patient(
            &self,
            patient_id: Uuid,
        ) -> Result<Vec<Prescription>, RepositoryError> {
            Ok(self
                .prescriptions
                .iter()
                .filter(|p| p.patient_id == patient_id)
                .cloned()
                .collect())
        }

        async fn find_active_by_patient(
            &self,
            patient_id: Uuid,
        ) -> Result<Vec<Prescription>, RepositoryError> {
            Ok(self
                .prescriptions
                .iter()
                .filter(|p| p.patient_id == patient_id && p.is_active)
                .cloned()
                .collect())
        }

        async fn create(
            &self,
            prescription: Prescription,
        ) -> Result<Prescription, RepositoryError> {
            Ok(prescription)
        }

        async fn update(
            &self,
            prescription: Prescription,
        ) -> Result<Prescription, RepositoryError> {
            Ok(prescription)
        }

        async fn cancel(&self, _id: Uuid) -> Result<(), RepositoryError> {
            Ok(())
        }
    }

    struct MockAuditRepository;

    #[async_trait]
    impl AuditRepository for MockAuditRepository {
        async fn create(&self, entry: AuditEntry) -> Result<AuditEntry, AuditRepositoryError> {
            Ok(entry)
        }

        async fn find_by_entity(
            &self,
            _entity_type: &str,
            _entity_id: Uuid,
        ) -> Result<Vec<AuditEntry>, AuditRepositoryError> {
            Ok(vec![])
        }

        async fn find_by_user(
            &self,
            _user_id: Uuid,
        ) -> Result<Vec<AuditEntry>, AuditRepositoryError> {
            Ok(vec![])
        }

        async fn find_by_time_range(
            &self,
            _start_time: chrono::DateTime<Utc>,
            _end_time: chrono::DateTime<Utc>,
        ) -> Result<Vec<AuditEntry>, AuditRepositoryError> {
            Ok(vec![])
        }

        async fn find_by_entity_and_time_range(
            &self,
            _entity_type: &str,
            _entity_id: Uuid,
            _start_time: chrono::DateTime<Utc>,
            _end_time: chrono::DateTime<Utc>,
        ) -> Result<Vec<AuditEntry>, AuditRepositoryError> {
            Ok(vec![])
        }
    }

    fn create_test_medication() -> Medication {
        Medication {
            generic_name: "Amoxicillin".to_string(),
            brand_name: Some("Amoxil".to_string()),
            strength: "500mg".to_string(),
            form: MedicationForm::Capsule,
            amt_code: Some("12345".to_string()),
        }
    }

    fn create_test_prescription_data() -> NewPrescriptionData {
        NewPrescriptionData {
            patient_id: Uuid::new_v4(),
            practitioner_id: Uuid::new_v4(),
            consultation_id: Some(Uuid::new_v4()),
            medication: create_test_medication(),
            dosage: "500mg".to_string(),
            quantity: 20,
            repeats: 2,
            authority_required: false,
            authority_approval_number: None,
            authority_type: None,
            pbs_status: PBSStatus::GeneralSchedule,
            pbs_item_code: Some("1234".to_string()),
            indication: Some("Bacterial infection".to_string()),
            directions: "Take one capsule three times daily with food".to_string(),
            notes: None,
            prescription_type: PrescriptionType::Electronic,
            prescription_date: Utc::now(),
            expiry_date: Some(NaiveDate::from_ymd_opt(2027, 12, 31).expect("Valid date")),
        }
    }

    #[tokio::test]
    async fn test_create_prescription() {
        let repo = Arc::new(MockPrescriptionRepository {
            prescriptions: vec![],
        });
        let audit_repo = Arc::new(MockAuditRepository);
        let audit_service = Arc::new(AuditService::new(audit_repo));
        let service = PrescriptionService::new(repo, audit_service);

        let data = create_test_prescription_data();
        let user_id = Uuid::new_v4();

        let result = service.create_prescription(data, user_id).await;
        assert!(result.is_ok());

        let prescription = result.unwrap();
        assert_eq!(prescription.medication.generic_name, "Amoxicillin");
        assert_eq!(prescription.quantity, 20);
        assert_eq!(prescription.repeats, 2);
        assert!(prescription.is_active);
    }

    #[tokio::test]
    async fn test_pbs_authority_validation() {
        let repo = Arc::new(MockPrescriptionRepository {
            prescriptions: vec![],
        });
        let audit_repo = Arc::new(MockAuditRepository);
        let audit_service = Arc::new(AuditService::new(audit_repo));
        let service = PrescriptionService::new(repo, audit_service);

        let mut data = create_test_prescription_data();
        data.pbs_status = PBSStatus::AuthorityRequired;
        // Authority approval number not provided - should fail
        data.authority_approval_number = None;

        let user_id = Uuid::new_v4();

        let result = service.create_prescription(data, user_id).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ServiceError::PBSAuthorityRequired(_)
        ));
    }

    #[tokio::test]
    async fn test_pbs_authority_validation_with_approval() {
        let repo = Arc::new(MockPrescriptionRepository {
            prescriptions: vec![],
        });
        let audit_repo = Arc::new(MockAuditRepository);
        let audit_service = Arc::new(AuditService::new(audit_repo));
        let service = PrescriptionService::new(repo, audit_service);

        let mut data = create_test_prescription_data();
        data.pbs_status = PBSStatus::AuthorityRequired;
        data.authority_approval_number = Some("AUTH123456".to_string());
        data.authority_type = Some(AuthorityType::Streamlined);

        let user_id = Uuid::new_v4();

        let result = service.create_prescription(data, user_id).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_cancel_prescription() {
        let patient_id = Uuid::new_v4();
        let prescription_id = Uuid::new_v4();

        let prescription = Prescription {
            id: prescription_id,
            patient_id,
            practitioner_id: Uuid::new_v4(),
            consultation_id: None,
            medication: create_test_medication(),
            dosage: "500mg".to_string(),
            quantity: 20,
            repeats: 2,
            authority_required: false,
            authority_approval_number: None,
            authority_type: None,
            pbs_status: PBSStatus::GeneralSchedule,
            pbs_item_code: None,
            indication: None,
            directions: "Take one capsule three times daily".to_string(),
            notes: None,
            prescription_type: PrescriptionType::Electronic,
            prescription_date: Utc::now(),
            expiry_date: None,
            is_active: true,
            cancelled_at: None,
            cancellation_reason: None,
            created_at: Utc::now(),
            created_by: Uuid::new_v4(),
        };

        let repo = Arc::new(MockPrescriptionRepository {
            prescriptions: vec![prescription],
        });
        let audit_repo = Arc::new(MockAuditRepository);
        let audit_service = Arc::new(AuditService::new(audit_repo));
        let service = PrescriptionService::new(repo, audit_service);

        let user_id = Uuid::new_v4();
        let result = service
            .cancel_prescription(
                prescription_id,
                "Patient discontinued medication".to_string(),
                user_id,
            )
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_cancel_already_cancelled_prescription() {
        let patient_id = Uuid::new_v4();
        let prescription_id = Uuid::new_v4();

        let mut prescription = Prescription {
            id: prescription_id,
            patient_id,
            practitioner_id: Uuid::new_v4(),
            consultation_id: None,
            medication: create_test_medication(),
            dosage: "500mg".to_string(),
            quantity: 20,
            repeats: 2,
            authority_required: false,
            authority_approval_number: None,
            authority_type: None,
            pbs_status: PBSStatus::GeneralSchedule,
            pbs_item_code: None,
            indication: None,
            directions: "Take one capsule three times daily".to_string(),
            notes: None,
            prescription_type: PrescriptionType::Electronic,
            prescription_date: Utc::now(),
            expiry_date: None,
            is_active: false, // Already cancelled
            cancelled_at: Some(Utc::now()),
            cancellation_reason: Some("Previous reason".to_string()),
            created_at: Utc::now(),
            created_by: Uuid::new_v4(),
        };
        prescription.is_active = false;

        let repo = Arc::new(MockPrescriptionRepository {
            prescriptions: vec![prescription],
        });
        let audit_repo = Arc::new(MockAuditRepository);
        let audit_service = Arc::new(AuditService::new(audit_repo));
        let service = PrescriptionService::new(repo, audit_service);

        let user_id = Uuid::new_v4();
        let result = service
            .cancel_prescription(prescription_id, "New reason".to_string(), user_id)
            .await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ServiceError::AlreadyCancelled
        ));
    }
}

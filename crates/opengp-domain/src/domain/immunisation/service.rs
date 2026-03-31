use std::sync::Arc;
use uuid::Uuid;

use crate::service;

use super::error::{ServiceError, ValidationError};
use super::model::{Immunisation, VaccinationSchedule};
use super::repository::ImmunisationRepository;

service! {
    /// Service layer for recording immunisations and managing
    /// vaccination schedules.
    ImmunisationService {
        repository: Arc<dyn ImmunisationRepository>,
    }
}

impl ImmunisationService {
    fn validate_immunisation(&self, immunisation: &Immunisation) -> Result<(), ServiceError> {
        if immunisation.dose_number == 0 {
            return Err(ValidationError::InvalidDoseNumber.into());
        }

        if immunisation.batch_number.trim().is_empty() {
            return Err(ValidationError::EmptyBatchNumber.into());
        }

        Ok(())
    }

    /// Record a new immunisation for a patient.
    ///
    /// # Errors
    /// * [`ServiceError::Validation`] if the dose number or batch
    ///   number are invalid.
    pub async fn record_immunisation(
        &self,
        immunisation: Immunisation,
    ) -> Result<Immunisation, ServiceError> {
        self.validate_immunisation(&immunisation)?;
        let saved = self.repository.create(immunisation).await?;
        Ok(saved)
    }

    /// List immunisations recorded for a patient.
    ///
    /// # Errors
    /// Returns [`ServiceError::Repository`] if the repository query
    /// fails.
    pub async fn find_by_patient(
        &self,
        patient_id: Uuid,
    ) -> Result<Vec<Immunisation>, ServiceError> {
        Ok(self.repository.find_by_patient(patient_id).await?)
    }

    /// Return due vaccination schedule entries for a patient.
    ///
    /// # Errors
    /// Returns [`ServiceError::Repository`] if the repository query
    /// fails.
    pub async fn due_schedule(
        &self,
        patient_id: Uuid,
    ) -> Result<Vec<VaccinationSchedule>, ServiceError> {
        Ok(self.repository.find_due_schedules(patient_id).await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::immunisation::{
        AdministrationRoute, AnatomicalSite, ConsentType, RepositoryError, ScheduleStatus, Vaccine,
        VaccineType,
    };
    use async_trait::async_trait;
    use chrono::{NaiveDate, Utc};

    struct MockImmunisationRepository {
        items: Vec<Immunisation>,
        schedules: Vec<VaccinationSchedule>,
    }

    #[async_trait]
    impl ImmunisationRepository for MockImmunisationRepository {
        async fn find_by_id(&self, id: Uuid) -> Result<Option<Immunisation>, RepositoryError> {
            Ok(self.items.iter().find(|item| item.id == id).cloned())
        }

        async fn find_by_patient(
            &self,
            patient_id: Uuid,
        ) -> Result<Vec<Immunisation>, RepositoryError> {
            Ok(self
                .items
                .iter()
                .filter(|item| item.patient_id == patient_id)
                .cloned()
                .collect())
        }

        async fn create(
            &self,
            immunisation: Immunisation,
        ) -> Result<Immunisation, RepositoryError> {
            Ok(immunisation)
        }

        async fn update(
            &self,
            immunisation: Immunisation,
        ) -> Result<Immunisation, RepositoryError> {
            Ok(immunisation)
        }

        async fn find_due_schedules(
            &self,
            patient_id: Uuid,
        ) -> Result<Vec<VaccinationSchedule>, RepositoryError> {
            Ok(self
                .schedules
                .iter()
                .filter(|schedule| schedule.patient_id == patient_id)
                .cloned()
                .collect())
        }
    }

    fn new_service(
        items: Vec<Immunisation>,
        schedules: Vec<VaccinationSchedule>,
    ) -> ImmunisationService {
        ImmunisationService::new(Arc::new(MockImmunisationRepository { items, schedules }))
    }

    fn test_immunisation(patient_id: Uuid) -> Immunisation {
        Immunisation {
            id: Uuid::new_v4(),
            patient_id,
            practitioner_id: Uuid::new_v4(),
            consultation_id: None,
            vaccine: Vaccine {
                name: "Influenza Quadrivalent".to_string(),
                vaccine_type: VaccineType::Influenza,
                brand_name: Some("Fluad Quad".to_string()),
                snomed_code: None,
                amt_code: None,
            },
            vaccination_date: NaiveDate::from_ymd_opt(2026, 3, 1).expect("valid date"),
            dose_number: 1,
            total_doses: Some(1),
            batch_number: "BATCH-123".to_string(),
            expiry_date: None,
            manufacturer: Some("Seqirus".to_string()),
            route: AdministrationRoute::Intramuscular,
            site: AnatomicalSite::LeftDeltoid,
            dose_quantity: Some(0.5),
            dose_unit: Some("mL".to_string()),
            consent_obtained: true,
            consent_type: Some(ConsentType::Verbal),
            air_reported: false,
            air_report_date: None,
            air_transaction_id: None,
            adverse_event: false,
            adverse_event_details: None,
            notes: None,
            created_at: Utc::now(),
            created_by: Uuid::new_v4(),
        }
    }

    #[tokio::test]
    async fn test_record_immunisation_rejects_empty_batch_number() {
        let service = new_service(vec![], vec![]);
        let mut immunisation = test_immunisation(Uuid::new_v4());
        immunisation.batch_number = "   ".to_string();

        let result = service.record_immunisation(immunisation).await;

        assert!(matches!(
            result,
            Err(ServiceError::Validation(ValidationError::EmptyBatchNumber))
        ));
    }

    #[tokio::test]
    async fn test_find_by_patient_returns_matching_records() {
        let target_patient = Uuid::new_v4();
        let other_patient = Uuid::new_v4();

        let service = new_service(
            vec![
                test_immunisation(target_patient),
                test_immunisation(other_patient),
            ],
            vec![],
        );

        let result = service.find_by_patient(target_patient).await;

        assert!(result.is_ok());
        let items = result.expect("result should be ok");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].patient_id, target_patient);
    }

    #[tokio::test]
    async fn test_due_schedule_filters_by_patient() {
        let target_patient = Uuid::new_v4();

        let schedules = vec![
            VaccinationSchedule {
                patient_id: target_patient,
                vaccine_type: VaccineType::Influenza,
                dose_number: 2,
                due_date: NaiveDate::from_ymd_opt(2026, 6, 1).expect("valid date"),
                status: ScheduleStatus::Due,
                completed_immunisation_id: None,
            },
            VaccinationSchedule {
                patient_id: Uuid::new_v4(),
                vaccine_type: VaccineType::COVID19,
                dose_number: 3,
                due_date: NaiveDate::from_ymd_opt(2026, 7, 1).expect("valid date"),
                status: ScheduleStatus::Overdue,
                completed_immunisation_id: None,
            },
        ];

        let service = new_service(vec![], schedules);
        let result = service.due_schedule(target_patient).await;

        assert!(result.is_ok());
        let due = result.expect("result should be ok");
        assert_eq!(due.len(), 1);
        assert_eq!(due[0].patient_id, target_patient);
    }
}

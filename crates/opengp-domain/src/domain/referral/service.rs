use std::sync::Arc;
use uuid::Uuid;

use crate::service;

use super::error::{ServiceError, ValidationError};
use super::model::{Referral, ReferralStatus};
use super::repository::ReferralRepository;

service! {
    /// Service layer for managing outgoing referrals.
    ///
    /// Validates referral details and coordinates persistence via the
    /// [`ReferralRepository`].
    ReferralService {
        repository: Arc<dyn ReferralRepository>,
    }
}

impl ReferralService {
    fn validate_referral(&self, referral: &Referral) -> Result<(), ServiceError> {
        if referral.specialty.trim().is_empty() {
            return Err(ValidationError::EmptySpecialty.into());
        }

        if referral.reason.trim().is_empty() {
            return Err(ValidationError::EmptyReason.into());
        }

        Ok(())
    }

    /// Create a new referral after validating mandatory fields.
    ///
    /// # Errors
    /// * [`ServiceError::Validation`] if specialty or reason are empty.
    ///
    /// # Examples
    /// ```ignore
    /// let saved = referral_service.create_referral(referral).await?;
    /// # Ok::<(), opengp_domain::domain::referral::ServiceError>(())
    /// ```
    pub async fn create_referral(&self, referral: Referral) -> Result<Referral, ServiceError> {
        self.validate_referral(&referral)?;
        Ok(self.repository.create(referral).await?)
    }

    /// Mark a referral as sent by a particular user.
    pub async fn mark_sent(&self, id: Uuid, user_id: Uuid) -> Result<Referral, ServiceError> {
        let mut referral = self
            .repository
            .find_by_id(id)
            .await?
            .ok_or(ServiceError::NotFound(id))?;

        if referral.status == ReferralStatus::Sent {
            return Err(ServiceError::AlreadySent);
        }

        referral.status = ReferralStatus::Sent;
        referral.updated_by = Some(user_id);
        referral.updated_at = chrono::Utc::now();

        Ok(self.repository.update(referral).await?)
    }

    /// Find referrals filtered by their current status.
    pub async fn find_by_status(
        &self,
        status: ReferralStatus,
    ) -> Result<Vec<Referral>, ServiceError> {
        Ok(self.repository.find_by_status(status).await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::referral::{
        ReferralDeliveryMethod, ReferralType, ReferralUrgency, RepositoryError,
    };
    use async_trait::async_trait;
    use chrono::Utc;

    struct MockReferralRepository {
        items: Vec<Referral>,
    }

    #[async_trait]
    impl ReferralRepository for MockReferralRepository {
        async fn find_by_id(&self, id: Uuid) -> Result<Option<Referral>, RepositoryError> {
            Ok(self.items.iter().find(|item| item.id == id).cloned())
        }

        async fn find_by_patient(
            &self,
            patient_id: Uuid,
        ) -> Result<Vec<Referral>, RepositoryError> {
            Ok(self
                .items
                .iter()
                .filter(|item| item.patient_id == patient_id)
                .cloned()
                .collect())
        }

        async fn create(&self, referral: Referral) -> Result<Referral, RepositoryError> {
            Ok(referral)
        }

        async fn update(&self, referral: Referral) -> Result<Referral, RepositoryError> {
            Ok(referral)
        }

        async fn find_by_status(
            &self,
            status: ReferralStatus,
        ) -> Result<Vec<Referral>, RepositoryError> {
            Ok(self
                .items
                .iter()
                .filter(|item| item.status == status)
                .cloned()
                .collect())
        }
    }

    fn new_service(items: Vec<Referral>) -> ReferralService {
        ReferralService::new(Arc::new(MockReferralRepository { items }))
    }

    fn test_referral(status: ReferralStatus) -> Referral {
        let now = Utc::now();
        Referral {
            id: Uuid::new_v4(),
            patient_id: Uuid::new_v4(),
            referring_practitioner_id: Uuid::new_v4(),
            consultation_id: None,
            referral_type: ReferralType::Specialist,
            specialty: "Cardiology".to_string(),
            recipient_name: Some("Dr Smith".to_string()),
            recipient_address: None,
            recipient_phone: None,
            recipient_fax: None,
            recipient_email: None,
            reason: "Chest pain investigation".to_string(),
            clinical_notes: None,
            urgency: ReferralUrgency::Routine,
            referral_date: now.date_naive(),
            valid_until: None,
            status,
            sent_via: Some(ReferralDeliveryMethod::SecureMessaging),
            sent_at: None,
            appointment_made: false,
            appointment_date: None,
            response_received: false,
            response_date: None,
            created_at: now,
            updated_at: now,
            created_by: Uuid::new_v4(),
            updated_by: None,
        }
    }

    #[tokio::test]
    async fn test_create_referral_rejects_empty_specialty() {
        let service = new_service(vec![]);
        let mut referral = test_referral(ReferralStatus::Draft);
        referral.specialty = "  ".to_string();

        let result = service.create_referral(referral).await;

        assert!(matches!(
            result,
            Err(ServiceError::Validation(ValidationError::EmptySpecialty))
        ));
    }

    #[tokio::test]
    async fn test_mark_sent_sets_status_and_updated_by() {
        let mut referral = test_referral(ReferralStatus::Draft);
        let id = referral.id;
        let user_id = Uuid::new_v4();
        referral.updated_by = None;

        let service = new_service(vec![referral]);
        let result = service.mark_sent(id, user_id).await;

        assert!(result.is_ok());
        let updated = result.expect("mark_sent should succeed");
        assert_eq!(updated.status, ReferralStatus::Sent);
        assert_eq!(updated.updated_by, Some(user_id));
    }

    #[tokio::test]
    async fn test_find_by_status_returns_matching_referrals() {
        let service = new_service(vec![
            test_referral(ReferralStatus::Draft),
            test_referral(ReferralStatus::Sent),
        ]);

        let result = service.find_by_status(ReferralStatus::Sent).await;

        assert!(result.is_ok());
        let items = result.expect("result should be ok");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].status, ReferralStatus::Sent);
    }
}

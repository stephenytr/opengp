use async_trait::async_trait;
use uuid::Uuid;

use super::error::RepositoryError;
use super::model::{Referral, ReferralStatus};

/// Repository boundary for referral records.
#[async_trait]
pub trait ReferralRepository: Send + Sync {
    /// Find a referral by identifier.
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Referral>, RepositoryError>;

    /// List referrals for a given patient.
    async fn find_by_patient(&self, patient_id: Uuid) -> Result<Vec<Referral>, RepositoryError>;

    /// Create a new referral.
    async fn create(&self, referral: Referral) -> Result<Referral, RepositoryError>;

    /// Persist changes to an existing referral.
    async fn update(&self, referral: Referral) -> Result<Referral, RepositoryError>;

    /// Find referrals filtered by status (eg Sent, Draft).
    async fn find_by_status(
        &self,
        status: ReferralStatus,
    ) -> Result<Vec<Referral>, RepositoryError>;
}

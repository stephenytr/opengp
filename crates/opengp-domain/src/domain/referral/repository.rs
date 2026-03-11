use async_trait::async_trait;
use uuid::Uuid;

use super::error::RepositoryError;
use super::model::{Referral, ReferralStatus};

#[async_trait]
pub trait ReferralRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Referral>, RepositoryError>;
    async fn find_by_patient(&self, patient_id: Uuid) -> Result<Vec<Referral>, RepositoryError>;
    async fn create(&self, referral: Referral) -> Result<Referral, RepositoryError>;
    async fn update(&self, referral: Referral) -> Result<Referral, RepositoryError>;
    async fn find_by_status(
        &self,
        status: ReferralStatus,
    ) -> Result<Vec<Referral>, RepositoryError>;
}

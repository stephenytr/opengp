use async_trait::async_trait;
use uuid::Uuid;

use super::error::RepositoryError;
use super::model::{OrderStatus, PathologyOrder, PathologyResult};

#[async_trait]
pub trait PathologyRepository: Send + Sync {
    async fn find_order_by_id(&self, id: Uuid) -> Result<Option<PathologyOrder>, RepositoryError>;
    async fn find_result_by_id(&self, id: Uuid) -> Result<Option<PathologyResult>, RepositoryError>;
    async fn find_orders_by_patient(
        &self,
        patient_id: Uuid,
    ) -> Result<Vec<PathologyOrder>, RepositoryError>;
    async fn create_order(&self, order: PathologyOrder) -> Result<PathologyOrder, RepositoryError>;
    async fn update_order(&self, order: PathologyOrder) -> Result<PathologyOrder, RepositoryError>;
    async fn create_result(
        &self,
        result: PathologyResult,
    ) -> Result<PathologyResult, RepositoryError>;
    async fn find_orders_by_status(
        &self,
        status: OrderStatus,
    ) -> Result<Vec<PathologyOrder>, RepositoryError>;
}

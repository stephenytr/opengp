use async_trait::async_trait;

use crate::domain::error::RepositoryError;

#[async_trait]
pub trait CrudRepository<TEntity, TId>: Send + Sync
where
    TEntity: Send + Sync + 'static,
    TId: Send + Sync + 'static,
{
    async fn find_by_id(&self, _id: TId) -> Result<Option<TEntity>, RepositoryError> {
        Err(RepositoryError::Database(
            "Repository operation 'find_by_id' is not implemented".to_string(),
        ))
    }

    async fn create(&self, _entity: TEntity) -> Result<TEntity, RepositoryError> {
        Err(RepositoryError::Database(
            "Repository operation 'create' is not implemented".to_string(),
        ))
    }

    async fn update(&self, _entity: TEntity) -> Result<TEntity, RepositoryError> {
        Err(RepositoryError::Database(
            "Repository operation 'update' is not implemented".to_string(),
        ))
    }

    async fn delete(&self, _id: TId) -> Result<(), RepositoryError> {
        Err(RepositoryError::Database(
            "Repository operation 'delete' is not implemented".to_string(),
        ))
    }

    async fn find_all(&self) -> Result<Vec<TEntity>, RepositoryError> {
        Err(RepositoryError::Database(
            "Repository operation 'find_all' is not implemented".to_string(),
        ))
    }
}

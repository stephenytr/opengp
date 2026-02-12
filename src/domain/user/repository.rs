use async_trait::async_trait;
use uuid::Uuid;

use super::model::Practitioner;

#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("Database error: {0}")]
    Database(String),
    
    #[error("Not found")]
    NotFound,
}

/// Repository trait for practitioner persistence
#[async_trait]
pub trait PractitionerRepository: Send + Sync {
    /// List all active practitioners
    ///
    /// # Returns
    /// * `Ok(Vec<Practitioner>)` - List of active practitioners
    /// * `Err(RepositoryError)` - Database error
    async fn list_active(&self) -> Result<Vec<Practitioner>, RepositoryError>;
    
    /// Find practitioner by ID
    ///
    /// # Returns
    /// * `Ok(Some(Practitioner))` - Practitioner found
    /// * `Ok(None)` - Practitioner not found
    /// * `Err(RepositoryError)` - Database error
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Practitioner>, RepositoryError>;
}

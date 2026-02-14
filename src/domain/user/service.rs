use std::sync::Arc;
use tracing::{error, info};
use uuid::Uuid;

use super::model::Practitioner;
use super::repository::PractitionerRepository;

#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("Repository error: {0}")]
    Repository(String),

    #[error("Practitioner not found: {0}")]
    NotFound(Uuid),
}

/// Service layer for practitioner business logic
pub struct PractitionerService {
    repository: Arc<dyn PractitionerRepository>,
}

impl PractitionerService {
    pub fn new(repository: Arc<dyn PractitionerRepository>) -> Self {
        Self { repository }
    }

    /// Get all active practitioners
    ///
    /// # Returns
    /// * `Ok(Vec<Practitioner>)` - List of active practitioners
    /// * `Err(ServiceError)` - Database error
    pub async fn get_active_practitioners(&self) -> Result<Vec<Practitioner>, ServiceError> {
        info!("Fetching active practitioners");

        match self.repository.list_active().await {
            Ok(practitioners) => {
                info!("Found {} active practitioners", practitioners.len());
                Ok(practitioners)
            }
            Err(e) => {
                error!("Failed to fetch practitioners: {}", e);
                Err(ServiceError::Repository(e.to_string()))
            }
        }
    }
}

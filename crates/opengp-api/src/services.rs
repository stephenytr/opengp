use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use opengp_domain::domain::audit::{AuditEntry, AuditRepository, AuditRepositoryError, AuditService};
use opengp_infrastructure::infrastructure::crypto::EncryptionService;
use uuid::Uuid;

use crate::{ApiConfig, ApiError};

#[derive(Clone)]
pub struct ApiServices {
    pub audit_service: Arc<AuditService>,
    pub encryption_service: Arc<EncryptionService>,
}

impl ApiServices {
    pub fn new(config: &ApiConfig) -> Result<Self, ApiError> {
        unsafe {
            std::env::set_var("ENCRYPTION_KEY", &config.encryption_key);
        }

        let encryption_service = Arc::new(
            EncryptionService::new().map_err(|e| ApiError::EncryptionInit(e.to_string()))?,
        );
        let audit_repository: Arc<dyn AuditRepository> = Arc::new(NoopAuditRepository);
        let audit_service = Arc::new(AuditService::new(audit_repository));

        Ok(Self {
            audit_service,
            encryption_service,
        })
    }
}

struct NoopAuditRepository;

#[async_trait]
impl AuditRepository for NoopAuditRepository {
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

    async fn find_by_user(&self, _user_id: Uuid) -> Result<Vec<AuditEntry>, AuditRepositoryError> {
        Ok(vec![])
    }

    async fn find_by_time_range(
        &self,
        _start_time: DateTime<Utc>,
        _end_time: DateTime<Utc>,
    ) -> Result<Vec<AuditEntry>, AuditRepositoryError> {
        Ok(vec![])
    }

    async fn find_by_entity_and_time_range(
        &self,
        _entity_type: &str,
        _entity_id: Uuid,
        _start_time: DateTime<Utc>,
        _end_time: DateTime<Utc>,
    ) -> Result<Vec<AuditEntry>, AuditRepositoryError> {
        Ok(vec![])
    }
}

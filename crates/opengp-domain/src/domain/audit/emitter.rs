use async_trait::async_trait;

use super::error::AuditEmitterError;
use super::model::AuditEntry;

#[async_trait]
pub trait AuditEmitter: Send + Sync {
    async fn emit(&self, entry: AuditEntry) -> Result<(), AuditEmitterError>;
}

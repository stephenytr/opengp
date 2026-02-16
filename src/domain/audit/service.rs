use std::sync::Arc;
use tracing::{error, info};
use uuid::Uuid;

use crate::service;

use super::error::ServiceError;
use super::model::AuditEntry;
use super::repository::AuditRepository;

service! {
    AuditService {
        repository: Arc<dyn AuditRepository>,
    }
}

impl AuditService {

    /// Log a new audit entry
    ///
    /// # Arguments
    /// * `entry` - The audit entry to log
    ///
    /// # Returns
    /// * `Ok(AuditEntry)` - Successfully logged audit entry
    /// * `Err(ServiceError::Repository)` - Database error
    pub async fn log(&self, entry: AuditEntry) -> Result<AuditEntry, ServiceError> {
        info!(
            "Logging audit entry: {} for entity {} ({})",
            entry.action, entry.entity_id, entry.entity_type
        );

        match self.repository.create(entry.clone()).await {
            Ok(saved) => {
                info!("Audit entry logged successfully: {}", saved.id);
                Ok(saved)
            }
            Err(e) => {
                error!("Failed to log audit entry: {}", e);
                Err(e.into())
            }
        }
    }

    /// Get full audit history for an appointment
    ///
    /// Returns all audit entries for a specific appointment in reverse chronological order
    /// (newest first).
    ///
    /// # Arguments
    /// * `appointment_id` - ID of the appointment
    ///
    /// # Returns
    /// * `Ok(Vec<AuditEntry>)` - List of audit entries sorted by changed_at DESC
    /// * `Err(ServiceError::Repository)` - Database error
    ///
    /// # Example
    /// ```ignore
    /// use opengp::domain::audit::AuditService;
    /// use uuid::Uuid;
    ///
    /// async fn show_appointment_history(
    ///     audit_service: &AuditService,
    ///     appointment_id: Uuid,
    /// ) {
    ///     match audit_service.get_appointment_history(appointment_id).await {
    ///         Ok(entries) => {
    ///             for entry in entries {
    ///                 println!(
    ///                     "[{}] {} by {} - {}",
    ///                     entry.changed_at.format("%Y-%m-%d %H:%M"),
    ///                     entry.action,
    ///                     entry.changed_by,
    ///                     entry.entity_type
    ///                 );
    ///             }
    ///         }
    ///         Err(e) => eprintln!("Failed to fetch history: {}", e),
    ///     }
    /// }
    /// ```
    pub async fn get_appointment_history(
        &self,
        appointment_id: Uuid,
    ) -> Result<Vec<AuditEntry>, ServiceError> {
        info!("Fetching audit history for appointment: {}", appointment_id);

        let mut entries = self
            .repository
            .find_by_entity("appointment", appointment_id)
            .await?;

        // Sort by changed_at DESC (newest first)
        entries.sort_by(|a, b| b.changed_at.cmp(&a.changed_at));

        info!(
            "Found {} audit entries for appointment {}",
            entries.len(),
            appointment_id
        );

        Ok(entries)
    }

    /// Get all actions performed by a specific user
    ///
    /// Returns all audit entries created by a specific user in reverse chronological order
    /// (newest first).
    ///
    /// # Arguments
    /// * `user_id` - ID of the user
    ///
    /// # Returns
    /// * `Ok(Vec<AuditEntry>)` - List of audit entries by the user
    /// * `Err(ServiceError::Repository)` - Database error
    pub async fn get_user_activity(&self, user_id: Uuid) -> Result<Vec<AuditEntry>, ServiceError> {
        info!("Fetching activity for user: {}", user_id);

        let entries = self.repository.find_by_user(user_id).await?;

        info!("Found {} audit entries for user {}", entries.len(), user_id);

        Ok(entries)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::audit::AuditRepositoryError;
    use async_trait::async_trait;
    use chrono::Utc;
    // Mock repository for testing
    struct MockAuditRepository {
        entries: Vec<AuditEntry>,
    }

    #[async_trait]
    impl AuditRepository for MockAuditRepository {
        async fn create(&self, entry: AuditEntry) -> Result<AuditEntry, AuditRepositoryError> {
            Ok(entry)
        }

        async fn find_by_entity(
            &self,
            _entity_type: &str,
            _entity_id: Uuid,
        ) -> Result<Vec<AuditEntry>, AuditRepositoryError> {
            Ok(self.entries.clone())
        }

        async fn find_by_user(
            &self,
            _user_id: Uuid,
        ) -> Result<Vec<AuditEntry>, AuditRepositoryError> {
            Ok(self.entries.clone())
        }

        async fn find_by_time_range(
            &self,
            _start_time: chrono::DateTime<Utc>,
            _end_time: chrono::DateTime<Utc>,
        ) -> Result<Vec<AuditEntry>, AuditRepositoryError> {
            Ok(self.entries.clone())
        }

        async fn find_by_entity_and_time_range(
            &self,
            _entity_type: &str,
            _entity_id: Uuid,
            _start_time: chrono::DateTime<Utc>,
            _end_time: chrono::DateTime<Utc>,
        ) -> Result<Vec<AuditEntry>, AuditRepositoryError> {
            Ok(self.entries.clone())
        }
    }

    #[tokio::test]
    async fn test_log_audit_entry() {
        let repo = Arc::new(MockAuditRepository { entries: vec![] });
        let service = AuditService::new(repo);

        let entry = AuditEntry::new_created(
            "appointment",
            Uuid::new_v4(),
            r#"{"patient_id":"123"}"#,
            Uuid::new_v4(),
        );

        let result = service.log(entry.clone()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().id, entry.id);
    }

    #[tokio::test]
    async fn test_get_appointment_history() {
        let appointment_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        // Create entries with different timestamps
        let entry1 = AuditEntry::new_created("appointment", appointment_id, "{}", user_id);

        // Wait a bit to ensure different timestamps
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        let entry2 = AuditEntry::new_status_changed(
            "appointment",
            appointment_id,
            "Scheduled",
            "Arrived",
            user_id,
        );

        let repo = Arc::new(MockAuditRepository {
            entries: vec![entry1.clone(), entry2.clone()],
        });
        let service = AuditService::new(repo);

        let history = service
            .get_appointment_history(appointment_id)
            .await
            .unwrap();

        assert_eq!(history.len(), 2);
        // Verify newest first (DESC order)
        assert!(history[0].changed_at >= history[1].changed_at);
    }

    #[tokio::test]
    async fn test_get_user_activity() {
        let user_id = Uuid::new_v4();
        let entry = AuditEntry::new_created("appointment", Uuid::new_v4(), "{}", user_id);

        let repo = Arc::new(MockAuditRepository {
            entries: vec![entry],
        });
        let service = AuditService::new(repo);

        let activity = service.get_user_activity(user_id).await.unwrap();
        assert_eq!(activity.len(), 1);
        assert_eq!(activity[0].changed_by, user_id);
    }
}

use chrono::{Duration, Utc};
use rand::seq::SliceRandom;
use rand::Rng;
use uuid::Uuid;

use opengp_domain::domain::audit::{AuditAction, AuditEntry};

/// Configuration for audit entry generation
///
/// Controls how many audit entries are generated and their characteristics.
#[derive(Debug, Clone)]
pub struct AuditGeneratorConfig {
    /// Number of audit entries to generate
    pub count: usize,
    /// Maximum days in the past for audit entries (0 = today only)
    pub max_days_past: i64,
    /// Percentage of entries that are Created actions (0.0-1.0)
    pub created_percentage: f32,
    /// Percentage of entries that are Updated actions (0.0-1.0)
    pub updated_percentage: f32,
    /// Percentage of entries that are StatusChanged actions (0.0-1.0)
    pub status_changed_percentage: f32,
}

impl Default for AuditGeneratorConfig {
    fn default() -> Self {
        Self {
            count: 10,
            max_days_past: 30,
            created_percentage: 0.30,
            updated_percentage: 0.40,
            status_changed_percentage: 0.30,
        }
    }
}

/// Generator for realistic audit entry test data
///
/// Creates audit entries with realistic actions and chronological timestamps.
/// Supports configurable entity types and user IDs.
pub struct AuditGenerator {
    config: AuditGeneratorConfig,
    rng: rand::rngs::ThreadRng,
}

impl AuditGenerator {
    /// Create a new audit generator with the given configuration
    pub fn new(config: AuditGeneratorConfig) -> Self {
        Self {
            config,
            rng: rand::thread_rng(),
        }
    }

    /// Generate a vector of audit entries
    pub fn generate(&mut self) -> Vec<AuditEntry> {
        (0..self.config.count)
            .map(|_| self.generate_audit_entry())
            .collect()
    }

    /// Generate a single audit entry with random data
    fn generate_audit_entry(&mut self) -> AuditEntry {
        let entity_type = self.random_entity_type();
        let entity_id = Uuid::new_v4();
        let changed_by = Uuid::new_v4();
        let changed_at = self.random_timestamp();

        let action = self.random_action();

        let (old_value, new_value) = match &action {
            AuditAction::Created => (None, Some(self.random_json_value())),
            AuditAction::Updated => (
                Some(self.random_json_value()),
                Some(self.random_json_value()),
            ),
            AuditAction::Read => (None, None),
            AuditAction::StatusChanged { from, to } => (Some(from.clone()), Some(to.clone())),
            AuditAction::Rescheduled { from, to } => {
                (Some(from.to_rfc3339()), Some(to.to_rfc3339()))
            }
            AuditAction::Cancelled { reason } => (None, Some(reason.clone())),
        };

        AuditEntry {
            id: Uuid::new_v4(),
            entity_type,
            entity_id,
            action,
            old_value,
            new_value,
            changed_by,
            changed_at,
            source: "database".to_string(),
        }
    }

    /// Generate a random entity type
    fn random_entity_type(&mut self) -> String {
        let types = [
            "appointment",
            "patient",
            "prescription",
            "immunisation",
            "clinical_note",
            "user",
        ];

        types
            .choose(&mut self.rng)
            .unwrap_or(&types[0])
            .to_string()
    }

    /// Generate a random audit action
    fn random_action(&mut self) -> AuditAction {
        let choice = self.rng.gen_range(0.0..1.0);

        if choice < self.config.created_percentage as f64 {
            AuditAction::Created
        } else if choice < (self.config.created_percentage + self.config.updated_percentage) as f64
        {
            AuditAction::Updated
        } else if choice
            < (self.config.created_percentage
                + self.config.updated_percentage
                + self.config.status_changed_percentage) as f64
        {
            AuditAction::StatusChanged {
                from: self.random_status(),
                to: self.random_status(),
            }
        } else if self.rng.gen_bool(0.5) {
            let from = Utc::now() - Duration::hours(1);
            let to = Utc::now();
            AuditAction::Rescheduled { from, to }
        } else {
            AuditAction::Cancelled {
                reason: self.random_cancellation_reason(),
            }
        }
    }

    /// Generate a random status string
    fn random_status(&mut self) -> String {
        let statuses = [
            "Scheduled",
            "Confirmed",
            "Arrived",
            "InProgress",
            "Completed",
            "Cancelled",
            "NoShow",
        ];

        statuses
            .choose(&mut self.rng)
            .unwrap_or(&statuses[0])
            .to_string()
    }

    /// Generate a random cancellation reason
    fn random_cancellation_reason(&mut self) -> String {
        let reasons = [
            "Patient requested",
            "Practitioner unavailable",
            "Scheduling conflict",
            "Patient no-show",
            "Emergency",
            "Administrative error",
        ];

        reasons
            .choose(&mut self.rng)
            .unwrap_or(&reasons[0])
            .to_string()
    }

    /// Generate a random JSON value string
    fn random_json_value(&mut self) -> String {
        let values = [
            r#"{"status":"active"}"#,
            r#"{"name":"John Doe"}"#,
            r#"{"email":"john@example.com"}"#,
            r#"{"phone":"0412345678"}"#,
            r#"{"notes":"Updated patient record"}"#,
            r#"{"medication":"Paracetamol"}"#,
        ];

        values
            .choose(&mut self.rng)
            .unwrap_or(&values[0])
            .to_string()
    }

    /// Generate a random timestamp within the configured range
    fn random_timestamp(&mut self) -> chrono::DateTime<Utc> {
        let days_ago = self.rng.gen_range(0..=self.config.max_days_past);
        let hours_ago = self.rng.gen_range(0..24);
        let minutes_ago = self.rng.gen_range(0..60);

        Utc::now()
            - Duration::days(days_ago)
            - Duration::hours(hours_ago)
            - Duration::minutes(minutes_ago)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_audit_entries() {
        let config = AuditGeneratorConfig {
            count: 5,
            ..Default::default()
        };

        let mut generator = AuditGenerator::new(config);
        let entries = generator.generate();

        assert_eq!(entries.len(), 5);

        for entry in &entries {
            assert_ne!(entry.entity_id, Uuid::nil());
            assert_ne!(entry.changed_by, Uuid::nil());
            assert!(!entry.entity_type.is_empty());
        }
    }

    #[test]
    fn test_audit_entries_have_valid_timestamps() {
        let config = AuditGeneratorConfig {
            count: 10,
            max_days_past: 30,
            ..Default::default()
        };

        let mut generator = AuditGenerator::new(config);
        let entries = generator.generate();

        let now = Utc::now();
        for entry in &entries {
            assert!(entry.changed_at <= now, "Timestamp in future");
            let days_old = (now - entry.changed_at).num_days();
            assert!(days_old <= 30, "Timestamp older than max_days_past");
        }
    }

    #[test]
    fn test_audit_entries_have_actions() {
        let config = AuditGeneratorConfig {
            count: 10,
            ..Default::default()
        };

        let mut generator = AuditGenerator::new(config);
        let entries = generator.generate();

        for entry in &entries {
            match &entry.action {
                AuditAction::Created => assert!(entry.new_value.is_some()),
                AuditAction::Updated => {
                    assert!(entry.old_value.is_some());
                    assert!(entry.new_value.is_some());
                }
                AuditAction::Read => {
                    // Read actions don't require old/new values
                }
                AuditAction::StatusChanged { from, to } => {
                    assert!(!from.is_empty());
                    assert!(!to.is_empty());
                }
                AuditAction::Rescheduled { from, to } => {
                    assert!(from < to);
                }
                AuditAction::Cancelled { reason } => {
                    assert!(!reason.is_empty());
                }
            }
        }
    }

    #[test]
    fn test_config_action_distribution() {
        let config = AuditGeneratorConfig {
            count: 100,
            created_percentage: 0.50,
            updated_percentage: 0.30,
            status_changed_percentage: 0.20,
            ..Default::default()
        };

        let mut generator = AuditGenerator::new(config);
        let entries = generator.generate();

        let created_count = entries
            .iter()
            .filter(|e| matches!(e.action, AuditAction::Created))
            .count();

        assert!(created_count > 30, "Expected more Created actions");
    }

    #[test]
    fn test_entity_types_are_valid() {
        let config = AuditGeneratorConfig {
            count: 20,
            ..Default::default()
        };

        let mut generator = AuditGenerator::new(config);
        let entries = generator.generate();

        let valid_types = [
            "appointment",
            "patient",
            "prescription",
            "immunisation",
            "clinical_note",
            "user",
        ];

        for entry in &entries {
            assert!(
                valid_types.contains(&entry.entity_type.as_str()),
                "Invalid entity type: {}",
                entry.entity_type
            );
        }
    }
}

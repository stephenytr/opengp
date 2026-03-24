//! Audit domain models
//!
//! This module contains the core audit entities used for tracking changes to appointments,
//! patients, and other domain entities. All changes to critical data are recorded for
//! compliance and audit trail requirements.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Core audit entry entity
///
/// Represents a single change or action performed on a domain entity.
/// Stores both the old and new values for change tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// Unique identifier for this audit entry
    pub id: Uuid,

    /// Type of entity being audited (e.g., "appointment", "patient", "prescription")
    pub entity_type: String,

    /// ID of the entity that was changed
    pub entity_id: Uuid,

    /// The action that was performed
    pub action: AuditAction,

    /// Previous value (JSON serialized, if applicable)
    pub old_value: Option<String>,

    /// New value (JSON serialized, if applicable)
    pub new_value: Option<String>,

    /// User who performed the action
    pub changed_by: Uuid,

    /// Timestamp when the action occurred
    pub changed_at: DateTime<Utc>,

    /// Source of the data access: "cache" or "database"
    pub source: String,
}

impl AuditEntry {
    /// Create a new audit entry for a create action
    ///
    /// # Arguments
    /// * `entity_type` - Type of entity (e.g., "appointment", "patient")
    /// * `entity_id` - ID of the created entity
    /// * `new_value` - JSON representation of the new entity
    /// * `changed_by` - User who created the entity
    ///
    /// # Returns
    /// A new audit entry with action set to Created
    pub fn new_created(
        entity_type: impl Into<String>,
        entity_id: Uuid,
        new_value: impl Into<String>,
        changed_by: Uuid,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            entity_type: entity_type.into(),
            entity_id,
            action: AuditAction::Created,
            old_value: None,
            new_value: Some(new_value.into()),
            changed_by,
            changed_at: Utc::now(),
            source: "database".to_string(),
        }
    }

    /// Create a new audit entry for an update action
    ///
    /// # Arguments
    /// * `entity_type` - Type of entity (e.g., "appointment", "patient")
    /// * `entity_id` - ID of the updated entity
    /// * `old_value` - JSON representation of the entity before update
    /// * `new_value` - JSON representation of the entity after update
    /// * `changed_by` - User who updated the entity
    ///
    /// # Returns
    /// A new audit entry with action set to Updated
    pub fn new_updated(
        entity_type: impl Into<String>,
        entity_id: Uuid,
        old_value: impl Into<String>,
        new_value: impl Into<String>,
        changed_by: Uuid,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            entity_type: entity_type.into(),
            entity_id,
            action: AuditAction::Updated,
            old_value: Some(old_value.into()),
            new_value: Some(new_value.into()),
            changed_by,
            changed_at: Utc::now(),
            source: "database".to_string(),
        }
    }

    /// Create a new audit entry for a read action
    ///
    /// # Arguments
    /// * `entity_type` - Type of entity (e.g., "patient", "appointment")
    /// * `entity_id` - ID of the entity that was read
    /// * `changed_by` - User who read the entity
    ///
    /// # Returns
    /// A new audit entry with action set to Read
    pub fn new_read(entity_type: impl Into<String>, entity_id: Uuid, changed_by: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            entity_type: entity_type.into(),
            entity_id,
            action: AuditAction::Read,
            old_value: None,
            new_value: None,
            changed_by,
            changed_at: Utc::now(),
            source: "database".to_string(),
        }
    }

    /// Create a new audit entry for a status change
    ///
    /// # Arguments
    /// * `entity_type` - Type of entity (e.g., "appointment")
    /// * `entity_id` - ID of the entity
    /// * `from_status` - Previous status
    /// * `to_status` - New status
    /// * `changed_by` - User who changed the status
    ///
    /// # Returns
    /// A new audit entry with action set to StatusChanged
    pub fn new_status_changed(
        entity_type: impl Into<String>,
        entity_id: Uuid,
        from_status: impl Into<String>,
        to_status: impl Into<String>,
        changed_by: Uuid,
    ) -> Self {
        let from = from_status.into();
        let to = to_status.into();

        Self {
            id: Uuid::new_v4(),
            entity_type: entity_type.into(),
            entity_id,
            action: AuditAction::StatusChanged {
                from: from.clone(),
                to: to.clone(),
            },
            old_value: Some(from),
            new_value: Some(to),
            changed_by,
            changed_at: Utc::now(),
            source: "database".to_string(),
        }
    }

    /// Create a new audit entry for an appointment reschedule
    ///
    /// # Arguments
    /// * `entity_id` - ID of the appointment
    /// * `from_time` - Previous appointment time
    /// * `to_time` - New appointment time
    /// * `changed_by` - User who rescheduled the appointment
    ///
    /// # Returns
    /// A new audit entry with action set to Rescheduled
    pub fn new_rescheduled(
        entity_id: Uuid,
        from_time: DateTime<Utc>,
        to_time: DateTime<Utc>,
        changed_by: Uuid,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            entity_type: "appointment".to_string(),
            entity_id,
            action: AuditAction::Rescheduled {
                from: from_time,
                to: to_time,
            },
            old_value: Some(from_time.to_rfc3339()),
            new_value: Some(to_time.to_rfc3339()),
            changed_by,
            changed_at: Utc::now(),
            source: "database".to_string(),
        }
    }

    /// Create a new audit entry for a cancellation
    ///
    /// # Arguments
    /// * `entity_type` - Type of entity (e.g., "appointment", "prescription")
    /// * `entity_id` - ID of the cancelled entity
    /// * `reason` - Reason for cancellation
    /// * `changed_by` - User who cancelled the entity
    ///
    /// # Returns
    /// A new audit entry with action set to Cancelled
    pub fn new_cancelled(
        entity_type: impl Into<String>,
        entity_id: Uuid,
        reason: impl Into<String>,
        changed_by: Uuid,
    ) -> Self {
        let reason_str = reason.into();

        Self {
            id: Uuid::new_v4(),
            entity_type: entity_type.into(),
            entity_id,
            action: AuditAction::Cancelled {
                reason: reason_str.clone(),
            },
            old_value: None,
            new_value: Some(reason_str),
            changed_by,
            changed_at: Utc::now(),
            source: "database".to_string(),
        }
    }
}

/// Types of actions that can be audited
///
/// This enum represents all possible actions that can be recorded in the audit log.
/// Variants with payloads (like `StatusChanged`, `Rescheduled`, `Cancelled`) serialize
/// their payload as JSON for storage in the database.
///
/// # JSON Serialization
/// The `AuditAction` enum is serialized to JSON for storage in the `action` column
/// of the audit entries table. This allows capturing detailed information about
/// each action type while maintaining a consistent schema.
///
/// # Example
/// ```json
/// {
///   "StatusChanged": { "from": "Scheduled", "to": "Confirmed" }
/// }
/// ```
///
/// # Display Format
/// Each variant implements [`std::fmt::Display`] for human-readable logging.
/// Use `entry.action.to_string()` to get a formatted description of the action.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuditAction {
    /// Entity was created
    Created,

    /// Entity was updated (general update)
    Updated,

    /// Entity was read (for compliance logging of data access)
    Read,

    /// Entity status changed
    StatusChanged {
        /// Previous status
        from: String,
        /// New status
        to: String,
    },

    /// Appointment was rescheduled
    Rescheduled {
        /// Previous appointment time
        from: DateTime<Utc>,
        /// New appointment time
        to: DateTime<Utc>,
    },

    /// Entity was cancelled
    Cancelled {
        /// Reason for cancellation
        reason: String,
    },
}

impl std::fmt::Display for AuditAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuditAction::Created => write!(f, "Created"),
            AuditAction::Updated => write!(f, "Updated"),
            AuditAction::Read => write!(f, "Read"),
            AuditAction::StatusChanged { from, to } => {
                write!(f, "Status Changed: {} → {}", from, to)
            }
            AuditAction::Rescheduled { from, to } => {
                write!(
                    f,
                    "Rescheduled: {} → {}",
                    from.format("%d/%m/%Y %H:%M"),
                    to.format("%d/%m/%Y %H:%M")
                )
            }
            AuditAction::Cancelled { reason } => write!(f, "Cancelled: {}", reason),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_created_audit_entry() {
        let entity_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let entry =
            AuditEntry::new_created("appointment", entity_id, r#"{"patient_id":"123"}"#, user_id);

        assert_eq!(entry.entity_type, "appointment");
        assert_eq!(entry.entity_id, entity_id);
        assert_eq!(entry.action, AuditAction::Created);
        assert!(entry.old_value.is_none());
        assert!(entry.new_value.is_some());
        assert_eq!(entry.changed_by, user_id);
    }

    #[test]
    fn test_new_updated_audit_entry() {
        let entity_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let entry = AuditEntry::new_updated(
            "patient",
            entity_id,
            r#"{"name":"John"}"#,
            r#"{"name":"Jane"}"#,
            user_id,
        );

        assert_eq!(entry.entity_type, "patient");
        assert_eq!(entry.action, AuditAction::Updated);
        assert!(entry.old_value.is_some());
        assert!(entry.new_value.is_some());
    }

    #[test]
    fn test_new_status_changed_audit_entry() {
        let entity_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let entry = AuditEntry::new_status_changed(
            "appointment",
            entity_id,
            "Scheduled",
            "Confirmed",
            user_id,
        );

        match entry.action {
            AuditAction::StatusChanged { ref from, ref to } => {
                assert_eq!(from, "Scheduled");
                assert_eq!(to, "Confirmed");
            }
            _ => panic!("Expected StatusChanged action"),
        }
    }

    #[test]
    fn test_new_rescheduled_audit_entry() {
        let entity_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let from_time = Utc::now();
        let to_time = from_time + chrono::Duration::hours(2);

        let entry = AuditEntry::new_rescheduled(entity_id, from_time, to_time, user_id);

        assert_eq!(entry.entity_type, "appointment");
        match entry.action {
            AuditAction::Rescheduled { from, to } => {
                assert_eq!(from, from_time);
                assert_eq!(to, to_time);
            }
            _ => panic!("Expected Rescheduled action"),
        }
    }

    #[test]
    fn test_new_cancelled_audit_entry() {
        let entity_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let entry = AuditEntry::new_cancelled(
            "appointment",
            entity_id,
            "Patient requested cancellation",
            user_id,
        );

        match entry.action {
            AuditAction::Cancelled { ref reason } => {
                assert_eq!(reason, "Patient requested cancellation");
            }
            _ => panic!("Expected Cancelled action"),
        }
    }

    #[test]
    fn test_audit_action_display() {
        let action = AuditAction::Created;
        assert_eq!(action.to_string(), "Created");

        let action = AuditAction::StatusChanged {
            from: "Scheduled".to_string(),
            to: "Confirmed".to_string(),
        };
        assert_eq!(action.to_string(), "Status Changed: Scheduled → Confirmed");

        let action = AuditAction::Cancelled {
            reason: "Test reason".to_string(),
        };
        assert_eq!(action.to_string(), "Cancelled: Test reason");
    }
}

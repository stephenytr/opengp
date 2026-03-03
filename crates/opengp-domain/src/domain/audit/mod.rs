//! Audit domain module
//!
//! This module provides the audit logging functionality for OpenGP, tracking all
//! changes to critical domain entities for compliance and accountability.
//!
//! # Overview
//!
//! The audit system follows an append-only design pattern - audit entries are
//! never modified or deleted once created. This ensures a complete, tamper-proof
//! audit trail as required by Australian healthcare regulations.
//!
//! # Key Components
//!
//! - [`AuditEntry`] - The core entity representing a single audit record
//! - [`AuditAction`] - Enum representing the type of action performed
//! - [`AuditRepository`] - Trait for persisting audit entries
//! - [`AuditService`] - Business logic layer for audit operations
//!
//! # Compliance
//!
//! This module supports compliance with:
//! - Privacy Act 1988 (Australian Privacy Principles)
//! - RACGP Standards for General Practices
//! - My Health Records Act 2012
//!
//! All patient data access and modifications are logged with:
//! - User identity (who performed the action)
//! - Timestamp (when the action occurred)
//! - Entity details (what was changed)
//! - Before/after values (what the change was)
//!
//! # Usage Example
//!
//! ```ignore
//! use opengp::domain::audit::{AuditEntry, AuditService};
//! use uuid::Uuid;
//!
//! // Log an appointment creation
//! let entry = AuditEntry::new_created(
//!     "appointment",
//!     appointment_id,
//!     r#"{"patient_id": "123", "practitioner_id": "456"}"#,
//!     user_id,
//! );
//!
//! let saved = audit_service.log(entry).await?;
//! ```
//!
//! # Architecture
//!
//! The module follows the standard domain layer pattern:
//!
//! ```text
//! +-------------+     +---------------+     +------------------+
//! |   Service   | --> |  Repository   | --> | Infrastructure   |
//! |   Layer     |     |    (Trait)    |     | (SQLx impl)      |
//! +-------------+     +---------------+     +------------------+
//! ```
//!
//! The service layer provides business-logic-level operations while the repository
//! trait enables dependency injection for testing and different storage backends.

mod error;
mod model;
mod repository;
mod service;

pub use error::*;
pub use model::*;
pub use repository::*;
pub use service::*;

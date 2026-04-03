//! UI Services Module
//!
//! Bridge between UI components and domain layer.
//! Provides async services for data operations.

pub mod appointment_service;
pub mod billing_service;
pub mod clinical_service;
pub mod patient_service;
pub mod shared;

pub use appointment_service::AppointmentUiService;
pub use billing_service::BillingUiService;
pub use clinical_service::ClinicalUiService;
pub use patient_service::PatientUiService;
pub use shared::{ToUiError, UiResult, UiServiceError};

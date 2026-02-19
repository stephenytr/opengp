//! UI Services Module
//!
//! Bridge between UI components and domain layer.
//! Provides async services for data operations.

pub mod appointment_service;
pub mod patient_service;

pub use appointment_service::AppointmentUiService;
pub use patient_service::PatientUiService;

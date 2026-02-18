//! UI Services Module
//!
//! Bridge between UI components and domain layer.
//! Provides async services for data operations.

pub mod patient_service;

pub use patient_service::PatientUiService;

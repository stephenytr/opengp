//! Test fixtures and data generation
//!
//! This module provides utilities for generating realistic test data for development,
//! testing, and seeding databases.

pub mod appointment_generator;
pub mod audit_generator;
pub mod clinical_generator;
pub mod comprehensive_generator;
#[cfg(feature = "immunisation")]
pub mod immunisation_generator;
pub mod patient_generator;
#[cfg(feature = "prescription")]
pub mod prescription_generator;
pub mod schedule_scenarios;
pub mod working_hours_generator;

pub use appointment_generator::{
    AppointmentGenerator, AppointmentGeneratorConfig, GenerationStats,
};
pub use audit_generator::{AuditGenerator, AuditGeneratorConfig};
pub use clinical_generator::{ClinicalDataGenerator, ClinicalDataGeneratorConfig};
pub use comprehensive_generator::{
    ComprehensivePatientGenerator, ComprehensivePatientGeneratorConfig, ComprehensivePatientProfile,
};
#[cfg(feature = "immunisation")]
pub use immunisation_generator::{ImmunisationGenerator, ImmunisationGeneratorConfig};
pub use patient_generator::{PatientGenerator, PatientGeneratorConfig};
#[cfg(feature = "prescription")]
pub use prescription_generator::{PrescriptionGenerator, PrescriptionGeneratorConfig};
pub use schedule_scenarios::ScheduleScenario;
pub use working_hours_generator::seed_working_hours;

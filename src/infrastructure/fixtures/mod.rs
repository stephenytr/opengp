//! Test fixtures and data generation
//!
//! This module provides utilities for generating realistic test data for development,
//! testing, and seeding databases.

pub mod patient_generator;

pub use patient_generator::{PatientGenerator, PatientGeneratorConfig};

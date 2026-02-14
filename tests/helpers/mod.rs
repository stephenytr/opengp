//! Test helper utilities for integration tests
//!
//! This module provides assertion helpers and other utilities for testing domain entities.

pub mod assertions;

pub use assertions::{
    assert_appointment_eq, assert_audit_entry_eq, assert_consultation_eq, assert_immunisation_eq,
    assert_patient_eq, assert_prescription_eq,
};

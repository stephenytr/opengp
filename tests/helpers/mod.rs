//! Test helper utilities for integration tests
//!
//! This module provides assertion helpers and other utilities for testing domain entities.

pub mod assertions;

#[cfg(feature = "immunisation")]
pub use assertions::assert_immunisation_eq;
#[cfg(feature = "prescription")]
pub use assertions::assert_prescription_eq;
pub use assertions::{
    assert_appointment_eq, assert_audit_entry_eq, assert_consultation_eq, assert_patient_eq,
};

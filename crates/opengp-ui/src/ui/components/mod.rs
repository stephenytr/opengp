//! OpenGP UI Components
//!
//! Reusable UI components for the TUI application.

pub mod appointment;
pub mod billing;
pub mod clinical;
pub mod clinical_row;
pub mod help;
pub mod patient;
pub mod patient_tab_bar;
pub mod shared;
pub mod status_bar;
pub mod tabs;
pub mod welcome_panel;
pub mod workspace;

pub use clinical_row::{ClinicalMenuKind, ClinicalRow};
pub use patient_tab_bar::{PatientTab, PatientTabBar};
pub use workspace::SubtabKind;

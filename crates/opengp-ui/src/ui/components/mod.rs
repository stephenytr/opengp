//! OpenGP UI Components
//!
//! Reusable UI components for the TUI application.

pub mod appointment;
pub mod billing;
pub mod clinical;
pub mod help;
pub mod patient;
pub mod patient_tab_bar;
pub mod shared;
pub mod status_bar;
pub mod subtab_bar;
pub mod tabs;
pub mod welcome_panel;
pub mod workspace;

pub use patient_tab_bar::{PatientTab, PatientTabBar};
pub use subtab_bar::{SubtabBar, SubtabKind};

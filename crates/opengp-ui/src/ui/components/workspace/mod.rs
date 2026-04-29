//! Patient workspace management module.
//!
//! Manages multiple concurrent patient workspaces with support for:
//! - Idempotent patient opening (opening same patient returns existing index)
//! - Maximum concurrent open patients
//! - Subtab state tracking (clinical, billing, appointments)
//! - Per-workspace colour assignment via round-robin palette

pub mod appointment_state;
pub mod appointments_view;
pub mod disabled_subtab_view;
pub mod manager;
pub mod workspace;

pub use appointment_state::PatientAppointmentState;
pub use appointments_view::{AppointmentViewAction, PatientAppointmentsView};
pub use disabled_subtab_view::DisabledSubtabView;
pub use manager::WorkspaceManager;
pub use workspace::{PatientWorkspace, SubtabKind, WorkspaceError};

//! Patient workspace management module.
//!
//! Manages multiple concurrent patient workspaces with support for:
//! - Idempotent patient opening (opening same patient returns existing index)
//! - Maximum concurrent open patients
//! - Subtab state tracking (clinical, billing, appointments)
//! - Per-workspace colour assignment via round-robin palette

pub mod workspace;
pub mod manager;
pub mod appointment_state;
pub mod appointments_view;
pub mod disabled_subtab_view;

pub use workspace::{PatientWorkspace, WorkspaceError, SubtabKind};
pub use manager::WorkspaceManager;
pub use appointment_state::PatientAppointmentState;
pub use appointments_view::{PatientAppointmentsView, AppointmentViewAction};
pub use disabled_subtab_view::DisabledSubtabView;

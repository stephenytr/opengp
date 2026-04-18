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
pub mod demographics_view;
pub mod demographics_view_list;
pub mod summary_view;
pub mod disabled_subtab_view;

pub use crate::ui::components::SubtabKind;
pub use workspace::{PatientWorkspace, WorkspaceError};
pub use manager::WorkspaceManager;
pub use appointment_state::PatientAppointmentState;
pub use appointments_view::{PatientAppointmentsView, AppointmentViewAction};
pub use demographics_view::DemographicsView;
pub use demographics_view_list::DemographicsViewList;
pub use summary_view::SummaryView;
pub use disabled_subtab_view::DisabledSubtabView;

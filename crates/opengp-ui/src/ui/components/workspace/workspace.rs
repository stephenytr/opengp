use std::collections::HashSet;
use uuid::Uuid;
use ratatui::style::Color;
use crate::ui::view_models::PatientListItem;
use crate::ui::components::clinical::ClinicalState;
use crate::ui::components::billing::PatientBillingState;
use crate::ui::components::SubtabKind;
use super::appointment_state::PatientAppointmentState;

#[derive(Debug, Clone)]
pub enum WorkspaceError {
    AlreadyAtLimit,
    FormOpen,
    TimerActive,
    IndexOutOfRange,
}

impl std::fmt::Display for WorkspaceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkspaceError::AlreadyAtLimit => write!(f, "Already at maximum open patient limit"),
            WorkspaceError::FormOpen => write!(f, "Cannot close workspace while form is open"),
            WorkspaceError::TimerActive => write!(f, "Cannot close workspace while timer is active"),
            WorkspaceError::IndexOutOfRange => write!(f, "Workspace index out of range"),
        }
    }
}

impl std::error::Error for WorkspaceError {}

#[derive(Clone)]
pub struct PatientWorkspace {
    pub patient_id: Uuid,
    pub patient_snapshot: PatientListItem,
    pub colour: Color,
    pub active_subtab: SubtabKind,
    pub loaded: HashSet<SubtabKind>,
    pub clinical: Option<ClinicalState>,
    pub billing: Option<PatientBillingState>,
    pub appointments: Option<PatientAppointmentState>,
}

impl PatientWorkspace {
    pub fn new(
        patient_snapshot: PatientListItem,
        colour: Color,
    ) -> Self {
        Self {
            patient_id: patient_snapshot.id,
            patient_snapshot,
            colour,
            active_subtab: SubtabKind::Clinical,
            loaded: HashSet::new(),
            clinical: None,
            billing: None,
            appointments: None,
        }
    }

    pub fn mark_loaded(&mut self, subtab: SubtabKind) {
        self.loaded.insert(subtab);
    }

    pub fn is_loaded(&self, subtab: SubtabKind) -> bool {
        self.loaded.contains(&subtab)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use opengp_domain::domain::patient::Gender;

    fn create_test_patient() -> PatientListItem {
        PatientListItem {
            id: Uuid::new_v4(),
            full_name: "Test Patient".to_string(),
            date_of_birth: NaiveDate::from_ymd_opt(1990, 1, 1).unwrap(),
            gender: Gender::Male,
            medicare_number: None,
            medicare_irn: None,
            ihi: None,
            phone_mobile: None,
        }
    }

    #[test]
    fn patient_workspace_new() {
        let patient = create_test_patient();
        let colour = Color::Cyan;
        let workspace = PatientWorkspace::new(patient.clone(), colour);

        assert_eq!(workspace.patient_id, patient.id);
        assert_eq!(workspace.colour, colour);
        assert_eq!(workspace.active_subtab, SubtabKind::Clinical);
        assert!(workspace.loaded.is_empty());
        assert!(workspace.clinical.is_none());
        assert!(workspace.billing.is_none());
        assert!(workspace.appointments.is_none());
    }

    #[test]
    fn mark_loaded_and_is_loaded() {
        let patient = create_test_patient();
        let mut workspace = PatientWorkspace::new(patient, Color::Cyan);

        assert!(!workspace.is_loaded(SubtabKind::Clinical));
        workspace.mark_loaded(SubtabKind::Clinical);
        assert!(workspace.is_loaded(SubtabKind::Clinical));
    }

    #[test]
    fn is_loaded_multiple_subtabs() {
        let patient = create_test_patient();
        let mut workspace = PatientWorkspace::new(patient, Color::Cyan);

        workspace.mark_loaded(SubtabKind::Clinical);
        workspace.mark_loaded(SubtabKind::Billing);

        assert!(workspace.is_loaded(SubtabKind::Clinical));
        assert!(workspace.is_loaded(SubtabKind::Billing));
        assert!(!workspace.is_loaded(SubtabKind::Appointments));
    }
}

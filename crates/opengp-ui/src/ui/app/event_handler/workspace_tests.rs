//! RED tests for workspace event handling
//! These tests define the expected behavior of workspace keybind dispatch
//! Tests should FAIL initially (RED), then pass after implementation (GREEN)

use crate::ui::view_models::PatientListItem;
use crate::ui::components::workspace::{WorkspaceManager, WorkspaceError};
use crate::ui::components::SubtabKind;
use crate::ui::theme::Theme;
use chrono::NaiveDate;
use opengp_domain::domain::patient::Gender;
use uuid::Uuid;

// Helper function to create a test patient
fn test_patient(id: Option<Uuid>) -> PatientListItem {
    PatientListItem {
        id: id.unwrap_or_else(Uuid::new_v4),
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
fn test_workspace_manager_cycle_next_basic() {
    let theme = Theme::default();
    let mut manager = WorkspaceManager::new(theme, 5);
    let p1 = test_patient(None);
    let p2 = test_patient(None);
    let p3 = test_patient(None);
    
    manager.open_patient(p1).unwrap();
    manager.open_patient(p2).unwrap();
    manager.open_patient(p3).unwrap();
    manager.active_index = Some(0);
    
    manager.cycle_next();
    assert_eq!(manager.active_index, Some(1));
    
    manager.cycle_next();
    assert_eq!(manager.active_index, Some(2));
    
    manager.cycle_next();
    assert_eq!(manager.active_index, Some(0));
}

#[test]
fn test_workspace_manager_cycle_prev_basic() {
    let theme = Theme::default();
    let mut manager = WorkspaceManager::new(theme, 5);
    let p1 = test_patient(None);
    let p2 = test_patient(None);
    let p3 = test_patient(None);
    
    manager.open_patient(p1).unwrap();
    manager.open_patient(p2).unwrap();
    manager.open_patient(p3).unwrap();
    manager.active_index = Some(0);
    
    manager.cycle_prev();
    assert_eq!(manager.active_index, Some(2));
    
    manager.cycle_prev();
    assert_eq!(manager.active_index, Some(1));
}

#[test]
fn test_workspace_manager_close_active() {
    let theme = Theme::default();
    let mut manager = WorkspaceManager::new(theme, 5);
    let p1 = test_patient(None);
    let p2 = test_patient(None);
    
    manager.open_patient(p1).unwrap();
    manager.open_patient(p2).unwrap();
    assert_eq!(manager.workspaces.len(), 2);
    
    manager.close_active().unwrap();
    assert_eq!(manager.workspaces.len(), 1);
    assert_eq!(manager.active_index, Some(0));
}

#[test]
fn test_workspace_manager_select_by_index_valid() {
    let theme = Theme::default();
    let mut manager = WorkspaceManager::new(theme, 5);
    let p1 = test_patient(None);
    let p2 = test_patient(None);
    
    manager.open_patient(p1).unwrap();
    manager.open_patient(p2).unwrap();
    
    manager.select_by_index(1).unwrap();
    assert_eq!(manager.active_index, Some(1));
}

#[test]
fn test_workspace_manager_select_by_index_out_of_range() {
    let theme = Theme::default();
    let mut manager = WorkspaceManager::new(theme, 5);
    let p1 = test_patient(None);
    
    manager.open_patient(p1).unwrap();
    
    let result = manager.select_by_index(5);
    assert!(matches!(result, Err(WorkspaceError::IndexOutOfRange)));
}

#[test]
fn test_workspace_manager_open_at_limit() {
    let theme = Theme::default();
    let mut manager = WorkspaceManager::new(theme, 2);
    let p1 = test_patient(None);
    let p2 = test_patient(None);
    let p3 = test_patient(None);
    
    manager.open_patient(p1).unwrap();
    manager.open_patient(p2).unwrap();
    
    let result = manager.open_patient(p3);
    assert!(matches!(result, Err(WorkspaceError::AlreadyAtLimit)));
}

#[test]
fn test_subtab_kind_sequence() {
    // Test that subtab kinds cycle in expected order
    // Clinical -> Billing -> Appointments -> Clinical
    let theme = Theme::default();
    let mut manager = WorkspaceManager::new(theme, 5);
    let patient = test_patient(None);
    
    manager.open_patient(patient).unwrap();
    let workspace = manager.active_mut().unwrap();
    
    assert_eq!(workspace.active_subtab, SubtabKind::Clinical);
    
    // Helper function to get next subtab in sequence
    fn next_subtab(current: SubtabKind) -> SubtabKind {
        match current {
            SubtabKind::Summary | SubtabKind::Demographics => SubtabKind::Clinical,
            SubtabKind::Clinical => SubtabKind::Billing,
            SubtabKind::Billing => SubtabKind::Appointments,
            SubtabKind::Appointments => SubtabKind::Clinical,
            #[cfg(feature = "pathology")]
            SubtabKind::Pathology => SubtabKind::Clinical,
            #[cfg(feature = "prescription")]
            SubtabKind::Prescription => SubtabKind::Clinical,
            #[cfg(feature = "referral")]
            SubtabKind::Referral => SubtabKind::Clinical,
            #[cfg(feature = "immunisation")]
            SubtabKind::Immunisation => SubtabKind::Clinical,
        }
    }
    
    let next = next_subtab(workspace.active_subtab);
    assert_eq!(next, SubtabKind::Billing);
}

#[test]
fn test_workspace_is_loaded_tracking() {
    let theme = Theme::default();
    let mut manager = WorkspaceManager::new(theme, 5);
    let patient = test_patient(None);
    
    manager.open_patient(patient).unwrap();
    
    assert!(!manager.is_subtab_loaded(SubtabKind::Clinical));
    
    manager.mark_subtab_loaded(SubtabKind::Clinical);
    assert!(manager.is_subtab_loaded(SubtabKind::Clinical));
    
    assert!(!manager.is_subtab_loaded(SubtabKind::Billing));
}

#[test]
fn test_subtab_cycle_next_wraps_clinical_to_billing() {
    let mut workspace_manager = WorkspaceManager::new(Theme::default(), 5);
    let patient = test_patient(None);
    workspace_manager.open_patient(patient).unwrap();
    
    let workspace = workspace_manager.active_mut().unwrap();
    assert_eq!(workspace.active_subtab, SubtabKind::Clinical);
    
    workspace.active_subtab = SubtabKind::Billing;
    assert_eq!(workspace.active_subtab, SubtabKind::Billing);
}

#[test]
fn test_subtab_cycle_prev_wraps_clinical_to_appointments() {
    let mut workspace_manager = WorkspaceManager::new(Theme::default(), 5);
    let patient = test_patient(None);
    workspace_manager.open_patient(patient).unwrap();
    
    let workspace = workspace_manager.active_mut().unwrap();
    assert_eq!(workspace.active_subtab, SubtabKind::Clinical);
    
    workspace.active_subtab = SubtabKind::Appointments;
    assert_eq!(workspace.active_subtab, SubtabKind::Appointments);
}

#[test]
fn test_close_active_removes_workspace() {
    let theme = Theme::default();
    let mut manager = WorkspaceManager::new(theme, 5);
    let p1 = test_patient(None);
    let p2 = test_patient(None);
    
    manager.open_patient(p1).unwrap();
    manager.open_patient(p2).unwrap();
    assert_eq!(manager.workspaces.len(), 2);
    assert_eq!(manager.active_index, Some(1));
    
    manager.close_active().unwrap();
    
    assert_eq!(manager.workspaces.len(), 1);
    assert_eq!(manager.active_index, Some(0));
}

#[test]
fn test_next_prev_tab_cycle_through_patients() {
    let theme = Theme::default();
    let mut manager = WorkspaceManager::new(theme, 5);
    let p1 = test_patient(None);
    let p2 = test_patient(None);
    let p3 = test_patient(None);
    
    manager.open_patient(p1).unwrap();
    manager.open_patient(p2).unwrap();
    manager.open_patient(p3).unwrap();
    
    assert_eq!(manager.active_index, Some(2));
    
    manager.cycle_next();
    assert_eq!(manager.active_index, Some(0));
    
    manager.cycle_prev();
    assert_eq!(manager.active_index, Some(2));
}

#[test]
fn test_select_by_index_valid() {
    let theme = Theme::default();
    let mut manager = WorkspaceManager::new(theme, 5);
    let p1 = test_patient(None);
    let p2 = test_patient(None);
    
    manager.open_patient(p1).unwrap();
    manager.open_patient(p2).unwrap();
    assert_eq!(manager.active_index, Some(1));
    
    manager.select_by_index(0).unwrap();
    assert_eq!(manager.active_index, Some(0));
}

#[test]
fn test_open_patient_idempotent_returns_existing_index() {
    let theme = Theme::default();
    let mut manager = WorkspaceManager::new(theme, 5);
    let patient_id = Uuid::new_v4();
    let p1 = test_patient(Some(patient_id));
    let p2 = test_patient(None);
    
    let idx1 = manager.open_patient(p1.clone()).unwrap();
    manager.open_patient(p2).unwrap();
    
    let idx2 = manager.open_patient(p1).unwrap();
    
    assert_eq!(idx1, idx2);
    assert_eq!(manager.workspaces.len(), 2);
}

#[test]
fn test_open_at_max_limit_returns_error() {
    let theme = Theme::default();
    let mut manager = WorkspaceManager::new(theme, 1);
    let p1 = test_patient(None);
    let p2 = test_patient(None);
    
    manager.open_patient(p1).unwrap();
    
    let result = manager.open_patient(p2);
    assert!(matches!(result, Err(WorkspaceError::AlreadyAtLimit)));
    assert_eq!(manager.workspaces.len(), 1);
}

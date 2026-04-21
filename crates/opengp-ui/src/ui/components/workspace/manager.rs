use uuid::Uuid;
use crossterm::event::{MouseEvent, MouseEventKind};
use ratatui::layout::{Position, Rect};
use crate::ui::view_models::PatientListItem;
use crate::ui::theme::Theme;
use crate::ui::components::SubtabKind;
use crate::ui::input::{HoverState, DoubleClickDetector};
use super::workspace::{PatientWorkspace, WorkspaceError};

pub struct WorkspaceManager {
    pub workspaces: Vec<PatientWorkspace>,
    pub active_index: Option<usize>,
    pub max_open: usize,
    pub colour_counter: usize,
    pub theme: Theme,
    /// Tracks which patient tab is currently hovered
    pub hovered_tab: HoverState<usize>,
    /// Detects double-clicks on patient tabs
    pub double_click_detector: DoubleClickDetector,
}

impl WorkspaceManager {
    pub fn new(theme: Theme, max_open: usize) -> Self {
        Self {
            workspaces: Vec::new(),
            active_index: None,
            max_open,
            colour_counter: 0,
            theme,
            hovered_tab: HoverState::new(),
            double_click_detector: DoubleClickDetector::default(),
        }
    }

    pub fn open_patient(
        &mut self,
        patient: PatientListItem,
    ) -> Result<usize, WorkspaceError> {
        if let Some(idx) = self.find_patient(patient.id) {
            self.active_index = Some(idx);
            return Ok(idx);
        }

        if self.workspaces.len() >= self.max_open {
            return Err(WorkspaceError::AlreadyAtLimit);
        }

        let colour = self.theme.colors.patient_colour(self.colour_counter);
        self.colour_counter += 1;

        let workspace = PatientWorkspace::new(patient, colour);
        self.workspaces.push(workspace);
        let idx = self.workspaces.len() - 1;
        self.active_index = Some(idx);
        Ok(idx)
    }

    pub fn close_active(&mut self) -> Result<(), WorkspaceError> {
        if let Some(idx) = self.active_index {
            self.workspaces.remove(idx);
            if !self.workspaces.is_empty() {
                self.active_index = Some(idx.min(self.workspaces.len() - 1));
            } else {
                self.active_index = None;
            }
            Ok(())
        } else {
            Ok(())
        }
    }

    pub fn active(&self) -> Option<&PatientWorkspace> {
        self.active_index.and_then(|idx| self.workspaces.get(idx))
    }

    pub fn active_mut(&mut self) -> Option<&mut PatientWorkspace> {
        let idx = self.active_index?;
        self.workspaces.get_mut(idx)
    }

    pub fn is_at_limit(&self) -> bool {
        self.workspaces.len() >= self.max_open
    }

    pub fn find_patient(&self, patient_id: Uuid) -> Option<usize> {
        self.workspaces.iter().position(|w| w.patient_id == patient_id)
    }

    pub fn cycle_next(&mut self) {
        if self.workspaces.is_empty() {
            self.active_index = None;
        } else if let Some(idx) = self.active_index {
            self.active_index = Some((idx + 1) % self.workspaces.len());
        } else {
            self.active_index = Some(0);
        }
    }

    pub fn cycle_prev(&mut self) {
        if self.workspaces.is_empty() {
            self.active_index = None;
        } else if let Some(idx) = self.active_index {
            self.active_index = Some(if idx == 0 {
                self.workspaces.len() - 1
            } else {
                idx - 1
            });
        } else {
            self.active_index = Some(0);
        }
    }

    pub fn select_by_index(&mut self, index: usize) -> Result<(), WorkspaceError> {
        if index >= self.workspaces.len() {
            return Err(WorkspaceError::IndexOutOfRange);
        }
        self.active_index = Some(index);
        Ok(())
    }

    pub fn mark_subtab_loaded(&mut self, subtab: SubtabKind) {
        if let Some(workspace) = self.active_mut() {
            workspace.mark_loaded(subtab);
        }
    }

    pub fn is_subtab_loaded(&self, subtab: SubtabKind) -> bool {
        self.active().map(|w| w.is_loaded(subtab)).unwrap_or(false)
    }

    pub fn handle_patient_tab_mouse(&mut self, mouse: MouseEvent, tab_area: Rect) -> Option<usize> {
        if !tab_area.contains(Position::new(mouse.column, mouse.row)) {
            self.hovered_tab.clear_hover();
            return None;
        }

        match mouse.kind {
            MouseEventKind::Moved => {
                if !self.workspaces.is_empty() {
                    let tab_width = (tab_area.width as usize / self.workspaces.len()).max(1);
                    let hovered_idx =
                        (mouse.column.saturating_sub(tab_area.x)) as usize / tab_width;
                    if hovered_idx < self.workspaces.len() {
                        self.hovered_tab.set_hovered(hovered_idx, (mouse.column, mouse.row));
                    } else {
                        self.hovered_tab.clear_hover();
                    }
                }
                None
            }
            MouseEventKind::Down(crossterm::event::MouseButton::Left) => {
                if !self.workspaces.is_empty() {
                    let tab_width = (tab_area.width as usize / self.workspaces.len()).max(1);
                    let clicked_idx = (mouse.column.saturating_sub(tab_area.x)) as usize / tab_width;
                    if clicked_idx < self.workspaces.len() {
                        if self.double_click_detector.check_double_click(&mouse, &crate::ui::input::SystemClock) {
                            self.active_index = Some(clicked_idx);
                            return Some(clicked_idx);
                        }
                    }
                }
                None
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use opengp_domain::domain::patient::Gender;

    fn create_test_patient(id: Option<Uuid>) -> PatientListItem {
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

    fn create_manager(max: usize) -> WorkspaceManager {
        let theme = Theme::default();
        WorkspaceManager::new(theme, max)
    }

    #[test]
    fn open_patient_idempotent() {
        let mut manager = create_manager(5);
        let patient_id = Uuid::new_v4();
        let patient = create_test_patient(Some(patient_id));

        let idx1 = manager.open_patient(patient.clone()).unwrap();
        let idx2 = manager.open_patient(patient.clone()).unwrap();

        assert_eq!(idx1, idx2);
        assert_eq!(manager.workspaces.len(), 1);
    }

    #[test]
    fn open_patient_at_max_returns_error() {
        let mut manager = create_manager(2);
        let p1 = create_test_patient(None);
        let p2 = create_test_patient(None);
        let p3 = create_test_patient(None);

        manager.open_patient(p1).unwrap();
        manager.open_patient(p2).unwrap();
        
        let result = manager.open_patient(p3);
        assert!(matches!(result, Err(WorkspaceError::AlreadyAtLimit)));
        assert_eq!(manager.workspaces.len(), 2);
    }

    #[test]
    fn find_patient() {
        let mut manager = create_manager(5);
        let patient_id = Uuid::new_v4();
        let patient = create_test_patient(Some(patient_id));

        manager.open_patient(patient).unwrap();
        assert_eq!(manager.find_patient(patient_id), Some(0));
        assert_eq!(manager.find_patient(Uuid::new_v4()), None);
    }

    #[test]
    fn close_active_removes_workspace() {
        let mut manager = create_manager(5);
        let p1 = create_test_patient(None);
        let p2 = create_test_patient(None);

        manager.open_patient(p1).unwrap();
        manager.open_patient(p2).unwrap();
        assert_eq!(manager.workspaces.len(), 2);

        manager.close_active().unwrap();
        assert_eq!(manager.workspaces.len(), 1);
    }

    #[test]
    fn close_active_updates_active_index() {
        let mut manager = create_manager(5);
        let p1 = create_test_patient(None);
        let p2 = create_test_patient(None);

        manager.open_patient(p1).unwrap();
        manager.open_patient(p2).unwrap();
        assert_eq!(manager.active_index, Some(1));

        manager.close_active().unwrap();
        assert_eq!(manager.active_index, Some(0));
    }

    #[test]
    fn cycle_next() {
        let mut manager = create_manager(5);
        let p1 = create_test_patient(None);
        let p2 = create_test_patient(None);
        let p3 = create_test_patient(None);

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
    fn cycle_prev() {
        let mut manager = create_manager(5);
        let p1 = create_test_patient(None);
        let p2 = create_test_patient(None);
        let p3 = create_test_patient(None);

        manager.open_patient(p1).unwrap();
        manager.open_patient(p2).unwrap();
        manager.open_patient(p3).unwrap();
        manager.active_index = Some(0);

        manager.cycle_prev();
        assert_eq!(manager.active_index, Some(2));
        manager.cycle_prev();
        assert_eq!(manager.active_index, Some(1));
        manager.cycle_prev();
        assert_eq!(manager.active_index, Some(0));
    }

    #[test]
    fn select_by_index_valid() {
        let mut manager = create_manager(5);
        let p1 = create_test_patient(None);
        let p2 = create_test_patient(None);

        manager.open_patient(p1).unwrap();
        manager.open_patient(p2).unwrap();
        manager.active_index = Some(0);

        manager.select_by_index(1).unwrap();
        assert_eq!(manager.active_index, Some(1));
    }

    #[test]
    fn select_by_index_out_of_range() {
        let mut manager = create_manager(5);
        let p1 = create_test_patient(None);

        manager.open_patient(p1).unwrap();

        let result = manager.select_by_index(5);
        assert!(matches!(result, Err(WorkspaceError::IndexOutOfRange)));
    }

    #[test]
    fn is_at_limit() {
        let mut manager = create_manager(2);
        let p1 = create_test_patient(None);
        let p2 = create_test_patient(None);

        assert!(!manager.is_at_limit());
        manager.open_patient(p1).unwrap();
        assert!(!manager.is_at_limit());
        manager.open_patient(p2).unwrap();
        assert!(manager.is_at_limit());
    }

    #[test]
    fn mark_subtab_loaded() {
        let mut manager = create_manager(5);
        let patient = create_test_patient(None);

        manager.open_patient(patient).unwrap();
        assert!(!manager.is_subtab_loaded(SubtabKind::Clinical));

        manager.mark_subtab_loaded(SubtabKind::Clinical);
        assert!(manager.is_subtab_loaded(SubtabKind::Clinical));
    }
}

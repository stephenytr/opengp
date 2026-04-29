use std::rc::Rc;

use crate::ui::theme::Theme;
use crate::ui::widgets::{UnifiedColumnDef, UnifiedList, UnifiedListAction, UnifiedListConfig};
use crossterm::event::{KeyEvent, MouseEvent};
use opengp_domain::domain::clinical::Allergy;
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
use rat_focus::{FocusFlag, HasFocus, FocusBuilder};

#[derive(Clone)]
pub struct AllergyList {
    pub allergies: Vec<Allergy>,
    pub selected_index: usize,
    pub show_inactive: bool,
    pub scroll_offset: usize,
    pub loading: bool,
    pub theme: Theme,
    pub hovered_index: Option<usize>,
    pub focus: FocusFlag,
}

#[derive(Debug, Clone)]
pub enum AllergyListAction {
    Select(usize),
    Open(Allergy),
    New,
    ToggleInactive,
    Delete(Allergy),
    ContextMenu { index: usize, x: u16, y: u16 },
}

impl AllergyList {
    pub fn new(theme: Theme) -> Self {
        Self {
            allergies: Vec::new(),
            selected_index: 0,
            show_inactive: true,
            scroll_offset: 0,
            loading: false,
            theme,
            hovered_index: None,
            focus: FocusFlag::default(),
        }
    }

    pub fn next(&mut self) {
        self.selected_index = (self.selected_index + 1).min(self.allergies.len().saturating_sub(1));
    }

    pub fn prev(&mut self) {
        self.selected_index = self.selected_index.saturating_sub(1);
    }

    pub fn move_first(&mut self) {
        self.selected_index = 0;
    }

    pub fn adjust_scroll(&mut self, visible_rows: usize) {
        if visible_rows == 0 {
            return;
        }
        if self.selected_index < self.scroll_offset {
            self.scroll_offset = self.selected_index;
        } else if self.selected_index >= self.scroll_offset + visible_rows {
            self.scroll_offset = self.selected_index.saturating_sub(visible_rows) + 1;
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Option<AllergyListAction> {
        let mut list = self.as_list()?;
        let action = list.handle_key(key);
        self.sync_from(&list);
        action.map(|a| match a {
            UnifiedListAction::Select(i) => AllergyListAction::Select(i),
            UnifiedListAction::Open(a) => AllergyListAction::Open(a),
            UnifiedListAction::New => AllergyListAction::New,
            UnifiedListAction::Delete(a) => AllergyListAction::Delete(a),
            UnifiedListAction::ToggleInactive => {
                self.show_inactive = !self.show_inactive;
                AllergyListAction::ToggleInactive
            }
            UnifiedListAction::Edit(_) | UnifiedListAction::ContextMenu { .. } => {
                AllergyListAction::Select(self.selected_index)
            }
        })
    }

    pub fn handle_mouse(&mut self, mouse: MouseEvent, area: Rect) -> Option<AllergyListAction> {
        let mut list = self.as_list()?;
        let action = list.handle_mouse(mouse, area);
        self.sync_from(&list);
        action.map(|a| match a {
            UnifiedListAction::Select(i) => AllergyListAction::Select(i),
            UnifiedListAction::Open(a) => AllergyListAction::Open(a),
            UnifiedListAction::New => AllergyListAction::New,
            UnifiedListAction::ContextMenu { index, x, y } => AllergyListAction::ContextMenu { index, x, y },
            UnifiedListAction::Edit(_) | UnifiedListAction::Delete(_) | UnifiedListAction::ToggleInactive => {
                AllergyListAction::Select(self.selected_index)
            }
        })
    }

    fn as_list(&self) -> Option<UnifiedList<Allergy>> {
        let mut list = UnifiedList::new(
            self.allergies.clone(),
            columns(),
            self.theme.clone(),
            UnifiedListConfig::new("Allergies", 2, "No allergies found. Press n to add a new allergy."),
        );
        list.selected_index = self.selected_index;
        list.scroll_offset = self.scroll_offset;
        list.loading = self.loading;
        list.hovered_index = self.hovered_index;
        Some(list)
    }

    fn sync_from(&mut self, list: &UnifiedList<Allergy>) {
        self.selected_index = list.selected_index;
        self.scroll_offset = list.scroll_offset;
        self.loading = list.loading;
        self.hovered_index = list.hovered_index;
    }
}

impl Widget for AllergyList {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut list = match self.as_list() {
            Some(l) => l,
            None => return,
        };
        list.loading = self.loading;
        list.render(area, buf);
    }
}

fn col(
    title: &'static str,
    width: u16,
    render: impl Fn(&Allergy) -> String + 'static,
) -> UnifiedColumnDef<Allergy> {
    UnifiedColumnDef::new(title, width, render)
}

fn columns() -> Vec<UnifiedColumnDef<Allergy>> {
    vec![
        col("Allergen", 20, |a| a.allergen.clone()),
        col("Type", 15, |a| a.allergy_type.to_string()),
        col("Severity", 10, |a| a.severity.to_string()),
        col("Reaction", 30, |a| {
            a.reaction.clone().unwrap_or_else(|| "-".into())
        }),
        col("Status", 10, |a| {
            ["Inactive", "Active"][a.is_active as usize].into()
        }),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::theme::Theme;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use opengp_domain::domain::clinical::AllergyType;
    use uuid::Uuid;

    fn make_test_allergy() -> Allergy {
        Allergy {
            id: Uuid::new_v4(),
            patient_id: Uuid::new_v4(),
            allergen: "Peanuts".to_string(),
            allergy_type: AllergyType::Food,
            severity: opengp_domain::domain::clinical::Severity::Severe,
            reaction: Some("Anaphylaxis".to_string()),
            onset_date: None,
            notes: None,
            is_active: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            created_by: Uuid::new_v4(),
            updated_by: None,
        }
    }

    #[test]
    fn test_allergy_list_up_down_key_navigation() {
        let theme = Theme::dark();
        let mut list = AllergyList::new(theme);

        let allergies = vec![
            make_test_allergy(),
            make_test_allergy(),
            make_test_allergy(),
        ];
        list.allergies = allergies;

        let key_down = KeyEvent::new(KeyCode::Down, KeyModifiers::NONE);
        let key_up = KeyEvent::new(KeyCode::Up, KeyModifiers::NONE);

        assert_eq!(list.selected_index, 0);

        let action = list.handle_key(key_down);
        assert!(matches!(action, Some(AllergyListAction::Select(1))));
        assert_eq!(list.selected_index, 1);

        let action = list.handle_key(key_down);
        assert!(matches!(action, Some(AllergyListAction::Select(2))));
        assert_eq!(list.selected_index, 2);

        let action = list.handle_key(key_up);
        assert!(matches!(action, Some(AllergyListAction::Select(1))));
        assert_eq!(list.selected_index, 1);

        let action = list.handle_key(key_up);
        assert!(matches!(action, Some(AllergyListAction::Select(0))));
        assert_eq!(list.selected_index, 0);
    }

    #[test]
    fn test_allergy_list_j_k_key_navigation() {
        let theme = Theme::dark();
        let mut list = AllergyList::new(theme);

        let allergies = vec![make_test_allergy(), make_test_allergy()];
        list.allergies = allergies;

        let key_j = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
        let key_k = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE);

        assert_eq!(list.selected_index, 0);

        let action = list.handle_key(key_j);
        assert!(matches!(action, Some(AllergyListAction::Select(1))));
        assert_eq!(list.selected_index, 1);

        let action = list.handle_key(key_k);
        assert!(matches!(action, Some(AllergyListAction::Select(0))));
        assert_eq!(list.selected_index, 0);
    }
}

impl HasFocus for AllergyList {
    fn build(&self, builder: &mut FocusBuilder) {
        builder.leaf_widget(self);
    }

    fn focus(&self) -> FocusFlag {
        self.focus.clone()
    }

    fn area(&self) -> Rect {
        Rect::default()
    }
}

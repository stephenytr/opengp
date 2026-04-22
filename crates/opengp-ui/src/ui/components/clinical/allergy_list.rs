use crate::ui::theme::Theme;
use crate::ui::widgets::{ClinicalTableList, ColumnDef, ListAction};
use crossterm::event::{KeyEvent, MouseEvent};
use opengp_domain::domain::clinical::Allergy;
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

#[derive(Clone)]
pub struct AllergyList {
    pub allergies: Vec<Allergy>,
    pub selected_index: usize,
    pub show_inactive: bool,
    pub scroll_offset: usize,
    pub loading: bool,
    theme: Theme,
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
        }
    }

    pub fn next(&mut self) {
        let mut table = self.table();
        table.move_down();
        self.selected_index = table.selected_index;
        self.scroll_offset = table.scroll_offset;
    }

    pub fn prev(&mut self) {
        let mut table = self.table();
        table.move_up();
        self.selected_index = table.selected_index;
        self.scroll_offset = table.scroll_offset;
    }

    pub fn move_first(&mut self) {
        let mut table = self.table();
        table.move_first();
        self.selected_index = table.selected_index;
        self.scroll_offset = table.scroll_offset;
    }

    pub fn adjust_scroll(&mut self, visible_rows: usize) {
        let mut table = self.table();
        table.adjust_scroll(visible_rows);
        self.selected_index = table.selected_index;
        self.scroll_offset = table.scroll_offset;
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Option<AllergyListAction> {
        let mut table = self.table();
        let out = match table.handle_key(key) {
            Some(ListAction::Select(i)) => Some(AllergyListAction::Select(i)),
            Some(ListAction::Open(a)) => Some(AllergyListAction::Open(a)),
            Some(ListAction::New) => Some(AllergyListAction::New),
            Some(ListAction::Delete(a)) => Some(AllergyListAction::Delete(a)),
            Some(ListAction::ToggleInactive) => {
                self.show_inactive = !self.show_inactive;
                Some(AllergyListAction::ToggleInactive)
            }
            _ => None,
        };
        self.selected_index = table.selected_index;
        self.scroll_offset = table.scroll_offset;
        out
    }

    pub fn handle_mouse(&mut self, mouse: MouseEvent, area: Rect) -> Option<AllergyListAction> {
        let mut table = self.table();
        let action = table.handle_mouse(mouse, area).and_then(|a| match a {
            ListAction::Select(i) => Some(AllergyListAction::Select(i)),
            ListAction::Open(allergy) => Some(AllergyListAction::Open(allergy)),
            ListAction::New => Some(AllergyListAction::New),
            ListAction::ContextMenu { index, x, y } => Some(AllergyListAction::ContextMenu { index, x, y }),
            ListAction::Edit(_) | ListAction::Delete(_) | ListAction::ToggleInactive => None,
        });
        self.selected_index = table.selected_index;
        self.scroll_offset = table.scroll_offset;
        action
    }

    fn table(&self) -> ClinicalTableList<Allergy> {
        let mut table = ClinicalTableList::new(
            self.allergies.clone(),
            columns(),
            self.theme.clone(),
            "Allergies",
            None,
        );
        table.selected_index = self.selected_index;
        table.scroll_offset = self.scroll_offset;
        table.loading = self.loading;
        table.empty_message = "No allergies found. Press n to add a new allergy.".into();
        table
    }
}

impl Widget for AllergyList {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.table().render(area, buf);
    }
}

fn col(
    title: &'static str,
    width: u16,
    render: impl Fn(&Allergy) -> String + 'static,
) -> ColumnDef<Allergy> {
    ColumnDef {
        title,
        width,
        render: Box::new(render),
    }
}

fn columns() -> Vec<ColumnDef<Allergy>> {
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

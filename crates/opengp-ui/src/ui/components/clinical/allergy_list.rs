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
        let out = match table.handle_mouse(mouse, area) {
            Some(ListAction::Select(i)) => Some(AllergyListAction::Select(i)),
            _ => None,
        };
        self.selected_index = table.selected_index;
        self.scroll_offset = table.scroll_offset;
        out
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

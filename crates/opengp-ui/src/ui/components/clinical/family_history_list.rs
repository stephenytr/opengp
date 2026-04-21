use crate::ui::{
    theme::Theme,
    widgets::{ClinicalTableList, ColumnDef, ListAction},
};
use crossterm::event::{KeyEvent, MouseEvent};
use opengp_domain::domain::clinical::FamilyHistory;
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
const EMPTY_MESSAGE: &str = "No family history found. Press n to add an entry.";
#[derive(Clone)]
pub struct FamilyHistoryList {
    pub entries: Vec<FamilyHistory>,
    pub selected_index: usize,
    pub scroll_offset: usize,
    pub loading: bool,
    theme: Theme,
}
#[derive(Debug, Clone)]
pub enum FamilyHistoryListAction {
    Select(usize),
    Open(FamilyHistory),
    New,
    Delete(FamilyHistory),
}
impl FamilyHistoryList {
    pub fn new(theme: Theme) -> Self {
        Self::with_entries(Vec::new(), theme)
    }
    pub fn with_entries(entries: Vec<FamilyHistory>, theme: Theme) -> Self {
        Self {
            entries,
            selected_index: 0,
            scroll_offset: 0,
            loading: false,
            theme,
        }
    }
    pub fn next(&mut self) {
        if self.selected_index < self.entries.len().saturating_sub(1) {
            self.selected_index += 1;
        }
    }
    pub fn prev(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
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
    pub fn handle_key(&mut self, key: KeyEvent) -> Option<FamilyHistoryListAction> {
        let mut table = self.as_table();
        let action = table.handle_key(key).and_then(map_action);
        self.selected_index = table.selected_index;
        self.scroll_offset = table.scroll_offset;
        action
    }
    pub fn handle_mouse(
        &mut self,
        mouse: MouseEvent,
        area: Rect,
    ) -> Option<FamilyHistoryListAction> {
        let mut table = self.as_table();
        let action = table.handle_mouse(mouse, area).and_then(map_action);
        self.selected_index = table.selected_index;
        self.scroll_offset = table.scroll_offset;
        action
    }
    fn as_table(&self) -> ClinicalTableList<FamilyHistory> {
        let mut table = ClinicalTableList::new(
            self.entries.clone(),
            columns(),
            self.theme.clone(),
            "Family History",
            None,
        );
        table.selected_index = self.selected_index;
        table.scroll_offset = self.scroll_offset;
        table.loading = self.loading;
        table.empty_message = EMPTY_MESSAGE.to_string();
        table
    }
}
impl Widget for FamilyHistoryList {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.as_table().render(area, buf);
    }
}
fn map_action(action: ListAction<FamilyHistory>) -> Option<FamilyHistoryListAction> {
    match action {
        ListAction::Select(index) => Some(FamilyHistoryListAction::Select(index)),
        ListAction::Open(entry) => Some(FamilyHistoryListAction::Open(entry)),
        ListAction::New => Some(FamilyHistoryListAction::New),
        ListAction::Delete(entry) => Some(FamilyHistoryListAction::Delete(entry)),
        ListAction::Edit(_) | ListAction::ToggleInactive | ListAction::ContextMenu { .. } => None,
    }
}
fn columns() -> Vec<ColumnDef<FamilyHistory>> {
    vec![
        ColumnDef {
            title: "Condition",
            width: 25,
            render: Box::new(|e| e.condition.clone()),
        },
        ColumnDef {
            title: "Relationship",
            width: 20,
            render: Box::new(|e| e.relative_relationship.clone()),
        },
        ColumnDef {
            title: "Age",
            width: 10,
            render: Box::new(|e| {
                e.age_at_diagnosis
                    .map(|age| format!("{} years", age))
                    .unwrap_or_else(|| "-".to_string())
            }),
        },
        ColumnDef {
            title: "Notes",
            width: 30,
            render: Box::new(|e| {
                e.notes
                    .as_ref()
                    .map(|n| {
                        if n.len() > 28 {
                            format!("{}...", &n[..28])
                        } else {
                            n.clone()
                        }
                    })
                    .unwrap_or_else(|| "-".to_string())
            }),
        },
    ]
}

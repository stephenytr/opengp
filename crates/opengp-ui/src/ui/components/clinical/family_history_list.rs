use std::rc::Rc;

use crate::ui::theme::Theme;
use crate::ui::widgets::{UnifiedColumnDef, UnifiedList, UnifiedListAction, UnifiedListConfig};
use crossterm::event::{KeyEvent, MouseEvent};
use opengp_domain::domain::clinical::FamilyHistory;
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
use rat_focus::{FocusFlag, HasFocus, FocusBuilder};

const EMPTY_MESSAGE: &str = "No family history found. Press n to add an entry.";

#[derive(Clone)]
pub struct FamilyHistoryList {
    pub entries: Vec<FamilyHistory>,
    pub selected_index: usize,
    pub scroll_offset: usize,
    pub loading: bool,
    pub theme: Theme,
    pub hovered_index: Option<usize>,
    pub focus: FocusFlag,
}

#[derive(Debug, Clone)]
pub enum FamilyHistoryListAction {
    Select(usize),
    Open(FamilyHistory),
    New,
    Delete(FamilyHistory),
    ContextMenu { index: usize, x: u16, y: u16 },
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
            hovered_index: None,
            focus: FocusFlag::default(),
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
        let mut list = self.as_list();
        let action = list.handle_key(key).and_then(map_action);
        self.sync_from(&list);
        action
    }

    pub fn handle_mouse(&mut self, mouse: MouseEvent, area: Rect) -> Option<FamilyHistoryListAction> {
        let mut list = self.as_list();
        let action = list.handle_mouse(mouse, area).and_then(map_action);
        self.sync_from(&list);
        action
    }

    fn as_list(&self) -> UnifiedList<FamilyHistory> {
        let mut list = UnifiedList::new(
            self.entries.clone(),
            columns(),
            self.theme.clone(),
            UnifiedListConfig::new("Family History", 2, EMPTY_MESSAGE),
        );
        list.selected_index = self.selected_index;
        list.scroll_offset = self.scroll_offset;
        list.loading = self.loading;
        list.hovered_index = self.hovered_index;
        list
    }

    fn sync_from(&mut self, list: &UnifiedList<FamilyHistory>) {
        self.selected_index = list.selected_index;
        self.scroll_offset = list.scroll_offset;
        self.loading = list.loading;
        self.hovered_index = list.hovered_index;
    }
}

impl Widget for FamilyHistoryList {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.as_list().render(area, buf);
    }
}

fn map_action(action: UnifiedListAction<FamilyHistory>) -> Option<FamilyHistoryListAction> {
    match action {
        UnifiedListAction::Select(index) => Some(FamilyHistoryListAction::Select(index)),
        UnifiedListAction::Open(entry) => Some(FamilyHistoryListAction::Open(entry)),
        UnifiedListAction::New => Some(FamilyHistoryListAction::New),
        UnifiedListAction::Delete(entry) => Some(FamilyHistoryListAction::Delete(entry)),
        UnifiedListAction::ContextMenu { index, x, y } => Some(FamilyHistoryListAction::ContextMenu { index, x, y }),
        UnifiedListAction::Edit(_) | UnifiedListAction::ToggleInactive => None,
    }
}

fn columns() -> Vec<UnifiedColumnDef<FamilyHistory>> {
    vec![
        UnifiedColumnDef::<FamilyHistory>::new("Condition", 25, |e| e.condition.clone()),
        UnifiedColumnDef::<FamilyHistory>::new("Relationship", 20, |e| e.relative_relationship.clone()),
        UnifiedColumnDef::<FamilyHistory>::new("Age", 10, |e| {
            e.age_at_diagnosis
                .map(|age| format!("{} years", age))
                .unwrap_or_else(|| "-".to_string())
        }),
        UnifiedColumnDef::<FamilyHistory>::new("Notes", 30, |e| {
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
    ]
}

impl HasFocus for FamilyHistoryList {
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

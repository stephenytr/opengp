use std::rc::Rc;

use crate::ui::theme::Theme;
use crate::ui::widgets::{UnifiedColumnDef, UnifiedList, UnifiedListAction, UnifiedListConfig};
use crossterm::event::{KeyEvent, MouseEvent};
use opengp_domain::domain::clinical::{ConditionStatus, MedicalHistory, Severity};
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
use rat_focus::{FocusFlag, HasFocus, FocusBuilder};

pub type MedicalHistoryListAction = UnifiedListAction<MedicalHistory>;

#[derive(Clone)]
pub struct MedicalHistoryList {
    pub conditions: Vec<MedicalHistory>,
    pub selected_index: usize,
    pub scroll_offset: usize,
    pub loading: bool,
    pub theme: Theme,
    pub hovered_index: Option<usize>,
    pub focus: FocusFlag,
}

impl MedicalHistoryList {
    pub fn new(theme: Theme) -> Self {
        Self {
            conditions: Vec::new(),
            selected_index: 0,
            scroll_offset: 0,
            loading: false,
            theme,
            hovered_index: None,
            focus: FocusFlag::default(),
        }
    }

    pub fn adjust_scroll(&mut self, visible_rows: usize) {
        let mut list = self.as_list();
        list.adjust_scroll(visible_rows);
        self.sync_from(&list);
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Option<MedicalHistoryListAction> {
        let mut list = self.as_list();
        let action = list.handle_key(key);
        self.sync_from(&list);
        action
    }

    pub fn handle_mouse(&mut self, mouse: MouseEvent, area: Rect) -> Option<MedicalHistoryListAction> {
        let mut list = self.as_list();
        let action = list.handle_mouse(mouse, area);
        self.sync_from(&list);
        action
    }

    fn as_list(&self) -> UnifiedList<MedicalHistory> {
        let mut list = UnifiedList::new(
            self.conditions.clone(),
            Self::columns(),
            self.theme.clone(),
            UnifiedListConfig::new("Medical History", 2, "No medical history found. Press n to add a condition."),
        );
        list.selected_index = self.selected_index;
        list.scroll_offset = self.scroll_offset;
        list.loading = self.loading;
        list.hovered_index = self.hovered_index;
        list
    }

    fn sync_from(&mut self, list: &UnifiedList<MedicalHistory>) {
        self.selected_index = list.selected_index;
        self.scroll_offset = list.scroll_offset;
        self.loading = list.loading;
        self.hovered_index = list.hovered_index;
    }

    fn columns() -> Vec<UnifiedColumnDef<MedicalHistory>> {
        vec![
            UnifiedColumnDef::<MedicalHistory>::new("Condition", 25, |c| c.condition.clone()),
            UnifiedColumnDef::<MedicalHistory>::new("DiagDate", 12, |c| {
                c.diagnosis_date
                    .map(|d| d.format("%d/%m/%Y").to_string())
                    .unwrap_or_else(|| "-".to_string())
            }),
            UnifiedColumnDef::<MedicalHistory>::new("Status", 12, |c| {
                match c.status {
                    ConditionStatus::Active => "Active",
                    ConditionStatus::Resolved => "Resolved",
                    ConditionStatus::Chronic => "Chronic",
                    ConditionStatus::Recurring => "Recurring",
                    ConditionStatus::InRemission => "In Remission",
                }
                .to_string()
            }),
            UnifiedColumnDef::<MedicalHistory>::new("Severity", 10, |c| {
                match c.severity {
                    Some(Severity::Mild) => "Mild",
                    Some(Severity::Moderate) => "Moderate",
                    Some(Severity::Severe) => "Severe",
                    None => "-",
                }
                .to_string()
            }),
            UnifiedColumnDef::<MedicalHistory>::new("Notes", 25, |c| {
                c.notes
                    .as_ref()
                    .map(|s| {
                        if s.len() > 23 {
                            format!("{}...", &s[..23])
                        } else {
                            s.clone()
                        }
                    })
                    .unwrap_or_else(|| "-".to_string())
            }),
        ]
    }
}

impl Widget for MedicalHistoryList {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.as_list().render(area, buf);
    }
}

impl HasFocus for MedicalHistoryList {
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

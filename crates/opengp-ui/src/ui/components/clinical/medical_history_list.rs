use crate::ui::theme::Theme;
use crate::ui::widgets::{ClinicalTableList, ColumnDef, ListAction};
use crossterm::event::{KeyEvent, MouseEvent};
use opengp_domain::domain::clinical::{ConditionStatus, MedicalHistory, Severity};
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

pub type MedicalHistoryListAction = ListAction<MedicalHistory>;

#[derive(Clone)]
pub struct MedicalHistoryList {
    pub conditions: Vec<MedicalHistory>,
    pub selected_index: usize,
    pub scroll_offset: usize,
    pub loading: bool,
    pub theme: Theme,
}

impl MedicalHistoryList {
    pub fn new(theme: Theme) -> Self {
        Self {
            conditions: Vec::new(),
            selected_index: 0,
            scroll_offset: 0,
            loading: false,
            theme,
        }
    }

    pub fn adjust_scroll(&mut self, visible_rows: usize) {
        let mut list = self.table_list();
        list.adjust_scroll(visible_rows);
        self.sync_nav(&list);
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Option<MedicalHistoryListAction> {
        let mut list = self.table_list();
        let action = list.handle_key(key);
        self.sync_nav(&list);
        action
    }

    pub fn handle_mouse(
        &mut self,
        mouse: MouseEvent,
        area: Rect,
    ) -> Option<MedicalHistoryListAction> {
        let mut list = self.table_list();
        let action = list.handle_mouse(mouse, area);
        self.sync_nav(&list);
        action
    }

    fn sync_nav(&mut self, list: &ClinicalTableList<MedicalHistory>) {
        self.selected_index = list.selected_index;
        self.scroll_offset = list.scroll_offset;
    }

    fn table_list(&self) -> ClinicalTableList<MedicalHistory> {
        let mut list = ClinicalTableList::new(
            self.conditions.clone(),
            Self::columns(),
            self.theme.clone(),
            "Medical History",
            None,
        );
        list.selected_index = self.selected_index;
        list.scroll_offset = self.scroll_offset;
        list.loading = self.loading;
        list.empty_message = "No medical history found. Press n to add a condition.".to_string();
        list
    }

    fn columns() -> Vec<ColumnDef<MedicalHistory>> {
        vec![
            ColumnDef {
                title: "Condition",
                width: 25,
                render: Box::new(|c| c.condition.clone()),
            },
            ColumnDef {
                title: "DiagDate",
                width: 12,
                render: Box::new(|c| {
                    c.diagnosis_date
                        .map(|d| d.format("%d/%m/%Y").to_string())
                        .unwrap_or_else(|| "-".to_string())
                }),
            },
            ColumnDef {
                title: "Status",
                width: 12,
                render: Box::new(|c| {
                    match c.status {
                        ConditionStatus::Active => "Active",
                        ConditionStatus::Resolved => "Resolved",
                        ConditionStatus::Chronic => "Chronic",
                        ConditionStatus::Recurring => "Recurring",
                        ConditionStatus::InRemission => "In Remission",
                    }
                    .to_string()
                }),
            },
            ColumnDef {
                title: "Severity",
                width: 10,
                render: Box::new(|c| {
                    match c.severity {
                        Some(Severity::Mild) => "Mild",
                        Some(Severity::Moderate) => "Moderate",
                        Some(Severity::Severe) => "Severe",
                        None => "-",
                    }
                    .to_string()
                }),
            },
            ColumnDef {
                title: "Notes",
                width: 25,
                render: Box::new(|c| {
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
            },
        ]
    }
}

impl Widget for MedicalHistoryList {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.table_list().render(area, buf);
    }
}

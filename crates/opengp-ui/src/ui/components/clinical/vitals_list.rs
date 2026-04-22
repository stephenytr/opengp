use crate::ui::theme::Theme;
use crate::ui::widgets::{ClinicalTableList, ColumnDef, ListAction};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, MouseEvent};
use opengp_domain::domain::clinical::VitalSigns;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::Widget;

#[derive(Clone)]
pub struct VitalSignsList {
    pub vitals: Vec<VitalSigns>,
    pub selected_index: usize,
    pub scroll_offset: usize,
    pub loading: bool,
    pub theme: Theme,
}

#[derive(Debug, Clone)]
pub enum VitalSignsListAction {
    Select(usize),
    Open(VitalSigns),
    New,
    NextPage,
    PrevPage,
    ContextMenu { index: usize, x: u16, y: u16 },
}

fn fmt_date(v: &VitalSigns) -> String {
    v.measured_at.format("%d/%m/%Y").to_string()
}
fn fmt_bp(v: &VitalSigns) -> String {
    match (v.systolic_bp, v.diastolic_bp) {
        (Some(sys), Some(dia)) => format!("{}/{}", sys, dia),
        _ => "-".to_string(),
    }
}
fn fmt_hr(v: &VitalSigns) -> String {
    v.heart_rate
        .map_or_else(|| "-".to_string(), |h| h.to_string())
}
fn fmt_rr(v: &VitalSigns) -> String {
    v.respiratory_rate
        .map_or_else(|| "-".to_string(), |r| r.to_string())
}
fn fmt_temp(v: &VitalSigns) -> String {
    v.temperature
        .map_or_else(|| "-".to_string(), |t| format!("{:.1}", t))
}
fn fmt_spo2(v: &VitalSigns) -> String {
    v.oxygen_saturation
        .map_or_else(|| "-".to_string(), |s| s.to_string())
}
fn fmt_bmi(v: &VitalSigns) -> String {
    v.bmi
        .map_or_else(|| "-".to_string(), |b| format!("{:.1}", b))
}

fn columns() -> Vec<ColumnDef<VitalSigns>> {
    [
        ("Date", 12, fmt_date as fn(&VitalSigns) -> String),
        ("BP", 14, fmt_bp),
        ("HR", 8, fmt_hr),
        ("RR", 8, fmt_rr),
        ("Temp", 8, fmt_temp),
        ("SpO2", 8, fmt_spo2),
        ("BMI", 8, fmt_bmi),
    ]
    .into_iter()
    .map(|(title, width, render)| ColumnDef {
        title,
        width,
        render: Box::new(render),
    })
    .collect()
}

impl VitalSignsList {
    pub fn new(theme: Theme) -> Self {
        Self::with_vitals(Vec::new(), theme)
    }

    pub fn with_vitals(mut vitals: Vec<VitalSigns>, theme: Theme) -> Self {
        vitals.sort_by(|a, b| b.measured_at.cmp(&a.measured_at));
        Self {
            vitals,
            selected_index: 0,
            scroll_offset: 0,
            loading: false,
            theme,
        }
    }

    pub fn next(&mut self) {
        self.selected_index = (self.selected_index + 1).min(self.vitals.len().saturating_sub(1));
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

    fn table(&self) -> ClinicalTableList<VitalSigns> {
        let mut table = ClinicalTableList::new(
            self.vitals.clone(),
            columns(),
            self.theme.clone(),
            "Vital Signs",
            Some(Box::new(|a: &VitalSigns, b: &VitalSigns| {
                b.measured_at.cmp(&a.measured_at)
            })),
        );
        table.selected_index = self.selected_index.min(table.items.len().saturating_sub(1));
        table.scroll_offset = self.scroll_offset;
        table.loading = self.loading;
        table
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Option<VitalSignsListAction> {
        if key.kind == KeyEventKind::Press {
            if matches!(key.code, KeyCode::Char('+') | KeyCode::Char('=')) {
                return Some(VitalSignsListAction::NextPage);
            }
            if key.code == KeyCode::Char('-') {
                return Some(VitalSignsListAction::PrevPage);
            }
        }
        let mut table = self.table();
        let action = table.handle_key(key).and_then(|a| match a {
            ListAction::Select(i) => Some(VitalSignsListAction::Select(i)),
            ListAction::Open(v) => Some(VitalSignsListAction::Open(v)),
            ListAction::New => Some(VitalSignsListAction::New),
            ListAction::Edit(_) | ListAction::Delete(_) | ListAction::ToggleInactive | ListAction::ContextMenu { .. } => None,
        });
        self.vitals = table.items;
        self.selected_index = table.selected_index;
        self.scroll_offset = table.scroll_offset;
        action
    }

    pub fn handle_mouse(&mut self, mouse: MouseEvent, area: Rect) -> Option<VitalSignsListAction> {
        let mut table = self.table();
        let action = table.handle_mouse(mouse, area).and_then(|a| match a {
            ListAction::Select(i) => Some(VitalSignsListAction::Select(i)),
            ListAction::ContextMenu { index, x, y } => Some(VitalSignsListAction::ContextMenu { index, x, y }),
            ListAction::Open(_) | ListAction::New | ListAction::Edit(_) | ListAction::Delete(_) | ListAction::ToggleInactive => None,
        });
        self.vitals = table.items;
        self.selected_index = table.selected_index;
        self.scroll_offset = table.scroll_offset;
        action
    }
}

impl Widget for VitalSignsList {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.table().render(area, buf);
    }
}

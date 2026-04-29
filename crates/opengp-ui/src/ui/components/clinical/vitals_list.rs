use std::rc::Rc;

use crate::ui::theme::Theme;
use crate::ui::widgets::{UnifiedColumnDef, UnifiedList, UnifiedListAction, UnifiedListConfig};
use crossterm::event::{Event, KeyEvent, KeyEventKind, MouseEvent};
use rat_event::ct_event;
use opengp_domain::domain::clinical::VitalSigns;
use rat_focus::{FocusBuilder, FocusFlag, HasFocus};
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

#[derive(Clone)]
pub struct VitalSignsList {
    pub vitals: Vec<VitalSigns>,
    pub selected_index: usize,
    pub scroll_offset: usize,
    pub loading: bool,
    pub theme: Theme,
    pub hovered_index: Option<usize>,
    pub focus: FocusFlag,
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

fn columns() -> Vec<UnifiedColumnDef<VitalSigns>> {
    vec![
        UnifiedColumnDef::new("Date", 12, fmt_date),
        UnifiedColumnDef::new("BP", 14, fmt_bp),
        UnifiedColumnDef::new("HR", 8, fmt_hr),
        UnifiedColumnDef::new("RR", 8, fmt_rr),
        UnifiedColumnDef::new("Temp", 8, fmt_temp),
        UnifiedColumnDef::new("SpO2", 8, fmt_spo2),
        UnifiedColumnDef::new("BMI", 8, fmt_bmi),
    ]
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
            hovered_index: None,
            focus: FocusFlag::default(),
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

    fn as_list(&self) -> UnifiedList<VitalSigns> {
        let mut list = UnifiedList::new(
            self.vitals.clone(),
            columns(),
            self.theme.clone(),
            UnifiedListConfig::new("Vital Signs", 2, "No vital signs recorded."),
        );
        list.selected_index = self.selected_index.min(list.items.len().saturating_sub(1));
        list.scroll_offset = self.scroll_offset;
        list.loading = self.loading;
        list.hovered_index = self.hovered_index;
        list
    }

    fn sync_from(&mut self, list: &UnifiedList<VitalSigns>) {
        self.vitals = list.items.clone();
        self.selected_index = list.selected_index;
        self.scroll_offset = list.scroll_offset;
        self.loading = list.loading;
        self.hovered_index = list.hovered_index;
    }

     pub fn handle_key(&mut self, key: KeyEvent) -> Option<VitalSignsListAction> {
         if key.kind == KeyEventKind::Press {
             let event = Event::Key(key);
             if matches!(&event, ct_event!(key press '+') | ct_event!(key press '=')) {
                 return Some(VitalSignsListAction::NextPage);
             }
             if matches!(&event, ct_event!(key press '-')) {
                 return Some(VitalSignsListAction::PrevPage);
             }
         }
         let mut list = self.as_list();
         let action = list.handle_key(key).and_then(|a| match a {
            UnifiedListAction::Select(i) => Some(VitalSignsListAction::Select(i)),
            UnifiedListAction::Open(v) => Some(VitalSignsListAction::Open(v)),
            UnifiedListAction::New => Some(VitalSignsListAction::New),
            UnifiedListAction::Edit(_)
            | UnifiedListAction::Delete(_)
            | UnifiedListAction::ToggleInactive
            | UnifiedListAction::ContextMenu { .. } => None,
        });
        self.sync_from(&list);
        action
    }

    pub fn handle_mouse(&mut self, mouse: MouseEvent, area: Rect) -> Option<VitalSignsListAction> {
        let mut list = self.as_list();
        let action = list.handle_mouse(mouse, area).and_then(|a| match a {
            UnifiedListAction::Select(i) => Some(VitalSignsListAction::Select(i)),
            UnifiedListAction::ContextMenu { index, x, y } => {
                Some(VitalSignsListAction::ContextMenu { index, x, y })
            }
            UnifiedListAction::Open(_)
            | UnifiedListAction::New
            | UnifiedListAction::Edit(_)
            | UnifiedListAction::Delete(_)
            | UnifiedListAction::ToggleInactive => None,
        });
        self.sync_from(&list);
        action
    }
}

impl Widget for VitalSignsList {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.as_list().render(area, buf);
    }
}

impl HasFocus for VitalSignsList {
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

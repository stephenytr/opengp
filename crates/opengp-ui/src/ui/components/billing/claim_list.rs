use std::rc::Rc;

use crate::ui::theme::Theme;
use crate::ui::widgets::{UnifiedColumnDef, UnifiedList, UnifiedListAction, UnifiedListConfig};
use crossterm::event::{MouseEvent, MouseButton, MouseEventKind};
use opengp_domain::domain::billing::{ClaimStatus, MedicareClaim};
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
use uuid::Uuid;
use rat_focus::{FocusFlag, HasFocus, FocusBuilder};

#[derive(Debug, Clone)]
pub struct ClaimList {
    pub claims: Vec<MedicareClaim>,
    pub selected_index: usize,
    pub scroll_offset: usize,
    pub hovered_index: Option<usize>,
    pub theme: Theme,
    pub focus: FocusFlag,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClaimListAction {
    Select(usize),
    ViewDetail,
    PrepareNew,
    Back,
    ContextMenu { x: u16, y: u16, claim_id: Uuid },
}

impl ClaimList {
    pub fn new(claims: Vec<MedicareClaim>, theme: Theme) -> Self {
        let mut scroll_state = ratatui::widgets::ListState::default();
        if !claims.is_empty() {
            scroll_state.select(Some(0));
        }

        Self {
            claims,
            selected_index: 0,
            scroll_offset: 0,
            hovered_index: None,
            theme,
            focus: FocusFlag::default(),
        }
    }

    pub fn select_next(&mut self) {
        self.selected_index = (self.selected_index + 1).min(self.claims.len().saturating_sub(1));
    }

    pub fn select_prev(&mut self) {
        self.selected_index = self.selected_index.saturating_sub(1);
    }

    pub fn handle_mouse(&mut self, mouse: MouseEvent, area: Rect) -> Option<ClaimListAction> {
        let mut list = self.as_list();
        let action = list.handle_mouse(mouse, area);
        self.sync_from(&list);
        action.map(|a| match a {
            UnifiedListAction::Select(i) => ClaimListAction::Select(i),
            UnifiedListAction::Open(_) => ClaimListAction::ViewDetail,
            UnifiedListAction::ContextMenu { index, x, y } => {
                if let Some(claim) = self.claims.get(index) {
                    ClaimListAction::ContextMenu { x, y, claim_id: claim.id }
                } else {
                    ClaimListAction::Select(index)
                }
            }
            UnifiedListAction::New | UnifiedListAction::Edit(_) | UnifiedListAction::Delete(_) | UnifiedListAction::ToggleInactive => {
                ClaimListAction::Select(self.selected_index)
            }
        })
    }

    fn as_list(&self) -> UnifiedList<MedicareClaim> {
        let mut list = UnifiedList::new(
            self.claims.clone(),
            columns(),
            self.theme.clone(),
            UnifiedListConfig::new("Claims", 2, "No claims found."),
        );
        list.selected_index = self.selected_index;
        list.scroll_offset = self.scroll_offset;
        list.hovered_index = self.hovered_index;
        list
    }

    fn sync_from(&mut self, list: &UnifiedList<MedicareClaim>) {
        self.selected_index = list.selected_index;
        self.scroll_offset = list.scroll_offset;
        self.hovered_index = list.hovered_index;
    }
}

impl Widget for ClaimList {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.as_list().render(area, buf);
    }
}

fn col(
    title: &'static str,
    width: u16,
    render: impl Fn(&MedicareClaim) -> String + 'static,
) -> UnifiedColumnDef<MedicareClaim> {
    UnifiedColumnDef::new(title, width, render)
}

fn columns() -> Vec<UnifiedColumnDef<MedicareClaim>> {
    vec![
        col("Reference", 18, |c| {
            c.claim_reference.clone().unwrap_or_else(|| "-".to_string())
        }),
        col("Date", 12, |c| c.service_date.format("%d/%m/%Y").to_string()),
        col("Patient", 12, |c| short_patient(c)),
        col("Type", 14, |c| c.claim_type.to_string()),
        col("Total Claimed", 14, |c| format!("${:.2}", c.total_claimed)),
        col("Status", 12, |c| c.status.to_string()),
    ]
}

fn short_patient(claim: &MedicareClaim) -> String {
    claim.patient_id.to_string().chars().take(8).collect()
}

impl HasFocus for ClaimList {
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

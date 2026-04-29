use std::rc::Rc;

use crate::ui::theme::Theme;
use crate::ui::widgets::{UnifiedColumnDef, UnifiedList, UnifiedListAction, UnifiedListConfig};
use crossterm::event::{KeyEvent, MouseEvent};
use opengp_domain::domain::clinical::Consultation;
use rat_focus::{FocusBuilder, FocusFlag, HasFocus};
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

#[derive(Clone)]
pub struct ConsultationList {
    pub consultations: Vec<Consultation>,
    pub selected_index: usize,
    pub scroll_offset: usize,
    pub loading: bool,
    pub theme: Theme,
    pub hovered_index: Option<usize>,
    pub focus: FocusFlag,
}

#[derive(Debug, Clone)]
pub enum ConsultationListAction {
    Select(usize),
    Open(Box<Consultation>),
    New,
    NextPage,
    PrevPage,
    ContextMenu { index: usize, x: u16, y: u16 },
}

impl ConsultationList {
    pub fn new(theme: Theme) -> Self {
        Self {
            consultations: Vec::new(),
            selected_index: 0,
            scroll_offset: 0,
            loading: false,
            theme,
            hovered_index: None,
            focus: FocusFlag::default(),
        }
    }

    pub fn set_consultations(&mut self, consultations: Vec<Consultation>) {
        self.consultations = consultations;
    }

    pub fn next(&mut self) {
        if self.selected_index < self.consultations.len().saturating_sub(1) {
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

    pub fn move_last(&mut self) {
        self.selected_index = self.consultations.len().saturating_sub(1);
    }

    pub fn move_up(&mut self) {
        self.prev();
    }

    pub fn move_down(&mut self) {
        self.next();
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

    pub fn move_up_and_scroll(&mut self, visible_rows: usize) {
        self.move_up();
        self.adjust_scroll(visible_rows);
    }

    pub fn move_down_and_scroll(&mut self, visible_rows: usize) {
        self.move_down();
        self.adjust_scroll(visible_rows);
    }

    pub fn selected(&self) -> Option<&Consultation> {
        self.consultations.get(self.selected_index)
    }

    pub fn selected_id(&self) -> Option<uuid::Uuid> {
        self.selected().map(|c| c.id)
    }

    pub fn selected_index(&self) -> usize {
        self.selected_index
    }

    pub fn has_selection(&self) -> bool {
        !self.consultations.is_empty()
    }

    pub fn count(&self) -> usize {
        self.consultations.len()
    }

    pub fn is_loading(&self) -> bool {
        self.loading
    }

    pub fn set_loading(&mut self, loading: bool) {
        self.loading = loading;
    }

    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Option<ConsultationListAction> {
        let mut list = self.as_list()?;
        let action = list.handle_key(key);
        self.sync_from(&list);
        action.map(|a| match a {
            UnifiedListAction::Select(i) => ConsultationListAction::Select(i),
            UnifiedListAction::Open(c) => ConsultationListAction::Open(Box::new(c)),
            UnifiedListAction::New => ConsultationListAction::New,
            _ => ConsultationListAction::Select(self.selected_index),
        })
    }

    pub fn handle_mouse(
        &mut self,
        mouse: MouseEvent,
        area: Rect,
    ) -> Option<ConsultationListAction> {
        let mut list = self.as_list()?;
        let action = list.handle_mouse(mouse, area);
        self.sync_from(&list);
        action.map(|a| match a {
            UnifiedListAction::Select(i) => ConsultationListAction::Select(i),
            UnifiedListAction::Open(c) => ConsultationListAction::Open(Box::new(c)),
            UnifiedListAction::New => ConsultationListAction::New,
            UnifiedListAction::ContextMenu { index, x, y, .. } => {
                ConsultationListAction::ContextMenu { index, x, y }
            }
            _ => ConsultationListAction::Select(self.selected_index),
        })
    }

    fn as_list(&self) -> Option<UnifiedList<Consultation>> {
        let mut list = UnifiedList::new(
            self.consultations.clone(),
            columns(),
            self.theme.clone(),
            UnifiedListConfig::new(
                "Consultations",
                2,
                "No consultations found. Press n to add a new consultation.",
            ),
        );
        list.selected_index = self.selected_index;
        list.scroll_offset = self.scroll_offset;
        list.loading = self.loading;
        list.hovered_index = self.hovered_index;
        Some(list)
    }

    fn sync_from(&mut self, list: &UnifiedList<Consultation>) {
        self.selected_index = list.selected_index;
        self.scroll_offset = list.scroll_offset;
        self.loading = list.loading;
        self.hovered_index = list.hovered_index;
    }
}

impl Widget for ConsultationList {
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
    render: impl Fn(&Consultation) -> String + 'static,
) -> UnifiedColumnDef<Consultation> {
    UnifiedColumnDef {
        title,
        width,
        render: Rc::new(render),
    }
}

fn columns() -> Vec<UnifiedColumnDef<Consultation>> {
    vec![
        col("Date", 12, |c| {
            c.consultation_date.format("%d/%m/%Y").to_string()
        }),
        col("Practitioner", 20, |c| format!("ID: {}", c.practitioner_id)),
        col("Reason", 30, |c| {
            c.reason
                .clone()
                .unwrap_or_else(|| "-".into())
                .chars()
                .take(28)
                .collect()
        }),
        col("Status", 10, |c| {
            if c.is_signed {
                "Signed".into()
            } else {
                "Unsigned".into()
            }
        }),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyCode;
    use uuid::Uuid;

    #[test]
    fn test_consultation_list_empty() {
        let theme = Theme::dark();
        let list = ConsultationList::new(theme);
        assert_eq!(list.count(), 0);
    }

    #[test]
    fn test_consultation_list_navigation() {
        let theme = Theme::dark();
        let mut list = ConsultationList::new(theme);

        let consultations = vec![
            Consultation::new(Uuid::new_v4(), Uuid::new_v4(), None, Uuid::new_v4()),
            Consultation::new(Uuid::new_v4(), Uuid::new_v4(), None, Uuid::new_v4()),
        ];
        list.consultations = consultations;

        assert!(list.has_selection());

        list.move_first();
        assert_eq!(list.selected_index(), 0);

        list.move_down();
        assert_eq!(list.selected_index(), 1);

        list.move_up();
        assert_eq!(list.selected_index(), 0);
    }

    #[test]
    fn test_consultation_list_up_down_key_navigation() {
        let theme = Theme::dark();
        let mut list = ConsultationList::new(theme);

        let consultations = vec![
            Consultation::new(Uuid::new_v4(), Uuid::new_v4(), None, Uuid::new_v4()),
            Consultation::new(Uuid::new_v4(), Uuid::new_v4(), None, Uuid::new_v4()),
            Consultation::new(Uuid::new_v4(), Uuid::new_v4(), None, Uuid::new_v4()),
        ];
        list.consultations = consultations;

        let key_down = KeyEvent::new(KeyCode::Down, crossterm::event::KeyModifiers::NONE);
        let key_up = KeyEvent::new(KeyCode::Up, crossterm::event::KeyModifiers::NONE);

        assert_eq!(list.selected_index(), 0);

        let action = list.handle_key(key_down);
        assert!(matches!(action, Some(ConsultationListAction::Select(1))));
        assert_eq!(list.selected_index(), 1);

        let action = list.handle_key(key_down);
        assert!(matches!(action, Some(ConsultationListAction::Select(2))));
        assert_eq!(list.selected_index(), 2);

        let action = list.handle_key(key_up);
        assert!(matches!(action, Some(ConsultationListAction::Select(1))));
        assert_eq!(list.selected_index(), 1);

        let action = list.handle_key(key_up);
        assert!(matches!(action, Some(ConsultationListAction::Select(0))));
        assert_eq!(list.selected_index(), 0);
    }

    #[test]
    fn test_consultation_list_j_k_key_navigation() {
        let theme = Theme::dark();
        let mut list = ConsultationList::new(theme);

        let consultations = vec![
            Consultation::new(Uuid::new_v4(), Uuid::new_v4(), None, Uuid::new_v4()),
            Consultation::new(Uuid::new_v4(), Uuid::new_v4(), None, Uuid::new_v4()),
        ];
        list.consultations = consultations;

        let key_j = KeyEvent::new(KeyCode::Char('j'), crossterm::event::KeyModifiers::NONE);
        let key_k = KeyEvent::new(KeyCode::Char('k'), crossterm::event::KeyModifiers::NONE);

        assert_eq!(list.selected_index(), 0);

        let action = list.handle_key(key_j);
        assert!(matches!(action, Some(ConsultationListAction::Select(1))));
        assert_eq!(list.selected_index(), 1);

        let action = list.handle_key(key_k);
        assert!(matches!(action, Some(ConsultationListAction::Select(0))));
        assert_eq!(list.selected_index(), 0);
    }
}

impl HasFocus for ConsultationList {
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

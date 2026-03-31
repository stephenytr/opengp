use crate::ui::theme::Theme;
use crate::ui::widgets::{ClinicalTableList, ColumnDef, ListAction};
use crossterm::event::{KeyEvent, MouseEvent};
use opengp_domain::domain::clinical::Consultation;
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

#[derive(Clone)]
pub struct ConsultationList {
    pub consultations: Vec<Consultation>,
    pub selected_index: usize,
    pub scroll_offset: usize,
    pub loading: bool,
    theme: Theme,
}

#[derive(Debug, Clone)]
pub enum ConsultationListAction {
    Select(usize),
    Open(Consultation),
    New,
    NextPage,
    PrevPage,
}

impl ConsultationList {
    pub fn new(theme: Theme) -> Self {
        Self {
            consultations: Vec::new(),
            selected_index: 0,
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

    pub fn move_last(&mut self) {
        let mut table = self.table();
        table.move_last();
        self.selected_index = table.selected_index;
        self.scroll_offset = table.scroll_offset;
    }

    pub fn move_up(&mut self) {
        self.prev();
    }

    pub fn move_down(&mut self) {
        self.next();
    }

    pub fn adjust_scroll(&mut self, visible_rows: usize) {
        let mut table = self.table();
        table.adjust_scroll(visible_rows);
        self.selected_index = table.selected_index;
        self.scroll_offset = table.scroll_offset;
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
        let mut table = self.table();
        let out = match table.handle_key(key) {
            Some(ListAction::Select(i)) => Some(ConsultationListAction::Select(i)),
            Some(ListAction::Open(c)) => Some(ConsultationListAction::Open(c)),
            Some(ListAction::New) => Some(ConsultationListAction::New),
            _ => None,
        };
        self.selected_index = table.selected_index;
        self.scroll_offset = table.scroll_offset;
        out
    }

    pub fn handle_mouse(
        &mut self,
        mouse: MouseEvent,
        area: Rect,
    ) -> Option<ConsultationListAction> {
        let mut table = self.table();
        let out = match table.handle_mouse(mouse, area) {
            Some(ListAction::Select(i)) => Some(ConsultationListAction::Select(i)),
            _ => None,
        };
        self.selected_index = table.selected_index;
        self.scroll_offset = table.scroll_offset;
        out
    }

    fn table(&self) -> ClinicalTableList<Consultation> {
        let mut table = ClinicalTableList::new(
            self.consultations.clone(),
            columns(),
            self.theme.clone(),
            "Consultations",
            None,
        );
        table.selected_index = self.selected_index;
        table.scroll_offset = self.scroll_offset;
        table.loading = self.loading;
        table.empty_message = "No consultations found. Press n to add a new consultation.".into();
        table
    }
}

impl Widget for ConsultationList {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.table().render(area, buf);
    }
}

fn col(
    title: &'static str,
    width: u16,
    render: impl Fn(&Consultation) -> String + 'static,
) -> ColumnDef<Consultation> {
    ColumnDef {
        title,
        width,
        render: Box::new(render),
    }
}

fn columns() -> Vec<ColumnDef<Consultation>> {
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
}

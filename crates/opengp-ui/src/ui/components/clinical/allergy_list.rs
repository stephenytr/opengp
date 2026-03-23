use crate::ui::layout::HEADER_HEIGHT;
use crate::ui::theme::Theme;
use crate::ui::widgets::LoadingState;
use crossterm::event::{KeyEvent, MouseEvent, MouseEventKind};
use opengp_domain::domain::clinical::Allergy;
use opengp_domain::domain::clinical::AllergyType;
use opengp_domain::domain::clinical::Severity;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Position, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Row, Table, Widget};

pub struct AllergyList {
    pub allergies: Vec<Allergy>,
    pub selected_index: usize,
    pub show_inactive: bool,
    pub scroll_offset: usize,
    pub loading: bool,
    loading_state: LoadingState,
    theme: Theme,
}

impl Clone for AllergyList {
    fn clone(&self) -> Self {
        Self {
            allergies: self.allergies.clone(),
            selected_index: self.selected_index,
            show_inactive: self.show_inactive,
            scroll_offset: self.scroll_offset,
            loading: self.loading,
            loading_state: self.loading_state.clone(),
            theme: self.theme.clone(),
        }
    }
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
            loading_state: LoadingState::new().message("Loading allergies..."),
            theme,
        }
    }

    pub fn with_allergies(allergies: Vec<Allergy>, theme: Theme) -> Self {
        Self {
            allergies,
            selected_index: 0,
            show_inactive: true,
            scroll_offset: 0,
            loading: false,
            loading_state: LoadingState::new().message("Loading allergies..."),
            theme,
        }
    }

    pub fn selected(&self) -> Option<&Allergy> {
        self.allergies.get(self.selected_index)
    }

    pub fn selected_id(&self) -> Option<uuid::Uuid> {
        self.selected().map(|a| a.id)
    }

    pub fn select(&mut self, index: usize) {
        if index < self.allergies.len() {
            self.selected_index = index;
        }
    }

    pub fn is_loading(&self) -> bool {
        self.loading
    }

    pub fn set_loading(&mut self, loading: bool) {
        self.loading = loading;
    }

    pub fn next(&mut self) {
        if self.selected_index + 1 < self.allergies.len() {
            self.selected_index += 1;
        }
    }

    pub fn prev(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    pub fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    pub fn move_down(&mut self) {
        if self.selected_index < self.allergies.len().saturating_sub(1) {
            self.selected_index += 1;
        }
    }

    pub fn move_first(&mut self) {
        self.selected_index = 0;
    }

    pub fn move_last(&mut self) {
        self.selected_index = self.allergies.len().saturating_sub(1);
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

    pub fn toggle_inactive(&mut self) {
        self.show_inactive = !self.show_inactive;
    }

    pub fn filtered_allergies(&self) -> Vec<&Allergy> {
        if self.show_inactive {
            self.allergies.iter().collect()
        } else {
            self.allergies.iter().filter(|a| a.is_active).collect()
        }
    }

    pub fn has_selection(&self) -> bool {
        !self.allergies.is_empty()
    }

    pub fn count(&self) -> usize {
        self.allergies.len()
    }

    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Option<AllergyListAction> {
        use crossterm::event::{KeyCode, KeyEventKind};

        if key.kind != KeyEventKind::Press {
            return None;
        }

        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.move_up();
                Some(AllergyListAction::Select(self.selected_index))
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.move_down();
                Some(AllergyListAction::Select(self.selected_index))
            }
            KeyCode::Home => {
                self.move_first();
                Some(AllergyListAction::Select(self.selected_index))
            }
            KeyCode::End => {
                self.move_last();
                Some(AllergyListAction::Select(self.selected_index))
            }
            KeyCode::PageUp => {
                let new_index = self.selected_index.saturating_sub(10);
                self.selected_index = new_index;
                Some(AllergyListAction::Select(self.selected_index))
            }
            KeyCode::PageDown => {
                let new_index =
                    (self.selected_index + 10).min(self.allergies.len().saturating_sub(1));
                self.selected_index = new_index;
                Some(AllergyListAction::Select(self.selected_index))
            }
            KeyCode::Enter => self
                .selected()
                .map(|allergy| AllergyListAction::Open(allergy.clone())),
            KeyCode::Char('n') => Some(AllergyListAction::New),
            KeyCode::Char('i') => {
                self.toggle_inactive();
                Some(AllergyListAction::ToggleInactive)
            }
            KeyCode::Char('d') => self
                .selected()
                .map(|allergy| AllergyListAction::Delete(allergy.clone())),
            _ => None,
        }
    }

    pub fn handle_mouse(&mut self, mouse: MouseEvent, area: Rect) -> Option<AllergyListAction> {
        if let MouseEventKind::ScrollUp = mouse.kind {
            if self.scroll_offset > 0 {
                self.scroll_offset = self.scroll_offset.saturating_sub(3).max(0);
            }
            return Some(AllergyListAction::Select(self.selected_index));
        }
        if let MouseEventKind::ScrollDown = mouse.kind {
            let visible_rows = area.height.saturating_sub(3) as usize;
            let max_scroll = self.allergies.len().saturating_sub(visible_rows);
            self.scroll_offset = (self.scroll_offset + 3).min(max_scroll);
            return Some(AllergyListAction::Select(self.selected_index));
        }

        if mouse.kind != MouseEventKind::Up(crossterm::event::MouseButton::Left) {
            return None;
        }

        if !area.contains(Position::new(mouse.column, mouse.row)) {
            return None;
        }

        if mouse.row < area.y + HEADER_HEIGHT {
            return None;
        }

        let row_index = (mouse.row - area.y - HEADER_HEIGHT) as usize;
        let actual_index = self.scroll_offset + row_index;
        if actual_index < self.allergies.len() {
            self.selected_index = actual_index;
            Some(AllergyListAction::Select(self.selected_index))
        } else {
            None
        }
    }
}

const COL_ALLERGEN: u16 = 20;
const COL_TYPE: u16 = 15;
const COL_SEVERITY: u16 = 10;
const COL_REACTION: u16 = 30;
const COL_STATUS: u16 = 10;

fn format_allergen(allergy: &Allergy) -> String {
    allergy.allergen.clone()
}

fn format_type(allergy: &Allergy) -> String {
    match allergy.allergy_type {
        AllergyType::Drug => "Drug".to_string(),
        AllergyType::Food => "Food".to_string(),
        AllergyType::Environmental => "Environmental".to_string(),
        AllergyType::Other => "Other".to_string(),
    }
}

fn format_severity(allergy: &Allergy) -> String {
    match allergy.severity {
        Severity::Mild => "Mild".to_string(),
        Severity::Moderate => "Moderate".to_string(),
        Severity::Severe => "Severe".to_string(),
    }
}

fn format_reaction(allergy: &Allergy) -> String {
    allergy
        .reaction
        .as_ref()
        .map(|s| {
            if s.len() > 28 {
                format!("{}...", &s[..28])
            } else {
                s.clone()
            }
        })
        .unwrap_or_else(|| "-".to_string())
}

fn format_status(allergy: &Allergy) -> String {
    if allergy.is_active {
        "Active".to_string()
    } else {
        "Inactive".to_string()
    }
}

impl Widget for AllergyList {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.is_empty() {
            return;
        }

        let block = Block::default()
            .title(" Allergies ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.colors.border));

        block.clone().render(area, buf);

        let inner = block.inner(area);
        if inner.is_empty() {
            return;
        }

        if self.loading {
            let mut loading_state = self.loading_state.clone();
            loading_state.tick();
            let indicator = loading_state.to_indicator(self.theme.clone());
            indicator.render(inner, buf);
            return;
        }

        if self.allergies.is_empty() {
            let message = "No allergies found. Press n to add a new allergy.";
            let text = Line::from(vec![Span::styled(
                message,
                Style::default().fg(self.theme.colors.disabled),
            )]);
            let x = inner.x + (inner.width.saturating_sub(message.len() as u16)) / 2;
            let y = inner.y + inner.height / 2;
            buf.set_line(x, y, &text, inner.width);
            return;
        }

        let col_widths = [
            Constraint::Length(COL_ALLERGEN),
            Constraint::Length(COL_TYPE),
            Constraint::Length(COL_SEVERITY),
            Constraint::Length(COL_REACTION),
            Constraint::Length(COL_STATUS),
        ];

        let header = Row::new(vec!["Allergen", "Type", "Severity", "Reaction", "Status"])
            .style(Style::default().fg(self.theme.colors.primary).bold());

        let visible_rows = inner.height as usize;
        let max_scroll = self.allergies.len().saturating_sub(visible_rows);
        let scroll_offset = self.scroll_offset.min(max_scroll);

        let rows: Vec<Row> = self
            .allergies
            .iter()
            .skip(scroll_offset)
            .take(visible_rows)
            .enumerate()
            .map(|(i, allergy)| {
                let actual_index = scroll_offset + i;
                let style = if actual_index == self.selected_index {
                    Style::default()
                        .bg(self.theme.colors.selected)
                        .fg(self.theme.colors.foreground)
                } else {
                    Style::default().fg(self.theme.colors.foreground)
                };

                Row::new(vec![
                    format_allergen(allergy),
                    format_type(allergy),
                    format_severity(allergy),
                    format_reaction(allergy),
                    format_status(allergy),
                ])
                .style(style)
                .height(1)
            })
            .collect();

        let table = Table::new(rows, col_widths)
            .header(header)
            .block(Block::default().borders(Borders::NONE))
            .widths(col_widths);

        table.render(inner, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
    use uuid::Uuid;

    fn create_test_allergy(allergen: &str, is_active: bool) -> Allergy {
        Allergy {
            id: Uuid::new_v4(),
            patient_id: Uuid::new_v4(),
            allergen: allergen.to_string(),
            allergy_type: AllergyType::Drug,
            severity: Severity::Moderate,
            reaction: Some("Rash".to_string()),
            onset_date: None,
            notes: None,
            is_active,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_by: Uuid::new_v4(),
            updated_by: None,
        }
    }

    #[test]
    fn test_next_at_end_stays_at_last() {
        let allergies = vec![
            create_test_allergy("Penicillin", true),
            create_test_allergy("Aspirin", true),
            create_test_allergy("Ibuprofen", true),
        ];
        let mut list = AllergyList::with_allergies(allergies, Theme::dark());
        list.selected_index = 2;
        list.next();
        assert_eq!(list.selected_index, 2);
    }

    #[test]
    fn test_next_advances_selection() {
        let allergies = vec![
            create_test_allergy("Penicillin", true),
            create_test_allergy("Aspirin", true),
        ];
        let mut list = AllergyList::with_allergies(allergies, Theme::dark());
        list.next();
        assert_eq!(list.selected_index, 1);
    }

    #[test]
    fn test_prev_at_zero_stays_at_zero() {
        let allergies = vec![
            create_test_allergy("Penicillin", true),
            create_test_allergy("Aspirin", true),
        ];
        let mut list = AllergyList::with_allergies(allergies, Theme::dark());
        list.prev();
        assert_eq!(list.selected_index, 0);
    }

    #[test]
    fn test_prev_decreases_selection() {
        let allergies = vec![
            create_test_allergy("Penicillin", true),
            create_test_allergy("Aspirin", true),
        ];
        let mut list = AllergyList::with_allergies(allergies, Theme::dark());
        list.selected_index = 1;
        list.prev();
        assert_eq!(list.selected_index, 0);
    }

    #[test]
    fn test_move_up_decreases_selection() {
        let allergies = vec![
            create_test_allergy("Penicillin", true),
            create_test_allergy("Aspirin", true),
        ];
        let mut list = AllergyList::with_allergies(allergies, Theme::dark());
        list.selected_index = 1;
        list.move_up();
        assert_eq!(list.selected_index, 0);
    }

    #[test]
    fn test_move_down_increases_selection() {
        let allergies = vec![
            create_test_allergy("Penicillin", true),
            create_test_allergy("Aspirin", true),
        ];
        let mut list = AllergyList::with_allergies(allergies, Theme::dark());
        list.move_down();
        assert_eq!(list.selected_index, 1);
    }

    #[test]
    fn test_move_first_sets_to_zero() {
        let allergies = vec![
            create_test_allergy("Penicillin", true),
            create_test_allergy("Aspirin", true),
            create_test_allergy("Ibuprofen", true),
        ];
        let mut list = AllergyList::with_allergies(allergies, Theme::dark());
        list.selected_index = 2;
        list.move_first();
        assert_eq!(list.selected_index, 0);
    }

    #[test]
    fn test_move_last_sets_to_end() {
        let allergies = vec![
            create_test_allergy("Penicillin", true),
            create_test_allergy("Aspirin", true),
            create_test_allergy("Ibuprofen", true),
        ];
        let mut list = AllergyList::with_allergies(allergies, Theme::dark());
        list.move_last();
        assert_eq!(list.selected_index, 2);
    }

    #[test]
    fn test_selected_returns_correct_allergy() {
        let allergy1 = create_test_allergy("Penicillin", true);
        let allergy2 = create_test_allergy("Aspirin", true);
        let id1 = allergy1.id;
        let list = AllergyList::with_allergies(vec![allergy1, allergy2], Theme::dark());
        assert_eq!(list.selected().map(|a| a.id), Some(id1));
    }

    #[test]
    fn test_selected_id_returns_correct_uuid() {
        let allergy = create_test_allergy("Penicillin", true);
        let id = allergy.id;
        let list = AllergyList::with_allergies(vec![allergy], Theme::dark());
        assert_eq!(list.selected_id(), Some(id));
    }

    #[test]
    fn test_toggle_inactive_flips_flag() {
        let allergies = vec![create_test_allergy("Penicillin", true)];
        let mut list = AllergyList::with_allergies(allergies, Theme::dark());
        assert!(list.show_inactive);
        list.toggle_inactive();
        assert!(!list.show_inactive);
        list.toggle_inactive();
        assert!(list.show_inactive);
    }

    #[test]
    fn test_filtered_allergies_shows_all_when_show_inactive_true() {
        let allergies = vec![
            create_test_allergy("Penicillin", true),
            create_test_allergy("Aspirin", false),
            create_test_allergy("Ibuprofen", true),
        ];
        let list = AllergyList::with_allergies(allergies, Theme::dark());
        let filtered = list.filtered_allergies();
        assert_eq!(filtered.len(), 3);
    }

    #[test]
    fn test_filtered_allergies_excludes_inactive_when_show_inactive_false() {
        let allergies = vec![
            create_test_allergy("Penicillin", true),
            create_test_allergy("Aspirin", false),
            create_test_allergy("Ibuprofen", true),
        ];
        let mut list = AllergyList::with_allergies(allergies, Theme::dark());
        list.show_inactive = false;
        let filtered = list.filtered_allergies();
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|a| a.is_active));
    }

    #[test]
    fn test_adjust_scroll_moves_selection_into_view_top() {
        let allergies = vec![
            create_test_allergy("Penicillin", true),
            create_test_allergy("Aspirin", true),
            create_test_allergy("Ibuprofen", true),
        ];
        let mut list = AllergyList::with_allergies(allergies, Theme::dark());
        list.scroll_offset = 2;
        list.selected_index = 0;
        list.adjust_scroll(10);
        assert_eq!(list.scroll_offset, 0);
    }

    #[test]
    fn test_adjust_scroll_moves_selection_into_view_bottom() {
        let allergies = vec![
            create_test_allergy("Penicillin", true),
            create_test_allergy("Aspirin", true),
            create_test_allergy("Ibuprofen", true),
        ];
        let mut list = AllergyList::with_allergies(allergies, Theme::dark());
        list.scroll_offset = 0;
        list.selected_index = 2;
        list.adjust_scroll(2);
        assert_eq!(list.scroll_offset, 1);
    }

    #[test]
    fn test_adjust_scroll_returns_early_when_visible_rows_zero() {
        let allergies = vec![create_test_allergy("Penicillin", true)];
        let mut list = AllergyList::with_allergies(allergies, Theme::dark());
        list.scroll_offset = 5;
        list.adjust_scroll(0);
        assert_eq!(list.scroll_offset, 5);
    }

    #[test]
    fn test_handle_key_up_moves_up() {
        let allergies = vec![
            create_test_allergy("Penicillin", true),
            create_test_allergy("Aspirin", true),
        ];
        let mut list = AllergyList::with_allergies(allergies, Theme::dark());
        list.selected_index = 1;
        let key = KeyEvent::new(KeyCode::Up, KeyModifiers::NONE);
        let action = list.handle_key(key);
        assert_eq!(list.selected_index, 0);
        assert!(matches!(action, Some(AllergyListAction::Select(0))));
    }

    #[test]
    fn test_handle_key_k_moves_up() {
        let allergies = vec![
            create_test_allergy("Penicillin", true),
            create_test_allergy("Aspirin", true),
        ];
        let mut list = AllergyList::with_allergies(allergies, Theme::dark());
        list.selected_index = 1;
        let key = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE);
        let action = list.handle_key(key);
        assert_eq!(list.selected_index, 0);
        assert!(matches!(action, Some(AllergyListAction::Select(0))));
    }

    #[test]
    fn test_handle_key_down_moves_down() {
        let allergies = vec![
            create_test_allergy("Penicillin", true),
            create_test_allergy("Aspirin", true),
        ];
        let mut list = AllergyList::with_allergies(allergies, Theme::dark());
        let key = KeyEvent::new(KeyCode::Down, KeyModifiers::NONE);
        let action = list.handle_key(key);
        assert_eq!(list.selected_index, 1);
        assert!(matches!(action, Some(AllergyListAction::Select(1))));
    }

    #[test]
    fn test_handle_key_j_moves_down() {
        let allergies = vec![
            create_test_allergy("Penicillin", true),
            create_test_allergy("Aspirin", true),
        ];
        let mut list = AllergyList::with_allergies(allergies, Theme::dark());
        let key = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
        let action = list.handle_key(key);
        assert_eq!(list.selected_index, 1);
        assert!(matches!(action, Some(AllergyListAction::Select(1))));
    }

    #[test]
    fn test_handle_key_enter_opens_selected() {
        let allergy = create_test_allergy("Penicillin", true);
        let id = allergy.id;
        let mut list = AllergyList::with_allergies(vec![allergy], Theme::dark());
        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let action = list.handle_key(key);
        assert!(matches!(action, Some(AllergyListAction::Open(_))));
        if let Some(AllergyListAction::Open(a)) = action {
            assert_eq!(a.id, id);
        }
    }

    #[test]
    fn test_handle_key_n_creates_new() {
        let allergies = vec![create_test_allergy("Penicillin", true)];
        let mut list = AllergyList::with_allergies(allergies, Theme::dark());
        let key = KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE);
        let action = list.handle_key(key);
        assert!(matches!(action, Some(AllergyListAction::New)));
    }

    #[test]
    fn test_handle_key_i_toggles_inactive() {
        let allergies = vec![create_test_allergy("Penicillin", true)];
        let mut list = AllergyList::with_allergies(allergies, Theme::dark());
        let key = KeyEvent::new(KeyCode::Char('i'), KeyModifiers::NONE);
        let action = list.handle_key(key);
        assert!(!list.show_inactive);
        assert!(matches!(action, Some(AllergyListAction::ToggleInactive)));
    }

    #[test]
    fn test_handle_key_d_deletes_selected() {
        let allergy = create_test_allergy("Penicillin", true);
        let id = allergy.id;
        let mut list = AllergyList::with_allergies(vec![allergy], Theme::dark());
        let key = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE);
        let action = list.handle_key(key);
        assert!(matches!(action, Some(AllergyListAction::Delete(_))));
        if let Some(AllergyListAction::Delete(a)) = action {
            assert_eq!(a.id, id);
        }
    }

    #[test]
    fn test_handle_key_home_moves_to_first() {
        let allergies = vec![
            create_test_allergy("Penicillin", true),
            create_test_allergy("Aspirin", true),
            create_test_allergy("Ibuprofen", true),
        ];
        let mut list = AllergyList::with_allergies(allergies, Theme::dark());
        list.selected_index = 2;
        let key = KeyEvent::new(KeyCode::Home, KeyModifiers::NONE);
        let action = list.handle_key(key);
        assert_eq!(list.selected_index, 0);
        assert!(matches!(action, Some(AllergyListAction::Select(0))));
    }

    #[test]
    fn test_handle_key_end_moves_to_last() {
        let allergies = vec![
            create_test_allergy("Penicillin", true),
            create_test_allergy("Aspirin", true),
            create_test_allergy("Ibuprofen", true),
        ];
        let mut list = AllergyList::with_allergies(allergies, Theme::dark());
        let key = KeyEvent::new(KeyCode::End, KeyModifiers::NONE);
        let action = list.handle_key(key);
        assert_eq!(list.selected_index, 2);
        assert!(matches!(action, Some(AllergyListAction::Select(2))));
    }

    #[test]
    fn test_handle_key_release_event_returns_none() {
        let allergies = vec![create_test_allergy("Penicillin", true)];
        let mut list = AllergyList::with_allergies(allergies, Theme::dark());
        let mut key = KeyEvent::new(KeyCode::Up, KeyModifiers::NONE);
        key.kind = KeyEventKind::Release;
        let action = list.handle_key(key);
        assert!(action.is_none());
    }

    #[test]
    fn test_has_selection_true_when_non_empty() {
        let allergies = vec![create_test_allergy("Penicillin", true)];
        let list = AllergyList::with_allergies(allergies, Theme::dark());
        assert!(list.has_selection());
    }

    #[test]
    fn test_has_selection_false_when_empty() {
        let list = AllergyList::new(Theme::dark());
        assert!(!list.has_selection());
    }

    #[test]
    fn test_selected_returns_none_when_empty() {
        let list = AllergyList::new(Theme::dark());
        assert!(list.selected().is_none());
    }

    #[test]
    fn test_selected_id_returns_none_when_empty() {
        let list = AllergyList::new(Theme::dark());
        assert!(list.selected_id().is_none());
    }

    #[test]
    fn test_move_last_on_empty_list_stays_at_zero() {
        let mut list = AllergyList::new(Theme::dark());
        list.move_last();
        assert_eq!(list.selected_index, 0);
    }

    #[test]
    fn test_enter_on_empty_list_returns_none() {
        let mut list = AllergyList::new(Theme::dark());
        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let action = list.handle_key(key);
        assert!(action.is_none());
    }

    #[test]
    fn test_delete_on_empty_list_returns_none() {
        let mut list = AllergyList::new(Theme::dark());
        let key = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE);
        let action = list.handle_key(key);
        assert!(action.is_none());
    }

    #[test]
    fn test_allergy_list_snapshot_with_three_allergies() {
        use ratatui::backend::TestBackend;
        use ratatui::Terminal;

        let allergies = vec![
            create_test_allergy("Penicillin", true),
            create_test_allergy("Aspirin", true),
            create_test_allergy("Ibuprofen", false),
        ];

        let mut terminal = Terminal::new(TestBackend::new(80, 10)).unwrap();
        let list = AllergyList::with_allergies(allergies, Theme::dark());

        terminal
            .draw(|f| {
                let rect = f.area();
                f.render_widget(list.clone(), rect);
            })
            .unwrap();

        insta::assert_snapshot!(terminal.backend());
    }

    #[test]
    fn test_allergy_list_snapshot_with_selection_highlight() {
        use ratatui::backend::TestBackend;
        use ratatui::Terminal;

        let allergies = vec![
            create_test_allergy("Penicillin", true),
            create_test_allergy("Aspirin", true),
            create_test_allergy("Ibuprofen", false),
        ];

        let mut terminal = Terminal::new(TestBackend::new(80, 10)).unwrap();
        let mut list = AllergyList::with_allergies(allergies, Theme::dark());
        list.selected_index = 1;

        terminal
            .draw(|f| {
                let rect = f.area();
                f.render_widget(list.clone(), rect);
            })
            .unwrap();

        insta::assert_snapshot!(terminal.backend());
    }
}

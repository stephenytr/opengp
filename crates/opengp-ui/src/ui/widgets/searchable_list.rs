use std::marker::PhantomData;

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Borders, Widget};
use sublime_fuzzy::best_match;
use uuid::Uuid;

use crate::ui::theme::Theme;
use crate::ui::view_models::{PatientListItem, PractitionerViewItem};
use crate::ui::widgets::ScrollableState;

pub trait Searchable: Clone {
    fn id(&self) -> Uuid;
    fn display_text(&self) -> &str;
    fn search_text(&self) -> &str;
}

impl Searchable for PatientListItem {
    fn id(&self) -> Uuid {
        self.id
    }

    fn display_text(&self) -> &str {
        &self.full_name
    }

    fn search_text(&self) -> &str {
        &self.full_name
    }
}

impl Searchable for PractitionerViewItem {
    fn id(&self) -> Uuid {
        self.id
    }

    fn display_text(&self) -> &str {
        &self.display_name
    }

    fn search_text(&self) -> &str {
        &self.display_name
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SearchableListAction {
    Selected(Uuid, String),
    Cancelled,
    None,
}

#[derive(Clone)]
pub struct SearchableListState<T: Searchable> {
    pub items: Vec<T>,
    pub filtered: Vec<T>,
    pub query: String,
    scrollable: ScrollableState,
    pub open: bool,
    pub focused: bool,
    _marker: PhantomData<T>,
}

impl<T: Searchable> SearchableListState<T> {
    pub fn new(items: Vec<T>) -> Self {
        let filtered = items.clone();
        Self {
            items,
            filtered,
            query: String::new(),
            scrollable: ScrollableState::new(),
            open: false,
            focused: false,
            _marker: PhantomData,
        }
    }

    pub fn set_items(&mut self, items: Vec<T>) {
        self.items = items.clone();
        if self.query.is_empty() {
            self.filtered = items;
        } else {
            self.filter_fuzzy();
        }
        self.scrollable.set_item_count(self.filtered.len());
    }

    pub fn open(&mut self) {
        self.open = true;
        self.query.clear();
        self.scrollable = ScrollableState::new();
        self.filtered = self.items.clone();
    }

    pub fn close(&mut self) {
        self.open = false;
        self.query.clear();
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    fn filter_fuzzy(&mut self) {
        if self.query.is_empty() {
            self.filtered = self.items.clone();
            return;
        }

        let query = self.query.to_lowercase();

        let mut matches: Vec<(usize, i64)> = self
            .items
            .iter()
            .enumerate()
            .filter_map(|(i, item)| {
                best_match(&query, item.search_text()).map(|result| (i, result.score() as i64))
            })
            .collect();

        matches.sort_by(|a, b| b.1.cmp(&a.1));

        self.filtered = matches
            .into_iter()
            .map(|(i, _)| self.items[i].clone())
            .collect();
        self.scrollable = ScrollableState::new();
    }

    fn filter_substring(&mut self) {
        if self.query.is_empty() {
            self.filtered = self.items.clone();
            return;
        }

        let query = self.query.to_lowercase();
        self.filtered = self
            .items
            .iter()
            .filter(|item| item.search_text().to_lowercase().contains(&query))
            .cloned()
            .collect();
        self.scrollable = ScrollableState::new();
    }

    pub fn set_query(&mut self, query: String, fuzzy: bool) {
        self.query = query;
        if fuzzy {
            self.filter_fuzzy();
        } else {
            self.filter_substring();
        }
    }

    pub fn move_up(&mut self) {
        self.scrollable.move_up();
    }

    pub fn move_down(&mut self) {
        self.scrollable.set_item_count(self.filtered.len());
        self.scrollable.move_down();
    }

    pub fn selected_item(&self) -> Option<&T> {
        self.filtered.get(self.scrollable.selected_index())
    }

    pub fn selected_id(&self) -> Option<Uuid> {
        self.selected_item().map(|item| item.id())
    }

    pub fn selected_display(&self) -> Option<String> {
        self.selected_item()
            .map(|item| item.display_text().to_string())
    }
}

impl<T: Searchable> Default for SearchableListState<T> {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

pub struct SearchableList<'a, T: Searchable> {
    state: &'a mut SearchableListState<T>,
    theme: &'a Theme,
    label: &'a str,
    fuzzy: bool,
}

impl<'a, T: Searchable> SearchableList<'a, T> {
    pub fn new(
        state: &'a mut SearchableListState<T>,
        theme: &'a Theme,
        label: &'a str,
        fuzzy: bool,
    ) -> Self {
        Self {
            state,
            theme,
            label,
            fuzzy,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> SearchableListAction {
        if key.kind != KeyEventKind::Press {
            return SearchableListAction::None;
        }

        if !self.state.open {
            if key.code == KeyCode::Enter {
                self.state.open();
                return SearchableListAction::None;
            }
            return SearchableListAction::None;
        }

        match key.code {
            KeyCode::Esc => {
                self.state.close();
                SearchableListAction::Cancelled
            }
            KeyCode::Enter => {
                if let (Some(id), Some(name)) =
                    (self.state.selected_id(), self.state.selected_display())
                {
                    self.state.close();
                    SearchableListAction::Selected(id, name)
                } else {
                    SearchableListAction::Cancelled
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.state.move_up();
                SearchableListAction::None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.state.move_down();
                SearchableListAction::None
            }
            KeyCode::Backspace => {
                self.state.query.pop();
                if self.fuzzy {
                    self.state.filter_fuzzy();
                } else {
                    self.state.filter_substring();
                }
                SearchableListAction::None
            }
            KeyCode::Char(c) => {
                self.state.query.push(c);
                if self.fuzzy {
                    self.state.filter_fuzzy();
                } else {
                    self.state.filter_substring();
                }
                SearchableListAction::None
            }
            _ => SearchableListAction::None,
        }
    }
}

impl<'a, T: Searchable> Widget for SearchableList<'a, T> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if !self.state.open {
            return;
        }

        ratatui::widgets::Clear.render(area, buf);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(
                Style::default()
                    .fg(self.theme.colors.border)
                    .bg(self.theme.colors.background_dark),
            )
            .title(self.label);

        block.clone().render(area, buf);

        let inner = block.inner(area);
        if inner.height < 3 {
            return;
        }

        buf.set_style(
            inner,
            Style::default().bg(self.theme.colors.background_dark),
        );

        let query_area = Rect::new(inner.x, inner.y, inner.width, 3);
        buf.set_string(
            query_area.x + 1,
            query_area.y + 1,
            format!("Search: {}_", self.state.query),
            Style::default()
                .fg(self.theme.colors.foreground)
                .bg(self.theme.colors.background_dark),
        );

        let list_area = Rect::new(
            inner.x,
            inner.y + 3,
            inner.width,
            inner.height.saturating_sub(3),
        );

        if self.state.filtered.is_empty() {
            buf.set_string(
                list_area.x + 1,
                list_area.y + 1,
                "No results",
                Style::default()
                    .fg(self.theme.colors.disabled)
                    .bg(self.theme.colors.background_dark),
            );
            return;
        }

        let visible_rows = list_area.height as usize;
        self.state
            .scrollable
            .set_item_count(self.state.filtered.len());
        self.state.scrollable.adjust_scroll(visible_rows);
        let start_idx = self.state.scrollable.scroll_offset();
        let end_idx = (start_idx + visible_rows).min(self.state.filtered.len());

        for (i, item) in self.state.filtered[start_idx..end_idx].iter().enumerate() {
            let row_y = list_area.y + i as u16;
            if row_y >= list_area.y + list_area.height {
                break;
            }

            let is_selected = start_idx + i == self.state.scrollable.selected_index();
            let style = if is_selected {
                Style::default()
                    .fg(self.theme.colors.primary)
                    .bg(self.theme.colors.background_dark)
            } else {
                Style::default()
                    .fg(self.theme.colors.foreground)
                    .bg(self.theme.colors.background_dark)
            };

            let display = if item.display_text().len() > list_area.width as usize - 2 {
                format!(
                    "{}...",
                    &item.display_text()[..(list_area.width as usize - 5)]
                )
            } else {
                item.display_text().to_string()
            };

            if is_selected {
                buf.set_string(
                    list_area.x + 1,
                    row_y,
                    &display,
                    style.add_modifier(Modifier::BOLD),
                );
            } else {
                buf.set_string(list_area.x + 1, row_y, &display, style);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyModifiers;

    // Test struct implementing Searchable trait
    #[derive(Debug, Clone, PartialEq, Eq)]
    struct TestItem {
        id: Uuid,
        name: String,
    }

    impl Searchable for TestItem {
        fn id(&self) -> Uuid {
            self.id
        }

        fn display_text(&self) -> &str {
            &self.name
        }

        fn search_text(&self) -> &str {
            &self.name
        }
    }

    // Helper function to create test items
    fn create_test_item(name: &str) -> TestItem {
        TestItem {
            id: Uuid::new_v4(),
            name: name.to_string(),
        }
    }

    // Test 1: SearchableListState::new() — filtered == items initially
    #[test]
    fn test_new_initializes_filtered_to_items() {
        let items = vec![create_test_item("Alice"), create_test_item("Bob")];
        let state = SearchableListState::new(items.clone());

        assert_eq!(state.items.len(), 2);
        assert_eq!(state.filtered.len(), 2);
        assert_eq!(state.filtered, state.items);
        assert_eq!(state.query, "");
        assert!(!state.open);
        assert!(!state.focused);
    }

    // Test 2: set_items() — updates items, re-filters if query active
    #[test]
    fn test_set_items_updates_items_and_refilters() {
        let initial_items = vec![create_test_item("Alice"), create_test_item("Bob")];
        let mut state = SearchableListState::new(initial_items);

        // No query, filtered should match items
        assert_eq!(state.filtered.len(), 2);

        // Set items with new data
        let new_items = vec![
            create_test_item("Charlie"),
            create_test_item("Diana"),
            create_test_item("Eve"),
        ];
        state.set_items(new_items);

        assert_eq!(state.items.len(), 3);
        assert_eq!(state.filtered.len(), 3);
        assert_eq!(state.filtered, state.items);
    }

    // Test 3: set_items() with active query re-filters
    #[test]
    fn test_set_items_with_active_query_refilters() {
        let items = vec![create_test_item("Smith"), create_test_item("Johnson")];
        let mut state = SearchableListState::new(items);

        // Set a query
        state.set_query("smith".to_string(), true);
        assert_eq!(state.filtered.len(), 1); // "smith" matches "Smith"

        // Update items
        let new_items = vec![
            create_test_item("Smith"),
            create_test_item("Smithson"),
            create_test_item("Jones"),
        ];
        state.set_items(new_items);

        // Query should still be active and filter should apply
        assert_eq!(state.filtered.len(), 2); // "smith" matches "Smith" and "Smithson"
    }

    // Test 4: open() — sets open=true, clears query, resets scroll, filtered = all items
    #[test]
    fn test_open_sets_state_correctly() {
        let items = vec![create_test_item("Alice"), create_test_item("Bob")];
        let mut state = SearchableListState::new(items);

        // Simulate some prior state
        state.query = "test".to_string();
        state.open = false;
        state.filtered = vec![]; // Filtered to empty

        state.open();

        assert!(state.open);
        assert_eq!(state.query, "");
        assert_eq!(state.filtered.len(), 2); // Back to all items
    }

    // Test 5: close() — sets open=false, clears query
    #[test]
    fn test_close_sets_state_correctly() {
        let items = vec![create_test_item("Alice")];
        let mut state = SearchableListState::new(items);

        state.open = true;
        state.query = "test".to_string();

        state.close();

        assert!(!state.open);
        assert_eq!(state.query, "");
    }

    // Test 6: filter_fuzzy() — "smi" matches "Smith" but not "Jones"
    #[test]
    fn test_filter_fuzzy_matches_substring() {
        let items = vec![
            create_test_item("Smith"),
            create_test_item("Jones"),
            create_test_item("Smithson"),
        ];
        let mut state = SearchableListState::new(items);

        state.set_query("smi".to_string(), true);

        // "smi" should match "Smith" and "Smithson", but not "Jones"
        assert_eq!(state.filtered.len(), 2);
        assert!(state.filtered.iter().any(|item| item.name == "Smith"));
        assert!(state.filtered.iter().any(|item| item.name == "Smithson"));
        assert!(!state.filtered.iter().any(|item| item.name == "Jones"));
    }

    // Test 7: filter_fuzzy() — results sorted by score
    #[test]
    fn test_filter_fuzzy_sorts_by_score() {
        let items = vec![
            create_test_item("Smith"),
            create_test_item("Smithson"),
            create_test_item("Smythe"),
        ];
        let mut state = SearchableListState::new(items);

        state.set_query("smith".to_string(), true);

        // "smith" should match "Smith" and "Smithson" (exact+substring match),
        // "Smythe" may or may not match depending on fuzzy score threshold
        assert!(state.filtered.len() >= 2);
        // First result should be exact match "Smith"
        assert_eq!(state.filtered[0].name, "Smith");
    }

    // Test 8: filter_substring() — case-insensitive substring match
    #[test]
    fn test_filter_substring_case_insensitive() {
        let items = vec![
            create_test_item("Smith"),
            create_test_item("JOHNSON"),
            create_test_item("alice"),
        ];
        let mut state = SearchableListState::new(items);

        state.set_query("smith".to_string(), false);

        assert_eq!(state.filtered.len(), 1);
        assert_eq!(state.filtered[0].name, "Smith");

        state.set_query("JOHN".to_string(), false);
        assert_eq!(state.filtered.len(), 1);
        assert_eq!(state.filtered[0].name, "JOHNSON");

        state.set_query("Alice".to_string(), false);
        assert_eq!(state.filtered.len(), 1);
        assert_eq!(state.filtered[0].name, "alice");
    }

    // Test 9: set_query() with fuzzy=true
    #[test]
    fn test_set_query_fuzzy_true() {
        let items = vec![create_test_item("Smith"), create_test_item("Jones")];
        let mut state = SearchableListState::new(items);

        state.set_query("smi".to_string(), true);

        assert_eq!(state.query, "smi");
        assert_eq!(state.filtered.len(), 1);
        assert_eq!(state.filtered[0].name, "Smith");
    }

    // Test 10: set_query() with fuzzy=false
    #[test]
    fn test_set_query_fuzzy_false() {
        let items = vec![create_test_item("Smith"), create_test_item("Smithson")];
        let mut state = SearchableListState::new(items);

        state.set_query("smith".to_string(), false);

        assert_eq!(state.query, "smith");
        assert_eq!(state.filtered.len(), 2);
    }

    // Test 11: move_up() and move_down() — delegates to ScrollableState
    #[test]
    fn test_move_up_and_down() {
        let items = vec![
            create_test_item("Alice"),
            create_test_item("Bob"),
            create_test_item("Charlie"),
        ];
        let mut state = SearchableListState::new(items);

        // Initial index is 0
        assert_eq!(state.scrollable.selected_index(), 0);

        state.move_down();
        assert_eq!(state.scrollable.selected_index(), 1);

        state.move_down();
        assert_eq!(state.scrollable.selected_index(), 2);

        state.move_up();
        assert_eq!(state.scrollable.selected_index(), 1);

        state.move_up();
        assert_eq!(state.scrollable.selected_index(), 0);
    }

    // Test 12: selected_item() — returns correct item after navigation
    #[test]
    fn test_selected_item_returns_correct_item() {
        let items = vec![
            create_test_item("Alice"),
            create_test_item("Bob"),
            create_test_item("Charlie"),
        ];
        let mut state = SearchableListState::new(items);

        assert_eq!(state.selected_item().unwrap().name, "Alice");

        state.move_down();
        assert_eq!(state.selected_item().unwrap().name, "Bob");

        state.move_down();
        assert_eq!(state.selected_item().unwrap().name, "Charlie");
    }

    // Test 13: selected_id() and selected_display() — return id/display_text
    #[test]
    fn test_selected_id_and_display() {
        let item = create_test_item("Alice Smith");
        let item_id = item.id;
        let state = SearchableListState::new(vec![item]);

        assert_eq!(state.selected_id(), Some(item_id));
        assert_eq!(state.selected_display(), Some("Alice Smith".to_string()));
    }

    // Test 14: empty list — selected_item() returns None
    #[test]
    fn test_empty_list_returns_none() {
        let state: SearchableListState<TestItem> = SearchableListState::new(vec![]);

        assert!(state.selected_item().is_none());
        assert!(state.selected_id().is_none());
        assert!(state.selected_display().is_none());
    }

    // Test 15: SearchableListAction enum variants
    #[test]
    fn test_searchable_list_action_enum() {
        let id = Uuid::new_v4();
        let name = "Test".to_string();

        let action1 = SearchableListAction::Selected(id, name.clone());
        let action2 = SearchableListAction::Cancelled;
        let action3 = SearchableListAction::None;

        // Verify enum variants can be instantiated
        match action1 {
            SearchableListAction::Selected(_, _) => {}
            _ => panic!("Should be Selected variant"),
        }

        match action2 {
            SearchableListAction::Cancelled => {}
            _ => panic!("Should be Cancelled variant"),
        }

        match action3 {
            SearchableListAction::None => {}
            _ => panic!("Should be None variant"),
        }
    }

    // Test 16: handle_key() — Enter opens when closed
    #[test]
    fn test_handle_key_enter_opens_closed_list() {
        use crate::ui::theme::Theme;

        let items = vec![create_test_item("Alice")];
        let mut state = SearchableListState::new(items);
        let theme = Theme::default();

        let mut list = SearchableList::new(&mut state, &theme, "Test", true);

        let key = KeyEvent {
            code: KeyCode::Enter,
            modifiers: KeyModifiers::empty(),
            kind: KeyEventKind::Press,
            state: crossterm::event::KeyEventState::empty(),
        };

        let action = list.handle_key(key);

        assert!(list.state.open);
        assert_eq!(action, SearchableListAction::None);
    }

    // Test 17: handle_key() — Esc cancels when open
    #[test]
    fn test_handle_key_esc_cancels_open_list() {
        use crate::ui::theme::Theme;

        let items = vec![create_test_item("Alice")];
        let mut state = SearchableListState::new(items);
        state.open = true;
        let theme = Theme::default();

        let mut list = SearchableList::new(&mut state, &theme, "Test", true);

        let key = KeyEvent {
            code: KeyCode::Esc,
            modifiers: KeyModifiers::empty(),
            kind: KeyEventKind::Press,
            state: crossterm::event::KeyEventState::empty(),
        };

        let action = list.handle_key(key);

        assert!(!list.state.open);
        assert_eq!(action, SearchableListAction::Cancelled);
    }

    // Test 18: handle_key() — Enter selects when open
    #[test]
    fn test_handle_key_enter_selects_when_open() {
        use crate::ui::theme::Theme;

        let item = create_test_item("Alice Smith");
        let item_id = item.id;
        let mut state = SearchableListState::new(vec![item]);
        state.open = true;
        let theme = Theme::default();

        let mut list = SearchableList::new(&mut state, &theme, "Test", true);

        let key = KeyEvent {
            code: KeyCode::Enter,
            modifiers: KeyModifiers::empty(),
            kind: KeyEventKind::Press,
            state: crossterm::event::KeyEventState::empty(),
        };

        let action = list.handle_key(key);

        match action {
            SearchableListAction::Selected(id, name) => {
                assert_eq!(id, item_id);
                assert_eq!(name, "Alice Smith");
            }
            _ => panic!("Should return Selected action"),
        }
        assert!(!list.state.open);
    }

    // Test 19: handle_key() — Char appends to query with fuzzy=true
    #[test]
    fn test_handle_key_char_appends_to_query_fuzzy() {
        use crate::ui::theme::Theme;

        let items = vec![
            create_test_item("David"),
            create_test_item("Bob"),
            create_test_item("Charles"),
        ];
        let mut state = SearchableListState::new(items);
        state.open = true;
        let theme = Theme::default();

        let mut list = SearchableList::new(&mut state, &theme, "Test", true);

        let key = KeyEvent {
            code: KeyCode::Char('d'),
            modifiers: KeyModifiers::empty(),
            kind: KeyEventKind::Press,
            state: crossterm::event::KeyEventState::empty(),
        };

        let action = list.handle_key(key);

        assert_eq!(list.state.query, "d");
        assert_eq!(action, SearchableListAction::None);
        assert_eq!(list.state.filtered.len(), 1); // "d" matches "David"
    }

    // Test 20: handle_key() — Backspace pops from query
    #[test]
    fn test_handle_key_backspace_pops_query() {
        use crate::ui::theme::Theme;

        let items = vec![create_test_item("Smith"), create_test_item("Jones")];
        let mut state = SearchableListState::new(items);
        state.open = true;
        state.query = "smi".to_string();
        state.filtered = vec![]; // Simulate filtered results
        let theme = Theme::default();

        let mut list = SearchableList::new(&mut state, &theme, "Test", true);

        let key = KeyEvent {
            code: KeyCode::Backspace,
            modifiers: KeyModifiers::empty(),
            kind: KeyEventKind::Press,
            state: crossterm::event::KeyEventState::empty(),
        };

        let action = list.handle_key(key);

        assert_eq!(list.state.query, "sm");
        assert_eq!(action, SearchableListAction::None);
    }

    // Test 21: handle_key() — Up/Down navigation
    #[test]
    fn test_handle_key_up_down_navigation() {
        use crate::ui::theme::Theme;

        let items = vec![
            create_test_item("Alice"),
            create_test_item("Bob"),
            create_test_item("Charlie"),
        ];
        let mut state = SearchableListState::new(items);
        state.open = true;
        let theme = Theme::default();

        let mut list = SearchableList::new(&mut state, &theme, "Test", true);

        // Initial selection is index 0
        assert_eq!(list.state.scrollable.selected_index(), 0);

        // Move down
        let down_key = KeyEvent {
            code: KeyCode::Down,
            modifiers: KeyModifiers::empty(),
            kind: KeyEventKind::Press,
            state: crossterm::event::KeyEventState::empty(),
        };
        list.handle_key(down_key);
        assert_eq!(list.state.scrollable.selected_index(), 1);

        // Move up
        let up_key = KeyEvent {
            code: KeyCode::Up,
            modifiers: KeyModifiers::empty(),
            kind: KeyEventKind::Press,
            state: crossterm::event::KeyEventState::empty(),
        };
        list.handle_key(up_key);
        assert_eq!(list.state.scrollable.selected_index(), 0);
    }

    // Test 22: is_open() returns correct state
    #[test]
    fn test_is_open() {
        let items = vec![create_test_item("Alice")];
        let mut state = SearchableListState::new(items);

        assert!(!state.is_open());
        state.open();
        assert!(state.is_open());
        state.close();
        assert!(!state.is_open());
    }
}

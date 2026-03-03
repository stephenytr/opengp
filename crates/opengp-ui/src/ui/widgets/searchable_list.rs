use std::marker::PhantomData;

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
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
                    .bg(Color::Black),
            )
            .title(self.label);

        block.clone().render(area, buf);

        let inner = block.inner(area);
        if inner.height < 3 {
            return;
        }

        buf.set_style(inner, Style::default().bg(Color::Black));

        let query_area = Rect::new(inner.x, inner.y, inner.width, 3);
        buf.set_string(
            query_area.x + 1,
            query_area.y + 1,
            format!("Search: {}_", self.state.query),
            Style::default()
                .fg(self.theme.colors.foreground)
                .bg(Color::Black),
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
                    .bg(Color::Black),
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
                    .bg(Color::Black)
            } else {
                Style::default()
                    .fg(self.theme.colors.foreground)
                    .bg(Color::Black)
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

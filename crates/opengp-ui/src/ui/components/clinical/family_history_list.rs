use crate::ui::layout::HEADER_HEIGHT;
use crate::ui::theme::Theme;
use crate::ui::widgets::LoadingState;
use crossterm::event::{KeyEvent, MouseEvent, MouseEventKind};
use opengp_domain::domain::clinical::FamilyHistory;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Position, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Row, Table, Widget};

pub struct FamilyHistoryList {
    pub entries: Vec<FamilyHistory>,
    pub selected_index: usize,
    pub scroll_offset: usize,
    pub loading: bool,
    loading_state: LoadingState,
    theme: Theme,
}

impl Clone for FamilyHistoryList {
    fn clone(&self) -> Self {
        Self {
            entries: self.entries.clone(),
            selected_index: self.selected_index,
            scroll_offset: self.scroll_offset,
            loading: self.loading,
            loading_state: self.loading_state.clone(),
            theme: self.theme.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum FamilyHistoryListAction {
    Select(usize),
    Open(FamilyHistory),
    New,
    Delete(FamilyHistory),
}

impl FamilyHistoryList {
    pub fn new(theme: Theme) -> Self {
        Self {
            entries: Vec::new(),
            selected_index: 0,
            scroll_offset: 0,
            loading: false,
            loading_state: LoadingState::new().message("Loading family history..."),
            theme,
        }
    }

    pub fn with_entries(entries: Vec<FamilyHistory>, theme: Theme) -> Self {
        Self {
            entries,
            selected_index: 0,
            scroll_offset: 0,
            loading: false,
            loading_state: LoadingState::new().message("Loading family history..."),
            theme,
        }
    }

    pub fn selected(&self) -> Option<&FamilyHistory> {
        self.entries.get(self.selected_index)
    }

    pub fn selected_id(&self) -> Option<uuid::Uuid> {
        self.selected().map(|e| e.id)
    }

    pub fn select(&mut self, index: usize) {
        if index < self.entries.len() {
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
        if self.selected_index + 1 < self.entries.len() {
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
        if self.selected_index < self.entries.len().saturating_sub(1) {
            self.selected_index += 1;
        }
    }

    pub fn move_first(&mut self) {
        self.selected_index = 0;
    }

    pub fn move_last(&mut self) {
        self.selected_index = self.entries.len().saturating_sub(1);
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

    pub fn has_selection(&self) -> bool {
        !self.entries.is_empty()
    }

    pub fn count(&self) -> usize {
        self.entries.len()
    }

    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Option<FamilyHistoryListAction> {
        use crossterm::event::{KeyCode, KeyEventKind};

        if key.kind != KeyEventKind::Press {
            return None;
        }

        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.move_up();
                Some(FamilyHistoryListAction::Select(self.selected_index))
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.move_down();
                Some(FamilyHistoryListAction::Select(self.selected_index))
            }
            KeyCode::Home => {
                self.move_first();
                Some(FamilyHistoryListAction::Select(self.selected_index))
            }
            KeyCode::End => {
                self.move_last();
                Some(FamilyHistoryListAction::Select(self.selected_index))
            }
            KeyCode::PageUp => {
                let new_index = self.selected_index.saturating_sub(10);
                self.selected_index = new_index;
                Some(FamilyHistoryListAction::Select(self.selected_index))
            }
            KeyCode::PageDown => {
                let new_index =
                    (self.selected_index + 10).min(self.entries.len().saturating_sub(1));
                self.selected_index = new_index;
                Some(FamilyHistoryListAction::Select(self.selected_index))
            }
            KeyCode::Enter => {
                if let Some(entry) = self.selected() {
                    Some(FamilyHistoryListAction::Open(entry.clone()))
                } else {
                    None
                }
            }
            KeyCode::Char('n') => Some(FamilyHistoryListAction::New),
            KeyCode::Char('d') => {
                if let Some(entry) = self.selected() {
                    Some(FamilyHistoryListAction::Delete(entry.clone()))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub fn handle_mouse(
        &mut self,
        mouse: MouseEvent,
        area: Rect,
    ) -> Option<FamilyHistoryListAction> {
        if let MouseEventKind::ScrollUp = mouse.kind {
            if self.scroll_offset > 0 {
                self.scroll_offset = self.scroll_offset.saturating_sub(3).max(0);
            }
            return Some(FamilyHistoryListAction::Select(self.selected_index));
        }
        if let MouseEventKind::ScrollDown = mouse.kind {
            let visible_rows = area.height.saturating_sub(3) as usize;
            let max_scroll = self.entries.len().saturating_sub(visible_rows);
            self.scroll_offset = (self.scroll_offset + 3).min(max_scroll);
            return Some(FamilyHistoryListAction::Select(self.selected_index));
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
        if actual_index < self.entries.len() {
            self.selected_index = actual_index;
            Some(FamilyHistoryListAction::Select(self.selected_index))
        } else {
            None
        }
    }
}

const COL_CONDITION: u16 = 25;
const COL_RELATIONSHIP: u16 = 20;
const COL_AGE: u16 = 10;
const COL_NOTES: u16 = 30;

fn format_condition(entry: &FamilyHistory) -> String {
    entry.condition.clone()
}

fn format_relationship(entry: &FamilyHistory) -> String {
    entry.relative_relationship.clone()
}

fn format_age(entry: &FamilyHistory) -> String {
    entry
        .age_at_diagnosis
        .map(|a| format!("{} years", a))
        .unwrap_or_else(|| "-".to_string())
}

fn format_notes(entry: &FamilyHistory) -> String {
    entry
        .notes
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

impl Widget for FamilyHistoryList {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.is_empty() {
            return;
        }

        let block = Block::default()
            .title(" Family History ")
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

        if self.entries.is_empty() {
            let message = "No family history found. Press n to add an entry.";
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
            Constraint::Length(COL_CONDITION),
            Constraint::Length(COL_RELATIONSHIP),
            Constraint::Length(COL_AGE),
            Constraint::Length(COL_NOTES),
        ];

        let header = Row::new(vec!["Condition", "Relationship", "Age at Dx", "Notes"])
            .style(Style::default().fg(self.theme.colors.primary).bold());

        let visible_rows = inner.height as usize;
        let max_scroll = self.entries.len().saturating_sub(visible_rows);
        let scroll_offset = self.scroll_offset.min(max_scroll);

        let rows: Vec<Row> = self
            .entries
            .iter()
            .skip(scroll_offset)
            .take(visible_rows)
            .enumerate()
            .map(|(i, entry)| {
                let actual_index = scroll_offset + i;
                let style = if actual_index == self.selected_index {
                    Style::default()
                        .bg(self.theme.colors.selected)
                        .fg(self.theme.colors.foreground)
                } else {
                    Style::default().fg(self.theme.colors.foreground)
                };

                Row::new(vec![
                    format_condition(entry),
                    format_relationship(entry),
                    format_age(entry),
                    format_notes(entry),
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

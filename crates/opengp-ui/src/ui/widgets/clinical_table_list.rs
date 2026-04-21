use std::cmp::Ordering;

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, MouseButton, MouseEvent, MouseEventKind};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Position, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Row, Table, Widget};

const TABLE_HEADER_ROWS: u16 = 1;
use crate::ui::input::DoubleClickDetector;
use crate::ui::shared::{hover_style, selected_hover_style};
use crate::ui::theme::Theme;
use crate::ui::widgets::LoadingIndicator;

/// Configuration for a single column in a [`ClinicalTableList`].
pub struct ColumnDef<T> {
    /// Column header text.
    pub title: &'static str,
    /// Fixed width for this column in terminal cells.
    pub width: u16,
    /// Function that renders a row item into the cell text for this column.
    pub render: Box<dyn Fn(&T) -> String>,
}

/// Actions that can be produced by interacting with a [`ClinicalTableList`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ListAction<T> {
    /// A different row was selected by index.
    Select(usize),
    /// The currently selected row was opened.
    Open(T),
    /// The user requested to create a new item.
    New,
    /// The user requested to edit the selected item.
    Edit(T),
    /// The user requested to delete the selected item.
    Delete(T),
    /// The user toggled whether inactive items are shown.
    ToggleInactive,
    /// User right-clicked on a row - show context menu
    ContextMenu { index: usize, x: u16, y: u16 },
}

/// Generic table widget used for clinical lists such as allergies or medical history.
pub struct ClinicalTableList<T> {
    pub items: Vec<T>,
    pub columns: Vec<ColumnDef<T>>,
    pub selected_index: usize,
    pub scroll_offset: usize,
    pub theme: Theme,
    pub title: String,
    pub loading: bool,
    pub empty_message: String,
    /// Tracks the hovered row index for visual feedback
    pub hovered_index: Option<usize>,
    /// Detects double-click interactions on rows
    pub double_click_detector: DoubleClickDetector,
}

impl<T> ClinicalTableList<T> {
    #[allow(clippy::type_complexity)]
    /// Creates a new clinical table list from items and column definitions.
    ///
    /// An optional sort function can be supplied to control the initial
    /// ordering of items.
    pub fn new(
        mut items: Vec<T>,
        columns: Vec<ColumnDef<T>>,
        theme: Theme,
        title: impl Into<String>,
        sort_fn: Option<Box<dyn Fn(&T, &T) -> Ordering>>,
    ) -> Self {
        if let Some(sort_fn) = sort_fn {
            items.sort_by(|a, b| sort_fn(a, b));
        }

        Self {
            items,
            columns,
            selected_index: 0,
            scroll_offset: 0,
            theme,
            title: title.into(),
            loading: false,
            empty_message: "No entries found. Press n to add an entry.".to_string(),
            hovered_index: None,
            double_click_detector: DoubleClickDetector::default(),
        }
    }

    /// Moves the selection up by one row if possible.
    pub fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    /// Moves the selection down by one row if possible.
    pub fn move_down(&mut self) {
        if self.selected_index < self.items.len().saturating_sub(1) {
            self.selected_index += 1;
        }
    }

    /// Moves the selection to the first row.
    pub fn move_first(&mut self) {
        self.selected_index = 0;
    }

    /// Moves the selection to the last row.
    pub fn move_last(&mut self) {
        self.selected_index = self.items.len().saturating_sub(1);
    }

    /// Adjusts the scroll offset so the selected row stays within view.
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

    /// Handles keyboard navigation and commands for the list.
    ///
    /// Returns a [`ListAction`] that the caller can use to drive the rest of
    /// the UI, or `None` when the key is ignored.
    pub fn handle_key(&mut self, _key: KeyEvent) -> Option<ListAction<T>>
    where
        T: Clone,
    {
        let key = _key;

        if key.kind != KeyEventKind::Press {
            return None;
        }

        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.move_up();
                self.adjust_scroll(10);
                Some(ListAction::Select(self.selected_index))
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.move_down();
                self.adjust_scroll(10);
                Some(ListAction::Select(self.selected_index))
            }
            KeyCode::Home => {
                self.move_first();
                self.adjust_scroll(10);
                Some(ListAction::Select(self.selected_index))
            }
            KeyCode::End => {
                self.move_last();
                self.adjust_scroll(10);
                Some(ListAction::Select(self.selected_index))
            }
            KeyCode::PageUp => {
                self.selected_index = self.selected_index.saturating_sub(10);
                self.adjust_scroll(10);
                Some(ListAction::Select(self.selected_index))
            }
            KeyCode::PageDown => {
                self.selected_index =
                    (self.selected_index + 10).min(self.items.len().saturating_sub(1));
                self.adjust_scroll(10);
                Some(ListAction::Select(self.selected_index))
            }
            KeyCode::Enter => self
                .items
                .get(self.selected_index)
                .cloned()
                .map(ListAction::Open),
            KeyCode::Char('n') => Some(ListAction::New),
            KeyCode::Char('e') => self
                .items
                .get(self.selected_index)
                .cloned()
                .map(ListAction::Edit),
            KeyCode::Char('d') => self
                .items
                .get(self.selected_index)
                .cloned()
                .map(ListAction::Delete),
            KeyCode::Char('i') => Some(ListAction::ToggleInactive),
            _ => None,
        }
    }

    /// Handles mouse scrolling and click selection inside the list area.
    ///
    /// Returns a [`ListAction`] when the mouse event changes selection,
    /// or `None` for events that are outside the list or ignored.
    pub fn handle_mouse(&mut self, mouse: MouseEvent, area: Rect) -> Option<ListAction<T>>
    where
        T: Clone,
    {
        if let MouseEventKind::ScrollUp = mouse.kind {
            if self.scroll_offset > 0 {
                self.scroll_offset = self.scroll_offset.saturating_sub(3);
            }
            self.hovered_index = None;
            return Some(ListAction::Select(self.selected_index));
        }

        if let MouseEventKind::ScrollDown = mouse.kind {
            let visible_rows = area.height.saturating_sub(3) as usize;
            let max_scroll = self.items.len().saturating_sub(visible_rows);
            self.scroll_offset = (self.scroll_offset + 3).min(max_scroll);
            self.hovered_index = None;
            return Some(ListAction::Select(self.selected_index));
        }

        // Track hover state on mouse movement
        if let MouseEventKind::Moved = mouse.kind {
            if area.contains(Position::new(mouse.column, mouse.row))
                && mouse.row >= area.y + TABLE_HEADER_ROWS
            {
                let row_index = (mouse.row - area.y - TABLE_HEADER_ROWS) as usize;
                let actual_index = self.scroll_offset + row_index;
                if actual_index < self.items.len() {
                    self.hovered_index = Some(actual_index);
                } else {
                    self.hovered_index = None;
                }
            } else {
                self.hovered_index = None;
            }
            return None;
        }

        // Handle right-click for context menu
        if let MouseEventKind::Down(MouseButton::Right) = mouse.kind {
            if !area.contains(Position::new(mouse.column, mouse.row)) {
                return None;
            }

            if mouse.row < area.y + TABLE_HEADER_ROWS {
                return None;
            }

            let row_index = (mouse.row - area.y - TABLE_HEADER_ROWS) as usize;
            let actual_index = self.scroll_offset + row_index;
            if actual_index < self.items.len() {
                self.selected_index = actual_index;
                return Some(ListAction::ContextMenu {
                    index: actual_index,
                    x: mouse.column,
                    y: mouse.row,
                });
            }
            return None;
        }

        // Handle double-click for open action
        if let MouseEventKind::Down(MouseButton::Left) = mouse.kind {
            if !area.contains(Position::new(mouse.column, mouse.row)) {
                return None;
            }

            if mouse.row < area.y + TABLE_HEADER_ROWS {
                return None;
            }

            let row_index = (mouse.row - area.y - TABLE_HEADER_ROWS) as usize;
            let actual_index = self.scroll_offset + row_index;

            if actual_index >= self.items.len() {
                return None;
            }

            // Check for double-click
            if self.double_click_detector.check_double_click_now(&mouse) {
                if let Some(item) = self.items.get(actual_index).cloned() {
                    return Some(ListAction::Open(item));
                }
            }
            return None;
        }

        // Only process left mouse up for normal selection
        if mouse.kind != MouseEventKind::Up(MouseButton::Left) {
            return None;
        }

        if !area.contains(Position::new(mouse.column, mouse.row)) {
            return None;
        }

        if mouse.row < area.y + TABLE_HEADER_ROWS {
            return None;
        }

        let row_index = (mouse.row - area.y - TABLE_HEADER_ROWS) as usize;
        let actual_index = self.scroll_offset + row_index;
        if actual_index < self.items.len() {
            self.selected_index = actual_index;
            Some(ListAction::Select(self.selected_index))
        } else {
            None
        }
    }
}

impl<T> Widget for ClinicalTableList<T> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.is_empty() {
            return;
        }

        let block = Block::default()
            .title(format!(" {} ", self.title))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.colors.border));

        block.clone().render(area, buf);

        let inner = block.inner(area);
        if inner.is_empty() {
            return;
        }

        if self.loading {
            let indicator = LoadingIndicator::new(self.theme.clone()).message("Loading...");
            indicator.render(inner, buf);
            return;
        }

        if self.items.is_empty() {
            let message = self.empty_message;
            let text = Line::from(vec![Span::styled(
                message.clone(),
                Style::default().fg(self.theme.colors.disabled),
            )]);
            let x = inner.x + (inner.width.saturating_sub(message.len() as u16)) / 2;
            let y = inner.y + inner.height / 2;
            buf.set_line(x, y, &text, inner.width);
            return;
        }

        let col_widths: Vec<Constraint> = self
            .columns
            .iter()
            .map(|col| Constraint::Length(col.width))
            .collect();

        let header_cells = self.columns.iter().map(|col| col.title);
        let header = Row::new(header_cells)
            .style(Style::default().fg(self.theme.colors.primary).bold())
            .height(1);

        let visible_rows = inner.height.saturating_sub(1) as usize;
        let max_scroll = self.items.len().saturating_sub(visible_rows);
        let scroll_offset = self.scroll_offset.min(max_scroll);

        let rows: Vec<Row> = self
            .items
            .iter()
            .skip(scroll_offset)
            .take(visible_rows)
            .enumerate()
            .map(|(i, item)| {
                let actual_index = scroll_offset + i;
                let is_selected = actual_index == self.selected_index;
                let is_hovered = self.hovered_index == Some(actual_index);

                let style = match (is_selected, is_hovered) {
                    (true, true) => selected_hover_style(&self.theme),
                    (true, false) => Style::default()
                        .bg(self.theme.colors.selected)
                        .fg(self.theme.colors.foreground),
                    (false, true) => hover_style(&self.theme),
                    (false, false) => Style::default().fg(self.theme.colors.foreground),
                };

                let cells = self
                    .columns
                    .iter()
                    .map(|col| (col.render)(item))
                    .collect::<Vec<String>>();

                Row::new(cells).style(style).height(1)
            })
            .collect();

        let table = Table::new(rows, col_widths.clone())
            .header(header)
            .block(Block::default().borders(Borders::NONE))
            .widths(col_widths);

        table.render(inner, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct TestItem {
        id: u32,
        name: &'static str,
    }

    fn make_items() -> Vec<TestItem> {
        vec![
            TestItem {
                id: 3,
                name: "Gamma",
            },
            TestItem {
                id: 1,
                name: "Alpha",
            },
            TestItem {
                id: 2,
                name: "Beta",
            },
        ]
    }

    fn make_columns() -> Vec<ColumnDef<TestItem>> {
        vec![
            ColumnDef {
                title: "ID",
                width: 5,
                render: Box::new(|i| i.id.to_string()),
            },
            ColumnDef {
                title: "Name",
                width: 20,
                render: Box::new(|i| i.name.to_string()),
            },
        ]
    }

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, crossterm::event::KeyModifiers::NONE)
    }

    fn click(x: u16, y: u16) -> MouseEvent {
        MouseEvent {
            kind: MouseEventKind::Up(MouseButton::Left),
            column: x,
            row: y,
            modifiers: crossterm::event::KeyModifiers::NONE,
        }
    }

    #[test]
    fn clinical_table_list_renders_table_with_configurable_columns() {
        let mut terminal = Terminal::new(TestBackend::new(60, 8)).unwrap();
        let list = ClinicalTableList::new(
            make_items(),
            make_columns(),
            Theme::dark(),
            "Test",
            Some(Box::new(|a: &TestItem, b: &TestItem| a.id.cmp(&b.id))),
        );

        terminal
            .draw(|f| {
                let rect = f.area();
                f.render_widget(list, rect);
            })
            .unwrap();

        let rendered = format!("{:?}", terminal.backend());
        assert!(rendered.contains("ID"));
        assert!(rendered.contains("Name"));
        assert!(rendered.contains("Alpha"));
        assert!(rendered.contains("Beta"));
        assert!(rendered.contains("Gamma"));
    }

    #[test]
    fn clinical_table_list_key_navigation_and_actions_work() {
        let mut list =
            ClinicalTableList::new(make_items(), make_columns(), Theme::dark(), "Test", None);

        assert!(matches!(
            list.handle_key(key(KeyCode::Down)),
            Some(ListAction::Select(1))
        ));
        assert_eq!(list.selected_index, 1);

        assert!(matches!(
            list.handle_key(key(KeyCode::Up)),
            Some(ListAction::Select(0))
        ));
        assert_eq!(list.selected_index, 0);

        assert!(matches!(
            list.handle_key(key(KeyCode::End)),
            Some(ListAction::Select(2))
        ));
        assert_eq!(list.selected_index, 2);

        assert!(matches!(
            list.handle_key(key(KeyCode::Home)),
            Some(ListAction::Select(0))
        ));
        assert_eq!(list.selected_index, 0);

        list.selected_index = 2;
        assert!(matches!(
            list.handle_key(key(KeyCode::PageUp)),
            Some(ListAction::Select(0))
        ));

        list.selected_index = 0;
        assert!(matches!(
            list.handle_key(key(KeyCode::PageDown)),
            Some(ListAction::Select(2))
        ));

        let open = list.handle_key(key(KeyCode::Enter));
        assert!(matches!(
            open,
            Some(ListAction::Open(TestItem { id: 2, .. }))
        ));

        assert!(matches!(
            list.handle_key(key(KeyCode::Char('n'))),
            Some(ListAction::New)
        ));
    }

    #[test]
    fn clinical_table_list_mouse_scroll_and_click_work() {
        let mut list = ClinicalTableList::new(
            (0..30).map(|n| TestItem { id: n, name: "X" }).collect(),
            make_columns(),
            Theme::dark(),
            "Test",
            None,
        );

        let area = Rect::new(0, 0, 40, 8);

        let scroll_down = MouseEvent {
            kind: MouseEventKind::ScrollDown,
            column: 1,
            row: 4,
            modifiers: crossterm::event::KeyModifiers::NONE,
        };
        let scroll_up = MouseEvent {
            kind: MouseEventKind::ScrollUp,
            column: 1,
            row: 4,
            modifiers: crossterm::event::KeyModifiers::NONE,
        };

        assert!(matches!(
            list.handle_mouse(scroll_down, area),
            Some(ListAction::Select(0))
        ));
        assert!(list.scroll_offset > 0);

        assert!(matches!(
            list.handle_mouse(scroll_up, area),
            Some(ListAction::Select(0))
        ));

        assert!(matches!(
            list.handle_mouse(click(2, 4), area),
            Some(ListAction::Select(_))
        ));
        assert_eq!(list.selected_index, list.scroll_offset + 3);
    }

    #[test]
    fn clinical_table_list_highlights_selected_row() {
        let mut terminal = Terminal::new(TestBackend::new(50, 8)).unwrap();
        let mut list =
            ClinicalTableList::new(make_items(), make_columns(), Theme::dark(), "Test", None);
        list.selected_index = 1;
        let selected_bg = list.theme.colors.selected;

        terminal
            .draw(|f| {
                let rect = f.area();
                f.render_widget(list, rect);
            })
            .unwrap();

        let selected_row_y = 3u16;
        let has_selected_bg = (0..50u16).any(|x| {
            terminal
                .backend()
                .buffer()
                .cell((x, selected_row_y))
                .is_some_and(|cell| cell.style().bg == Some(selected_bg))
        });
        assert!(has_selected_bg);
    }

    #[test]
    fn clinical_table_list_shows_loading_state() {
        let mut terminal = Terminal::new(TestBackend::new(60, 8)).unwrap();
        let mut list =
            ClinicalTableList::new(make_items(), make_columns(), Theme::dark(), "Test", None);
        list.loading = true;

        terminal
            .draw(|f| {
                let rect = f.area();
                f.render_widget(list, rect);
            })
            .unwrap();

        let rendered = format!("{:?}", terminal.backend());
        assert!(rendered.contains("Loading"));
    }

    #[test]
    fn clinical_table_list_shows_empty_state_message() {
        let mut terminal = Terminal::new(TestBackend::new(60, 8)).unwrap();
        let mut list = ClinicalTableList::new(
            Vec::<TestItem>::new(),
            make_columns(),
            Theme::dark(),
            "Test",
            None,
        );
        list.empty_message = "No clinical entries".to_string();

        terminal
            .draw(|f| {
                let rect = f.area();
                f.render_widget(list, rect);
            })
            .unwrap();

        let rendered = format!("{:?}", terminal.backend());
        assert!(rendered.contains("No clinical entries"));
    }

    #[test]
    fn clinical_table_list_ignores_non_press_key_events() {
        let mut list =
            ClinicalTableList::new(make_items(), make_columns(), Theme::dark(), "Test", None);
        let mut key = key(KeyCode::Down);
        key.kind = KeyEventKind::Release;
        assert_eq!(list.handle_key(key), None::<ListAction<TestItem>>);
    }
}

use std::cmp::Ordering;
use std::rc::Rc;

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, MouseButton, MouseEvent, MouseEventKind};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Position, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Row, Table, Widget};

use crate::ui::input::DoubleClickDetector;
use crate::ui::shared::{hover_style, invert_color, selected_hover_style};
use crate::ui::theme::Theme;
use crate::ui::widgets::{LoadingIndicator, SCROLL_LINES};

pub struct UnifiedColumnDef<T> {
    pub title: &'static str,
    pub width: u16,
    pub render: Rc<dyn Fn(&T) -> String>,
}

impl<T> UnifiedColumnDef<T> {
    pub fn new(title: &'static str, width: u16, render: impl Fn(&T) -> String + 'static) -> Self {
        Self {
            title,
            width,
            render: Rc::new(render),
        }
    }
}

impl<T> Clone for UnifiedColumnDef<T> {
    fn clone(&self) -> Self {
        Self {
            title: self.title,
            width: self.width,
            render: self.render.clone(),
        }
    }
}

pub enum UnifiedListAction<T> {
    Select(usize),
    Open(T),
    New,
    Edit(T),
    Delete(T),
    ToggleInactive,
    ContextMenu { index: usize, x: u16, y: u16 },
}

impl<T: Clone> Clone for UnifiedListAction<T> {
    fn clone(&self) -> Self {
        match self {
            UnifiedListAction::Select(i) => UnifiedListAction::Select(*i),
            UnifiedListAction::Open(t) => UnifiedListAction::Open(t.clone()),
            UnifiedListAction::New => UnifiedListAction::New,
            UnifiedListAction::Edit(t) => UnifiedListAction::Edit(t.clone()),
            UnifiedListAction::Delete(t) => UnifiedListAction::Delete(t.clone()),
            UnifiedListAction::ToggleInactive => UnifiedListAction::ToggleInactive,
            UnifiedListAction::ContextMenu { index, x, y } => UnifiedListAction::ContextMenu { index: *index, x: *x, y: *y },
        }
    }
}

impl<T: PartialEq> PartialEq for UnifiedListAction<T> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (UnifiedListAction::Select(a), UnifiedListAction::Select(b)) => a == b,
            (UnifiedListAction::Open(a), UnifiedListAction::Open(b)) => a == b,
            (UnifiedListAction::New, UnifiedListAction::New) => true,
            (UnifiedListAction::Edit(a), UnifiedListAction::Edit(b)) => a == b,
            (UnifiedListAction::Delete(a), UnifiedListAction::Delete(b)) => a == b,
            (UnifiedListAction::ToggleInactive, UnifiedListAction::ToggleInactive) => true,
            (UnifiedListAction::ContextMenu { index: ia, x: xa, y: ya }, UnifiedListAction::ContextMenu { index: ib, x: xb, y: yb }) => ia == ib && xa == xb && ya == yb,
            _ => false,
        }
    }
}

impl<T: Eq> Eq for UnifiedListAction<T> {}

pub struct UnifiedListConfig<T> {
    pub title: String,
    pub header_rows: u16,
    pub empty_message: String,
    pub sort_fn: Option<Box<dyn Fn(&T, &T) -> Ordering>>,
}

impl<T> UnifiedListConfig<T> {
    pub fn new(title: impl Into<String>, header_rows: u16, empty_message: impl Into<String>) -> UnifiedListConfig<T> {
        UnifiedListConfig {
            title: title.into(),
            header_rows,
            empty_message: empty_message.into(),
            sort_fn: None,
        }
    }

    pub fn with_sort(mut self, sort_fn: impl Fn(&T, &T) -> Ordering + 'static) -> Self {
        self.sort_fn = Some(Box::new(sort_fn));
        self
    }
}

pub struct UnifiedList<T: Clone> {
    pub items: Vec<T>,
    pub columns: Vec<UnifiedColumnDef<T>>,
    pub selected_index: usize,
    pub scroll_offset: usize,
    pub theme: Theme,
    pub loading: bool,
    pub hovered_index: Option<usize>,
    pub double_click_detector: DoubleClickDetector,
    pub config: UnifiedListConfig<T>,
}

impl<T: Clone> UnifiedList<T> {
    pub fn new(
        mut items: Vec<T>,
        columns: Vec<UnifiedColumnDef<T>>,
        theme: Theme,
        config: UnifiedListConfig<T>,
    ) -> Self {
        if let Some(ref sort_fn) = config.sort_fn {
            items.sort_by(|a, b| sort_fn(a, b));
        }
        Self {
            items,
            columns,
            selected_index: 0,
            scroll_offset: 0,
            theme,
            loading: false,
            hovered_index: None,
            double_click_detector: DoubleClickDetector::default(),
            config,
        }
    }

    pub fn with_sorted(mut self, sort_fn: impl Fn(&T, &T) -> Ordering + 'static) -> Self {
        self.items.sort_by(|a, b| sort_fn(a, b));
        self
    }

    pub fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    pub fn move_down(&mut self) {
        if self.selected_index < self.items.len().saturating_sub(1) {
            self.selected_index += 1;
        }
    }

    pub fn move_first(&mut self) {
        self.selected_index = 0;
    }

    pub fn move_last(&mut self) {
        self.selected_index = self.items.len().saturating_sub(1);
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

    pub fn next(&mut self) {
        self.move_down();
    }

    pub fn prev(&mut self) {
        self.move_up();
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Option<UnifiedListAction<T>> {
        if key.kind != KeyEventKind::Press {
            return None;
        }

        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.move_up();
                self.adjust_scroll(10);
                Some(UnifiedListAction::Select(self.selected_index))
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.move_down();
                self.adjust_scroll(10);
                Some(UnifiedListAction::Select(self.selected_index))
            }
            KeyCode::Home => {
                self.move_first();
                self.adjust_scroll(10);
                Some(UnifiedListAction::Select(self.selected_index))
            }
            KeyCode::End => {
                self.move_last();
                self.adjust_scroll(10);
                Some(UnifiedListAction::Select(self.selected_index))
            }
            KeyCode::PageUp => {
                self.selected_index = self.selected_index.saturating_sub(10);
                self.adjust_scroll(10);
                Some(UnifiedListAction::Select(self.selected_index))
            }
            KeyCode::PageDown => {
                self.selected_index = (self.selected_index + 10).min(self.items.len().saturating_sub(1));
                self.adjust_scroll(10);
                Some(UnifiedListAction::Select(self.selected_index))
            }
            KeyCode::Enter => self
                .items
                .get(self.selected_index)
                .cloned()
                .map(UnifiedListAction::Open),
            KeyCode::Char('n') => Some(UnifiedListAction::New),
            KeyCode::Char('e') => self
                .items
                .get(self.selected_index)
                .cloned()
                .map(UnifiedListAction::Edit),
            KeyCode::Char('d') => self
                .items
                .get(self.selected_index)
                .cloned()
                .map(UnifiedListAction::Delete),
            KeyCode::Char('i') => Some(UnifiedListAction::ToggleInactive),
            _ => None,
        }
    }

    pub fn handle_mouse(&mut self, mouse: MouseEvent, area: Rect) -> Option<UnifiedListAction<T>> {
        if let MouseEventKind::ScrollUp = mouse.kind {
            if self.scroll_offset > 0 {
                self.scroll_offset = self.scroll_offset.saturating_sub(SCROLL_LINES);
            }
            self.hovered_index = None;
            return Some(UnifiedListAction::Select(self.selected_index));
        }

        if let MouseEventKind::ScrollDown = mouse.kind {
            let visible_rows = area.height.saturating_sub(self.config.header_rows).max(1) as usize;
            let max_scroll = self.items.len().saturating_sub(visible_rows);
            self.scroll_offset = (self.scroll_offset + SCROLL_LINES).min(max_scroll);
            self.hovered_index = None;
            return Some(UnifiedListAction::Select(self.selected_index));
        }

        if let MouseEventKind::Moved = mouse.kind {
            if area.contains(Position::new(mouse.column, mouse.row))
                && mouse.row >= area.y + self.config.header_rows
            {
                let row_index = (mouse.row - area.y - self.config.header_rows) as usize;
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

        if let MouseEventKind::Down(MouseButton::Right) = mouse.kind {
            if !area.contains(Position::new(mouse.column, mouse.row)) {
                return None;
            }
            if mouse.row < area.y + self.config.header_rows {
                return None;
            }
            let row_index = (mouse.row - area.y - self.config.header_rows) as usize;
            let actual_index = self.scroll_offset + row_index;
            if actual_index < self.items.len() {
                self.selected_index = actual_index;
                return Some(UnifiedListAction::ContextMenu {
                    index: actual_index,
                    x: mouse.column,
                    y: mouse.row,
                });
            }
            return None;
        }

        if let MouseEventKind::Down(MouseButton::Left) = mouse.kind {
            if !area.contains(Position::new(mouse.column, mouse.row)) {
                return None;
            }
            if mouse.row < area.y + self.config.header_rows {
                return None;
            }
            let row_index = (mouse.row - area.y - self.config.header_rows) as usize;
            let actual_index = self.scroll_offset + row_index;
            if actual_index >= self.items.len() {
                return None;
            }
            if self.double_click_detector.check_double_click_now(&mouse) {
                if let Some(item) = self.items.get(actual_index).cloned() {
                    return Some(UnifiedListAction::Open(item));
                }
            }
            return None;
        }

        if mouse.kind != MouseEventKind::Up(MouseButton::Left) {
            return None;
        }

        if !area.contains(Position::new(mouse.column, mouse.row)) {
            return None;
        }
        if mouse.row < area.y + self.config.header_rows {
            return None;
        }

        let row_index = (mouse.row - area.y - self.config.header_rows) as usize;
        let actual_index = self.scroll_offset + row_index;
        if actual_index < self.items.len() {
            self.selected_index = actual_index;
            Some(UnifiedListAction::Select(self.selected_index))
        } else {
            None
        }
    }
}

impl<T: Clone> Widget for UnifiedList<T> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.is_empty() {
            return;
        }

        let block = Block::default()
            .title(format!(" {} ", self.config.title))
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
            let message = self.config.empty_message.clone();
            let msg_len = message.len() as u16;
            let text = Line::from(vec![Span::styled(
                message,
                Style::default().fg(self.theme.colors.disabled),
            )]);
            let x = inner.x + (inner.width.saturating_sub(msg_len)) / 2;
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

        let visible_rows = inner.height.saturating_sub(self.config.header_rows).max(1) as usize;
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
                    (true, false) => {
                        let bg = self.theme.colors.selected;
                        Style::default().bg(bg).fg(invert_color(bg))
                    }
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
            TestItem { id: 3, name: "Gamma" },
            TestItem { id: 1, name: "Alpha" },
            TestItem { id: 2, name: "Beta" },
        ]
    }

    fn make_columns() -> Vec<UnifiedColumnDef<TestItem>> {
        vec![
            UnifiedColumnDef { title: "ID", width: 5, render: Rc::new(|i: &TestItem| i.id.to_string()) },
            UnifiedColumnDef { title: "Name", width: 20, render: Rc::new(|i: &TestItem| i.name.to_string()) },
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
    fn unified_list_renders_table_with_configurable_columns() {
        let mut terminal = Terminal::new(TestBackend::new(60, 8)).unwrap();
        let items = make_items();
        let columns = make_columns();
        let list = UnifiedList::new(
            items,
            columns,
            Theme::dark(),
            UnifiedListConfig::new("Test", 1, "No items"),
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
    fn unified_list_key_navigation_and_actions_work() {
        let items = make_items();
        let columns = make_columns();
        let mut list = UnifiedList::new(
            items,
            columns,
            Theme::dark(),
            UnifiedListConfig::new("Test", 1, "No items"),
        );

        assert!(matches!(
            list.handle_key(key(KeyCode::Down)),
            Some(UnifiedListAction::Select(1))
        ));
        assert_eq!(list.selected_index, 1);

        assert!(matches!(
            list.handle_key(key(KeyCode::Up)),
            Some(UnifiedListAction::Select(0))
        ));
        assert_eq!(list.selected_index, 0);

        assert!(matches!(
            list.handle_key(key(KeyCode::End)),
            Some(UnifiedListAction::Select(2))
        ));
        assert_eq!(list.selected_index, 2);

        assert!(matches!(
            list.handle_key(key(KeyCode::Home)),
            Some(UnifiedListAction::Select(0))
        ));
        assert_eq!(list.selected_index, 0);

let open = list.handle_key(key(KeyCode::Enter));
        assert!(matches!(
            open,
            Some(UnifiedListAction::Open(TestItem { id: 3, .. }))
        ));

        assert!(matches!(
            list.handle_key(key(KeyCode::Char('n'))),
            Some(UnifiedListAction::New)
        ));
    }

    #[test]
    fn unified_list_mouse_scroll_and_click_work() {
        let mut list = UnifiedList::new(
            (0..30).map(|n| TestItem { id: n, name: "X" }).collect(),
            make_columns(),
            Theme::dark(),
            UnifiedListConfig::new("Test", 1, "No items"),
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
            Some(UnifiedListAction::Select(0))
        ));
        assert!(list.scroll_offset > 0);

        assert!(matches!(
            list.handle_mouse(scroll_up, area),
            Some(UnifiedListAction::Select(0))
        ));

        assert!(matches!(
            list.handle_mouse(click(2, 3), area),
            Some(UnifiedListAction::Select(_))
        ));
        assert_eq!(list.selected_index, list.scroll_offset + 2);
    }

    #[test]
    fn unified_list_highlights_selected_row() {
        let mut terminal = Terminal::new(TestBackend::new(50, 8)).unwrap();
        let mut list = UnifiedList::new(
            make_items(),
            make_columns(),
            Theme::dark(),
            UnifiedListConfig::new("Test", 1, "No items"),
        );
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
}
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
            KeyCode::Enter => {
                self.selected().map(|allergy| AllergyListAction::Open(allergy.clone()))
            }
            KeyCode::Char('n') => Some(AllergyListAction::New),
            KeyCode::Char('i') => {
                self.toggle_inactive();
                Some(AllergyListAction::ToggleInactive)
            }
            KeyCode::Char('d') => {
                self.selected().map(|allergy| AllergyListAction::Delete(allergy.clone()))
            }
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

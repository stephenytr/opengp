use crate::domain::clinical::{ConditionStatus, MedicalHistory, Severity};
use crate::ui::layout::HEADER_HEIGHT;
use crate::ui::theme::Theme;
use crate::ui::widgets::LoadingState;
use crossterm::event::{KeyEvent, MouseEvent, MouseEventKind};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Position, Rect};
use ratatui::prelude::Stylize;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Row, Table, Widget};

#[derive(Debug, Clone, Default)]
pub enum HistoryFilter {
    #[default]
    All,
    Active,
    Resolved,
}

pub struct MedicalHistoryList {
    pub conditions: Vec<MedicalHistory>,
    pub selected_index: usize,
    pub filter: HistoryFilter,
    pub scroll_offset: usize,
    pub loading: bool,
    loading_state: LoadingState,
    theme: Theme,
}

impl Clone for MedicalHistoryList {
    fn clone(&self) -> Self {
        Self {
            conditions: self.conditions.clone(),
            selected_index: self.selected_index,
            filter: self.filter.clone(),
            scroll_offset: self.scroll_offset,
            loading: self.loading,
            loading_state: self.loading_state.clone(),
            theme: self.theme.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum MedicalHistoryListAction {
    Select(usize),
    Open(MedicalHistory),
    New,
    SetFilter(HistoryFilter),
    Delete(MedicalHistory),
}

impl MedicalHistoryList {
    pub fn new(theme: Theme) -> Self {
        Self {
            conditions: Vec::new(),
            selected_index: 0,
            filter: HistoryFilter::All,
            scroll_offset: 0,
            loading: false,
            loading_state: LoadingState::new().message("Loading medical history..."),
            theme,
        }
    }

    pub fn with_conditions(conditions: Vec<MedicalHistory>, theme: Theme) -> Self {
        Self {
            conditions,
            selected_index: 0,
            filter: HistoryFilter::All,
            scroll_offset: 0,
            loading: false,
            loading_state: LoadingState::new().message("Loading medical history..."),
            theme,
        }
    }

    pub fn selected(&self) -> Option<&MedicalHistory> {
        self.conditions.get(self.selected_index)
    }

    pub fn selected_id(&self) -> Option<uuid::Uuid> {
        self.selected().map(|c| c.id)
    }

    pub fn select(&mut self, index: usize) {
        if index < self.conditions.len() {
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
        if self.selected_index + 1 < self.conditions.len() {
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
        if self.selected_index < self.conditions.len().saturating_sub(1) {
            self.selected_index += 1;
        }
    }

    pub fn move_first(&mut self) {
        self.selected_index = 0;
    }

    pub fn move_last(&mut self) {
        self.selected_index = self.conditions.len().saturating_sub(1);
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

    pub fn set_filter(&mut self, filter: HistoryFilter) {
        self.filter = filter;
        self.selected_index = 0;
    }

    pub fn filtered_conditions(&self) -> Vec<&MedicalHistory> {
        match self.filter {
            HistoryFilter::All => self.conditions.iter().collect(),
            HistoryFilter::Active => self
                .conditions
                .iter()
                .filter(|c| {
                    matches!(
                        c.status,
                        ConditionStatus::Active
                            | ConditionStatus::Chronic
                            | ConditionStatus::Recurring
                    )
                })
                .collect(),
            HistoryFilter::Resolved => self
                .conditions
                .iter()
                .filter(|c| {
                    matches!(
                        c.status,
                        ConditionStatus::Resolved | ConditionStatus::InRemission
                    )
                })
                .collect(),
        }
    }

    pub fn has_selection(&self) -> bool {
        !self.conditions.is_empty()
    }

    pub fn count(&self) -> usize {
        self.conditions.len()
    }

    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Option<MedicalHistoryListAction> {
        use crossterm::event::{KeyCode, KeyEventKind};

        if key.kind != KeyEventKind::Press {
            return None;
        }

        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.move_up();
                Some(MedicalHistoryListAction::Select(self.selected_index))
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.move_down();
                Some(MedicalHistoryListAction::Select(self.selected_index))
            }
            KeyCode::Home => {
                self.move_first();
                Some(MedicalHistoryListAction::Select(self.selected_index))
            }
            KeyCode::End => {
                self.move_last();
                Some(MedicalHistoryListAction::Select(self.selected_index))
            }
            KeyCode::PageUp => {
                let new_index = self.selected_index.saturating_sub(10);
                self.selected_index = new_index;
                Some(MedicalHistoryListAction::Select(self.selected_index))
            }
            KeyCode::PageDown => {
                let new_index =
                    (self.selected_index + 10).min(self.conditions.len().saturating_sub(1));
                self.selected_index = new_index;
                Some(MedicalHistoryListAction::Select(self.selected_index))
            }
            KeyCode::Enter => {
                if let Some(condition) = self.selected() {
                    Some(MedicalHistoryListAction::Open(condition.clone()))
                } else {
                    None
                }
            }
            KeyCode::Char('n') => Some(MedicalHistoryListAction::New),
            KeyCode::Char('f') => {
                self.filter = match self.filter {
                    HistoryFilter::All => HistoryFilter::Active,
                    HistoryFilter::Active => HistoryFilter::Resolved,
                    HistoryFilter::Resolved => HistoryFilter::All,
                };
                self.selected_index = 0;
                Some(MedicalHistoryListAction::SetFilter(self.filter.clone()))
            }
            KeyCode::Char('d') => {
                if let Some(condition) = self.selected() {
                    Some(MedicalHistoryListAction::Delete(condition.clone()))
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
    ) -> Option<MedicalHistoryListAction> {
        if let MouseEventKind::ScrollUp = mouse.kind {
            if self.scroll_offset > 0 {
                self.scroll_offset = self.scroll_offset.saturating_sub(3).max(0);
            }
            return Some(MedicalHistoryListAction::Select(self.selected_index));
        }
        if let MouseEventKind::ScrollDown = mouse.kind {
            let visible_rows = area.height.saturating_sub(3) as usize;
            let max_scroll = self.conditions.len().saturating_sub(visible_rows);
            self.scroll_offset = (self.scroll_offset + 3).min(max_scroll);
            return Some(MedicalHistoryListAction::Select(self.selected_index));
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
        if actual_index < self.conditions.len() {
            self.selected_index = actual_index;
            Some(MedicalHistoryListAction::Select(self.selected_index))
        } else {
            None
        }
    }
}

const COL_CONDITION: u16 = 25;
const COL_DIAGNOSIS_DATE: u16 = 12;
const COL_STATUS: u16 = 12;
const COL_SEVERITY: u16 = 10;
const COL_NOTES: u16 = 25;

fn format_condition(condition: &MedicalHistory) -> String {
    condition.condition.clone()
}

fn format_diagnosis_date(condition: &MedicalHistory) -> String {
    condition
        .diagnosis_date
        .map(|d| d.format("%d/%m/%Y").to_string())
        .unwrap_or_else(|| "-".to_string())
}

fn format_status(condition: &MedicalHistory) -> String {
    match condition.status {
        ConditionStatus::Active => "Active".to_string(),
        ConditionStatus::Resolved => "Resolved".to_string(),
        ConditionStatus::Chronic => "Chronic".to_string(),
        ConditionStatus::Recurring => "Recurring".to_string(),
        ConditionStatus::InRemission => "In Remission".to_string(),
    }
}

fn format_severity(condition: &MedicalHistory) -> String {
    match condition.severity {
        Some(Severity::Mild) => "Mild".to_string(),
        Some(Severity::Moderate) => "Moderate".to_string(),
        Some(Severity::Severe) => "Severe".to_string(),
        None => "-".to_string(),
    }
}

fn format_notes(condition: &MedicalHistory) -> String {
    condition
        .notes
        .as_ref()
        .map(|s| {
            if s.len() > 23 {
                format!("{}...", &s[..23])
            } else {
                s.clone()
            }
        })
        .unwrap_or_else(|| "-".to_string())
}

impl Widget for MedicalHistoryList {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.is_empty() {
            return;
        }

        let block = Block::default()
            .title(" Medical History ")
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

        if self.conditions.is_empty() {
            let message = "No medical history found. Press n to add a condition.";
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
            Constraint::Length(COL_DIAGNOSIS_DATE),
            Constraint::Length(COL_STATUS),
            Constraint::Length(COL_SEVERITY),
            Constraint::Length(COL_NOTES),
        ];

        let header = Row::new(vec![
            "Condition",
            "Diagnosis",
            "Status",
            "Severity",
            "Notes",
        ])
        .style(Style::default().fg(self.theme.colors.primary).bold());

        let visible_rows = inner.height as usize;
        let max_scroll = self.conditions.len().saturating_sub(visible_rows);
        let scroll_offset = self.scroll_offset.min(max_scroll);

        let rows: Vec<Row> = self
            .conditions
            .iter()
            .skip(scroll_offset)
            .take(visible_rows)
            .enumerate()
            .map(|(i, condition)| {
                let actual_index = scroll_offset + i;
                let style = if actual_index == self.selected_index {
                    Style::default()
                        .bg(self.theme.colors.selected)
                        .fg(self.theme.colors.foreground)
                } else {
                    Style::default().fg(self.theme.colors.foreground)
                };

                Row::new(vec![
                    format_condition(condition),
                    format_diagnosis_date(condition),
                    format_status(condition),
                    format_severity(condition),
                    format_notes(condition),
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

use crate::ui::layout::HEADER_HEIGHT;
use crate::ui::theme::Theme;
use crate::ui::widgets::LoadingState;
use crossterm::event::{KeyEvent, MouseEvent, MouseEventKind};
use opengp_domain::domain::clinical::VitalSigns;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Position, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Row, Table, Widget};

pub struct VitalSignsList {
    pub vitals: Vec<VitalSigns>,
    pub selected_index: usize,
    pub scroll_offset: usize,
    pub loading: bool,
    loading_state: LoadingState,
    theme: Theme,
}

impl Clone for VitalSignsList {
    fn clone(&self) -> Self {
        Self {
            vitals: self.vitals.clone(),
            selected_index: self.selected_index,
            scroll_offset: self.scroll_offset,
            loading: self.loading,
            loading_state: self.loading_state.clone(),
            theme: self.theme.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum VitalSignsListAction {
    Select(usize),
    Open(VitalSigns),
    New,
    NextPage,
    PrevPage,
}

impl VitalSignsList {
    pub fn new(theme: Theme) -> Self {
        Self {
            vitals: Vec::new(),
            selected_index: 0,
            scroll_offset: 0,
            loading: false,
            loading_state: LoadingState::new().message("Loading vital signs..."),
            theme,
        }
    }

    pub fn with_vitals(vitals: Vec<VitalSigns>, theme: Theme) -> Self {
        let mut sorted = vitals;
        sorted.sort_by(|a, b| b.measured_at.cmp(&a.measured_at));
        Self {
            vitals: sorted,
            selected_index: 0,
            scroll_offset: 0,
            loading: false,
            loading_state: LoadingState::new().message("Loading vital signs..."),
            theme,
        }
    }

    pub fn selected(&self) -> Option<&VitalSigns> {
        self.vitals.get(self.selected_index)
    }

    pub fn selected_id(&self) -> Option<uuid::Uuid> {
        self.selected().map(|v| v.id)
    }

    pub fn select(&mut self, index: usize) {
        if index < self.vitals.len() {
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
        if self.selected_index + 1 < self.vitals.len() {
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
        if self.selected_index < self.vitals.len().saturating_sub(1) {
            self.selected_index += 1;
        }
    }

    pub fn move_first(&mut self) {
        self.selected_index = 0;
    }

    pub fn move_last(&mut self) {
        self.selected_index = self.vitals.len().saturating_sub(1);
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
        !self.vitals.is_empty()
    }

    pub fn count(&self) -> usize {
        self.vitals.len()
    }

    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Option<VitalSignsListAction> {
        use crossterm::event::{KeyCode, KeyEventKind};

        if key.kind != KeyEventKind::Press {
            return None;
        }

        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.move_up();
                Some(VitalSignsListAction::Select(self.selected_index))
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.move_down();
                Some(VitalSignsListAction::Select(self.selected_index))
            }
            KeyCode::Home => {
                self.move_first();
                Some(VitalSignsListAction::Select(self.selected_index))
            }
            KeyCode::End => {
                self.move_last();
                Some(VitalSignsListAction::Select(self.selected_index))
            }
            KeyCode::PageUp => {
                let new_index = self.selected_index.saturating_sub(10);
                self.selected_index = new_index;
                Some(VitalSignsListAction::Select(self.selected_index))
            }
            KeyCode::PageDown => {
                let new_index = (self.selected_index + 10).min(self.vitals.len().saturating_sub(1));
                self.selected_index = new_index;
                Some(VitalSignsListAction::Select(self.selected_index))
            }
            KeyCode::Enter => {
                if let Some(vitals) = self.selected() {
                    Some(VitalSignsListAction::Open(vitals.clone()))
                } else {
                    None
                }
            }
            KeyCode::Char('n') => Some(VitalSignsListAction::New),
            KeyCode::Char('+') | KeyCode::Char('=') => Some(VitalSignsListAction::NextPage),
            KeyCode::Char('-') => Some(VitalSignsListAction::PrevPage),
            _ => None,
        }
    }

    pub fn handle_mouse(&mut self, mouse: MouseEvent, area: Rect) -> Option<VitalSignsListAction> {
        if let MouseEventKind::ScrollUp = mouse.kind {
            if self.scroll_offset > 0 {
                self.scroll_offset = self.scroll_offset.saturating_sub(3).max(0);
            }
            return Some(VitalSignsListAction::Select(self.selected_index));
        }
        if let MouseEventKind::ScrollDown = mouse.kind {
            let visible_rows = area.height.saturating_sub(3) as usize;
            let max_scroll = self.vitals.len().saturating_sub(visible_rows);
            self.scroll_offset = (self.scroll_offset + 3).min(max_scroll);
            return Some(VitalSignsListAction::Select(self.selected_index));
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
        if actual_index < self.vitals.len() {
            self.selected_index = actual_index;
            Some(VitalSignsListAction::Select(self.selected_index))
        } else {
            None
        }
    }
}

const COL_DATE: u16 = 12;
const COL_BP: u16 = 14;
const COL_HR: u16 = 8;
const COL_RR: u16 = 8;
const COL_TEMP: u16 = 8;
const COL_SPO2: u16 = 8;
const COL_BMI: u16 = 8;

fn format_date(vitals: &VitalSigns) -> String {
    vitals.measured_at.format("%d/%m/%Y").to_string()
}

fn format_bp(vitals: &VitalSigns) -> String {
    match (vitals.systolic_bp, vitals.diastolic_bp) {
        (Some(sys), Some(dia)) => format!("{}/{}", sys, dia),
        _ => "-".to_string(),
    }
}

fn format_hr(vitals: &VitalSigns) -> String {
    vitals
        .heart_rate
        .map(|h| format!("{} bpm", h))
        .unwrap_or_else(|| "-".to_string())
}

fn format_rr(vitals: &VitalSigns) -> String {
    vitals
        .respiratory_rate
        .map(|r| format!("{} /min", r))
        .unwrap_or_else(|| "-".to_string())
}

fn format_temp(vitals: &VitalSigns) -> String {
    vitals
        .temperature
        .map(|t| format!("{:.1}°C", t))
        .unwrap_or_else(|| "-".to_string())
}

fn format_spo2(vitals: &VitalSigns) -> String {
    vitals
        .oxygen_saturation
        .map(|s| format!("{}%", s))
        .unwrap_or_else(|| "-".to_string())
}

fn format_bmi(vitals: &VitalSigns) -> String {
    vitals
        .bmi
        .map(|b| format!("{:.1}", b))
        .unwrap_or_else(|| "-".to_string())
}

impl Widget for VitalSignsList {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.is_empty() {
            return;
        }

        let block = Block::default()
            .title(" Vital Signs ")
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

        if self.vitals.is_empty() {
            let message = "No vital signs recorded. Press n to add new vitals.";
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
            Constraint::Length(COL_DATE),
            Constraint::Length(COL_BP),
            Constraint::Length(COL_HR),
            Constraint::Length(COL_RR),
            Constraint::Length(COL_TEMP),
            Constraint::Length(COL_SPO2),
            Constraint::Length(COL_BMI),
        ];

        let header = Row::new(vec!["Date", "BP", "HR", "RR", "Temp", "SpO2", "BMI"])
            .style(Style::default().fg(self.theme.colors.primary).bold());

        let visible_rows = inner.height as usize;
        let max_scroll = self.vitals.len().saturating_sub(visible_rows);
        let scroll_offset = self.scroll_offset.min(max_scroll);

        let rows: Vec<Row> = self
            .vitals
            .iter()
            .skip(scroll_offset)
            .take(visible_rows)
            .enumerate()
            .map(|(i, vitals)| {
                let actual_index = scroll_offset + i;
                let style = if actual_index == self.selected_index {
                    Style::default()
                        .bg(self.theme.colors.selected)
                        .fg(self.theme.colors.foreground)
                } else {
                    Style::default().fg(self.theme.colors.foreground)
                };

                Row::new(vec![
                    format_date(vitals),
                    format_bp(vitals),
                    format_hr(vitals),
                    format_rr(vitals),
                    format_temp(vitals),
                    format_spo2(vitals),
                    format_bmi(vitals),
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

//! Help modal widget - displays context-aware keyboard shortcuts
//!
//! This modal shows all available keybinds for the current context with descriptions.

use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table};
use ratatui::Frame;

use crate::ui::keybinds::{KeybindContext, KeybindRegistry};

/// Help modal that displays keyboard shortcuts for a specific context
pub struct HelpModal {
    context: KeybindContext,
    scroll_offset: usize,
}

impl HelpModal {
    /// Create a new help modal for the given context
    pub fn new(context: KeybindContext) -> Self {
        Self {
            context,
            scroll_offset: 0,
        }
    }

    /// Scroll the help content up (decrease offset)
    pub fn scroll_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
    }

    /// Scroll the help content down (increase offset)
    pub fn scroll_down(&mut self, max_visible_lines: usize, total_keybinds: usize) {
        if self.scroll_offset + max_visible_lines < total_keybinds {
            self.scroll_offset += 1;
        }
    }

    /// Get the context name for display
    fn get_context_name(&self) -> &'static str {
        match self.context {
            KeybindContext::Global => "Global",
            KeybindContext::PatientList => "Patient List",
            KeybindContext::PatientListSearch => "Patient Search",
            KeybindContext::PatientForm => "Patient Form",
            KeybindContext::AppointmentList => "Appointment List",
            KeybindContext::CalendarMonthView => "Calendar - Month View",
            KeybindContext::CalendarDayView => "Calendar - Day View",
            KeybindContext::CalendarWeekView => "Calendar - Week View",
            KeybindContext::CalendarMultiSelect => "Calendar - Multi-Select",
            KeybindContext::CalendarDetailModal => "Appointment Details",
            KeybindContext::CalendarRescheduleModal => "Reschedule Appointment",
            KeybindContext::CalendarSearchModal => "Search Appointments",
            KeybindContext::CalendarFilterMenu => "Filter by Status",
            KeybindContext::CalendarPractitionerMenu => "Filter by Practitioner",
            KeybindContext::CalendarAuditModal => "Audit History",
            KeybindContext::CalendarConfirmation => "Confirmation",
            KeybindContext::CalendarErrorModal => "Error",
            KeybindContext::CalendarBatchMenu => "Batch Operations",
            KeybindContext::AppointmentForm => "Appointment Form",
            KeybindContext::AppointmentFormPatient => "Select Patient",
        }
    }

    /// Render the help modal as a centered overlay
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        // Calculate centered modal area: 60% width, 80% height
        let modal_width = (area.width as f32 * 0.6) as u16;
        let modal_height = (area.height as f32 * 0.8) as u16;
        let modal_x = (area.width.saturating_sub(modal_width)) / 2;
        let modal_y = (area.height.saturating_sub(modal_height)) / 2;

        let modal_area = Rect {
            x: area.x + modal_x,
            y: area.y + modal_y,
            width: modal_width,
            height: modal_height,
        };

        // Render semi-transparent background
        self.render_overlay_background(frame, area);

        // Get keybinds for the current context
        let keybinds = KeybindRegistry::get_keybinds(self.context.clone());

        // Build table rows
        let header = Row::new(vec![
            Cell::from("Key").style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Cell::from("Description").style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        ])
        .height(1)
        .style(Style::default().bg(Color::DarkGray));

        let mut rows: Vec<Row> = Vec::new();

        // Only show implemented keybinds
        let visible_keybinds: Vec<_> = keybinds.iter().filter(|kb| kb.implemented).collect();

        for kb in visible_keybinds.iter().skip(self.scroll_offset) {
            let key_str = KeybindRegistry::format_key(&kb.key, kb.modifiers);
            let row = Row::new(vec![
                Cell::from(key_str).style(Style::default().fg(Color::Cyan)),
                Cell::from(kb.action).style(Style::default().fg(Color::White)),
            ])
            .height(1);
            rows.push(row);
        }

        let widths = [Constraint::Length(20), Constraint::Min(30)];

        // Build title with context name
        let context_name = self.get_context_name();
        let title = format!(" Keyboard Shortcuts - {} ", context_name);

        // Build footer with instructions
        let footer = self.build_footer(visible_keybinds.len());

        let table = Table::new(rows, widths).header(header).block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(Style::default().fg(Color::Yellow)),
        );

        // Render the table
        frame.render_widget(table, modal_area);

        // Render footer below the table if there's space
        if modal_area.height > 2 {
            let footer_area = Rect {
                x: modal_area.x,
                y: modal_area.y + modal_area.height - 1,
                width: modal_area.width,
                height: 1,
            };
            frame.render_widget(footer, footer_area);
        }
    }

    /// Render semi-transparent background overlay
    fn render_overlay_background(&self, frame: &mut Frame, area: Rect) {
        let bg_style = Style::default().bg(Color::Black);
        let lines: Vec<Line> = (0..area.height)
            .map(|_| Line::from(Span::styled(" ".repeat(area.width as usize), bg_style)))
            .collect();

        let bg = Paragraph::new(lines);
        frame.render_widget(bg, area);
    }

    /// Build the footer paragraph with navigation instructions
    fn build_footer(&self, total_visible: usize) -> Paragraph<'static> {
        let footer_text = if total_visible > 10 {
            "↑↓: Scroll  Esc/?: Close"
        } else {
            "Esc/?: Close"
        };

        Paragraph::new(footer_text).style(
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
    }
}

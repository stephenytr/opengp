use async_trait::async_trait;
use chrono::{Duration, Utc};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Cell, Row, Table, TableState};
use ratatui::Frame;
use std::sync::Arc;

use crate::components::{Action, Component};
use crate::domain::appointment::{
    AppointmentSearchCriteria, AppointmentService, CalendarAppointment,
};
use crate::error::Result;
use crate::ui::keybinds::{KeybindContext, KeybindRegistry};
use crate::ui::widgets::HelpModal;

pub struct AppointmentListComponent {
    appointment_service: Arc<AppointmentService>,
    appointments: Vec<CalendarAppointment>,
    table_state: TableState,
    showing_help_modal: bool,
}

impl AppointmentListComponent {
    pub fn new(appointment_service: Arc<AppointmentService>) -> Self {
        let mut table_state = TableState::default();
        table_state.select(Some(0));

        Self {
            appointment_service,
            appointments: Vec::new(),
            table_state,
            showing_help_modal: false,
        }
    }

    fn next(&mut self) {
        if self.appointments.is_empty() {
            return;
        }
        let i = match self.table_state.selected() {
            Some(i) => {
                if i >= self.appointments.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    fn previous(&mut self) {
        if self.appointments.is_empty() {
            return;
        }
        let i = match self.table_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.appointments.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    fn select_first(&mut self) {
        if !self.appointments.is_empty() {
            self.table_state.select(Some(0));
        }
    }

    fn select_last(&mut self) {
        if !self.appointments.is_empty() {
            self.table_state.select(Some(self.appointments.len() - 1));
        }
    }
}

#[async_trait]
impl Component for AppointmentListComponent {
    async fn init(&mut self) -> Result<()> {
        let criteria = AppointmentSearchCriteria {
            patient_id: None,
            practitioner_id: None,
            date_from: Some(Utc::now() - Duration::days(7)),
            date_to: None,
            status: None,
            appointment_type: None,
            is_urgent: None,
            confirmed: None,
        };

        self.appointments = self
            .appointment_service
            .get_calendar_appointments(&criteria)
            .await
            .map_err(|e| crate::error::Error::App(format!("Failed to load appointments: {}", e)))?;

        Ok(())
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> Action {
        if self.showing_help_modal {
            match key.code {
                KeyCode::Esc | KeyCode::Char('?') => {
                    self.showing_help_modal = false;
                    return Action::Render;
                }
                _ => return Action::None,
            }
        }

        match key.code {
            KeyCode::Down | KeyCode::Char('j') => {
                self.next();
                Action::None
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.previous();
                Action::None
            }
            KeyCode::Char('g') => {
                self.select_first();
                Action::Render
            }
            KeyCode::Char('G') => {
                self.select_last();
                Action::Render
            }
            KeyCode::Char('n') => Action::AppointmentCreate,
            KeyCode::Char('?') => {
                self.showing_help_modal = true;
                Action::Render
            }
            _ => Action::None,
        }
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let header_cells = ["Date", "Time", "Patient", "Type", "Status"]
            .iter()
            .map(|h| {
                Cell::from(*h).style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
            });
        let header = Row::new(header_cells)
            .style(Style::default().bg(Color::DarkGray))
            .height(1);

        let rows = self.appointments.iter().map(|appointment| {
            let date = appointment.start_time.format("%Y-%m-%d").to_string();
            let time = appointment.start_time.format("%H:%M").to_string();
            let patient = appointment.patient_name.clone();
            let appt_type = format!("{}", appointment.appointment_type);
            let status = format!("{}", appointment.status);

            let cells = vec![
                Cell::from(date),
                Cell::from(time),
                Cell::from(patient),
                Cell::from(appt_type),
                Cell::from(status),
            ];
            Row::new(cells).height(1)
        });

        let widths = [
            Constraint::Length(12), // Date
            Constraint::Length(8),  // Time
            Constraint::Length(16), // Patient
            Constraint::Length(20), // Type
            Constraint::Length(12), // Status
        ];

        let help = KeybindRegistry::get_help_text(KeybindContext::AppointmentList);
        let title = format!(" Appointments - {} ", help);

        let table = Table::new(rows, widths)
            .header(header)
            .block(Block::default().borders(Borders::ALL).title(title))
            .row_highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");

        frame.render_stateful_widget(table, area, &mut self.table_state);

        if self.showing_help_modal {
            let help_modal = HelpModal::new(KeybindContext::AppointmentList);
            help_modal.render(frame, area);
        }
    }
}

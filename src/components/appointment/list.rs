use async_trait::async_trait;
use chrono::{Duration, Utc};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Cell, Row, Table, TableState};
use ratatui::Frame;
use std::sync::Arc;
use uuid::Uuid;

use crate::components::{Action, Component};
use crate::domain::appointment::{Appointment, AppointmentService, AppointmentType};
use crate::error::Result;

pub struct AppointmentListComponent {
    #[allow(dead_code)]
    appointment_service: Arc<AppointmentService>,
    appointments: Vec<Appointment>,
    table_state: TableState,
}

impl AppointmentListComponent {
    pub fn new(appointment_service: Arc<AppointmentService>) -> Self {
        let mut table_state = TableState::default();
        table_state.select(Some(0));
        
        Self {
            appointment_service,
            appointments: Vec::new(),
            table_state,
        }
    }

    fn generate_mock_appointments() -> Vec<Appointment> {
        let patient_id = Uuid::new_v4();
        let practitioner_id = Uuid::new_v4();
        
        vec![
            Appointment::new(
                patient_id,
                practitioner_id,
                Utc::now() + Duration::hours(2),
                Duration::minutes(15),
                AppointmentType::Standard,
                None,
            ),
            Appointment::new(
                Uuid::new_v4(),
                practitioner_id,
                Utc::now() + Duration::hours(4),
                Duration::minutes(30),
                AppointmentType::Long,
                None,
            ),
            Appointment::new(
                Uuid::new_v4(),
                practitioner_id,
                Utc::now() + Duration::days(1),
                Duration::minutes(45),
                AppointmentType::NewPatient,
                None,
            ),
        ]
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
}

#[async_trait]
impl Component for AppointmentListComponent {
    async fn init(&mut self) -> Result<()> {
        // Load mock data for now
        self.appointments = Self::generate_mock_appointments();
        Ok(())
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Down | KeyCode::Char('j') => {
                self.next();
                Action::None
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.previous();
                Action::None
            }
            KeyCode::Char('n') => Action::AppointmentCreate,
            _ => Action::None,
        }
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let header_cells = ["Date", "Time", "Patient", "Type", "Status"]
            .iter()
            .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
        let header = Row::new(header_cells)
            .style(Style::default().bg(Color::DarkGray))
            .height(1);

        let rows = self.appointments.iter().map(|appointment| {
            let date = appointment.start_time.format("%Y-%m-%d").to_string();
            let time = appointment.start_time.format("%H:%M").to_string();
            let patient = format!("{:.8}", appointment.patient_id.to_string());
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
            Constraint::Length(12),  // Date
            Constraint::Length(8),   // Time
            Constraint::Length(16),  // Patient
            Constraint::Length(20),  // Type
            Constraint::Length(12),  // Status
        ];

        let table = Table::new(rows, widths)
            .header(header)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Appointments (↑↓: Navigate, n: New) "),
            )
            .row_highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");

        frame.render_stateful_widget(table, area, &mut self.table_state);
    }
}

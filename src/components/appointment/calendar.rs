use async_trait::async_trait;
use chrono::{Datelike, Local, NaiveDate, Utc, Weekday};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState};
use ratatui::Frame;
use std::sync::Arc;
use uuid::Uuid;

use crate::components::{Action, Component};
use crate::domain::appointment::{Appointment, AppointmentService, AppointmentStatus};
use crate::domain::patient::{Patient, PatientService};
use crate::domain::user::{Practitioner, PractitionerService};
use crate::error::Result;
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FocusArea {
    MonthView,
    DayView,
}

/// View mode for the day schedule (single day or week view)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
enum ViewMode {
    Day,
    Week,
}

pub struct AppointmentCalendarComponent {
    appointment_service: Arc<AppointmentService>,
    practitioner_service: Arc<PractitionerService>,
    patient_service: Arc<PatientService>,
    
    current_date: NaiveDate,
    current_month_start: NaiveDate,
    practitioners: Vec<Practitioner>,
    appointments: Vec<Appointment>,
    
    focus_area: FocusArea,
    time_slot_state: TableState,
    selected_month_day: u32,
    #[allow(dead_code)]
    view_mode: ViewMode,
    #[allow(dead_code)]
    week_start_date: NaiveDate,
    
    // Modal state for appointment details
    selected_appointment: Option<Uuid>,
    showing_detail_modal: bool,
    modal_patient: Option<Patient>,
}

impl AppointmentCalendarComponent {
    pub fn new(
        appointment_service: Arc<AppointmentService>,
        practitioner_service: Arc<PractitionerService>,
#[allow(dead_code)]
    patient_service: Arc<PatientService>,
    ) -> Self {
        let mut table_state = TableState::default();
        table_state.select(Some(0));
        
        let today = Local::now().date_naive();
        let month_start = NaiveDate::from_ymd_opt(
            today.year(),
            today.month(),
            1
        ).expect("first day of month is always valid");
        
        // Calculate week start date (Monday of current week)
        let weekday = today.weekday();
        let days_from_monday = weekday.num_days_from_monday();
        let week_start = today - chrono::Duration::days(days_from_monday as i64);
        
        Self {
            appointment_service,
            practitioner_service,
            patient_service,
            current_date: today,
            current_month_start: month_start,
            practitioners: Vec::new(),
            appointments: Vec::new(),
            focus_area: FocusArea::MonthView,
            time_slot_state: table_state,
            selected_month_day: today.day(),
            view_mode: ViewMode::Day,
            week_start_date: week_start,
            selected_appointment: None,
            showing_detail_modal: false,
            modal_patient: None,
        }
    }
    
    fn generate_time_slots() -> Vec<String> {
        let mut slots = Vec::new();
        for hour in 8..18 {
            for minute in [0, 15, 30, 45] {
                slots.push(format!("{:02}:{:02}", hour, minute));
            }
        }
        slots
    }
    
    fn previous_day(&mut self) {
        if self.selected_month_day > 1 {
            self.selected_month_day -= 1;
        }
    }
    
    fn next_day(&mut self) {
        let days_in_month = self.days_in_current_month();
        if self.selected_month_day < days_in_month {
            self.selected_month_day += 1;
        }
    }
    
    fn previous_week(&mut self) {
        if self.selected_month_day > 7 {
            self.selected_month_day -= 7;
        } else {
            self.selected_month_day = 1;
        }
    }
    
    fn next_week(&mut self) {
        let days_in_month = self.days_in_current_month();
        if self.selected_month_day + 7 <= days_in_month {
            self.selected_month_day += 7;
        } else {
            self.selected_month_day = days_in_month;
        }
    }
    
    fn previous_month(&mut self) {
        if self.current_month_start.month() == 1 {
            self.current_month_start = NaiveDate::from_ymd_opt(
                self.current_month_start.year() - 1,
                12,
                1
            ).expect("first day of month is always valid");
        } else {
            self.current_month_start = NaiveDate::from_ymd_opt(
                self.current_month_start.year(),
                self.current_month_start.month() - 1,
                1
            ).expect("first day of month is always valid");
        }
        
        let days_in_month = self.days_in_current_month();
        if self.selected_month_day > days_in_month {
            self.selected_month_day = days_in_month;
        }
    }
    
    fn next_month(&mut self) {
        if self.current_month_start.month() == 12 {
            self.current_month_start = NaiveDate::from_ymd_opt(
                self.current_month_start.year() + 1,
                1,
                1
            ).expect("first day of month is always valid");
        } else {
            self.current_month_start = NaiveDate::from_ymd_opt(
                self.current_month_start.year(),
                self.current_month_start.month() + 1,
                1
            ).expect("first day of month is always valid");
        }
        
        let days_in_month = self.days_in_current_month();
        if self.selected_month_day > days_in_month {
            self.selected_month_day = days_in_month;
        }
    }
    
    fn jump_to_today(&mut self) {
        let today = Local::now().date_naive();
        self.current_date = today;
        self.current_month_start = NaiveDate::from_ymd_opt(
            today.year(),
            today.month(),
            1
        ).expect("first day of month is always valid");
        self.selected_month_day = today.day();
    }
    
    fn days_in_current_month(&self) -> u32 {
        let month = self.current_month_start.month();
        let year = self.current_month_start.year();
        
        match month {
            2 => {
                if (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0) {
                    29
                } else {
                    28
                }
            }
            4 | 6 | 9 | 11 => 30,
            _ => 31,
        }
    }
    
    fn get_month_name(&self) -> &'static str {
        match self.current_month_start.month() {
            1 => "January",
            2 => "February", 
            3 => "March",
            4 => "April",
            5 => "May",
            6 => "June",
            7 => "July",
            8 => "August",
            9 => "September",
            10 => "October",
            11 => "November",
            12 => "December",
            _ => "Unknown",
        }
    }
    
    async fn load_appointments_for_date(&mut self) -> Result<()> {
        let date = NaiveDate::from_ymd_opt(
            self.current_month_start.year(),
            self.current_month_start.month(),
            self.selected_month_day,
        ).expect("valid date from selected day");
        
        match self.appointment_service.get_day_appointments(date, None).await {
            Ok(appointments) => {
                self.appointments = appointments;
                Ok(())
            }
            Err(e) => {
                tracing::error!("Failed to load appointments: {}", e);
                self.appointments = Vec::new();
                Ok(())
            }
        }
    }
    
    fn find_appointment_for_slot(
        &self,
        practitioner_id: uuid::Uuid,
        slot_index: usize,
    ) -> Option<&Appointment> {
        let time_slots = Self::generate_time_slots();
        if slot_index >= time_slots.len() {
            return None;
        }
        
        let slot_time_str = &time_slots[slot_index];
        let (hour, minute) = slot_time_str.split_once(':')
            .and_then(|(h, m)| {
                let hour = h.parse::<u32>().ok()?;
                let minute = m.parse::<u32>().ok()?;
                Some((hour, minute))
            })?;
        
        let date = NaiveDate::from_ymd_opt(
            self.current_month_start.year(),
            self.current_month_start.month(),
            self.selected_month_day,
        ).expect("valid date");
        
        let slot_datetime = date.and_hms_opt(hour, minute, 0)
            .expect("valid time")
            .and_utc();
        
        // Find appointment that starts at or before this slot and ends after it
        self.appointments.iter().find(|appt| {
            appt.practitioner_id == practitioner_id
                && appt.start_time <= slot_datetime
                && appt.end_time > slot_datetime
        })
    }
    

    
    fn previous_time_slot(&mut self) {
        let time_slots = Self::generate_time_slots();
        if let Some(selected) = self.time_slot_state.selected() {
            if selected > 0 {
                self.time_slot_state.select(Some(selected - 1));
            } else {
                self.time_slot_state.select(Some(time_slots.len() - 1));
            }
        }
    }
    
    fn next_time_slot(&mut self) {
        let time_slots = Self::generate_time_slots();
        if let Some(selected) = self.time_slot_state.selected() {
            if selected < time_slots.len() - 1 {
                self.time_slot_state.select(Some(selected + 1));
            } else {
                self.time_slot_state.select(Some(0));
            }
        }
    }
    
    fn render_month_calendar(&self, frame: &mut Frame, area: Rect) {
        let month_year = format!("{} {}", self.get_month_name(), self.current_month_start.year());
        
        let first_weekday = self.current_month_start.weekday();
        let days_in_month = self.days_in_current_month();
        
        let mut lines = vec![
            Line::from(vec![
                Span::styled("Mon", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(" "),
                Span::styled("Tue", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(" "),
                Span::styled("Wed", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(" "),
                Span::styled("Thu", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(" "),
                Span::styled("Fri", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(" "),
                Span::styled("Sat", Style::default().fg(Color::Cyan)),
                Span::raw(" "),
                Span::styled("Sun", Style::default().fg(Color::Cyan)),
            ])
        ];
        
        let mut current_day = 1;
        let _week_start = 0;
        
        let first_day_offset = match first_weekday {
            Weekday::Mon => 0,
            Weekday::Tue => 1,
            Weekday::Wed => 2,
            Weekday::Thu => 3,
            Weekday::Fri => 4,
            Weekday::Sat => 5,
            Weekday::Sun => 6,
        };
        
        let mut is_first_week = true;
        
        while current_day <= days_in_month {
            let mut day_cells = Vec::new();
            
            for day_of_week in 0..7 {
                if (is_first_week && day_of_week < first_day_offset) || current_day > days_in_month {
                    day_cells.push(Span::raw("   "));
                } else {
                    let is_today = self.current_date.year() == self.current_month_start.year()
                        && self.current_date.month() == self.current_month_start.month()
                        && self.current_date.day() == current_day;
                    
                    let is_selected = current_day == self.selected_month_day;
                    let is_weekend = day_of_week >= 5;
                    
                    let style = if is_selected {
                        Style::default().fg(Color::Black).bg(Color::Yellow).add_modifier(Modifier::BOLD)
                    } else if is_today {
                        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
                    } else if is_weekend {
                        Style::default().fg(Color::Cyan)
                    } else {
                        Style::default().fg(Color::White)
                    };
                    
                    day_cells.push(Span::styled(format!("{:2} ", current_day), style));
                    current_day += 1;
                }
            }
            
            lines.push(Line::from(day_cells));
            is_first_week = false;
        }
        
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("↑↓", Style::default().fg(Color::Cyan)),
            Span::raw(": Day  "),
            Span::styled("h/l", Style::default().fg(Color::Cyan)),
            Span::raw(": Month"),
        ]));
        lines.push(Line::from(vec![
            Span::styled("t", Style::default().fg(Color::Cyan)),
            Span::raw(": Today  "),
            Span::styled("n", Style::default().fg(Color::Cyan)),
            Span::raw(": New  "),
            Span::styled("Enter", Style::default().fg(Color::Cyan)),
            Span::raw(": Day View"),
        ]));
        
        let paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(" {} (h/l: Month, t: Today, n: New, Enter: Day View) ", month_year))
                    .border_style(
                        if self.focus_area == FocusArea::MonthView {
                            Style::default().fg(Color::Yellow)
                        } else {
                            Style::default().fg(Color::White)
                        }
                    ),
            );
        
        frame.render_widget(paragraph, area);
    }
    
    fn render_day_schedule(&mut self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
            ])
            .split(area);
        
        self.render_practitioner_header(frame, chunks[0]);
        self.render_time_slots_grid(frame, chunks[1]);
    }
    
    fn render_practitioner_header(&self, frame: &mut Frame, area: Rect) {
        let mut header_cells = vec![Cell::from("Time").style(
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        )];
        
        for practitioner in &self.practitioners {
            let name = format!("Dr. {}", practitioner.last_name);
            header_cells.push(
                Cell::from(name)
                    .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            );
        }
        
        let header = Row::new(header_cells)
            .style(Style::default().bg(Color::DarkGray))
            .height(1);
        
        let mut widths = vec![Constraint::Length(8)];
        for _ in &self.practitioners {
            widths.push(Constraint::Min(15));
        }
        
        let table = Table::new(vec![header], widths)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Schedule ")
                    .border_style(
                        if self.focus_area == FocusArea::DayView {
                            Style::default().fg(Color::Yellow)
                        } else {
                            Style::default().fg(Color::White)
                        }
                    ),
            );
        
        frame.render_widget(table, area);
    }
    
    fn render_time_slots_grid(&mut self, frame: &mut Frame, area: Rect) {
        let time_slots = Self::generate_time_slots();
        let mut rows = Vec::new();
        let mut rendered_appointments = HashSet::new();
        
        for (slot_index, time_slot) in time_slots.iter().enumerate() {
            let mut cells = vec![Cell::from(time_slot.as_str())];
            
            for practitioner in &self.practitioners {
                if let Some(appt) = self.find_appointment_for_slot(practitioner.id, slot_index) {
                    let appt_key = (appt.id, practitioner.id, slot_index);
                    
                    if !rendered_appointments.contains(&appt_key) {
                        let duration_minutes = (appt.end_time - appt.start_time).num_minutes();
                        let slot_span = (duration_minutes / 15).max(1) as usize;
                        
                        for i in 0..slot_span {
                            rendered_appointments.insert((appt.id, practitioner.id, slot_index + i));
                        }
                        
                        let patient_name = format!("Patient {}", &appt.patient_id.to_string()[..8]);
                        
                        let mut appt_text = format!("{}\n{}", patient_name, appt.appointment_type);
                        if appt.is_urgent {
                            appt_text = format!("⚠ {}", appt_text);
                        }
                        
                        let style = match appt.status {
                            AppointmentStatus::Scheduled => Style::default().fg(Color::White).bg(Color::Blue),
                            AppointmentStatus::Confirmed => Style::default().fg(Color::Black).bg(Color::Cyan),
                            AppointmentStatus::Arrived => Style::default().fg(Color::Black).bg(Color::Yellow),
                            AppointmentStatus::InProgress => Style::default().fg(Color::White).bg(Color::Green),
                            AppointmentStatus::Completed => Style::default().fg(Color::White).bg(Color::DarkGray),
                            AppointmentStatus::NoShow => Style::default().fg(Color::White).bg(Color::Red),
                            AppointmentStatus::Cancelled => Style::default().fg(Color::Gray),
                            AppointmentStatus::Rescheduled => Style::default().fg(Color::Magenta),
                        };
                        
                        cells.push(Cell::from(appt_text).style(style));
                    } else {
                        cells.push(Cell::from(""));
                    }
                } else {
                    cells.push(Cell::from(""));
                }
            }
            
            let row = Row::new(cells).height(2);
            rows.push(row);
        }
        
        let mut widths = vec![Constraint::Length(8)];
        for _ in &self.practitioners {
            widths.push(Constraint::Min(15));
        }
        
        let table = Table::new(rows, widths)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" (↑↓/j/k: Navigate, n: New, Tab/Esc: Month View) "),
            )
            .row_highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");
        
        frame.render_stateful_widget(table, area, &mut self.time_slot_state);
    }
    
    /// Handle key events when appointment detail modal is open
    fn handle_modal_key_events(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Esc => {
                self.showing_detail_modal = false;
                self.selected_appointment = None;
                self.modal_patient = None;
                Action::Render
            }
            KeyCode::Char('a') | KeyCode::Char('A') => Action::AppointmentMarkArrived,
            KeyCode::Char('c') | KeyCode::Char('C') => Action::AppointmentMarkCompleted,
            KeyCode::Char('n') | KeyCode::Char('N') => Action::AppointmentMarkNoShow,
            _ => Action::None,
        }
    }
    
    /// Render appointment detail modal as a centered overlay
    fn render_appointment_detail_modal(&mut self, frame: &mut Frame, area: Rect) {
        // Calculate centered modal area: 60% width, 70% height
        let modal_area = Rect {
            x: area.width / 5,
            y: area.height / 6,
            width: area.width * 3 / 5,
            height: area.height * 2 / 3,
        };
        
        // Get appointment and patient data
        let mut lines = Vec::new();
        
        if let Some(appt_id) = self.selected_appointment {
            if let Some(appt) = self.appointments.iter().find(|a| a.id == appt_id) {
                // Header: Patient name and appointment type
                let patient_name = if let Some(ref patient) = self.modal_patient {
                    format!("{} {}", patient.first_name, patient.last_name)
                } else {
                    "Loading...".to_string()
                };
                
                lines.push(Line::from(vec![
                    Span::styled("Patient: ", Style::default().fg(Color::Yellow)),
                    Span::styled(patient_name, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                ]));
                
                lines.push(Line::from(""));
                lines.push(Line::from(vec![
                    Span::styled("Type: ", Style::default().fg(Color::Yellow)),
                    Span::styled(format!("{:?}", appt.appointment_type), Style::default().fg(Color::White)),
                ]));
                
                // Status with color coding
                let status_style = match appt.status {
                    AppointmentStatus::Scheduled => Style::default().fg(Color::Blue),
                    AppointmentStatus::Confirmed => Style::default().fg(Color::Cyan),
                    AppointmentStatus::Arrived => Style::default().fg(Color::Yellow),
                    AppointmentStatus::InProgress => Style::default().fg(Color::Green),
                    AppointmentStatus::Completed => Style::default().fg(Color::DarkGray),
                    AppointmentStatus::NoShow => Style::default().fg(Color::Red),
                    AppointmentStatus::Cancelled => Style::default().fg(Color::Gray),
                    AppointmentStatus::Rescheduled => Style::default().fg(Color::Magenta),
                };
                
                lines.push(Line::from(vec![
                    Span::styled("Status: ", Style::default().fg(Color::Yellow)),
                    Span::styled(format!("{:?}", appt.status), status_style),
                ]));
                
                if appt.is_urgent {
                    lines.push(Line::from(vec![
                        Span::styled("⚠ URGENT", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                    ]));
                }
                
                lines.push(Line::from(""));
                
                // Time details
                lines.push(Line::from(vec![
                    Span::styled("Date: ", Style::default().fg(Color::Yellow)),
                    Span::styled(appt.start_time.format("%Y-%m-%d").to_string(), Style::default().fg(Color::White)),
                ]));
                
                lines.push(Line::from(vec![
                    Span::styled("Time: ", Style::default().fg(Color::Yellow)),
                    Span::styled(
                        format!("{} - {}", 
                            appt.start_time.format("%H:%M"),
                            appt.end_time.format("%H:%M")
                        ),
                        Style::default().fg(Color::White)
                    ),
                ]));
                
                lines.push(Line::from(""));
                
                // Practitioner
                if let Some(practitioner) = self.practitioners.iter().find(|p| p.id == appt.practitioner_id) {
                    lines.push(Line::from(vec![
                        Span::styled("Practitioner: ", Style::default().fg(Color::Yellow)),
                        Span::styled(format!("Dr. {}", practitioner.last_name), Style::default().fg(Color::White)),
                    ]));
                }
                
                // Reason
                if let Some(ref reason) = appt.reason {
                    if !reason.is_empty() {
                        lines.push(Line::from(""));
                        lines.push(Line::from(vec![
                            Span::styled("Reason: ", Style::default().fg(Color::Yellow)),
                            Span::styled(reason, Style::default().fg(Color::White)),
                        ]));
                    }
                }
                
                // Notes
                if let Some(ref notes) = appt.notes {
                    if !notes.is_empty() {
                        lines.push(Line::from(""));
                        lines.push(Line::from(vec![
                            Span::styled("Notes: ", Style::default().fg(Color::Yellow)),
                            Span::styled(notes, Style::default().fg(Color::White)),
                        ]));
                    }
                }
            }
        }
        
        // Footer with keyboard hints
        lines.push(Line::from(""));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("A", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(": Arrived  ", Style::default().fg(Color::White)),
            Span::styled("C", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(": Completed  ", Style::default().fg(Color::White)),
            Span::styled("N", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(": No Show  ", Style::default().fg(Color::White)),
            Span::styled("Esc", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(": Close", Style::default().fg(Color::White)),
        ]));
        
        let modal_content = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Appointment Details ")
                    .border_style(Style::default().fg(Color::Yellow))
            )
            .wrap(ratatui::widgets::Wrap { trim: true });
        
        frame.render_widget(modal_content, modal_area);
    }
}

#[async_trait]
impl Component for AppointmentCalendarComponent {
    async fn init(&mut self) -> Result<()> {
        match self.practitioner_service.get_active_practitioners().await {
            Ok(practitioners) => {
                self.practitioners = practitioners;
            }
            Err(_e) => {
                self.practitioners = vec![
                    Practitioner {
                        id: Uuid::new_v4(),
                        user_id: None,
                        first_name: "Sarah".to_string(),
                        middle_name: None,
                        last_name: "Johnson".to_string(),
                        title: "Dr".to_string(),
                        hpi_i: Some("8003610000000000".to_string()),
                        ahpra_registration: Some("MED0001234567".to_string()),
                        prescriber_number: Some("123456".to_string()),
                        provider_number: "123456A".to_string(),
                        speciality: Some("General Practice".to_string()),
                        qualifications: vec!["MBBS".to_string(), "FRACGP".to_string()],
                        phone: Some("02 9876 5432".to_string()),
                        email: Some("s.johnson@clinic.com".to_string()),
                        is_active: true,
                        created_at: Utc::now(),
                        updated_at: Utc::now(),
                    },
                    Practitioner {
                        id: Uuid::new_v4(),
                        user_id: None,
                        first_name: "Michael".to_string(),
                        middle_name: Some("James".to_string()),
                        last_name: "Chen".to_string(),
                        title: "Dr".to_string(),
                        hpi_i: Some("8003610000000001".to_string()),
                        ahpra_registration: Some("MED0001234568".to_string()),
                        prescriber_number: Some("234567".to_string()),
                        provider_number: "234567B".to_string(),
                        speciality: Some("General Practice".to_string()),
                        qualifications: vec!["MBBS".to_string(), "FRACGP".to_string()],
                        phone: Some("02 9876 5433".to_string()),
                        email: Some("m.chen@clinic.com".to_string()),
                        is_active: true,
                        created_at: Utc::now(),
                        updated_at: Utc::now(),
                    },
                ];
            }
        }
        
        self.load_appointments_for_date().await?;
        
        Ok(())
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> Action {
        // Check if modal is open and handle modal-specific keys
        if self.showing_detail_modal {
            return self.handle_modal_key_events(key);
        }
        
        match self.focus_area {
            FocusArea::MonthView => {
                match key.code {
                    KeyCode::Left => {
                        self.previous_day();
                        Action::Render
                    }
                    KeyCode::Right => {
                        self.next_day();
                        Action::Render
                    }
                    KeyCode::Up => {
                        self.previous_week();
                        Action::Render
                    }
                    KeyCode::Down => {
                        self.next_week();
                        Action::Render
                    }
                    KeyCode::Char('h') => {
                        self.previous_month();
                        Action::Render
                    }
                    KeyCode::Char('l') => {
                        self.next_month();
                        Action::Render
                    }
                    KeyCode::Char('t') => {
                        self.jump_to_today();
                        Action::Render
                    }
                    KeyCode::Enter => {
                        self.focus_area = FocusArea::DayView;
                        Action::Render
                    }
                    KeyCode::Tab => {
                        self.focus_area = FocusArea::DayView;
                        Action::Render
                    }
                    KeyCode::Char('n') => Action::AppointmentCreate,
                    _ => Action::None,
                }
            }
            FocusArea::DayView => {
                match key.code {
                    KeyCode::Up | KeyCode::Char('k') => {
                        self.previous_time_slot();
                        Action::Render
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        self.next_time_slot();
                        Action::Render
                    }
                    KeyCode::Tab => {
                        self.focus_area = FocusArea::MonthView;
                        Action::Render
                    }
                    KeyCode::Esc => {
                        self.focus_area = FocusArea::MonthView;
                        Action::Render
                    }
                    KeyCode::Enter => {
                        if let Some(selected_slot) = self.time_slot_state.selected() {
                            // Find appointment at current slot for first practitioner
                            if let Some(practitioner) = self.practitioners.first() {
                                if let Some(appt) = self.find_appointment_for_slot(practitioner.id, selected_slot) {
                                    self.selected_appointment = Some(appt.id);
                                    self.showing_detail_modal = true;
                                    return Action::Render;
                                }
                            }
                        }
                        Action::None
                    }
                    KeyCode::Char('n') => Action::AppointmentCreate,
                    _ => Action::None,
                }
            }
        }
    }
    
    async fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::Render => {
                self.load_appointments_for_date().await?;
                
                // Load patient data when modal opens
                if self.showing_detail_modal && self.modal_patient.is_none() {
                    if let Some(appt_id) = self.selected_appointment {
                        if let Some(appt) = self.appointments.iter().find(|a| a.id == appt_id) {
                            match self.patient_service.find_patient(appt.patient_id).await {
                                Ok(Some(patient)) => {
                                    self.modal_patient = Some(patient);
                                }
                                Ok(None) => {
                                    tracing::warn!("Patient not found: {}", appt.patient_id);
                                }
                                Err(e) => {
                                    tracing::error!("Failed to load patient: {}", e);
                                }
                            }
                        }
                    }
                }
            }
            Action::AppointmentMarkArrived => {
                if let Some(appt_id) = self.selected_appointment {
                    let user_id = Uuid::parse_str("a1b2c3d4-e5f6-4789-a1b2-c3d4e5f64789")
                        .expect("valid UUID");
                    
                    match self.appointment_service.mark_arrived(appt_id, user_id).await {
                        Ok(_) => {
                            tracing::info!("Appointment {} marked as arrived", appt_id);
                            self.load_appointments_for_date().await?;
                        }
                        Err(e) => {
                            tracing::error!("Failed to mark appointment as arrived: {}", e);
                        }
                    }
                }
            }
            Action::AppointmentMarkCompleted => {
                if let Some(appt_id) = self.selected_appointment {
                    let user_id = Uuid::parse_str("a1b2c3d4-e5f6-4789-a1b2-c3d4e5f64789")
                        .expect("valid UUID");
                    
                    match self.appointment_service.mark_completed(appt_id, user_id).await {
                        Ok(_) => {
                            tracing::info!("Appointment {} marked as completed", appt_id);
                            self.load_appointments_for_date().await?;
                        }
                        Err(e) => {
                            tracing::error!("Failed to mark appointment as completed: {}", e);
                        }
                    }
                }
            }
            Action::AppointmentMarkNoShow => {
                if let Some(appt_id) = self.selected_appointment {
                    let user_id = Uuid::parse_str("a1b2c3d4-e5f6-4789-a1b2-c3d4e5f64789")
                        .expect("valid UUID");
                    
                    match self.appointment_service.mark_no_show(appt_id, user_id).await {
                        Ok(_) => {
                            tracing::info!("Appointment {} marked as no show", appt_id);
                            self.load_appointments_for_date().await?;
                        }
                        Err(e) => {
                            tracing::error!("Failed to mark appointment as no show: {}", e);
                        }
                    }
                }
            }
            _ => {}
        }
        Ok(None)
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(30),
                Constraint::Min(50),
            ])
            .split(area);
        
        self.render_month_calendar(frame, chunks[0]);
        self.render_day_schedule(frame, chunks[1]);
        
        // Render modal on top if showing
        if self.showing_detail_modal {
            self.render_appointment_detail_modal(frame, area);
        }
    }
}
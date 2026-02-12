use async_trait::async_trait;
use chrono::{Datelike, Local, NaiveDate, Timelike, Utc, Weekday};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
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
    view_mode: ViewMode,
    week_start_date: NaiveDate,
    
    // Modal state for appointment details
    selected_appointment: Option<Uuid>,
    showing_detail_modal: bool,
    modal_patient: Option<Patient>,
    
    // Reschedule modal state
    showing_reschedule_modal: bool,
    reschedule_new_start_time: Option<chrono::DateTime<Utc>>,
    reschedule_new_duration: i64,  // in minutes
    reschedule_conflict_warning: Option<String>,
    
    // Search modal state
    showing_search_modal: bool,
    search_query: String,
    search_results: Vec<Appointment>,
    search_selected_index: usize,
    
    // Filter state
    active_status_filters: HashSet<AppointmentStatus>,
    showing_filter_menu: bool,
    
    // Practitioner filter state
    active_practitioner_filters: HashSet<Uuid>,
    showing_practitioner_menu: bool,
}

impl AppointmentCalendarComponent {
    pub fn new(
        appointment_service: Arc<AppointmentService>,
        practitioner_service: Arc<PractitionerService>,
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
            showing_reschedule_modal: false,
            reschedule_new_start_time: None,
            reschedule_new_duration: 15,
            reschedule_conflict_warning: None,
            showing_search_modal: false,
            search_query: String::new(),
            search_results: Vec::new(),
            search_selected_index: 0,
            active_status_filters: HashSet::new(),
            showing_filter_menu: false,
            active_practitioner_filters: HashSet::new(),
            showing_practitioner_menu: false,
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
    
    fn toggle_status_filter(&mut self, status: AppointmentStatus) {
        if self.active_status_filters.contains(&status) {
            self.active_status_filters.remove(&status);
        } else {
            self.active_status_filters.insert(status);
        }
    }
    
    fn toggle_practitioner_filter(&mut self, practitioner_id: Uuid) {
        if self.active_practitioner_filters.contains(&practitioner_id) {
            self.active_practitioner_filters.remove(&practitioner_id);
        } else {
            self.active_practitioner_filters.insert(practitioner_id);
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
    
    /// Load appointments for all 7 days of the current week (Mon-Sun)
    /// based on `week_start_date`. Collects all appointments into a single Vec.
    async fn load_appointments_for_week(&mut self) -> Result<()> {
        let mut all_appointments = Vec::new();
        
        for i in 0..7 {
            let date = self.week_start_date + chrono::Duration::days(i);
            match self.appointment_service.get_day_appointments(date, None).await {
                Ok(mut appointments) => {
                    all_appointments.append(&mut appointments);
                }
                Err(e) => {
                    tracing::error!("Failed to load appointments for {}: {}", date, e);
                }
            }
        }
        
        self.appointments = all_appointments;
        Ok(())
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
                && (self.active_status_filters.is_empty() 
                    || self.active_status_filters.contains(&appt.status))
        })
    }
    
    /// Detect all overlapping appointments at a given slot for a practitioner
    /// Returns a vector of all appointments that overlap at this time slot
    fn detect_overlaps(
        &self,
        practitioner_id: uuid::Uuid,
        slot_index: usize,
    ) -> Vec<&Appointment> {
        let time_slots = Self::generate_time_slots();
        if slot_index >= time_slots.len() {
            return Vec::new();
        }
        
        let slot_time_str = &time_slots[slot_index];
        let (hour, minute) = slot_time_str.split_once(':')
            .and_then(|(h, m)| {
                let hour = h.parse::<u32>().ok()?;
                let minute = m.parse::<u32>().ok()?;
                Some((hour, minute))
            })
            .unwrap_or((0, 0));
        
        let date = NaiveDate::from_ymd_opt(
            self.current_month_start.year(),
            self.current_month_start.month(),
            self.selected_month_day,
        ).expect("valid date");
        
        let slot_datetime = date.and_hms_opt(hour, minute, 0)
            .expect("valid time")
            .and_utc();
        
        // Find ALL appointments that overlap at this slot
        self.appointments.iter()
            .filter(|appt| {
                appt.practitioner_id == practitioner_id
                    && appt.start_time <= slot_datetime
                    && appt.end_time > slot_datetime
                    && (self.active_status_filters.is_empty() 
                        || self.active_status_filters.contains(&appt.status))
            })
            .collect()
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
    
    fn render_week_schedule(&mut self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
            ])
            .split(area);
        
        self.render_week_header(frame, chunks[0]);
        self.render_week_time_slots_grid(frame, chunks[1]);
    }
    
    fn render_week_header(&self, frame: &mut Frame, area: Rect) {
        let dates: Vec<NaiveDate> = (0..7)
            .map(|i| self.week_start_date + chrono::Duration::days(i))
            .collect();
        
        fn get_day_name(weekday: Weekday) -> &'static str {
            match weekday {
                Weekday::Mon => "Mon",
                Weekday::Tue => "Tue",
                Weekday::Wed => "Wed",
                Weekday::Thu => "Thu",
                Weekday::Fri => "Fri",
                Weekday::Sat => "Sat",
                Weekday::Sun => "Sun",
            }
        }
        
        let mut header_cells = vec![Cell::from("Time").style(
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        )];
        
        for date in &dates {
            let day_name = get_day_name(date.weekday());
            let date_str = format!("{} {}", day_name, date.day());
            
            let style = if *date == self.current_date {
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            };
            
            header_cells.push(Cell::from(date_str).style(style));
        }
        
        let header = Row::new(header_cells)
            .style(Style::default().bg(Color::DarkGray))
            .height(1);
        
        let widths = vec![
            Constraint::Length(8),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
        ];
        
        let week_start_str = self.week_start_date.format("%b %d").to_string();
        let week_end_str = (self.week_start_date + chrono::Duration::days(6)).format("%b %d").to_string();
        let title = format!(" Week: {}-{} ", week_start_str, week_end_str);
        
        let table = Table::new(vec![header], widths)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(title)
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
    
    fn render_week_time_slots_grid(&mut self, frame: &mut Frame, area: Rect) {
        let time_slots = Self::generate_time_slots();
        let dates: Vec<NaiveDate> = (0..7)
            .map(|i| self.week_start_date + chrono::Duration::days(i))
            .collect();
        let mut rows = Vec::new();
        
        for time_slot in time_slots.iter() {
            let mut cells = vec![Cell::from(time_slot.as_str())];
            
            let (hour, minute) = time_slot.split_once(':')
                .and_then(|(h, m)| {
                    let hour = h.parse::<u32>().ok()?;
                    let minute = m.parse::<u32>().ok()?;
                    Some((hour, minute))
                })
                .expect("valid time slot format");
            
            for date in &dates {
                let slot_datetime = date.and_hms_opt(hour, minute, 0)
                    .expect("valid time")
                    .and_utc();
                
                let appts_at_slot: Vec<&Appointment> = self.appointments.iter()
                    .filter(|a| {
                        let appt_date = a.start_time.date_naive();
                        let same_day = appt_date == *date;
                        let overlaps_time = a.start_time <= slot_datetime && a.end_time > slot_datetime;
                        same_day && overlaps_time
                    })
                    .collect();
                
                let cell_content = match appts_at_slot.len() {
                    0 => String::new(),
                    1 => {
                        let appt = appts_at_slot[0];
                        appt.patient_id.to_string()[..3].to_string()
                    }
                    n => format!("{}", n),
                };
                
                let cell_style = match appts_at_slot.len() {
                    0 => Style::default(),
                    1 => {
                        let appt = appts_at_slot[0];
                        match appt.status {
                            AppointmentStatus::Scheduled => Style::default().fg(Color::White).bg(Color::Blue),
                            AppointmentStatus::Confirmed => Style::default().fg(Color::Black).bg(Color::Cyan),
                            AppointmentStatus::Arrived => Style::default().fg(Color::Black).bg(Color::Yellow),
                            AppointmentStatus::InProgress => Style::default().fg(Color::White).bg(Color::Green),
                            AppointmentStatus::Completed => Style::default().fg(Color::White).bg(Color::DarkGray),
                            AppointmentStatus::NoShow => Style::default().fg(Color::White).bg(Color::Red),
                            AppointmentStatus::Cancelled => Style::default().fg(Color::Gray),
                            AppointmentStatus::Rescheduled => Style::default().fg(Color::Magenta),
                        }
                    }
                    _ => Style::default().fg(Color::Yellow).bg(Color::DarkGray).add_modifier(Modifier::BOLD),
                };
                
                cells.push(Cell::from(cell_content).style(cell_style));
            }
            
            let row = Row::new(cells).height(1);
            rows.push(row);
        }
        
        let widths = vec![
            Constraint::Length(8),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
        ];
        
        let table = Table::new(rows, widths)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" (↑↓/j/k: Navigate, v: Day View, Shift+←/→: Week, n: New, Tab/Esc: Month) "),
            )
            .row_highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");
        
        frame.render_stateful_widget(table, area, &mut self.time_slot_state);
    }
    


    fn render_practitioner_header(&self, frame: &mut Frame, area: Rect) {
        let visible_practitioners: Vec<_> = self.practitioners.iter()
            .filter(|p| self.active_practitioner_filters.is_empty() 
                || self.active_practitioner_filters.contains(&p.id))
            .collect();
        
        let mut header_cells = vec![Cell::from("Time").style(
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        )];
        
        for practitioner in &visible_practitioners {
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
        for _ in &visible_practitioners {
            widths.push(Constraint::Min(15));
        }
        
        let mut title = " Schedule ".to_string();
        if !self.active_status_filters.is_empty() {
            title = format!(" Schedule [Status: {}] ", self.active_status_filters.len());
        }
        if !self.active_practitioner_filters.is_empty() {
            let practitioner_names: Vec<String> = visible_practitioners.iter()
                .map(|p| format!("Dr. {}", p.last_name))
                .collect();
            title = format!(" Schedule [{}] ", practitioner_names.join(", "));
        }
        
        let table = Table::new(vec![header], widths)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(title)
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
        let visible_practitioners: Vec<_> = self.practitioners.iter()
            .filter(|p| self.active_practitioner_filters.is_empty() 
                || self.active_practitioner_filters.contains(&p.id))
            .collect();
        
        let time_slots = Self::generate_time_slots();
        let mut rows = Vec::new();
        let mut rendered_appointments = HashSet::new();
        
        for (slot_index, time_slot) in time_slots.iter().enumerate() {
            let mut cells = vec![Cell::from(time_slot.as_str())];
            
            for practitioner in &visible_practitioners {
                let overlaps = self.detect_overlaps(practitioner.id, slot_index);
                
                if overlaps.len() > 1 {
                    let overlap_ids: Vec<String> = overlaps.iter()
                        .map(|a| a.id.to_string())
                        .collect();
                    tracing::warn!(
                        "Double-booking detected: {} appointments at slot {} for practitioner {} (IDs: {})",
                        overlaps.len(),
                        slot_index,
                        practitioner.id,
                        overlap_ids.join(", ")
                    );
                }
                
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
                        
                        if overlaps.len() > 1 {
                            appt_text = format!("⚠ {} conflicts\n{}", overlaps.len(), appt_text);
                        }
                        
                        let mut style = match appt.status {
                            AppointmentStatus::Scheduled => Style::default().fg(Color::White).bg(Color::Blue),
                            AppointmentStatus::Confirmed => Style::default().fg(Color::Black).bg(Color::Cyan),
                            AppointmentStatus::Arrived => Style::default().fg(Color::Black).bg(Color::Yellow),
                            AppointmentStatus::InProgress => Style::default().fg(Color::White).bg(Color::Green),
                            AppointmentStatus::Completed => Style::default().fg(Color::White).bg(Color::DarkGray),
                            AppointmentStatus::NoShow => Style::default().fg(Color::White).bg(Color::Red),
                            AppointmentStatus::Rescheduled => Style::default().fg(Color::Magenta),
                            AppointmentStatus::Cancelled => Style::default().fg(Color::Gray),
                        };
                        
                        if overlaps.len() > 1 {
                            style = style.fg(Color::Red).add_modifier(Modifier::BOLD);
                        }
                        
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
        for _ in &visible_practitioners {
            widths.push(Constraint::Min(15));
        }
        
        let table = Table::new(rows, widths)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" (↑↓/j/k: Navigate, v: Week View, n: New, Tab/Esc: Month View) "),
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
            KeyCode::Char('r') | KeyCode::Char('R') => {
                if let Some(appt_id) = self.selected_appointment {
                    if let Some(appt) = self.appointments.iter().find(|a| a.id == appt_id) {
                        self.reschedule_new_start_time = Some(appt.start_time);
                        self.reschedule_new_duration = appt.duration_minutes();
                        self.showing_detail_modal = false;
                        self.showing_reschedule_modal = true;
                        self.reschedule_conflict_warning = None;
                        return Action::Render;
                    }
                }
                Action::None
            }
            _ => Action::None,
        }
    }
    
    fn handle_reschedule_modal_key_events(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Esc => {
                self.showing_reschedule_modal = false;
                self.reschedule_new_start_time = None;
                self.reschedule_conflict_warning = None;
                self.showing_detail_modal = true;
                Action::Render
            }
            KeyCode::Up => {
                if let Some(current_time) = self.reschedule_new_start_time {
                    self.reschedule_new_start_time = Some(current_time - chrono::Duration::minutes(15));
                    Action::Render
                } else {
                    Action::None
                }
            }
            KeyCode::Down => {
                if let Some(current_time) = self.reschedule_new_start_time {
                    self.reschedule_new_start_time = Some(current_time + chrono::Duration::minutes(15));
                    Action::Render
                } else {
                    Action::None
                }
            }
            KeyCode::Char('+') => {
                self.reschedule_new_duration += 15;
                Action::Render
            }
            KeyCode::Char('-') => {
                if self.reschedule_new_duration > 15 {
                    self.reschedule_new_duration -= 15;
                }
                Action::Render
            }
            KeyCode::Enter => {
                Action::AppointmentReschedule
            }
            _ => Action::None,
        }
    }
    
    fn handle_search_key_events(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Esc => {
                self.showing_search_modal = false;
                self.search_query.clear();
                self.search_results.clear();
                self.search_selected_index = 0;
                Action::Render
            }
            KeyCode::Up => {
                if self.search_selected_index > 0 {
                    self.search_selected_index -= 1;
                }
                Action::Render
            }
            KeyCode::Down => {
                if !self.search_results.is_empty() && self.search_selected_index < self.search_results.len() - 1 {
                    self.search_selected_index += 1;
                }
                Action::Render
            }
            KeyCode::Enter => {
                if let Some(appt) = self.search_results.get(self.search_selected_index).cloned() {
                    self.navigate_to_appointment(&appt);
                    self.showing_search_modal = false;
                    self.search_query.clear();
                    self.search_results.clear();
                    self.search_selected_index = 0;
                }
                Action::Render
            }
            KeyCode::Char(c) => {
                self.search_query.push(c);
                self.filter_appointments_by_query();
                self.search_selected_index = 0;
                Action::Render
            }
            KeyCode::Backspace => {
                self.search_query.pop();
                self.filter_appointments_by_query();
                self.search_selected_index = 0;
                Action::Render
            }
            _ => Action::None,
        }
    }
    
    fn filter_appointments_by_query(&mut self) {
        if self.search_query.is_empty() {
            self.search_results.clear();
            return;
        }
        
        let query_lower = self.search_query.to_lowercase();
        
        self.search_results = self.appointments.iter()
            .filter(|appt| {
                let patient_id_str = appt.patient_id.to_string().to_lowercase();
                let type_str = format!("{:?}", appt.appointment_type).to_lowercase();
                let status_str = format!("{:?}", appt.status).to_lowercase();
                
                patient_id_str.contains(&query_lower) ||
                type_str.contains(&query_lower) ||
                status_str.contains(&query_lower)
            })
            .take(50)
            .cloned()
            .collect();
    }
    
    fn handle_filter_menu_key_events(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Esc => {
                self.showing_filter_menu = false;
                Action::Render
            }
            KeyCode::Char('0') => {
                self.active_status_filters.clear();
                Action::Render
            }
            KeyCode::Char('1') => {
                self.toggle_status_filter(AppointmentStatus::Scheduled);
                Action::Render
            }
            KeyCode::Char('2') => {
                self.toggle_status_filter(AppointmentStatus::Confirmed);
                Action::Render
            }
            KeyCode::Char('3') => {
                self.toggle_status_filter(AppointmentStatus::Arrived);
                Action::Render
            }
            KeyCode::Char('4') => {
                self.toggle_status_filter(AppointmentStatus::InProgress);
                Action::Render
            }
            KeyCode::Char('5') => {
                self.toggle_status_filter(AppointmentStatus::Completed);
                Action::Render
            }
            KeyCode::Char('6') => {
                self.toggle_status_filter(AppointmentStatus::NoShow);
                Action::Render
            }
            KeyCode::Char('7') => {
                self.toggle_status_filter(AppointmentStatus::Cancelled);
                Action::Render
            }
            KeyCode::Char('8') => {
                self.toggle_status_filter(AppointmentStatus::Rescheduled);
                Action::Render
            }
            _ => Action::None,
        }
    }
    
    fn handle_practitioner_menu_key_events(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Esc => {
                self.showing_practitioner_menu = false;
                Action::Render
            }
            KeyCode::Char('0') => {
                self.active_practitioner_filters.clear();
                Action::Render
            }
            KeyCode::Char(c) if c.is_ascii_digit() => {
                let digit = c.to_digit(10).unwrap() as usize;
                if digit > 0 && digit <= self.practitioners.len() {
                    let practitioner_id = self.practitioners[digit - 1].id;
                    self.toggle_practitioner_filter(practitioner_id);
                }
                Action::Render
            }
            _ => Action::None,
        }
    }
    
    fn navigate_to_appointment(&mut self, appt: &Appointment) {
        let appt_date = appt.start_time.date_naive();
        
        self.current_month_start = NaiveDate::from_ymd_opt(
            appt_date.year(),
            appt_date.month(),
            1
        ).expect("valid month start");
        
        self.selected_month_day = appt_date.day();
        
        let hour = appt.start_time.hour();
        let minute = appt.start_time.minute();
        let slot_index = ((hour - 8) * 4 + minute / 15) as usize;
        
        if slot_index < 40 {
            self.time_slot_state.select(Some(slot_index));
        }
        
        self.focus_area = FocusArea::DayView;
        self.view_mode = ViewMode::Day;
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
                
                let appt_start_hour = appt.start_time.hour();
                let appt_start_minute = appt.start_time.minute();
                let slot_index = ((appt_start_hour - 8) * 4 + appt_start_minute / 15) as usize;
                let overlaps = self.detect_overlaps(appt.practitioner_id, slot_index);
                
                if overlaps.len() > 1 {
                    lines.push(Line::from(vec![
                        Span::styled("⚠ CONFLICT: ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                        Span::styled(
                            format!("{} overlapping appointments detected", overlaps.len()),
                            Style::default().fg(Color::Red)
                        ),
                    ]));
                    
                    for overlap_appt in &overlaps {
                        if overlap_appt.id != appt.id {
                            lines.push(Line::from(vec![
                                Span::styled("  • ", Style::default().fg(Color::Red)),
                                Span::styled(
                                    format!("ID: {} ({} - {})",
                                        &overlap_appt.id.to_string()[..8],
                                        overlap_appt.start_time.format("%H:%M"),
                                        overlap_appt.end_time.format("%H:%M")
                                    ),
                                    Style::default().fg(Color::White)
                                ),
                            ]));
                        }
                    }
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
        
        lines.push(Line::from(""));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("A", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(": Arrived  ", Style::default().fg(Color::White)),
            Span::styled("C", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(": Completed  ", Style::default().fg(Color::White)),
            Span::styled("N", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(": No Show", Style::default().fg(Color::White)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("R", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(": Reschedule  ", Style::default().fg(Color::White)),
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
    
    fn render_search_modal(&self, frame: &mut Frame, area: Rect) {
        let modal_area = Rect {
            x: area.width / 5,
            y: area.height / 6,
            width: area.width * 3 / 5,
            height: area.height * 2 / 3,
        };
        
        let mut lines = Vec::new();
        
        lines.push(Line::from(vec![
            Span::styled("Search Appointments", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]));
        lines.push(Line::from(""));
        
        lines.push(Line::from(vec![
            Span::styled("Query: ", Style::default().fg(Color::Cyan)),
            Span::styled(&self.search_query, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled("█", Style::default().fg(Color::White)),
        ]));
        
        lines.push(Line::from(""));
        
        if self.search_query.is_empty() {
            lines.push(Line::from(vec![
                Span::styled("Start typing to search...", Style::default().fg(Color::DarkGray)),
            ]));
        } else if self.search_results.is_empty() {
            lines.push(Line::from(vec![
                Span::styled("No appointments found", Style::default().fg(Color::Red)),
            ]));
        } else {
            lines.push(Line::from(vec![
                Span::styled(
                    format!("Found {} result(s) (showing up to 50)", self.search_results.len()),
                    Style::default().fg(Color::Green)
                ),
            ]));
            lines.push(Line::from(""));
            
            for (idx, appt) in self.search_results.iter().enumerate() {
                let is_selected = idx == self.search_selected_index;
                let prefix = if is_selected { "→ " } else { "  " };
                
                let patient_id_short = &appt.patient_id.to_string()[..8];
                let date_str = appt.start_time.format("%Y-%m-%d").to_string();
                let time_str = appt.start_time.format("%H:%M").to_string();
                let type_str = format!("{:?}", appt.appointment_type);
                
                let style = if is_selected {
                    Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };
                
                lines.push(Line::from(vec![
                    Span::styled(prefix, style),
                    Span::styled(format!("Patient {}  ", patient_id_short), style),
                    Span::styled(format!("{} {}  ", date_str, time_str), style),
                    Span::styled(type_str, style),
                ]));
            }
        }
        
        lines.push(Line::from(""));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("↑↓", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(": Navigate  ", Style::default().fg(Color::White)),
            Span::styled("Enter", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(": Select  ", Style::default().fg(Color::White)),
            Span::styled("Esc", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(": Close", Style::default().fg(Color::White)),
        ]));
        
        let modal_content = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Search Appointments ")
                    .border_style(Style::default().fg(Color::Cyan))
            )
            .wrap(ratatui::widgets::Wrap { trim: true });
        
        frame.render_widget(modal_content, modal_area);
    }
    
    fn render_reschedule_modal(&mut self, frame: &mut Frame, area: Rect) {
        let modal_area = Rect {
            x: area.width / 5,
            y: area.height / 6,
            width: area.width * 3 / 5,
            height: area.height / 2,
        };
        
        let mut lines = Vec::new();
        
        if let Some(appt_id) = self.selected_appointment {
            if let Some(appt) = self.appointments.iter().find(|a| a.id == appt_id) {
                let patient_name = if let Some(ref patient) = self.modal_patient {
                    format!("{} {}", patient.first_name, patient.last_name)
                } else {
                    "Loading...".to_string()
                };
                
                lines.push(Line::from(vec![
                    Span::styled("Reschedule Appointment", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                ]));
                lines.push(Line::from(""));
                
                lines.push(Line::from(vec![
                    Span::styled("Patient: ", Style::default().fg(Color::Yellow)),
                    Span::styled(patient_name, Style::default().fg(Color::White)),
                ]));
                
                lines.push(Line::from(""));
                
                let current_time_str = format!("{}", appt.start_time.format("%Y-%m-%d %H:%M"));
                lines.push(Line::from(vec![
                    Span::styled("Current Time: ", Style::default().fg(Color::Yellow)),
                    Span::styled(current_time_str.clone(), Style::default().fg(Color::White)),
                ]));
                
                let current_duration = appt.duration_minutes();
                lines.push(Line::from(vec![
                    Span::styled("Current Duration: ", Style::default().fg(Color::Yellow)),
                    Span::styled(format!("{} minutes", current_duration), Style::default().fg(Color::White)),
                ]));
                
                lines.push(Line::from(""));
                lines.push(Line::from(vec![
                    Span::styled("New Time: ", Style::default().fg(Color::Yellow)),
                    Span::styled(
                        if let Some(new_time) = self.reschedule_new_start_time {
                            format!("{}", new_time.format("%Y-%m-%d %H:%M"))
                        } else {
                            current_time_str
                        },
                        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
                    ),
                ]));
                
                lines.push(Line::from(vec![
                    Span::styled("New Duration: ", Style::default().fg(Color::Yellow)),
                    Span::styled(
                        format!("{} minutes", self.reschedule_new_duration),
                        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
                    ),
                ]));
                
                if let Some(ref warning) = self.reschedule_conflict_warning {
                    lines.push(Line::from(""));
                    lines.push(Line::from(vec![
                        Span::styled("⚠ ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                        Span::styled(warning, Style::default().fg(Color::Red)),
                    ]));
                }
            }
        }
        
        lines.push(Line::from(""));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("↑↓", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(": Time  ", Style::default().fg(Color::White)),
            Span::styled("+/-", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(": Duration  ", Style::default().fg(Color::White)),
            Span::styled("Enter", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(": Save  ", Style::default().fg(Color::White)),
            Span::styled("Esc", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(": Cancel", Style::default().fg(Color::White)),
        ]));
        
        let modal_content = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Reschedule Appointment ")
                    .border_style(Style::default().fg(Color::Yellow))
            )
            .wrap(ratatui::widgets::Wrap { trim: true });
        
        frame.render_widget(modal_content, modal_area);
    }
    
    fn render_filter_menu(&self, frame: &mut Frame, area: Rect) {
        let modal_area = Rect {
            x: area.width / 5,
            y: area.height / 6,
            width: area.width * 3 / 5,
            height: area.height * 2 / 3,
        };
        
        let mut lines = Vec::new();
        
        lines.push(Line::from(vec![
            Span::styled("Filter by Status", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]));
        lines.push(Line::from(""));
        
        // Status filter options with checkmarks
        let statuses = vec![
            (AppointmentStatus::Scheduled, "1", "Scheduled"),
            (AppointmentStatus::Confirmed, "2", "Confirmed"),
            (AppointmentStatus::Arrived, "3", "Arrived"),
            (AppointmentStatus::InProgress, "4", "In Progress"),
            (AppointmentStatus::Completed, "5", "Completed"),
            (AppointmentStatus::NoShow, "6", "No Show"),
            (AppointmentStatus::Cancelled, "7", "Cancelled"),
            (AppointmentStatus::Rescheduled, "8", "Rescheduled"),
        ];
        
        for (status, key, label) in statuses {
            let is_active = self.active_status_filters.contains(&status);
            let checkbox = if is_active { "☑" } else { "☐" };
            let color = if is_active { Color::Green } else { Color::White };
            
            lines.push(Line::from(vec![
                Span::styled(format!("  {} ", checkbox), Style::default().fg(color)),
                Span::styled(format!("[{}] ", key), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled(label, Style::default().fg(color)),
            ]));
        }
        
        lines.push(Line::from(""));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("0", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(": Clear all  ", Style::default().fg(Color::White)),
            Span::styled("Esc", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(": Close", Style::default().fg(Color::White)),
        ]));
        
        let filter_count = self.active_status_filters.len();
        let title = if filter_count == 0 {
            " Status Filter (All) ".to_string()
        } else {
            format!(" Status Filter ({} active) ", filter_count)
        };
        
        let modal_content = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(title)
                    .border_style(Style::default().fg(Color::Green))
            )
            .wrap(ratatui::widgets::Wrap { trim: true });
        
        frame.render_widget(modal_content, modal_area);
    }
    
    fn render_practitioner_menu(&self, frame: &mut Frame, area: Rect) {
        let modal_area = Rect {
            x: area.width / 5,
            y: area.height / 6,
            width: area.width * 3 / 5,
            height: area.height * 2 / 3,
        };
        
        let mut lines = Vec::new();
        
        lines.push(Line::from(vec![
            Span::styled("Filter by Practitioner", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]));
        lines.push(Line::from(""));
        
        for (idx, practitioner) in self.practitioners.iter().enumerate() {
            let key = (idx + 1).to_string();
            let is_active = self.active_practitioner_filters.contains(&practitioner.id);
            let checkbox = if is_active { "☑" } else { "☐" };
            let color = if is_active { Color::Green } else { Color::White };
            let name = format!("Dr. {}", practitioner.last_name);
            
            lines.push(Line::from(vec![
                Span::styled(format!("  {} ", checkbox), Style::default().fg(color)),
                Span::styled(format!("[{}] ", key), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled(name, Style::default().fg(color)),
            ]));
        }
        
        lines.push(Line::from(""));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("0", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(": Clear all  ", Style::default().fg(Color::White)),
            Span::styled("Esc", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(": Close", Style::default().fg(Color::White)),
        ]));
        
        let filter_count = self.active_practitioner_filters.len();
        let title = if filter_count == 0 {
            " Practitioner Filter (All) ".to_string()
        } else {
            format!(" Practitioner Filter ({} active) ", filter_count)
        };
        
        let modal_content = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(title)
                    .border_style(Style::default().fg(Color::Magenta))
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
        if self.showing_filter_menu {
            return self.handle_filter_menu_key_events(key);
        }
        
        if self.showing_practitioner_menu {
            return self.handle_practitioner_menu_key_events(key);
        }
        
        if self.showing_search_modal {
            return self.handle_search_key_events(key);
        }
        
        if self.showing_reschedule_modal {
            return self.handle_reschedule_modal_key_events(key);
        }
        
        if self.showing_detail_modal {
            return self.handle_modal_key_events(key);
        }
        
        if key.code == KeyCode::Char('/') {
            self.showing_search_modal = true;
            self.search_query.clear();
            self.search_results.clear();
            self.search_selected_index = 0;
            return Action::Render;
        }
        
        if key.code == KeyCode::Char('f') {
            self.showing_filter_menu = true;
            return Action::Render;
        }
        
        if key.code == KeyCode::Char('p') {
            self.showing_practitioner_menu = true;
            return Action::Render;
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
                    KeyCode::Char('v') => {
                        self.view_mode = match self.view_mode {
                            ViewMode::Day => ViewMode::Week,
                            ViewMode::Week => ViewMode::Day,
                        };
                        Action::Render
                    }
                    KeyCode::Left if key.modifiers.contains(KeyModifiers::SHIFT) => {
                        self.week_start_date -= chrono::Duration::days(7);
                        Action::Render
                    }
                    KeyCode::Right if key.modifiers.contains(KeyModifiers::SHIFT) => {
                        self.week_start_date += chrono::Duration::days(7);
                        Action::Render
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
                match self.view_mode {
                    ViewMode::Day => self.load_appointments_for_date().await?,
                    ViewMode::Week => self.load_appointments_for_week().await?,
                }
                
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
            Action::AppointmentReschedule => {
                if let Some(appt_id) = self.selected_appointment {
                    if let Some(new_start_time) = self.reschedule_new_start_time {
                        let user_id = Uuid::parse_str("a1b2c3d4-e5f6-4789-a1b2-c3d4e5f64789")
                            .expect("valid UUID");
                        
                        match self.appointment_service.reschedule_appointment(
                            appt_id,
                            new_start_time,
                            self.reschedule_new_duration,
                            user_id
                        ).await {
                            Ok(_) => {
                                tracing::info!("Appointment {} rescheduled successfully", appt_id);
                                self.showing_reschedule_modal = false;
                                self.reschedule_new_start_time = None;
                                self.reschedule_conflict_warning = None;
                                match self.view_mode {
                                    ViewMode::Day => self.load_appointments_for_date().await?,
                                    ViewMode::Week => self.load_appointments_for_week().await?,
                                }
                            }
                            Err(e) => {
                                tracing::error!("Failed to reschedule appointment: {}", e);
                                self.reschedule_conflict_warning = Some(e.to_string());
                            }
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
        match self.view_mode {
            ViewMode::Day => self.render_day_schedule(frame, chunks[1]),
            ViewMode::Week => self.render_week_schedule(frame, chunks[1]),
        }
        
        if self.showing_detail_modal {
            self.render_appointment_detail_modal(frame, area);
        }
        
        if self.showing_reschedule_modal {
            self.render_reschedule_modal(frame, area);
        }
        
        if self.showing_search_modal {
            self.render_search_modal(frame, area);
        }
        
        if self.showing_filter_menu {
            self.render_filter_menu(frame, area);
        }
        
        if self.showing_practitioner_menu {
            self.render_practitioner_menu(frame, area);
        }
    }
}
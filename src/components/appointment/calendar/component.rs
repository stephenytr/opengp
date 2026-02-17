use async_trait::async_trait;
use chrono::{Datelike, Local, NaiveDate, TimeZone, Timelike, Utc, Weekday};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table};
use ratatui::Frame;
use std::sync::Arc;
use uuid::Uuid;

use crate::components::{Action, Component};
use crate::domain::appointment::{
    AppointmentSearchCriteria, AppointmentService, AppointmentStatus, CalendarAppointment,
};
use crate::domain::audit::AuditAction;
use crate::domain::patient::PatientService;
use crate::domain::user::{Practitioner, PractitionerService};
use crate::error::Result;
use crate::ui::key_dispatcher::KeyDispatcher;
use crate::ui::keybinds::{KeybindContext, KeybindRegistry};
use crate::ui::widgets::{
    is_click, is_scroll_down, is_scroll_up, HelpModal, ModalState, ModalType,
};

use super::renderers::{CalendarRenderer, ModalRenderer};
use super::state::{
    AuditModalData, BatchModalData, CalendarState, ConfirmationModalData, DetailModalData,
    ErrorModalData, FilterState, FocusArea, HistoryState, RescheduleModalData, SearchModalData,
    ViewMode,
};

pub struct AppointmentCalendarComponent {
    appointment_service: Arc<AppointmentService>,
    practitioner_service: Arc<PractitionerService>,
    patient_service: Arc<PatientService>,

    calendar_state: CalendarState,
    filter_state: FilterState,
    history_state: HistoryState,

    modal_state: ModalState,
    detail_data: DetailModalData,
    reschedule_data: RescheduleModalData,
    search_data: SearchModalData,
    confirmation_data: ConfirmationModalData,
    audit_data: AuditModalData,
    batch_data: BatchModalData,
    error_data: ErrorModalData,

    // Mouse interaction areas
    month_calendar_area: Option<Rect>,
    schedule_area: Option<Rect>,
}

impl AppointmentCalendarComponent {
    #[allow(dead_code)]
    pub fn new(
        appointment_service: Arc<AppointmentService>,
        practitioner_service: Arc<PractitionerService>,
        patient_service: Arc<PatientService>,
    ) -> Self {
        Self {
            appointment_service,
            practitioner_service,
            patient_service,
            calendar_state: CalendarState::new(),
            filter_state: FilterState::new(),
            history_state: HistoryState::new(),
            modal_state: ModalState::none(),
            detail_data: DetailModalData::default(),
            reschedule_data: RescheduleModalData::default(),
            search_data: SearchModalData::default(),
            confirmation_data: ConfirmationModalData::default(),
            audit_data: AuditModalData::default(),
            batch_data: BatchModalData::default(),
            error_data: ErrorModalData::default(),
            month_calendar_area: None,
            schedule_area: None,
        }
    }

    fn days_in_current_month(&self) -> u32 {
        CalendarRenderer::days_in_month(
            self.calendar_state.current_month_start.year(),
            self.calendar_state.current_month_start.month(),
        )
    }

    fn previous_day(&mut self) {
        if self.calendar_state.selected_month_day > 1 {
            self.calendar_state.selected_month_day -= 1;
        }
    }

    fn next_day(&mut self) {
        let days_in_month = CalendarRenderer::days_in_month(
            self.calendar_state.current_month_start.year(),
            self.calendar_state.current_month_start.month(),
        );
        if self.calendar_state.selected_month_day < days_in_month {
            self.calendar_state.selected_month_day += 1;
        }
    }

    fn previous_month(&mut self) {
        if self.calendar_state.current_month_start.month() == 1 {
            self.calendar_state.current_month_start =
                NaiveDate::from_ymd_opt(self.calendar_state.current_month_start.year() - 1, 12, 1)
                    .expect("first day of month is always valid");
        } else {
            self.calendar_state.current_month_start = NaiveDate::from_ymd_opt(
                self.calendar_state.current_month_start.year(),
                self.calendar_state.current_month_start.month() - 1,
                1,
            )
            .expect("first day of month is always valid");
        }

        let days_in_month = CalendarRenderer::days_in_month(
            self.calendar_state.current_month_start.year(),
            self.calendar_state.current_month_start.month(),
        );
        if self.calendar_state.selected_month_day > days_in_month {
            self.calendar_state.selected_month_day = days_in_month;
        }
    }

    fn next_month(&mut self) {
        if self.calendar_state.current_month_start.month() == 12 {
            self.calendar_state.current_month_start =
                NaiveDate::from_ymd_opt(self.calendar_state.current_month_start.year() + 1, 1, 1)
                    .expect("first day of month is always valid");
        } else {
            self.calendar_state.current_month_start = NaiveDate::from_ymd_opt(
                self.calendar_state.current_month_start.year(),
                self.calendar_state.current_month_start.month() + 1,
                1,
            )
            .expect("first day of month is always valid");
        }

        let days_in_month = self.days_in_current_month();
        if self.calendar_state.selected_month_day > days_in_month {
            self.calendar_state.selected_month_day = days_in_month;
        }
    }

    fn jump_to_today(&mut self) {
        let today = Local::now().date_naive();
        self.calendar_state.current_date = today;
        self.calendar_state.current_month_start =
            NaiveDate::from_ymd_opt(today.year(), today.month(), 1)
                .expect("first day of month is always valid");
        self.calendar_state.selected_month_day = today.day();
    }

    fn toggle_status_filter(&mut self, status: AppointmentStatus) {
        if self.filter_state.active_status_filters.contains(&status) {
            self.filter_state.active_status_filters.remove(&status);
        } else {
            self.filter_state.active_status_filters.insert(status);
        }
    }

    fn toggle_practitioner_filter(&mut self, practitioner_id: Uuid) {
        if self
            .filter_state
            .active_practitioner_filters
            .contains(&practitioner_id)
        {
            self.filter_state
                .active_practitioner_filters
                .remove(&practitioner_id);
        } else {
            self.filter_state
                .active_practitioner_filters
                .insert(practitioner_id);
        }
    }

    async fn load_appointments_for_date(&mut self) -> Result<()> {
        let date = NaiveDate::from_ymd_opt(
            self.calendar_state.current_month_start.year(),
            self.calendar_state.current_month_start.month(),
            self.calendar_state.selected_month_day,
        )
        .expect("valid date from selected day");

        let start_of_day = chrono::Utc
            .from_utc_datetime(&date.and_hms_opt(0, 0, 0).expect("00:00:00 is always valid"));
        let end_of_day = chrono::Utc.from_utc_datetime(
            &date
                .and_hms_opt(23, 59, 59)
                .expect("23:59:59 is always valid"),
        );

        let criteria = AppointmentSearchCriteria {
            patient_id: None,
            practitioner_id: None,
            date_from: Some(start_of_day),
            date_to: Some(end_of_day),
            status: None,
            appointment_type: None,
            is_urgent: None,
            confirmed: None,
        };

        match self
            .appointment_service
            .get_calendar_appointments(&criteria)
            .await
        {
            Ok(appointments) => {
                self.calendar_state.appointments = appointments;
                Ok(())
            }
            Err(e) => {
                tracing::error!("Failed to load appointments: {}", e);
                self.calendar_state.appointments = Vec::new();
                Ok(())
            }
        }
    }

    /// Load appointments for all 7 days of the current week (Mon-Sun)
    /// based on `week_start_date`. Collects all appointments into a single Vec.
    async fn load_appointments_for_week(&mut self) -> Result<()> {
        let mut all_appointments = Vec::new();

        for i in 0..7 {
            let date = self.calendar_state.week_start_date + chrono::Duration::days(i);
            let start_of_day = chrono::Utc
                .from_utc_datetime(&date.and_hms_opt(0, 0, 0).expect("00:00:00 is always valid"));
            let end_of_day = chrono::Utc.from_utc_datetime(
                &date
                    .and_hms_opt(23, 59, 59)
                    .expect("23:59:59 is always valid"),
            );

            let criteria = AppointmentSearchCriteria {
                patient_id: None,
                practitioner_id: None,
                date_from: Some(start_of_day),
                date_to: Some(end_of_day),
                status: None,
                appointment_type: None,
                is_urgent: None,
                confirmed: None,
            };

            match self
                .appointment_service
                .get_calendar_appointments(&criteria)
                .await
            {
                Ok(mut appointments) => {
                    all_appointments.append(&mut appointments);
                }
                Err(e) => {
                    tracing::error!("Failed to load appointments for {}: {}", date, e);
                }
            }
        }

        self.calendar_state.appointments = all_appointments;
        Ok(())
    }

    fn find_appointment_for_slot(
        &self,
        practitioner_id: uuid::Uuid,
        slot_index: usize,
    ) -> Option<&CalendarAppointment> {
        let time_slots = CalendarRenderer::generate_time_slots();
        if slot_index >= time_slots.len() {
            return None;
        }

        let slot_time_str = &time_slots[slot_index];
        let (hour, minute) = slot_time_str.split_once(':').and_then(|(h, m)| {
            let hour = h.parse::<u32>().ok()?;
            let minute = m.parse::<u32>().ok()?;
            Some((hour, minute))
        })?;

        let date = NaiveDate::from_ymd_opt(
            self.calendar_state.current_month_start.year(),
            self.calendar_state.current_month_start.month(),
            self.calendar_state.selected_month_day,
        )
        .expect("valid date");

        let slot_datetime = date
            .and_hms_opt(hour, minute, 0)
            .expect("valid time")
            .and_utc();

        // Find appointment that starts at or before this slot and ends after it
        self.calendar_state.appointments.iter().find(|appt| {
            appt.practitioner_id == practitioner_id
                && appt.start_time <= slot_datetime
                && appt.end_time > slot_datetime
                && (self.filter_state.active_status_filters.is_empty()
                    || self
                        .filter_state
                        .active_status_filters
                        .contains(&appt.status))
        })
    }

    #[allow(dead_code)]
    fn detect_overlaps(
        &self,
        practitioner_id: uuid::Uuid,
        slot_index: usize,
    ) -> Vec<&CalendarAppointment> {
        let time_slots = CalendarRenderer::generate_time_slots();
        if slot_index >= time_slots.len() {
            return Vec::new();
        }

        let slot_time_str = &time_slots[slot_index];
        let (hour, minute) = slot_time_str
            .split_once(':')
            .and_then(|(h, m)| {
                let hour = h.parse::<u32>().ok()?;
                let minute = m.parse::<u32>().ok()?;
                Some((hour, minute))
            })
            .unwrap_or((0, 0));

        let date = NaiveDate::from_ymd_opt(
            self.calendar_state.current_month_start.year(),
            self.calendar_state.current_month_start.month(),
            self.calendar_state.selected_month_day,
        )
        .expect("valid date");

        let slot_datetime = date
            .and_hms_opt(hour, minute, 0)
            .expect("valid time")
            .and_utc();

        // Find ALL appointments that overlap at this slot
        self.calendar_state
            .appointments
            .iter()
            .filter(|appt| {
                appt.practitioner_id == practitioner_id
                    && appt.start_time <= slot_datetime
                    && appt.end_time > slot_datetime
                    && (self.filter_state.active_status_filters.is_empty()
                        || self
                            .filter_state
                            .active_status_filters
                            .contains(&appt.status))
            })
            .collect()
    }

    fn previous_time_slot(&mut self) {
        let time_slots = CalendarRenderer::generate_time_slots();
        if let Some(selected) = self.calendar_state.time_slot_state.selected() {
            if selected > 0 {
                self.calendar_state
                    .time_slot_state
                    .select(Some(selected - 1));
            } else {
                self.calendar_state
                    .time_slot_state
                    .select(Some(time_slots.len() - 1));
            }
        }
    }

    fn next_time_slot(&mut self) {
        let time_slots = CalendarRenderer::generate_time_slots();
        if let Some(selected) = self.calendar_state.time_slot_state.selected() {
            if selected < time_slots.len() - 1 {
                self.calendar_state
                    .time_slot_state
                    .select(Some(selected + 1));
            } else {
                self.calendar_state.time_slot_state.select(Some(0));
            }
        }
    }

    fn previous_practitioner_column(&mut self) {
        let visible_count = self.get_visible_practitioner_count();
        if visible_count > 0 {
            if self.calendar_state.selected_practitioner_column > 0 {
                self.calendar_state.selected_practitioner_column -= 1;
            } else {
                self.calendar_state.selected_practitioner_column = visible_count - 1;
            }
        }
    }

    fn next_practitioner_column(&mut self) {
        let visible_count = self.get_visible_practitioner_count();
        if visible_count > 0 {
            if self.calendar_state.selected_practitioner_column < visible_count - 1 {
                self.calendar_state.selected_practitioner_column += 1;
            } else {
                self.calendar_state.selected_practitioner_column = 0;
            }
        }
    }

    fn get_visible_practitioner_count(&self) -> usize {
        self.calendar_state
            .practitioners
            .iter()
            .filter(|p| {
                self.filter_state.active_practitioner_filters.is_empty()
                    || self
                        .filter_state
                        .active_practitioner_filters
                        .contains(&p.id)
            })
            .count()
    }

    fn get_selected_practitioner(&self) -> Option<&Practitioner> {
        let visible_practitioners: Vec<_> = self
            .calendar_state
            .practitioners
            .iter()
            .filter(|p| {
                self.filter_state.active_practitioner_filters.is_empty()
                    || self
                        .filter_state
                        .active_practitioner_filters
                        .contains(&p.id)
            })
            .collect();

        visible_practitioners
            .get(self.calendar_state.selected_practitioner_column)
            .copied()
    }

    #[allow(dead_code)]
    fn render_month_calendar(&self, frame: &mut Frame, area: Rect) {
        let month_year = format!(
            "{} {}",
            CalendarRenderer::get_month_name(self.calendar_state.current_month_start.month()),
            self.calendar_state.current_month_start.year()
        );

        let first_weekday = self.calendar_state.current_month_start.weekday();
        let days_in_month = self.days_in_current_month();

        let mut lines = vec![Line::from(vec![
            Span::styled(
                "Mon",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
            Span::styled(
                "Tue",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
            Span::styled(
                "Wed",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
            Span::styled(
                "Thu",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
            Span::styled(
                "Fri",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
            Span::styled("Sat", Style::default().fg(Color::Cyan)),
            Span::raw(" "),
            Span::styled("Sun", Style::default().fg(Color::Cyan)),
        ])];

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
                if (is_first_week && day_of_week < first_day_offset) || current_day > days_in_month
                {
                    day_cells.push(Span::raw("   "));
                } else {
                    let is_today = self.calendar_state.current_date.year()
                        == self.calendar_state.current_month_start.year()
                        && self.calendar_state.current_date.month()
                            == self.calendar_state.current_month_start.month()
                        && self.calendar_state.current_date.day() == current_day;

                    let is_selected = current_day == self.calendar_state.selected_month_day;
                    let is_weekend = day_of_week >= 5;

                    let style = if is_selected {
                        Style::default()
                            .fg(Color::Black)
                            .bg(Color::Yellow)
                            .add_modifier(Modifier::BOLD)
                    } else if is_today {
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD)
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

        let help = KeybindRegistry::get_help_text(KeybindContext::CalendarMonthView);
        let paragraph = Paragraph::new(lines).block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" {} - {} ", month_year, help))
                .border_style(if self.calendar_state.focus_area == FocusArea::MonthView {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default().fg(Color::White)
                }),
        );

        frame.render_widget(paragraph, area);
    }

    #[allow(dead_code)]
    fn render_day_schedule(&mut self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(area);

        self.render_practitioner_header(frame, chunks[0]);
        self.render_time_slots_grid(frame, chunks[1]);
    }

    #[allow(dead_code)]
    fn render_week_schedule(&mut self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(area);

        self.render_week_header(frame, chunks[0]);
        self.render_week_time_slots_grid(frame, chunks[1]);
    }

    #[allow(dead_code)]
    fn render_week_header(&self, frame: &mut Frame, area: Rect) {
        let dates: Vec<NaiveDate> = (0..7)
            .map(|i| self.calendar_state.week_start_date + chrono::Duration::days(i))
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
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )];

        for date in &dates {
            let day_name = get_day_name(date.weekday());
            let date_str = format!("{} {}", day_name, date.day());

            let style = if *date == self.calendar_state.current_date {
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
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

        let week_start_str = self
            .calendar_state
            .week_start_date
            .format("%b %d")
            .to_string();
        let week_end_str = (self.calendar_state.week_start_date + chrono::Duration::days(6))
            .format("%b %d")
            .to_string();
        let title = format!(" Week: {}-{} ", week_start_str, week_end_str);

        let table = Table::new(vec![header], widths).block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(if self.calendar_state.focus_area == FocusArea::DayView {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default().fg(Color::White)
                }),
        );

        frame.render_widget(table, area);
    }

    #[allow(dead_code)]
    fn render_week_time_slots_grid(&mut self, frame: &mut Frame, area: Rect) {
        let time_slots = CalendarRenderer::generate_time_slots();
        let dates: Vec<NaiveDate> = (0..7)
            .map(|i| self.calendar_state.week_start_date + chrono::Duration::days(i))
            .collect();
        let mut rows = Vec::new();

        for time_slot in time_slots.iter() {
            let mut cells = vec![Cell::from(time_slot.as_str())];

            let (hour, minute) = time_slot
                .split_once(':')
                .and_then(|(h, m)| {
                    let hour = h.parse::<u32>().ok()?;
                    let minute = m.parse::<u32>().ok()?;
                    Some((hour, minute))
                })
                .expect("valid time slot format");

            for date in &dates {
                let slot_datetime = date
                    .and_hms_opt(hour, minute, 0)
                    .expect("valid time")
                    .and_utc();

                let appts_at_slot: Vec<&CalendarAppointment> = self
                    .calendar_state
                    .appointments
                    .iter()
                    .filter(|a| {
                        let appt_date = a.start_time.date_naive();
                        let same_day = appt_date == *date;
                        let overlaps_time =
                            a.start_time <= slot_datetime && a.end_time > slot_datetime;
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
                            AppointmentStatus::Scheduled => {
                                Style::default().fg(Color::White).bg(Color::Blue)
                            }
                            AppointmentStatus::Confirmed => {
                                Style::default().fg(Color::Black).bg(Color::Cyan)
                            }
                            AppointmentStatus::Arrived => {
                                Style::default().fg(Color::Black).bg(Color::Yellow)
                            }
                            AppointmentStatus::InProgress => {
                                Style::default().fg(Color::White).bg(Color::Green)
                            }
                            AppointmentStatus::Completed => {
                                Style::default().fg(Color::White).bg(Color::DarkGray)
                            }
                            AppointmentStatus::NoShow => {
                                Style::default().fg(Color::White).bg(Color::Red)
                            }
                            AppointmentStatus::Cancelled => Style::default().fg(Color::Gray),
                            AppointmentStatus::Rescheduled => Style::default().fg(Color::Magenta),
                        }
                    }
                    _ => Style::default()
                        .fg(Color::Yellow)
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD),
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

        let help = KeybindRegistry::get_help_text(KeybindContext::CalendarWeekView);
        let table = Table::new(rows, widths)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(" Week View - {} ", help)),
            )
            .row_highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");

        frame.render_stateful_widget(table, area, &mut self.calendar_state.time_slot_state);
    }

    #[allow(dead_code)]
    fn render_practitioner_header(&self, frame: &mut Frame, area: Rect) {
        let visible_practitioners: Vec<_> = self
            .calendar_state
            .practitioners
            .iter()
            .filter(|p| {
                self.filter_state.active_practitioner_filters.is_empty()
                    || self
                        .filter_state
                        .active_practitioner_filters
                        .contains(&p.id)
            })
            .collect();

        let mut header_cells = vec![Cell::from("Time").style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )];

        for practitioner in &visible_practitioners {
            let name = format!("Dr. {}", practitioner.last_name);
            header_cells.push(
                Cell::from(name).style(
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
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
        if !self.filter_state.active_status_filters.is_empty() {
            title = format!(
                " Schedule [Status: {}] ",
                self.filter_state.active_status_filters.len()
            );
        }
        if !self.filter_state.active_practitioner_filters.is_empty() {
            let practitioner_names: Vec<String> = visible_practitioners
                .iter()
                .map(|p| format!("Dr. {}", p.last_name))
                .collect();
            title = format!(" Schedule [{}] ", practitioner_names.join(", "));
        }

        let table = Table::new(vec![header], widths).block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(if self.calendar_state.focus_area == FocusArea::DayView {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default().fg(Color::White)
                }),
        );

        frame.render_widget(table, area);
    }

    #[allow(dead_code)]
    fn render_time_slots_grid(&mut self, frame: &mut Frame, area: Rect) {
        CalendarRenderer::render_time_slots_grid(
            &mut self.calendar_state,
            &self.filter_state,
            &self.history_state,
            frame,
            area,
        );
    }

    /// Handle key events when appointment detail modal is open
    fn handle_modal_key_events(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Esc => {
                self.detail_data.showing = false;
                self.detail_data.appointment_id = None;
                self.detail_data.patient = None;
                Action::Render
            }
            KeyCode::Char('a') | KeyCode::Char('A') => Action::AppointmentMarkArrived,
            KeyCode::Char('i') | KeyCode::Char('I') => Action::AppointmentMarkInProgress,
            KeyCode::Char('c') | KeyCode::Char('C') => Action::AppointmentMarkCompleted,
            KeyCode::Char('x') | KeyCode::Char('X') => Action::AppointmentMarkNoShow,
            KeyCode::Char('h') | KeyCode::Char('H') => {
                // Open audit history modal
                if let Some(_appt_id) = self.detail_data.appointment_id {
                    self.detail_data.showing = false;
                    self.audit_data.showing = true;
                    self.audit_data.entries.clear();
                    self.audit_data.selected_index = 0;
                    // TODO: Fetch audit history from service
                    return Action::Render;
                }
                Action::None
            }
            KeyCode::Char('r') | KeyCode::Char('R') => {
                if let Some(appt_id) = self.detail_data.appointment_id {
                    if let Some(appt) = self
                        .calendar_state
                        .appointments
                        .iter()
                        .find(|a| a.id == appt_id)
                    {
                        self.reschedule_data.new_start_time = Some(appt.start_time);
                        self.reschedule_data.new_duration = appt.duration_minutes();
                        self.detail_data.showing = false;
                        self.reschedule_data.showing = true;
                        self.reschedule_data.conflict_warning = None;
                        return Action::Render;
                    }
                }
                Action::None
            }
            KeyCode::Char('o') | KeyCode::Char('O') => {
                if let Some(patient_id) = self.detail_data.patient.as_ref().map(|p| p.id) {
                    return Action::NavigateToClinicalWithPatient(patient_id);
                }
                Action::None
            }
            _ => Action::None,
        }
    }

    fn handle_audit_modal_key_events(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Esc => {
                self.audit_data.showing = false;
                self.audit_data.entries.clear();
                self.audit_data.selected_index = 0;
                self.detail_data.showing = true;
                Action::Render
            }
            KeyCode::Up => {
                if self.audit_data.selected_index > 0 {
                    self.audit_data.selected_index -= 1;
                }
                Action::Render
            }
            KeyCode::Down => {
                if !self.audit_data.entries.is_empty()
                    && self.audit_data.selected_index < self.audit_data.entries.len() - 1
                {
                    self.audit_data.selected_index += 1;
                }
                Action::Render
            }
            _ => Action::None,
        }
    }

    fn handle_batch_menu_key_events(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Esc => {
                self.batch_data.showing_menu = false;
                Action::Render
            }
            KeyCode::Char('1') => {
                self.batch_data.showing_menu = false;
                Action::AppointmentBatchMarkArrived
            }
            KeyCode::Char('2') => {
                self.batch_data.showing_menu = false;
                Action::AppointmentBatchMarkCompleted
            }
            KeyCode::Char('3') => {
                self.batch_data.showing_menu = false;
                self.error_data.message =
                    "Batch cancel with reason not yet implemented. Use individual cancellation."
                        .to_string();
                self.error_data.showing = true;
                Action::Render
            }
            _ => Action::None,
        }
    }

    fn handle_reschedule_modal_key_events(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Esc => {
                self.reschedule_data.showing = false;
                self.reschedule_data.new_start_time = None;
                self.reschedule_data.conflict_warning = None;
                self.detail_data.showing = true;
                Action::Render
            }
            KeyCode::Up => {
                if let Some(current_time) = self.reschedule_data.new_start_time {
                    self.reschedule_data.new_start_time =
                        Some(current_time - chrono::Duration::minutes(15));
                    Action::Render
                } else {
                    Action::None
                }
            }
            KeyCode::Down => {
                if let Some(current_time) = self.reschedule_data.new_start_time {
                    self.reschedule_data.new_start_time =
                        Some(current_time + chrono::Duration::minutes(15));
                    Action::Render
                } else {
                    Action::None
                }
            }
            KeyCode::Char('+') => {
                self.reschedule_data.new_duration += 15;
                Action::Render
            }
            KeyCode::Char('-') => {
                if self.reschedule_data.new_duration > 15 {
                    self.reschedule_data.new_duration -= 15;
                }
                Action::Render
            }
            KeyCode::Enter => Action::AppointmentReschedule,
            _ => Action::None,
        }
    }

    fn handle_search_key_events(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Esc => {
                self.search_data.showing = false;
                self.search_data.query.clear();
                self.search_data.results.clear();
                self.search_data.selected_index = 0;
                Action::Render
            }
            KeyCode::Up => {
                if self.search_data.selected_index > 0 {
                    self.search_data.selected_index -= 1;
                }
                Action::Render
            }
            KeyCode::Down => {
                if !self.search_data.results.is_empty()
                    && self.search_data.selected_index < self.search_data.results.len() - 1
                {
                    self.search_data.selected_index += 1;
                }
                Action::Render
            }
            KeyCode::Enter => {
                if let Some(appt) = self
                    .search_data
                    .results
                    .get(self.search_data.selected_index)
                    .cloned()
                {
                    self.navigate_to_appointment(&appt);
                    self.search_data.showing = false;
                    self.search_data.query.clear();
                    self.search_data.results.clear();
                    self.search_data.selected_index = 0;
                }
                Action::Render
            }
            KeyCode::Char(c) => {
                self.search_data.query.push(c);
                self.filter_appointments_by_query();
                self.search_data.selected_index = 0;
                Action::Render
            }
            KeyCode::Backspace => {
                self.search_data.query.pop();
                self.filter_appointments_by_query();
                self.search_data.selected_index = 0;
                Action::Render
            }
            _ => Action::None,
        }
    }

    fn filter_appointments_by_query(&mut self) {
        if self.search_data.query.is_empty() {
            self.search_data.results.clear();
            return;
        }

        let query_lower = self.search_data.query.to_lowercase();

        self.search_data.results = self
            .calendar_state
            .appointments
            .iter()
            .filter(|appt| {
                let patient_id_str = appt.patient_id.to_string().to_lowercase();
                let type_str = format!("{:?}", appt.appointment_type).to_lowercase();
                let status_str = format!("{:?}", appt.status).to_lowercase();

                patient_id_str.contains(&query_lower)
                    || type_str.contains(&query_lower)
                    || status_str.contains(&query_lower)
            })
            .take(50)
            .cloned()
            .collect();
    }

    fn handle_filter_menu_key_events(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Esc => {
                self.filter_state.showing_filter_menu = false;
                Action::Render
            }
            KeyCode::Char('0') => {
                self.filter_state.active_status_filters.clear();
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
                self.filter_state.showing_practitioner_menu = false;
                Action::Render
            }
            KeyCode::Char('0') => {
                self.filter_state.active_practitioner_filters.clear();
                Action::Render
            }
            KeyCode::Char(c) if c.is_ascii_digit() => {
                let digit = c.to_digit(10).unwrap() as usize;
                if digit > 0 && digit <= self.calendar_state.practitioners.len() {
                    let practitioner_id = self.calendar_state.practitioners[digit - 1].id;
                    self.toggle_practitioner_filter(practitioner_id);
                }
                Action::Render
            }
            _ => Action::None,
        }
    }

    fn navigate_to_appointment(&mut self, appt: &CalendarAppointment) {
        let appt_date = appt.start_time.date_naive();

        self.calendar_state.current_month_start =
            NaiveDate::from_ymd_opt(appt_date.year(), appt_date.month(), 1)
                .expect("valid month start");

        self.calendar_state.selected_month_day = appt_date.day();

        let hour = appt.start_time.hour();
        let minute = appt.start_time.minute();
        let slot_index = ((hour - 8) * 4 + minute / 15) as usize;

        if slot_index < 40 {
            self.calendar_state.time_slot_state.select(Some(slot_index));
        }

        self.calendar_state.focus_area = FocusArea::DayView;
        self.calendar_state.view_mode = ViewMode::Day;
    }

    #[allow(dead_code)]
    fn get_user_display_name(&self, user_id: Uuid) -> String {
        if let Some(practitioner) = self
            .calendar_state
            .practitioners
            .iter()
            .find(|p| p.user_id.is_some_and(|uid| uid == user_id))
        {
            format!("{} {}", practitioner.title, practitioner.last_name)
        } else {
            format!("User {}...", &user_id.to_string()[..8])
        }
    }

    /// Render appointment detail modal as a centered overlay
    #[allow(dead_code)]
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

        if let Some(appt_id) = self.detail_data.appointment_id {
            if let Some(appt) = self
                .calendar_state
                .appointments
                .iter()
                .find(|a| a.id == appt_id)
            {
                // Header: Patient name and appointment type
                let patient_name = if let Some(ref patient) = self.detail_data.patient {
                    format!("{} {}", patient.first_name, patient.last_name)
                } else {
                    "Loading...".to_string()
                };

                lines.push(Line::from(vec![
                    Span::styled("Patient: ", Style::default().fg(Color::Yellow)),
                    Span::styled(
                        patient_name,
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]));

                lines.push(Line::from(""));
                lines.push(Line::from(vec![
                    Span::styled("Type: ", Style::default().fg(Color::Yellow)),
                    Span::styled(
                        format!("{:?}", appt.appointment_type),
                        Style::default().fg(Color::White),
                    ),
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
                    lines.push(Line::from(vec![Span::styled(
                        "⚠ URGENT",
                        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                    )]));
                }

                let appt_start_hour = appt.start_time.hour();
                let appt_start_minute = appt.start_time.minute();
                let slot_index = ((appt_start_hour - 8) * 4 + appt_start_minute / 15) as usize;
                let overlaps = self.detect_overlaps(appt.practitioner_id, slot_index);

                if overlaps.len() > 1 {
                    lines.push(Line::from(vec![
                        Span::styled(
                            "⚠ CONFLICT: ",
                            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(
                            format!("{} overlapping appointments detected", overlaps.len()),
                            Style::default().fg(Color::Red),
                        ),
                    ]));

                    for overlap_appt in &overlaps {
                        if overlap_appt.id != appt.id {
                            lines.push(Line::from(vec![
                                Span::styled("  • ", Style::default().fg(Color::Red)),
                                Span::styled(
                                    format!(
                                        "ID: {} ({} - {})",
                                        &overlap_appt.id.to_string()[..8],
                                        overlap_appt.start_time.format("%H:%M"),
                                        overlap_appt.end_time.format("%H:%M")
                                    ),
                                    Style::default().fg(Color::White),
                                ),
                            ]));
                        }
                    }
                }

                lines.push(Line::from(""));

                // Time details
                lines.push(Line::from(vec![
                    Span::styled("Date: ", Style::default().fg(Color::Yellow)),
                    Span::styled(
                        appt.start_time.format("%Y-%m-%d").to_string(),
                        Style::default().fg(Color::White),
                    ),
                ]));

                lines.push(Line::from(vec![
                    Span::styled("Time: ", Style::default().fg(Color::Yellow)),
                    Span::styled(
                        format!(
                            "{} - {}",
                            appt.start_time.format("%H:%M"),
                            appt.end_time.format("%H:%M")
                        ),
                        Style::default().fg(Color::White),
                    ),
                ]));

                lines.push(Line::from(""));

                // Practitioner
                if let Some(practitioner) = self
                    .calendar_state
                    .practitioners
                    .iter()
                    .find(|p| p.id == appt.practitioner_id)
                {
                    lines.push(Line::from(vec![
                        Span::styled("Practitioner: ", Style::default().fg(Color::Yellow)),
                        Span::styled(
                            format!("Dr. {}", practitioner.last_name),
                            Style::default().fg(Color::White),
                        ),
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
            Span::styled(
                "A",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(": Arrived  ", Style::default().fg(Color::White)),
            Span::styled(
                "I",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(": In Progress  ", Style::default().fg(Color::White)),
            Span::styled(
                "C",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(": Completed  ", Style::default().fg(Color::White)),
            Span::styled(
                "X",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(": No Show", Style::default().fg(Color::White)),
        ]));
        lines.push(Line::from(vec![
            Span::styled(
                "R",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(": Reschedule  ", Style::default().fg(Color::White)),
            Span::styled(
                "H",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(": History  ", Style::default().fg(Color::White)),
            Span::styled(
                "Esc",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(": Close", Style::default().fg(Color::White)),
        ]));

        let modal_content = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Appointment Details ")
                    .border_style(Style::default().fg(Color::Yellow)),
            )
            .wrap(ratatui::widgets::Wrap { trim: true });

        frame.render_widget(modal_content, modal_area);
    }

    #[allow(dead_code)]
    fn render_audit_history_modal(&self, frame: &mut Frame, area: Rect) {
        let modal_area = Rect {
            x: area.width / 5,
            y: area.height / 6,
            width: area.width * 3 / 5,
            height: area.height * 2 / 3,
        };

        let mut lines = Vec::new();

        if self.audit_data.entries.is_empty() {
            lines.push(Line::from(vec![Span::styled(
                "No audit history available",
                Style::default().fg(Color::Gray),
            )]));
        } else {
            lines.push(Line::from(vec![
                Span::styled(
                    "Timestamp",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" | "),
                Span::styled(
                    "Action",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" | "),
                Span::styled(
                    "Changed By",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
            ]));
            lines.push(Line::from("")); // Empty line for spacing

            for (index, entry) in self.audit_data.entries.iter().enumerate() {
                let timestamp = entry.changed_at.format("%Y-%m-%d %H:%M:%S").to_string();

                // Try to find practitioner name, fall back to UUID
                let user_display = self.get_user_display_name(entry.changed_by);

                let (action_text, action_color) = match &entry.action {
                    AuditAction::Created => ("Created".to_string(), Color::Green),
                    AuditAction::Updated => ("Updated".to_string(), Color::Cyan),
                    AuditAction::StatusChanged { from, to } => {
                        (format!("Status: {} → {}", from, to), Color::Yellow)
                    }
                    AuditAction::Rescheduled { from, to } => (
                        format!("Resched: {} → {}", from.format("%H:%M"), to.format("%H:%M")),
                        Color::Magenta,
                    ),
                    AuditAction::Cancelled { reason } => {
                        (format!("Cancelled: {}", reason), Color::Red)
                    }
                };

                let is_selected = index == self.audit_data.selected_index;
                let base_style = if is_selected {
                    Style::default()
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                let action_style = base_style.fg(action_color);
                let timestamp_style = base_style.fg(Color::Cyan);
                let user_style = base_style.fg(Color::White);

                lines.push(Line::from(vec![
                    Span::styled(timestamp, timestamp_style),
                    Span::raw(" | "),
                    Span::styled(action_text, action_style),
                    Span::raw(" | "),
                    Span::styled(user_display, user_style),
                ]));
            }
        }

        lines.push(Line::from(""));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled(
                "↑/↓",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(": Navigate  ", Style::default().fg(Color::White)),
            Span::styled(
                "Esc",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(": Close", Style::default().fg(Color::White)),
        ]));

        let content = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Audit History "),
            )
            .wrap(ratatui::widgets::Wrap { trim: true });

        frame.render_widget(content, modal_area);
    }

    #[allow(clippy::vec_init_then_push)]
    #[allow(dead_code)]
    fn render_search_modal(&self, frame: &mut Frame, area: Rect) {
        let modal_area = Rect {
            x: area.width / 5,
            y: area.height / 6,
            width: area.width * 3 / 5,
            height: area.height * 2 / 3,
        };

        let mut lines = Vec::new();

        lines.push(Line::from(vec![Span::styled(
            "Search Appointments",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]));
        lines.push(Line::from(""));

        lines.push(Line::from(vec![
            Span::styled("Query: ", Style::default().fg(Color::Cyan)),
            Span::styled(
                &self.search_data.query,
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("█", Style::default().fg(Color::White)),
        ]));

        lines.push(Line::from(""));

        if self.search_data.query.is_empty() {
            lines.push(Line::from(vec![Span::styled(
                "Start typing to search...",
                Style::default().fg(Color::DarkGray),
            )]));
        } else if self.search_data.results.is_empty() {
            lines.push(Line::from(vec![Span::styled(
                "No appointments found",
                Style::default().fg(Color::Red),
            )]));
        } else {
            lines.push(Line::from(vec![Span::styled(
                format!(
                    "Found {} result(s) (showing up to 50)",
                    self.search_data.results.len()
                ),
                Style::default().fg(Color::Green),
            )]));
            lines.push(Line::from(""));

            for (idx, appt) in self.search_data.results.iter().enumerate() {
                let is_selected = idx == self.search_data.selected_index;
                let prefix = if is_selected { "→ " } else { "  " };

                let patient_id_short = &appt.patient_id.to_string()[..8];
                let date_str = appt.start_time.format("%Y-%m-%d").to_string();
                let time_str = appt.start_time.format("%H:%M").to_string();
                let type_str = format!("{:?}", appt.appointment_type);

                let style = if is_selected {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
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
            Span::styled(
                "↑↓",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(": Navigate  ", Style::default().fg(Color::White)),
            Span::styled(
                "Enter",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(": Select  ", Style::default().fg(Color::White)),
            Span::styled(
                "Esc",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(": Close", Style::default().fg(Color::White)),
        ]));

        let modal_content = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Search Appointments ")
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .wrap(ratatui::widgets::Wrap { trim: true });

        frame.render_widget(modal_content, modal_area);
    }

    #[allow(dead_code)]
    fn render_reschedule_modal(&mut self, frame: &mut Frame, area: Rect) {
        let modal_area = Rect {
            x: area.width / 5,
            y: area.height / 6,
            width: area.width * 3 / 5,
            height: area.height / 2,
        };

        let mut lines = Vec::new();

        if let Some(appt_id) = self.detail_data.appointment_id {
            if let Some(appt) = self
                .calendar_state
                .appointments
                .iter()
                .find(|a| a.id == appt_id)
            {
                let patient_name = if let Some(ref patient) = self.detail_data.patient {
                    format!("{} {}", patient.first_name, patient.last_name)
                } else {
                    "Loading...".to_string()
                };

                lines.push(Line::from(vec![Span::styled(
                    "Reschedule Appointment",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )]));
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
                    Span::styled(
                        format!("{} minutes", current_duration),
                        Style::default().fg(Color::White),
                    ),
                ]));

                lines.push(Line::from(""));
                lines.push(Line::from(vec![
                    Span::styled("New Time: ", Style::default().fg(Color::Yellow)),
                    Span::styled(
                        if let Some(new_time) = self.reschedule_data.new_start_time {
                            format!("{}", new_time.format("%Y-%m-%d %H:%M"))
                        } else {
                            current_time_str
                        },
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]));

                lines.push(Line::from(vec![
                    Span::styled("New Duration: ", Style::default().fg(Color::Yellow)),
                    Span::styled(
                        format!("{} minutes", self.reschedule_data.new_duration),
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]));

                if let Some(ref warning) = self.reschedule_data.conflict_warning {
                    lines.push(Line::from(""));
                    lines.push(Line::from(vec![
                        Span::styled(
                            "⚠ ",
                            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(warning, Style::default().fg(Color::Red)),
                    ]));
                }
            }
        }

        lines.push(Line::from(""));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled(
                "↑↓",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(": Time  ", Style::default().fg(Color::White)),
            Span::styled(
                "+/-",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(": Duration  ", Style::default().fg(Color::White)),
            Span::styled(
                "Enter",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(": Save  ", Style::default().fg(Color::White)),
            Span::styled(
                "Esc",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(": Cancel", Style::default().fg(Color::White)),
        ]));

        let modal_content = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Reschedule Appointment ")
                    .border_style(Style::default().fg(Color::Yellow)),
            )
            .wrap(ratatui::widgets::Wrap { trim: true });

        frame.render_widget(modal_content, modal_area);
    }

    #[allow(dead_code)]
    fn render_filter_menu(&self, frame: &mut Frame, area: Rect) {
        let modal_area = Rect {
            x: area.width / 5,
            y: area.height / 6,
            width: area.width * 3 / 5,
            height: area.height * 2 / 3,
        };

        let mut lines = Vec::new();

        lines.push(Line::from(vec![Span::styled(
            "Filter by Status",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]));
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
            let is_active = self.filter_state.active_status_filters.contains(&status);
            let checkbox = if is_active { "☑" } else { "☐" };
            let color = if is_active {
                Color::Green
            } else {
                Color::White
            };

            lines.push(Line::from(vec![
                Span::styled(format!("  {} ", checkbox), Style::default().fg(color)),
                Span::styled(
                    format!("[{}] ", key),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(label, Style::default().fg(color)),
            ]));
        }

        lines.push(Line::from(""));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled(
                "0",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(": Clear all  ", Style::default().fg(Color::White)),
            Span::styled(
                "Esc",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(": Close", Style::default().fg(Color::White)),
        ]));

        let filter_count = self.filter_state.active_status_filters.len();
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
                    .border_style(Style::default().fg(Color::Green)),
            )
            .wrap(ratatui::widgets::Wrap { trim: true });

        frame.render_widget(modal_content, modal_area);
    }

    #[allow(dead_code)]
    fn render_practitioner_menu(&self, frame: &mut Frame, area: Rect) {
        let modal_area = Rect {
            x: area.width / 5,
            y: area.height / 6,
            width: area.width * 3 / 5,
            height: area.height * 2 / 3,
        };

        let mut lines = Vec::new();

        lines.push(Line::from(vec![Span::styled(
            "Filter by Practitioner",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]));
        lines.push(Line::from(""));

        for (idx, practitioner) in self.calendar_state.practitioners.iter().enumerate() {
            let key = (idx + 1).to_string();
            let is_active = self
                .filter_state
                .active_practitioner_filters
                .contains(&practitioner.id);
            let checkbox = if is_active { "☑" } else { "☐" };
            let color = if is_active {
                Color::Green
            } else {
                Color::White
            };
            let name = format!("Dr. {}", practitioner.last_name);

            lines.push(Line::from(vec![
                Span::styled(format!("  {} ", checkbox), Style::default().fg(color)),
                Span::styled(
                    format!("[{}] ", key),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(name, Style::default().fg(color)),
            ]));
        }

        lines.push(Line::from(""));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled(
                "0",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(": Clear all  ", Style::default().fg(Color::White)),
            Span::styled(
                "Esc",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(": Close", Style::default().fg(Color::White)),
        ]));

        let filter_count = self.filter_state.active_practitioner_filters.len();
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
                    .border_style(Style::default().fg(Color::Magenta)),
            )
            .wrap(ratatui::widgets::Wrap { trim: true });

        frame.render_widget(modal_content, modal_area);
    }

    fn initiate_status_change(&mut self, new_status: AppointmentStatus) -> Action {
        if let Some(selected_slot) = self.calendar_state.time_slot_state.selected() {
            if let Some(practitioner) = self.get_selected_practitioner() {
                let practitioner_id = practitioner.id;
                if let Some(appt) = self.find_appointment_for_slot(practitioner_id, selected_slot) {
                    let appt_id = appt.id;
                    let patient_id_str = appt.patient_id.to_string();
                    let patient_text = format!("Patient {}", &patient_id_str[..8]);

                    let status_text = match new_status {
                        AppointmentStatus::Arrived => "Arrived",
                        AppointmentStatus::Completed => "Completed",
                        AppointmentStatus::NoShow => "No Show",
                        _ => "Unknown",
                    };

                    let needs_confirmation = matches!(new_status, AppointmentStatus::NoShow);

                    if needs_confirmation {
                        self.confirmation_data.message = format!(
                            "Mark {} as {}?\n\nThis is a serious status change.\nPress 'y' to confirm or 'n' to cancel.",
                            patient_text, status_text
                        );
                        self.confirmation_data.pending_status = Some(new_status);
                        self.confirmation_data.pending_appointment_id = Some(appt_id);
                        self.confirmation_data.showing = true;
                        return Action::Render;
                    } else {
                        self.confirmation_data.pending_appointment_id = Some(appt_id);
                        return self.execute_status_change(appt_id, new_status);
                    }
                }
            }
        }
        Action::None
    }

    fn execute_status_change(&mut self, appt_id: Uuid, new_status: AppointmentStatus) -> Action {
        if let Some(appt) = self
            .calendar_state
            .appointments
            .iter()
            .find(|a| a.id == appt_id)
        {
            let old_status = appt.status;
            self.add_to_undo_stack(appt_id, old_status);
        }

        match new_status {
            AppointmentStatus::Arrived => Action::AppointmentMarkArrived,
            AppointmentStatus::InProgress => Action::AppointmentMarkInProgress,
            AppointmentStatus::Completed => Action::AppointmentMarkCompleted,
            AppointmentStatus::NoShow => Action::AppointmentMarkNoShow,
            _ => Action::None,
        }
    }

    fn handle_confirmation_key_events(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                self.confirmation_data.showing = false;
                if let (Some(appt_id), Some(status)) = (
                    self.confirmation_data.pending_appointment_id,
                    self.confirmation_data.pending_status,
                ) {
                    self.confirmation_data.pending_appointment_id = None;
                    self.confirmation_data.pending_status = None;
                    return self.execute_status_change(appt_id, status);
                }
                Action::None
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                self.confirmation_data.showing = false;
                self.confirmation_data.pending_appointment_id = None;
                self.confirmation_data.pending_status = None;
                Action::Render
            }
            _ => Action::None,
        }
    }

    fn add_to_undo_stack(&mut self, appt_id: Uuid, old_status: AppointmentStatus) {
        self.history_state
            .recent_status_changes
            .push((appt_id, old_status));
        if self.history_state.recent_status_changes.len() > 5 {
            self.history_state.recent_status_changes.remove(0);
        }
        self.history_state.undo_timestamp = Some(Utc::now());
    }

    fn handle_undo(&mut self) -> Action {
        self.clear_undo_if_expired();

        if self.history_state.recent_status_changes.is_empty() {
            return Action::None;
        }

        if let Some((appt_id, old_status)) = self.history_state.recent_status_changes.pop() {
            tracing::info!(
                "Undoing status change for appointment {}, restoring to {:?}",
                appt_id,
                old_status
            );

            self.detail_data.appointment_id = Some(appt_id);
            self.history_state.undo_timestamp = None;

            match old_status {
                AppointmentStatus::Arrived => Action::AppointmentMarkArrived,
                AppointmentStatus::InProgress => Action::AppointmentMarkInProgress,
                AppointmentStatus::Completed => Action::AppointmentMarkCompleted,
                AppointmentStatus::NoShow => Action::AppointmentMarkNoShow,
                _ => {
                    tracing::warn!("Cannot undo to status {:?} - not supported", old_status);
                    Action::None
                }
            }
        } else {
            Action::None
        }
    }

    fn clear_undo_if_expired(&mut self) {
        if let Some(timestamp) = self.history_state.undo_timestamp {
            let elapsed = Utc::now().signed_duration_since(timestamp);
            if elapsed.num_seconds() > 30 {
                self.clear_undo_stack();
            }
        }
    }

    fn clear_undo_stack(&mut self) {
        self.history_state.recent_status_changes.clear();
        self.history_state.undo_timestamp = None;
    }

    #[allow(dead_code)]
    fn render_confirmation_overlay(&self, frame: &mut Frame, area: Rect) {
        let modal_area = Rect {
            x: area.width / 4,
            y: area.height / 3,
            width: area.width / 2,
            height: area.height / 3,
        };

        let message = self.confirmation_data.message.clone();

        let lines = vec![
            Line::from(vec![Span::styled(
                "⚠ Confirmation Required",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(vec![Span::styled(
                message,
                Style::default().fg(Color::White),
            )]),
            Line::from(""),
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    "Y",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(": Confirm  ", Style::default().fg(Color::White)),
                Span::styled(
                    "N",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
                Span::styled(": Cancel  ", Style::default().fg(Color::White)),
                Span::styled(
                    "Esc",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(": Cancel", Style::default().fg(Color::White)),
            ]),
        ];

        let modal_content = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Confirm Action ")
                    .border_style(Style::default().fg(Color::Yellow)),
            )
            .wrap(ratatui::widgets::Wrap { trim: true });

        frame.render_widget(modal_content, modal_area);
    }

    #[allow(dead_code)]
    fn render_error_modal(&self, frame: &mut Frame, area: Rect) {
        let modal_area = Rect {
            x: area.width / 4,
            y: area.height / 3,
            width: area.width / 2,
            height: area.height / 3,
        };

        let lines = vec![
            Line::from(vec![Span::styled(
                "⚠ Error",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(vec![Span::styled(
                &self.error_data.message,
                Style::default().fg(Color::White),
            )]),
            Line::from(""),
            Line::from(""),
            Line::from(vec![
                Span::styled("Press ", Style::default().fg(Color::White)),
                Span::styled(
                    "Esc",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" or ", Style::default().fg(Color::White)),
                Span::styled(
                    "Enter",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" to close", Style::default().fg(Color::White)),
            ]),
        ];

        let modal_content = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Invalid Operation ")
                    .border_style(Style::default().fg(Color::Red)),
            )
            .wrap(ratatui::widgets::Wrap { trim: true });

        frame.render_widget(modal_content, modal_area);
    }

    #[allow(dead_code)]
    fn render_batch_menu(&self, frame: &mut Frame, area: Rect) {
        let modal_area = Rect {
            x: area.width / 4,
            y: area.height / 3,
            width: area.width / 2,
            height: area.height / 3,
        };

        let selection_count = self.history_state.selected_appointments.len();

        let lines = vec![
            Line::from(vec![Span::styled(
                format!("Batch Operations ({} appointments)", selection_count),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    "[1] ",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("Mark all as Arrived", Style::default().fg(Color::Green)),
            ]),
            Line::from(vec![
                Span::styled(
                    "[2] ",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("Mark all as Completed", Style::default().fg(Color::Green)),
            ]),
            Line::from(vec![
                Span::styled(
                    "[3] ",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "Cancel all (not yet implemented)",
                    Style::default().fg(Color::DarkGray),
                ),
            ]),
            Line::from(""),
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    "Select an option or press ",
                    Style::default().fg(Color::White),
                ),
                Span::styled(
                    "Esc",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" to cancel", Style::default().fg(Color::White)),
            ]),
        ];

        let modal_content = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Batch Operations ")
                    .border_style(Style::default().fg(Color::Yellow)),
            )
            .wrap(ratatui::widgets::Wrap { trim: true });

        frame.render_widget(modal_content, modal_area);
    }

    #[allow(dead_code)]
    fn render_batch_progress(&self, frame: &mut Frame, area: Rect) {
        let modal_area = Rect {
            x: area.width / 4,
            y: area.height / 3,
            width: area.width / 2,
            height: area.height / 4,
        };

        let progress_percent = if self.batch_data.progress_total > 0 {
            (self.batch_data.progress_current as f64 / self.batch_data.progress_total as f64
                * 100.0) as usize
        } else {
            0
        };

        let progress_bar_width = 40;
        let filled = (progress_bar_width * progress_percent) / 100;
        let progress_bar = format!(
            "[{}{}] {}%",
            "=".repeat(filled),
            " ".repeat(progress_bar_width - filled),
            progress_percent
        );

        let lines = vec![
            Line::from(vec![Span::styled(
                "Processing Batch Operation",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(vec![Span::styled(
                &self.batch_data.progress_message,
                Style::default().fg(Color::White),
            )]),
            Line::from(""),
            Line::from(vec![Span::styled(
                progress_bar,
                Style::default().fg(Color::Cyan),
            )]),
            Line::from(""),
            Line::from(vec![Span::styled(
                format!(
                    "Processing {} of {}",
                    self.batch_data.progress_current, self.batch_data.progress_total
                ),
                Style::default().fg(Color::DarkGray),
            )]),
        ];

        let modal_content = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Please Wait ")
                    .border_style(Style::default().fg(Color::Yellow)),
            )
            .wrap(ratatui::widgets::Wrap { trim: true });

        frame.render_widget(modal_content, modal_area);
    }

    fn handle_help_keys(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Esc | KeyCode::Char('?') => {
                self.modal_state.hide();
                Action::Render
            }
            _ => Action::None,
        }
    }

    fn handle_error_modal_keys(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Esc | KeyCode::Enter => {
                self.error_data.showing = false;
                self.error_data.message.clear();
                Action::Render
            }
            _ => Action::None,
        }
    }

    fn handle_multi_select_keys(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Esc => {
                self.history_state.multi_select_mode = false;
                self.history_state.selected_appointments.clear();
                tracing::info!("Exiting multi-select mode");
                Action::Render
            }
            KeyCode::Char(' ') if self.calendar_state.focus_area == FocusArea::DayView => {
                if let Some(selected_slot) = self.calendar_state.time_slot_state.selected() {
                    let visible_practitioners: Vec<_> = self
                        .calendar_state
                        .practitioners
                        .iter()
                        .filter(|p| {
                            self.filter_state.active_practitioner_filters.is_empty()
                                || self
                                    .filter_state
                                    .active_practitioner_filters
                                    .contains(&p.id)
                        })
                        .collect();

                    if let Some(practitioner) = visible_practitioners.first() {
                        if let Some(appt) =
                            self.find_appointment_for_slot(practitioner.id, selected_slot)
                        {
                            let appt_id = appt.id;
                            if self.history_state.selected_appointments.contains(&appt_id) {
                                self.history_state.selected_appointments.remove(&appt_id);
                                tracing::info!("Deselected appointment {}", appt_id);
                            } else {
                                self.history_state.selected_appointments.insert(appt_id);
                                tracing::info!("Selected appointment {}", appt_id);
                            }
                        }
                    }
                }
                Action::Render
            }
            KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.history_state.selected_appointments.clear();
                for appt in &self.calendar_state.appointments {
                    if (self.filter_state.active_status_filters.is_empty()
                        || self
                            .filter_state
                            .active_status_filters
                            .contains(&appt.status))
                        && (self.filter_state.active_practitioner_filters.is_empty()
                            || self
                                .filter_state
                                .active_practitioner_filters
                                .contains(&appt.practitioner_id))
                    {
                        self.history_state.selected_appointments.insert(appt.id);
                    }
                }
                tracing::info!(
                    "Selected all {} visible appointments",
                    self.history_state.selected_appointments.len()
                );
                Action::Render
            }
            KeyCode::Char('b') if !self.history_state.selected_appointments.is_empty() => {
                self.batch_data.showing_menu = true;
                tracing::info!(
                    "Batch operations menu requested for {} appointments",
                    self.history_state.selected_appointments.len()
                );
                Action::Render
            }
            _ => Action::None,
        }
    }

    fn handle_month_view_keys(&mut self, key: KeyEvent) -> Action {
        if KeyDispatcher::is_handled(&KeybindContext::CalendarMonthView, key) {
            if let Some(action_name) =
                KeybindRegistry::lookup_action(&KeybindContext::CalendarMonthView, key)
            {
                match action_name {
                    "Week Up" => {
                        let current_date = NaiveDate::from_ymd_opt(
                            self.calendar_state.current_month_start.year(),
                            self.calendar_state.current_month_start.month(),
                            self.calendar_state.selected_month_day,
                        )
                        .unwrap_or(self.calendar_state.current_month_start);
                        let new_date = current_date - chrono::Duration::days(7);
                        self.calendar_state.current_month_start =
                            NaiveDate::from_ymd_opt(new_date.year(), new_date.month(), 1)
                                .expect("first day of month is always valid");
                        self.calendar_state.selected_month_day = new_date.day().min(
                            CalendarRenderer::days_in_month(new_date.year(), new_date.month()),
                        );
                        return Action::Render;
                    }
                    "Week Down" => {
                        let current_date = NaiveDate::from_ymd_opt(
                            self.calendar_state.current_month_start.year(),
                            self.calendar_state.current_month_start.month(),
                            self.calendar_state.selected_month_day,
                        )
                        .unwrap_or(self.calendar_state.current_month_start);
                        let new_date = current_date + chrono::Duration::days(7);
                        self.calendar_state.current_month_start =
                            NaiveDate::from_ymd_opt(new_date.year(), new_date.month(), 1)
                                .expect("first day of month is always valid");
                        self.calendar_state.selected_month_day = new_date.day().min(
                            CalendarRenderer::days_in_month(new_date.year(), new_date.month()),
                        );
                        return Action::Render;
                    }
                    "Day Back" => {
                        self.previous_day();
                        return Action::Render;
                    }
                    "Day Forward" => {
                        self.next_day();
                        return Action::Render;
                    }
                    "Month Back" => {
                        self.previous_month();
                        return Action::Render;
                    }
                    "Month Forward" => {
                        self.next_month();
                        return Action::Render;
                    }
                    "Today" => {
                        self.jump_to_today();
                        return Action::Render;
                    }
                    "Day View" => {
                        self.calendar_state.focus_area = FocusArea::DayView;
                        return Action::Render;
                    }
                    "New" => {
                        return Action::AppointmentCreate;
                    }
                    _ => {}
                }
            }
        }

        Action::None
    }

    fn handle_day_view_keys(&mut self, key: KeyEvent) -> Action {
        let context = match self.calendar_state.view_mode {
            ViewMode::Day => KeybindContext::CalendarDayView,
            ViewMode::Week => KeybindContext::CalendarWeekView,
        };

        if KeyDispatcher::is_handled(&context, key) {
            if let Some(action_name) = KeybindRegistry::lookup_action(&context, key) {
                match action_name {
                    "Up" => {
                        self.clear_undo_if_expired();
                        self.previous_time_slot();
                        return Action::Render;
                    }
                    "Down" => {
                        self.clear_undo_if_expired();
                        self.next_time_slot();
                        return Action::Render;
                    }
                    "Month" => {
                        self.clear_undo_stack();
                        self.calendar_state.focus_area = FocusArea::MonthView;
                        return Action::Render;
                    }
                    "Details" => {
                        if let Some(selected_slot) =
                            self.calendar_state.time_slot_state.selected()
                        {
                            if let Some(practitioner) = self.get_selected_practitioner() {
                                if let Some(appt) =
                                    self.find_appointment_for_slot(practitioner.id, selected_slot)
                                {
                                    self.detail_data.appointment_id = Some(appt.id);
                                    self.detail_data.showing = true;
                                    return Action::Render;
                                }
                            }
                        }
                        return Action::None;
                    }
                    "Arrived" => {
                        return self.initiate_status_change(AppointmentStatus::Arrived);
                    }
                    "In Progress" => {
                        return self.initiate_status_change(AppointmentStatus::InProgress);
                    }
                    "Completed" => {
                        return self.initiate_status_change(AppointmentStatus::Completed);
                    }
                    "No Show" => {
                        return self.initiate_status_change(AppointmentStatus::NoShow);
                    }
                    "New" => {
                        return Action::AppointmentCreate;
                    }
                    "Week" => {
                        self.calendar_state.view_mode = match self.calendar_state.view_mode {
                            ViewMode::Day => ViewMode::Week,
                            ViewMode::Week => ViewMode::Day,
                        };
                        return Action::Render;
                    }
                    "Day" => {
                        self.calendar_state.view_mode = ViewMode::Day;
                        return Action::Render;
                    }
                    "Search" => {
                        self.search_data.showing = true;
                        self.search_data.query.clear();
                        self.search_data.results.clear();
                        self.search_data.selected_index = 0;
                        return Action::Render;
                    }
                    "Filter" => {
                        self.filter_state.showing_filter_menu = true;
                        return Action::Render;
                    }
                    "Practitioner" => {
                        self.filter_state.showing_practitioner_menu = true;
                        return Action::Render;
                    }
                    "Multi-Select" => {
                        self.history_state.multi_select_mode =
                            !self.history_state.multi_select_mode;
                        if !self.history_state.multi_select_mode {
                            self.history_state.selected_appointments.clear();
                        }
                        tracing::info!(
                            "Multi-select mode: {}",
                            if self.history_state.multi_select_mode {
                                "ON"
                            } else {
                                "OFF"
                            }
                        );
                        return Action::Render;
                    }
                    "Undo" => {
                        return self.handle_undo();
                    }
                    "Week Back" => {
                        self.calendar_state.week_start_date -= chrono::Duration::days(7);
                        return Action::Render;
                    }
                    "Week Forward" => {
                        self.calendar_state.week_start_date += chrono::Duration::days(7);
                        return Action::Render;
                    }
                    _ => {}
                }
            }
        }

        match key.code {
            KeyCode::Left if key.modifiers.contains(KeyModifiers::SHIFT) => {
                self.calendar_state.week_start_date -= chrono::Duration::days(7);
                return Action::Render;
            }
            KeyCode::Right if key.modifiers.contains(KeyModifiers::SHIFT) => {
                self.calendar_state.week_start_date += chrono::Duration::days(7);
                return Action::Render;
            }
            KeyCode::Left if !key.modifiers.contains(KeyModifiers::SHIFT) => {
                self.previous_practitioner_column();
                return Action::Render;
            }
            KeyCode::Right if !key.modifiers.contains(KeyModifiers::SHIFT) => {
                self.next_practitioner_column();
                return Action::Render;
            }
            _ => {}
        }

        Action::None
    }

    fn is_any_modal_active(&self) -> bool {
        self.modal_state.is_showing(ModalType::Help)
            || self.confirmation_data.showing
            || self.error_data.showing
            || self.filter_state.showing_filter_menu
            || self.filter_state.showing_practitioner_menu
            || self.search_data.showing
            || self.reschedule_data.showing
            || self.detail_data.showing
            || self.audit_data.showing
            || self.batch_data.showing_menu
    }

    fn handle_modal_events(&mut self, key: KeyEvent) -> Action {
        if self.modal_state.is_showing(ModalType::Help) {
            return self.handle_help_keys(key);
        }
        if self.confirmation_data.showing {
            return self.handle_confirmation_key_events(key);
        }
        if self.error_data.showing {
            return self.handle_error_modal_keys(key);
        }
        if self.filter_state.showing_filter_menu {
            return self.handle_filter_menu_key_events(key);
        }
        if self.filter_state.showing_practitioner_menu {
            return self.handle_practitioner_menu_key_events(key);
        }
        if self.search_data.showing {
            return self.handle_search_key_events(key);
        }
        if self.reschedule_data.showing {
            return self.handle_reschedule_modal_key_events(key);
        }
        if self.detail_data.showing {
            return self.handle_modal_key_events(key);
        }
        if self.audit_data.showing {
            return self.handle_audit_modal_key_events(key);
        }
        if self.batch_data.showing_menu {
            return self.handle_batch_menu_key_events(key);
        }

        Action::None
    }

    fn handle_calendar_events(&mut self, key: KeyEvent) -> Action {
        if key.code == KeyCode::Char('z') && key.modifiers.contains(KeyModifiers::CONTROL) {
            return self.handle_undo();
        }
        if key.code == KeyCode::Char('?') {
            self.modal_state.show(ModalType::Help);
            return Action::Render;
        }
        if key.code == KeyCode::Char('/') {
            self.search_data.showing = true;
            self.search_data.query.clear();
            self.search_data.results.clear();
            self.search_data.selected_index = 0;
            return Action::Render;
        }
        if key.code == KeyCode::Char('f') {
            self.filter_state.showing_filter_menu = true;
            return Action::Render;
        }
        if key.code == KeyCode::Char('p') {
            self.filter_state.showing_practitioner_menu = true;
            return Action::Render;
        }

        if key.code == KeyCode::Char('m') {
            self.history_state.multi_select_mode = !self.history_state.multi_select_mode;
            if !self.history_state.multi_select_mode {
                self.history_state.selected_appointments.clear();
            }
            tracing::info!(
                "Multi-select mode: {}",
                if self.history_state.multi_select_mode {
                    "ON"
                } else {
                    "OFF"
                }
            );
            return Action::Render;
        }

        if self.history_state.multi_select_mode {
            return self.handle_multi_select_keys(key);
        }

        match self.calendar_state.focus_area {
            FocusArea::MonthView => self.handle_month_view_keys(key),
            FocusArea::DayView => self.handle_day_view_keys(key),
        }
    }

    fn calculate_day_from_click(&self, col: u16, row: u16, area: Rect) -> Option<u32> {
        let relative_col = col.saturating_sub(area.x);
        let relative_row = row.saturating_sub(area.y);

        tracing::debug!(
            "Day calc: col={}, row={}, relative_col={}, relative_row={}, area={:?}",
            col,
            row,
            relative_col,
            relative_row,
            area
        );

        if relative_row < 2 || relative_col < 1 {
            return None;
        }

        let content_row = relative_row.saturating_sub(2);
        let content_col = relative_col.saturating_sub(1);

        tracing::debug!("Content: row={}, col={}", content_row, content_col);

        if content_row > 5 {
            return None;
        }

        let day_of_week = (content_col / 3) as usize;
        if day_of_week >= 7 {
            return None;
        }

        let week = content_row as usize;
        tracing::debug!("Week={}, day_of_week={}", week, day_of_week);

        let first_weekday = self.calendar_state.current_month_start.weekday();
        let first_day_offset = first_weekday.num_days_from_monday() as usize;

        let day_number = week * 7 + day_of_week;
        if day_number < first_day_offset {
            tracing::debug!(
                "Before month start: day_number={} < offset={}",
                day_number,
                first_day_offset
            );
            return None;
        }

        let day = (day_number - first_day_offset + 1) as u32;
        let days_in_month = CalendarRenderer::days_in_month(
            self.calendar_state.current_month_start.year(),
            self.calendar_state.current_month_start.month(),
        );

        tracing::debug!("Calculated day={} (max={})", day, days_in_month);

        if day > 0 && day <= days_in_month {
            Some(day)
        } else {
            None
        }
    }

    fn get_filtered_practitioners(&self) -> Vec<&Practitioner> {
        self.calendar_state
            .practitioners
            .iter()
            .filter(|p| {
                self.filter_state.active_practitioner_filters.is_empty()
                    || self
                        .filter_state
                        .active_practitioner_filters
                        .contains(&p.id)
            })
            .collect()
    }
}

#[async_trait]
impl Component for AppointmentCalendarComponent {
    async fn init(&mut self) -> Result<()> {
        match self.practitioner_service.get_active_practitioners().await {
            Ok(practitioners) => {
                self.calendar_state.practitioners = practitioners;
            }
            Err(_e) => {
                self.calendar_state.practitioners = vec![
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
                    Practitioner {
                        id: Uuid::new_v4(),
                        user_id: None,
                        first_name: "Emma".to_string(),
                        middle_name: None,
                        last_name: "Williams".to_string(),
                        title: "Dr".to_string(),
                        hpi_i: Some("8003610000000002".to_string()),
                        ahpra_registration: Some("MED0001234569".to_string()),
                        prescriber_number: Some("345678".to_string()),
                        provider_number: "345678C".to_string(),
                        speciality: Some("General Practice".to_string()),
                        qualifications: vec!["MBBS".to_string(), "FRACGP".to_string()],
                        phone: Some("02 9876 5434".to_string()),
                        email: Some("e.williams@clinic.com".to_string()),
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
        if self.is_any_modal_active() {
            return self.handle_modal_events(key);
        }

        self.handle_calendar_events(key)
    }

    fn handle_mouse_events(&mut self, mouse: MouseEvent) -> Action {
        use crate::ui::widgets::mouse_debug::log_mouse_event;

        log_mouse_event(&mouse, "AppointmentCalendar");

        if self.is_any_modal_active() {
            return Action::None;
        }

        if is_scroll_down(&mouse) {
            match self.calendar_state.focus_area {
                FocusArea::MonthView => self.next_day(),
                FocusArea::DayView => {
                    let current = self.calendar_state.time_slot_state.selected().unwrap_or(0);
                    let next = (current + 1).min(39);
                    self.calendar_state.time_slot_state.select(Some(next));
                }
            }
            return Action::Render;
        }

        if is_scroll_up(&mouse) {
            match self.calendar_state.focus_area {
                FocusArea::MonthView => self.previous_day(),
                FocusArea::DayView => {
                    let current = self.calendar_state.time_slot_state.selected().unwrap_or(0);
                    let prev = current.saturating_sub(1);
                    self.calendar_state.time_slot_state.select(Some(prev));
                }
            }
            return Action::Render;
        }

        if !is_click(&mouse) {
            return Action::None;
        }

        let col = mouse.column;
        let row = mouse.row;

        if let Some(month_area) = self.month_calendar_area {
            if col >= month_area.x
                && col < month_area.x + month_area.width
                && row >= month_area.y
                && row < month_area.y + month_area.height
            {
                if let Some(day) = self.calculate_day_from_click(col, row, month_area) {
                    self.calendar_state.selected_month_day = day;
                    self.calendar_state.focus_area = FocusArea::MonthView;
                    return Action::Render;
                }
            }
        }

        if let Some(schedule_area) = self.schedule_area {
            tracing::debug!(
                "Schedule area: x={}, y={}, w={}, h={}",
                schedule_area.x,
                schedule_area.y,
                schedule_area.width,
                schedule_area.height
            );

            if col >= schedule_area.x
                && col < schedule_area.x + schedule_area.width
                && row >= schedule_area.y
                && row < schedule_area.y + schedule_area.height
            {
                // Simple proportional hit test - no complex layout calculations
                // Layout: [3-row header][time slots grid with 1-char border]
                // Grid inside: x+1, y+4, w-2, h-4 (y+4 = 3 for header + 1 for top border)
                // Time column also has 1-char left border
                let grid_x = schedule_area.x + 2; // +2 = outer border + time column border
                let grid_y = schedule_area.y + 4;
                let grid_w = schedule_area.width.saturating_sub(2);
                let grid_h = schedule_area.height.saturating_sub(4);

                // Check if click is in the grid area (not in header or time column)
                if col >= grid_x && col < grid_x + grid_w && row >= grid_y && row < grid_y + grid_h
                {
                    let rel_col = col - grid_x;
                    let rel_row = row - grid_y;

                    let num_practitioners = self.get_filtered_practitioners().len();
                    if num_practitioners > 0 {
                        // Time column is 8 chars, rest is divided among practitioners
                        let time_col_width = 8u16;
                        let practitioner_area_width = grid_w.saturating_sub(time_col_width);
                        let practitioner_col_width =
                            practitioner_area_width / num_practitioners as u16;

                        // Check if click is past the time column
                        if rel_col > time_col_width {
                            let practitioner_idx =
                                ((rel_col - time_col_width) / practitioner_col_width) as usize;
                            let slot_idx = (rel_row / 2) as usize;

                            if practitioner_idx < num_practitioners && slot_idx < 40 {
                                tracing::debug!(
                                    "HIT: practitioner={}, slot={}",
                                    practitioner_idx,
                                    slot_idx
                                );
                                self.calendar_state.selected_practitioner_column = practitioner_idx;
                                self.calendar_state.time_slot_state.select(Some(slot_idx));
                                self.calendar_state.focus_area = FocusArea::DayView;
                                return Action::Render;
                            }
                        }
                    }
                }
                tracing::debug!("No hit at ({}, {})", col, row);
            }
        }

        Action::None
    }

    async fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::Render => {
                match self.calendar_state.view_mode {
                    ViewMode::Day => self.load_appointments_for_date().await?,
                    ViewMode::Week => self.load_appointments_for_week().await?,
                }

                // Load patient data when modal opens
                if self.detail_data.showing && self.detail_data.patient.is_none() {
                    if let Some(appt_id) = self.detail_data.appointment_id {
                        if let Some(appt) = self
                            .calendar_state
                            .appointments
                            .iter()
                            .find(|a| a.id == appt_id)
                        {
                            match self.patient_service.find_patient(appt.patient_id).await {
                                Ok(Some(patient)) => {
                                    self.detail_data.patient = Some(patient);
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
                if let Some(appt_id) = self.detail_data.appointment_id {
                    let user_id = Uuid::parse_str("a1b2c3d4-e5f6-4789-a1b2-c3d4e5f64789")
                        .expect("valid UUID");

                    match self
                        .appointment_service
                        .mark_arrived(appt_id, user_id)
                        .await
                    {
                        Ok(_) => {
                            tracing::info!("Appointment {} marked as arrived", appt_id);
                            self.detail_data.showing = false;
                            self.load_appointments_for_date().await?;
                        }
                        Err(e) => {
                            tracing::error!("Failed to mark appointment as arrived: {}", e);
                            self.error_data.message = e.to_string();
                            self.error_data.showing = true;
                            self.detail_data.showing = false;
                        }
                    }
                }
            }
            Action::AppointmentMarkInProgress => {
                if let Some(appt_id) = self.detail_data.appointment_id {
                    let user_id = Uuid::parse_str("a1b2c3d4-e5f6-4789-a1b2-c3d4e5f64789")
                        .expect("valid UUID");

                    match self
                        .appointment_service
                        .mark_in_progress(appt_id, user_id)
                        .await
                    {
                        Ok(_) => {
                            tracing::info!("Appointment {} marked as in progress", appt_id);
                            self.detail_data.showing = false;
                            self.load_appointments_for_date().await?;
                        }
                        Err(e) => {
                            tracing::error!("Failed to mark appointment as in progress: {}", e);
                            self.error_data.message = e.to_string();
                            self.error_data.showing = true;
                            self.detail_data.showing = false;
                        }
                    }
                }
            }
            Action::AppointmentMarkCompleted => {
                if let Some(appt_id) = self.detail_data.appointment_id {
                    let user_id = Uuid::parse_str("a1b2c3d4-e5f6-4789-a1b2-c3d4e5f64789")
                        .expect("valid UUID");

                    match self
                        .appointment_service
                        .mark_completed(appt_id, user_id)
                        .await
                    {
                        Ok(_) => {
                            tracing::info!("Appointment {} marked as completed", appt_id);
                            self.detail_data.showing = false;
                            self.load_appointments_for_date().await?;
                        }
                        Err(e) => {
                            tracing::error!("Failed to mark appointment as completed: {}", e);
                            self.error_data.message = e.to_string();
                            self.error_data.showing = true;
                            self.detail_data.showing = false;
                        }
                    }
                }
            }
            Action::AppointmentMarkNoShow => {
                if let Some(appt_id) = self.detail_data.appointment_id {
                    let user_id = Uuid::parse_str("a1b2c3d4-e5f6-4789-a1b2-c3d4e5f64789")
                        .expect("valid UUID");

                    match self
                        .appointment_service
                        .mark_no_show(appt_id, user_id)
                        .await
                    {
                        Ok(_) => {
                            tracing::info!("Appointment {} marked as no show", appt_id);
                            self.detail_data.showing = false;
                            self.load_appointments_for_date().await?;
                        }
                        Err(e) => {
                            tracing::error!("Failed to mark appointment as no show: {}", e);
                            self.error_data.message = e.to_string();
                            self.error_data.showing = true;
                            self.detail_data.showing = false;
                        }
                    }
                }
            }
            Action::AppointmentReschedule => {
                if let Some(appt_id) = self.detail_data.appointment_id {
                    if let Some(new_start_time) = self.reschedule_data.new_start_time {
                        let user_id = Uuid::parse_str("a1b2c3d4-e5f6-4789-a1b2-c3d4e5f64789")
                            .expect("valid UUID");

                        match self
                            .appointment_service
                            .reschedule_appointment(
                                appt_id,
                                new_start_time,
                                self.reschedule_data.new_duration,
                                user_id,
                            )
                            .await
                        {
                            Ok(_) => {
                                tracing::info!("Appointment {} rescheduled successfully", appt_id);
                                self.reschedule_data.showing = false;
                                self.reschedule_data.new_start_time = None;
                                self.reschedule_data.conflict_warning = None;
                                match self.calendar_state.view_mode {
                                    ViewMode::Day => self.load_appointments_for_date().await?,
                                    ViewMode::Week => self.load_appointments_for_week().await?,
                                }
                            }
                            Err(e) => {
                                tracing::error!("Failed to reschedule appointment: {}", e);
                                self.reschedule_data.conflict_warning = Some(e.to_string());
                            }
                        }
                    }
                }
            }
            Action::AppointmentBatchMarkArrived => {
                let user_id =
                    Uuid::parse_str("a1b2c3d4-e5f6-4789-a1b2-c3d4e5f64789").expect("valid UUID");

                let appointment_ids: Vec<Uuid> = self
                    .history_state
                    .selected_appointments
                    .iter()
                    .copied()
                    .collect();
                let total = appointment_ids.len();

                self.batch_data.operation_in_progress = true;
                self.batch_data.progress_total = total;
                self.batch_data.progress_current = 0;

                let mut success_count = 0;
                let mut error_count = 0;

                tracing::info!("Starting batch mark arrived for {} appointments", total);

                for (idx, appt_id) in appointment_ids.iter().enumerate() {
                    self.batch_data.progress_current = idx + 1;
                    self.batch_data.progress_message =
                        format!("Marking appointment {} as arrived...", idx + 1);

                    match self
                        .appointment_service
                        .mark_arrived(*appt_id, user_id)
                        .await
                    {
                        Ok(_) => {
                            success_count += 1;
                            tracing::info!("Batch: Appointment {} marked as arrived", appt_id);
                        }
                        Err(e) => {
                            error_count += 1;
                            tracing::error!(
                                "Batch: Failed to mark appointment {} as arrived: {}",
                                appt_id,
                                e
                            );
                        }
                    }
                }

                self.batch_data.operation_in_progress = false;
                self.history_state.multi_select_mode = false;
                self.history_state.selected_appointments.clear();

                tracing::info!(
                    "Batch mark arrived completed: {} succeeded, {} failed",
                    success_count,
                    error_count
                );

                if error_count > 0 {
                    self.error_data.message = format!(
                        "Batch operation completed with errors:\n{} succeeded, {} failed\n\nCheck logs for details.",
                        success_count, error_count
                    );
                    self.error_data.showing = true;
                }

                match self.calendar_state.view_mode {
                    ViewMode::Day => self.load_appointments_for_date().await?,
                    ViewMode::Week => self.load_appointments_for_week().await?,
                }
            }
            Action::AppointmentBatchMarkCompleted => {
                let user_id =
                    Uuid::parse_str("a1b2c3d4-e5f6-4789-a1b2-c3d4e5f64789").expect("valid UUID");

                let appointment_ids: Vec<Uuid> = self
                    .history_state
                    .selected_appointments
                    .iter()
                    .copied()
                    .collect();
                let total = appointment_ids.len();

                self.batch_data.operation_in_progress = true;
                self.batch_data.progress_total = total;
                self.batch_data.progress_current = 0;

                let mut success_count = 0;
                let mut error_count = 0;

                tracing::info!("Starting batch mark completed for {} appointments", total);

                for (idx, appt_id) in appointment_ids.iter().enumerate() {
                    self.batch_data.progress_current = idx + 1;
                    self.batch_data.progress_message =
                        format!("Marking appointment {} as completed...", idx + 1);

                    match self
                        .appointment_service
                        .mark_completed(*appt_id, user_id)
                        .await
                    {
                        Ok(_) => {
                            success_count += 1;
                            tracing::info!("Batch: Appointment {} marked as completed", appt_id);
                        }
                        Err(e) => {
                            error_count += 1;
                            tracing::error!(
                                "Batch: Failed to mark appointment {} as completed: {}",
                                appt_id,
                                e
                            );
                        }
                    }
                }

                self.batch_data.operation_in_progress = false;
                self.history_state.multi_select_mode = false;
                self.history_state.selected_appointments.clear();

                tracing::info!(
                    "Batch mark completed: {} succeeded, {} failed",
                    success_count,
                    error_count
                );

                if error_count > 0 {
                    self.error_data.message = format!(
                        "Batch operation completed with errors:\n{} succeeded, {} failed\n\nCheck logs for details.",
                        success_count, error_count
                    );
                    self.error_data.showing = true;
                }

                match self.calendar_state.view_mode {
                    ViewMode::Day => self.load_appointments_for_date().await?,
                    ViewMode::Week => self.load_appointments_for_week().await?,
                }
            }
            _ => {}
        }
        Ok(None)
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(30), Constraint::Min(50)])
            .split(area);

        self.month_calendar_area = Some(chunks[0]);
        self.schedule_area = Some(chunks[1]);

        CalendarRenderer::render_month_calendar(&self.calendar_state, frame, chunks[0]);
        match self.calendar_state.view_mode {
            ViewMode::Day => CalendarRenderer::render_day_schedule(
                &mut self.calendar_state,
                &self.filter_state,
                &self.history_state,
                frame,
                chunks[1],
            ),
            ViewMode::Week => {
                CalendarRenderer::render_week_schedule(&mut self.calendar_state, frame, chunks[1])
            }
        }

        if self.detail_data.showing {
            ModalRenderer::render_appointment_detail_modal(
                &self.detail_data,
                &self.calendar_state,
                &self.calendar_state.practitioners,
                frame,
                area,
            );
        }

        if self.audit_data.showing {
            ModalRenderer::render_audit_history_modal(
                &self.audit_data,
                &self.calendar_state.practitioners,
                frame,
                area,
            );
        }

        if self.reschedule_data.showing {
            ModalRenderer::render_reschedule_modal(
                &self.reschedule_data,
                &self.detail_data,
                &self.calendar_state,
                frame,
                area,
            );
        }

        if self.search_data.showing {
            ModalRenderer::render_search_modal(&self.search_data, frame, area);
        }

        if self.filter_state.showing_filter_menu {
            ModalRenderer::render_filter_menu(&self.filter_state, frame, area);
        }

        if self.filter_state.showing_practitioner_menu {
            ModalRenderer::render_practitioner_menu(
                &self.filter_state,
                &self.calendar_state,
                frame,
                area,
            );
        }

        if self.error_data.showing {
            ModalRenderer::render_error_modal(&self.error_data, frame, area);
        }

        if self.confirmation_data.showing {
            ModalRenderer::render_confirmation_overlay(&self.confirmation_data, frame, area);
        }

        if self.batch_data.showing_menu {
            ModalRenderer::render_batch_menu(&self.history_state, frame, area);
        }

        if self.batch_data.operation_in_progress {
            ModalRenderer::render_batch_progress(&self.batch_data, frame, area);
        }

        if self.modal_state.is_showing(ModalType::Help) {
            let context = if self.calendar_state.focus_area == FocusArea::MonthView {
                KeybindContext::CalendarMonthView
            } else if self.calendar_state.view_mode == ViewMode::Week {
                KeybindContext::CalendarWeekView
            } else if self.history_state.multi_select_mode {
                KeybindContext::CalendarMultiSelect
            } else {
                KeybindContext::CalendarDayView
            };
            let help_modal = HelpModal::new(context);
            help_modal.render(frame, area);
        }
    }
}

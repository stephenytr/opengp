//! Renderer structs for the AppointmentCalendarComponent
//!
//! This module contains stateless renderer structs that handle all rendering logic
//! for the calendar component. Each renderer struct groups related render methods
//! as static functions that take all required state as parameters.
//!
//! ## Renderer Structure
//!
//! - [`CalendarRenderer`] - Month/day/week view rendering
//! - [`ModalRenderer`] - All modal rendering

use chrono::{Datelike, Timelike, Weekday};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table};
use ratatui::Frame;
use std::collections::HashSet;

use crate::domain::appointment::{AppointmentStatus, CalendarAppointment};
use crate::domain::audit::AuditAction;
use crate::domain::user::Practitioner;
use crate::ui::widgets::MonthCalendar;
use crate::ui::Theme;

use super::state::{
    AuditModalData, BatchModalData, CalendarState, ConfirmationModalData, DetailModalData,
    ErrorModalData, FilterState, FocusArea, HistoryState, RescheduleFocus, RescheduleModalData,
    SearchModalData,
};

/// Renderer for calendar views (month, day, week)
///
/// This struct contains static methods for rendering the main calendar views.
/// All methods are stateless and take required state as parameters.
pub struct CalendarRenderer;

impl CalendarRenderer {
    /// Generate time slots for the day view (8:00 AM to 5:45 PM in 15-minute intervals)
    pub fn generate_time_slots() -> Vec<String> {
        let mut slots = Vec::new();
        for hour in 8..18 {
            for minute in [0, 15, 30, 45] {
                slots.push(format!("{:02}:{:02}", hour, minute));
            }
        }
        slots
    }

    /// Get the name of a month based on its number
    pub fn get_month_name(month: u32) -> &'static str {
        match month {
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

    /// Get the number of days in a given month
    pub fn days_in_month(year: i32, month: u32) -> u32 {
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

    /// Detect all overlapping appointments at a given slot for a practitioner
    pub fn detect_overlaps<'a>(
        appointments: &'a [CalendarAppointment],
        practitioner_id: uuid::Uuid,
        slot_index: usize,
        active_status_filters: &HashSet<AppointmentStatus>,
        date: chrono::NaiveDate,
    ) -> Vec<&'a CalendarAppointment> {
        let time_slots = Self::generate_time_slots();
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

        let slot_datetime = date
            .and_hms_opt(hour, minute, 0)
            .expect("valid time")
            .and_utc();

        appointments
            .iter()
            .filter(|appt| {
                appt.practitioner_id == practitioner_id
                    && appt.start_time <= slot_datetime
                    && appt.end_time > slot_datetime
                    && (active_status_filters.is_empty()
                        || active_status_filters.contains(&appt.status))
            })
            .collect()
    }

    /// Find an appointment for a given time slot
    pub fn find_appointment_for_slot<'a>(
        appointments: &'a [CalendarAppointment],
        practitioner_id: uuid::Uuid,
        slot_index: usize,
        active_status_filters: &HashSet<AppointmentStatus>,
        date: chrono::NaiveDate,
    ) -> Option<&'a CalendarAppointment> {
        let time_slots = Self::generate_time_slots();
        if slot_index >= time_slots.len() {
            return None;
        }

        let slot_time_str = &time_slots[slot_index];
        let (hour, minute) = slot_time_str.split_once(':').and_then(|(h, m)| {
            let hour = h.parse::<u32>().ok()?;
            let minute = m.parse::<u32>().ok()?;
            Some((hour, minute))
        })?;

        let slot_datetime = date
            .and_hms_opt(hour, minute, 0)
            .expect("valid time")
            .and_utc();

        appointments.iter().find(|appt| {
            appt.practitioner_id == practitioner_id
                && appt.start_time <= slot_datetime
                && appt.end_time > slot_datetime
                && (active_status_filters.is_empty()
                    || active_status_filters.contains(&appt.status))
        })
    }

    /// Render the month calendar view (left panel)
    pub fn render_month_calendar(calendar_state: &CalendarState, frame: &mut Frame, area: Rect) {
        let selected_date = calendar_state
            .current_month_start
            .with_day(calendar_state.selected_month_day)
            .unwrap_or(calendar_state.current_month_start);

        let calendar = MonthCalendar::new(selected_date);

        let is_focused = calendar_state.focus_area == FocusArea::MonthView;
        calendar.render(frame, area, is_focused);
    }

    /// Render the day schedule view
    pub fn render_day_schedule(
        calendar_state: &mut CalendarState,
        filter_state: &FilterState,
        history_state: &HistoryState,
        frame: &mut Frame,
        area: Rect,
    ) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(area);

        Self::render_practitioner_header(calendar_state, filter_state, frame, chunks[0]);
        Self::render_time_slots_grid(
            calendar_state,
            filter_state,
            history_state,
            frame,
            chunks[1],
        );
    }

    /// Render the week schedule view
    pub fn render_week_schedule(calendar_state: &mut CalendarState, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(area);

        Self::render_week_header(calendar_state, frame, chunks[0]);
        Self::render_week_time_slots_grid(calendar_state, frame, chunks[1]);
    }

    /// Render the week header (day names and dates)
    pub fn render_week_header(calendar_state: &CalendarState, frame: &mut Frame, area: Rect) {
        let dates: Vec<chrono::NaiveDate> = (0..7)
            .map(|i| calendar_state.week_start_date + chrono::Duration::days(i))
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

            let style = if *date == calendar_state.current_date {
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

        let week_start_str = calendar_state.week_start_date.format("%b %d").to_string();
        let week_end_str = (calendar_state.week_start_date + chrono::Duration::days(6))
            .format("%b %d")
            .to_string();
        let title = format!(" Week: {}-{} ", week_start_str, week_end_str);

        let table = Table::new(vec![header], widths).block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(if calendar_state.focus_area == FocusArea::DayView {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default().fg(Color::White)
                }),
        );

        frame.render_widget(table, area);
    }

    /// Render the week time slots grid (7-day view)
    pub fn render_week_time_slots_grid(
        calendar_state: &mut CalendarState,
        frame: &mut Frame,
        area: Rect,
    ) {
        let time_slots = Self::generate_time_slots();
        let dates: Vec<chrono::NaiveDate> = (0..7)
            .map(|i| calendar_state.week_start_date + chrono::Duration::days(i))
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

                let appts_at_slot: Vec<&CalendarAppointment> = calendar_state
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

        let table = Table::new(rows, widths)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Week View ".to_string()),
            )
            .row_highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");

        frame.render_stateful_widget(table, area, &mut calendar_state.time_slot_state);
    }

    /// Render the practitioner header row
    pub fn render_practitioner_header(
        calendar_state: &CalendarState,
        filter_state: &FilterState,
        frame: &mut Frame,
        area: Rect,
    ) {
        let visible_practitioners: Vec<_> = calendar_state
            .practitioners
            .iter()
            .filter(|p| {
                filter_state.active_practitioner_filters.is_empty()
                    || filter_state.active_practitioner_filters.contains(&p.id)
            })
            .collect();

        let mut header_cells = vec![Cell::from("Time").style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )];

        for (idx, practitioner) in visible_practitioners.iter().enumerate() {
            let name = format!("Dr. {}", practitioner.last_name);
            let is_selected = idx == calendar_state.selected_practitioner_column;
            let style = if is_selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            };
            header_cells.push(Cell::from(name).style(style));
        }

        let header = Row::new(header_cells)
            .style(Style::default().bg(Color::DarkGray))
            .height(1);

        let mut widths = vec![Constraint::Length(8)];
        for _ in &visible_practitioners {
            widths.push(Constraint::Min(15));
        }

        let mut title = " Schedule ".to_string();
        if !filter_state.active_status_filters.is_empty() {
            title = format!(
                " Schedule [Status: {}] ",
                filter_state.active_status_filters.len()
            );
        }
        if !filter_state.active_practitioner_filters.is_empty() {
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
                .border_style(if calendar_state.focus_area == FocusArea::DayView {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default().fg(Color::White)
                }),
        );

        frame.render_widget(table, area);
    }

    /// Render the time slots grid for day view
    pub fn render_time_slots_grid(
        calendar_state: &mut CalendarState,
        filter_state: &FilterState,
        history_state: &HistoryState,
        frame: &mut Frame,
        area: Rect,
    ) {
        let visible_practitioners: Vec<_> = calendar_state
            .practitioners
            .iter()
            .filter(|p| {
                filter_state.active_practitioner_filters.is_empty()
                    || filter_state.active_practitioner_filters.contains(&p.id)
            })
            .collect();

        let time_slots = Self::generate_time_slots();
        let mut rows = Vec::new();

        // Get the selected date from calendar state
        let selected_date = calendar_state
            .current_month_start
            .with_day(calendar_state.selected_month_day)
            .unwrap_or(calendar_state.current_date);

        for (slot_index, time_slot) in time_slots.iter().enumerate() {
            let mut cells = vec![Cell::from(time_slot.as_str())];

            for (practitioner_index, practitioner) in visible_practitioners.iter().enumerate() {
                let overlaps = Self::detect_overlaps(
                    &calendar_state.appointments,
                    practitioner.id,
                    slot_index,
                    &filter_state.active_status_filters,
                    selected_date,
                );

                if overlaps.len() > 1 {
                    let overlap_ids: Vec<String> =
                        overlaps.iter().map(|a| a.id.to_string()).collect();
                    tracing::warn!(
                        "Double-booking detected: {} appointments at slot {} for practitioner {} (IDs: {})",
                        overlaps.len(),
                        slot_index,
                        practitioner.id,
                        overlap_ids.join(", ")
                    );
                }

                if let Some(appt) = Self::find_appointment_for_slot(
                    &calendar_state.appointments,
                    practitioner.id,
                    slot_index,
                    &filter_state.active_status_filters,
                    selected_date,
                ) {
                    let slot_time_str = time_slot;
                    let (hour, minute) = slot_time_str
                        .split_once(':')
                        .and_then(|(h, m)| {
                            let hour = h.parse::<u32>().ok()?;
                            let minute = m.parse::<u32>().ok()?;
                            Some((hour, minute))
                        })
                        .unwrap_or((0, 0));

                    let slot_datetime = selected_date
                        .and_hms_opt(hour, minute, 0)
                        .expect("valid time")
                        .and_utc();

                    let is_first_slot = slot_datetime == appt.start_time;

                    let mut style = match appt.status {
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
                        AppointmentStatus::Rescheduled => Style::default().fg(Color::Magenta),
                        AppointmentStatus::Cancelled => Style::default().fg(Color::Gray),
                    };

                    if history_state.multi_select_mode
                        && history_state.selected_appointments.contains(&appt.id)
                    {
                        style = Style::default().fg(Color::Black).bg(Color::Yellow);
                    } else if let Some(selected_row) = calendar_state.time_slot_state.selected() {
                        if selected_row == slot_index
                            && practitioner_index == calendar_state.selected_practitioner_column
                        {
                            style = style.add_modifier(Modifier::REVERSED);
                        }
                    }

                    if overlaps.len() > 1 {
                        style = style.fg(Color::Red).add_modifier(Modifier::BOLD);
                    }

                    if is_first_slot {
                        let patient_name = &appt.patient_name;

                        let mut appt_text = if history_state.multi_select_mode
                            && history_state.selected_appointments.contains(&appt.id)
                        {
                            format!("☑ {}\n{}", patient_name, appt.appointment_type)
                        } else if history_state.multi_select_mode {
                            format!("☐ {}\n{}", patient_name, appt.appointment_type)
                        } else {
                            format!("{}\n{}", patient_name, appt.appointment_type)
                        };

                        if appt.is_urgent {
                            appt_text = format!("⚠ {}", appt_text);
                        }

                        if overlaps.len() > 1 {
                            appt_text = format!("⚠ {} conflicts\n{}", overlaps.len(), appt_text);
                        }

                        cells.push(Cell::from(appt_text).style(style));
                    } else {
                        cells.push(Cell::from("│").style(style));
                    }
                } else {
                    let is_selected = calendar_state.time_slot_state.selected() == Some(slot_index)
                        && practitioner_index == calendar_state.selected_practitioner_column;
                    let style = if is_selected {
                        Style::default().add_modifier(Modifier::REVERSED)
                    } else {
                        Style::default()
                    };
                    cells.push(Cell::from("").style(style));
                }
            }

            let row = Row::new(cells).height(2);
            rows.push(row);
        }

        let mut widths = vec![Constraint::Length(8)];
        for _ in &visible_practitioners {
            widths.push(Constraint::Min(15));
        }

        let title = if history_state.multi_select_mode {
            let count = history_state.selected_appointments.len();
            format!(" Multi-Select: {} selected ", count)
        } else {
            " Day View ".to_string()
        };

        let table = Table::new(rows, widths)
            .block(Block::default().borders(Borders::ALL).title(title))
            .row_highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");

        frame.render_stateful_widget(table, area, &mut calendar_state.time_slot_state);
    }
}

/// Renderer for all modal dialogs
///
/// This struct contains static methods for rendering modal dialogs
/// and overlays. All methods are stateless and take required state as parameters.
pub struct ModalRenderer;

impl ModalRenderer {
    fn render_modal_background(
        frame: &mut Frame,
        area: Rect,
        width_percent: u16,
        height_percent: u16,
    ) -> Rect {
        let theme = Theme::default();
        let vertical_margin = (100 - height_percent) / 2;
        let horizontal_margin = (100 - width_percent) / 2;

        let vertical = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(vertical_margin),
                Constraint::Percentage(height_percent),
                Constraint::Percentage(100 - height_percent - vertical_margin),
            ])
            .split(area);

        let horizontal = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(horizontal_margin),
                Constraint::Percentage(width_percent),
                Constraint::Percentage(100 - width_percent - horizontal_margin),
            ])
            .split(vertical[1]);

        let modal_area = horizontal[1];
        frame.render_widget(Clear, modal_area);

        let modal_block = Block::default()
            .borders(Borders::ALL)
            .style(theme.modal_background)
            .border_style(theme.normal);
        let inner_area = modal_block.inner(modal_area);
        frame.render_widget(modal_block, modal_area);

        inner_area
    }

    /// Get user display name from practitioners list
    pub fn get_user_display_name(practitioners: &[Practitioner], user_id: uuid::Uuid) -> String {
        if let Some(practitioner) = practitioners
            .iter()
            .find(|p| p.user_id.is_some_and(|uid| uid == user_id))
        {
            format!("{} {}", practitioner.title, practitioner.last_name)
        } else {
            format!("User {}...", &user_id.to_string()[..8])
        }
    }

    /// Render appointment detail modal
    pub fn render_appointment_detail_modal(
        detail_data: &DetailModalData,
        calendar_state: &CalendarState,
        practitioners: &[Practitioner],
        frame: &mut Frame,
        area: Rect,
    ) {
        let inner_area = Self::render_modal_background(frame, area, 60, 66);

        let mut lines = Vec::new();

        if let Some(appt_id) = detail_data.appointment_id {
            if let Some(appt) = calendar_state.appointments.iter().find(|a| a.id == appt_id) {
                let patient_name = if let Some(ref patient) = detail_data.patient {
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
                let appt_date = appt.start_time.date_naive();
                let overlaps = CalendarRenderer::detect_overlaps(
                    &calendar_state.appointments,
                    appt.practitioner_id,
                    slot_index,
                    &HashSet::new(),
                    appt_date,
                );

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

                if let Some(practitioner) =
                    practitioners.iter().find(|p| p.id == appt.practitioner_id)
                {
                    lines.push(Line::from(vec![
                        Span::styled("Practitioner: ", Style::default().fg(Color::Yellow)),
                        Span::styled(
                            format!("Dr. {}", practitioner.last_name),
                            Style::default().fg(Color::White),
                        ),
                    ]));
                }

                if let Some(ref reason) = appt.reason {
                    if !reason.is_empty() {
                        lines.push(Line::from(""));
                        lines.push(Line::from(vec![
                            Span::styled("Reason: ", Style::default().fg(Color::Yellow)),
                            Span::styled(reason, Style::default().fg(Color::White)),
                        ]));
                    }
                }

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

        frame.render_widget(modal_content, inner_area);
    }

    /// Render audit history modal
    pub fn render_audit_history_modal(
        audit_data: &AuditModalData,
        practitioners: &[Practitioner],
        frame: &mut Frame,
        area: Rect,
    ) {
        let inner_area = Self::render_modal_background(frame, area, 60, 66);

        let mut lines = Vec::new();

        if audit_data.entries.is_empty() {
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
            lines.push(Line::from(""));

            for (index, entry) in audit_data.entries.iter().enumerate() {
                let timestamp = entry.changed_at.format("%Y-%m-%d %H:%M:%S").to_string();

                let user_display = Self::get_user_display_name(practitioners, entry.changed_by);

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

                let is_selected = index == audit_data.selected_index;
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

        frame.render_widget(content, inner_area);
    }

    /// Render search modal
    pub fn render_search_modal(search_data: &SearchModalData, frame: &mut Frame, area: Rect) {
        let inner_area = Self::render_modal_background(frame, area, 60, 66);

        let mut lines = vec![
            Line::from(vec![Span::styled(
                "Search Appointments",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Query: ", Style::default().fg(Color::Cyan)),
                Span::styled(
                    &search_data.query,
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("█", Style::default().fg(Color::White)),
            ]),
            Line::from(""),
        ];

        if search_data.query.is_empty() {
            lines.push(Line::from(vec![Span::styled(
                "Start typing to search...",
                Style::default().fg(Color::DarkGray),
            )]));
        } else if search_data.results.is_empty() {
            lines.push(Line::from(vec![Span::styled(
                "No appointments found",
                Style::default().fg(Color::Red),
            )]));
        } else {
            lines.push(Line::from(vec![Span::styled(
                format!(
                    "Found {} result(s) (showing up to 50)",
                    search_data.results.len()
                ),
                Style::default().fg(Color::Green),
            )]));
            lines.push(Line::from(""));

            for (idx, appt) in search_data.results.iter().enumerate() {
                let is_selected = idx == search_data.selected_index;
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

        frame.render_widget(modal_content, inner_area);
    }

    /// Render reschedule modal
    pub fn render_reschedule_modal(
        reschedule_data: &RescheduleModalData,
        detail_data: &DetailModalData,
        calendar_state: &CalendarState,
        frame: &mut Frame,
        area: Rect,
    ) {
        let inner_area = Self::render_modal_background(frame, area, 80, 60);

        if let Some(appt_id) = detail_data.appointment_id {
            if let Some(appt) = calendar_state.appointments.iter().find(|a| a.id == appt_id) {
                let patient_name = if let Some(ref patient) = detail_data.patient {
                    format!("{} {}", patient.first_name, patient.last_name)
                } else {
                    "Loading...".to_string()
                };

                let main_layout = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(3),
                        Constraint::Length(3),
                        Constraint::Length(3),
                        Constraint::Length(3),
                        Constraint::Min(20),
                        Constraint::Length(3),
                        Constraint::Length(3),
                    ])
                    .split(inner_area);

                let title = Paragraph::new(vec![Line::from(vec![Span::styled(
                    "Reschedule Appointment",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )])]);
                frame.render_widget(title, main_layout[0]);

                let patient_info = Paragraph::new(vec![Line::from(vec![
                    Span::styled("Patient: ", Style::default().fg(Color::Yellow)),
                    Span::styled(patient_name, Style::default().fg(Color::White)),
                ])]);
                frame.render_widget(patient_info, main_layout[1]);

                let current_time_str = format!("{}", appt.start_time.format("%Y-%m-%d %H:%M"));
                let current_info = Paragraph::new(vec![Line::from(vec![
                    Span::styled("Current: ", Style::default().fg(Color::Yellow)),
                    Span::styled(
                        format!("{} ({} min)", current_time_str, appt.duration_minutes()),
                        Style::default().fg(Color::White),
                    ),
                ])]);
                frame.render_widget(current_info, main_layout[2]);

                let new_time_str = reschedule_data
                    .new_start_time
                    .map(|t| format!("{}", t.format("%Y-%m-%d %H:%M")))
                    .unwrap_or_else(|| "Select date and time".to_string());
                let new_info = Paragraph::new(vec![Line::from(vec![
                    Span::styled("New: ", Style::default().fg(Color::Yellow)),
                    Span::styled(
                        format!("{} ({} min)", new_time_str, reschedule_data.new_duration),
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                ])]);
                frame.render_widget(new_info, main_layout[3]);

                let widgets_layout = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .split(main_layout[4]);

                reschedule_data.calendar.render(
                    frame,
                    widgets_layout[0],
                    reschedule_data.focus == RescheduleFocus::Date,
                );
                reschedule_data.time_picker.render(frame, widgets_layout[1]);

                if let Some(ref warning) = reschedule_data.conflict_warning {
                    let warning_text = Paragraph::new(vec![Line::from(vec![
                        Span::styled(
                            "⚠ ",
                            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(warning, Style::default().fg(Color::Red)),
                    ])]);
                    frame.render_widget(warning_text, main_layout[5]);
                }

                let help_text = Paragraph::new(vec![Line::from(vec![
                    Span::styled(
                        "Tab",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(": Switch  ", Style::default().fg(Color::White)),
                    Span::styled(
                        "↑↓",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(": Navigate  ", Style::default().fg(Color::White)),
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
                ])]);
                frame.render_widget(help_text, main_layout[6]);
            }
        }
    }

    /// Render filter menu modal
    pub fn render_filter_menu(filter_state: &FilterState, frame: &mut Frame, area: Rect) {
        let inner_area = Self::render_modal_background(frame, area, 60, 66);

        let mut lines = Vec::new();

        lines.push(Line::from(vec![Span::styled(
            "Filter by Status",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]));
        lines.push(Line::from(""));

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
            let is_active = filter_state.active_status_filters.contains(&status);
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

        let filter_count = filter_state.active_status_filters.len();
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

        frame.render_widget(modal_content, inner_area);
    }

    /// Render practitioner filter menu
    pub fn render_practitioner_menu(
        filter_state: &FilterState,
        calendar_state: &CalendarState,
        frame: &mut Frame,
        area: Rect,
    ) {
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

        for (idx, practitioner) in calendar_state.practitioners.iter().enumerate() {
            let key = (idx + 1).to_string();
            let is_active = filter_state
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

        let filter_count = filter_state.active_practitioner_filters.len();
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

    /// Render confirmation overlay
    pub fn render_confirmation_overlay(
        confirmation_data: &ConfirmationModalData,
        frame: &mut Frame,
        area: Rect,
    ) {
        let modal_area = Rect {
            x: area.width / 4,
            y: area.height / 3,
            width: area.width / 2,
            height: area.height / 3,
        };

        let message = confirmation_data.message.clone();

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

    /// Render error modal
    pub fn render_error_modal(error_data: &ErrorModalData, frame: &mut Frame, area: Rect) {
        let inner_area = Self::render_modal_background(frame, area, 50, 33);

        let lines = vec![
            Line::from(vec![Span::styled(
                "⚠ Error",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(vec![Span::styled(
                &error_data.message,
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

        frame.render_widget(modal_content, inner_area);
    }

    /// Render batch menu modal
    pub fn render_batch_menu(history_state: &HistoryState, frame: &mut Frame, area: Rect) {
        let modal_area = Rect {
            x: area.width / 4,
            y: area.height / 3,
            width: area.width / 2,
            height: area.height / 3,
        };

        let selection_count = history_state.selected_appointments.len();

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

    /// Render batch progress modal
    pub fn render_batch_progress(batch_data: &BatchModalData, frame: &mut Frame, area: Rect) {
        let modal_area = Rect {
            x: area.width / 4,
            y: area.height / 3,
            width: area.width / 2,
            height: area.height / 4,
        };

        let progress_percent = if batch_data.progress_total > 0 {
            (batch_data.progress_current as f64 / batch_data.progress_total as f64 * 100.0) as usize
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
                &batch_data.progress_message,
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
                    batch_data.progress_current, batch_data.progress_total
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
}

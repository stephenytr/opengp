use async_trait::async_trait;
use chrono::{Duration, NaiveDate, NaiveTime, Timelike, Utc};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap};
use ratatui::Frame;
use std::collections::HashMap;
use std::sync::Arc;
use sublime_fuzzy::best_match;
use uuid::Uuid;

use crate::components::{Action, Component};
use crate::domain::appointment::{AppointmentService, AppointmentType, NewAppointmentData};
use crate::domain::patient::{Patient, PatientService};
use crate::domain::user::{Practitioner, PractitionerService};
use crate::error::Result;
use crate::ui::components::{InputWrapper, SelectWrapper};
use crate::ui::Theme;
use crate::ui::widgets::{MonthCalendar, TimeSlotPicker};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum FormField {
    Patient,
    Practitioner,
    Date,
    Time,
    Type,
    Reason,
}

impl FormField {
    fn next(&self) -> Self {
        match self {
            Self::Patient => Self::Practitioner,
            Self::Practitioner => Self::Date,
            Self::Date => Self::Time,
            Self::Time => Self::Type,
            Self::Type => Self::Reason,
            Self::Reason => Self::Patient,
        }
    }

    fn previous(&self) -> Self {
        match self {
            Self::Patient => Self::Reason,
            Self::Practitioner => Self::Patient,
            Self::Date => Self::Practitioner,
            Self::Time => Self::Date,
            Self::Type => Self::Time,
            Self::Reason => Self::Type,
        }
    }
}

pub struct AppointmentFormComponent {
    appointment_service: Arc<AppointmentService>,
    patient_service: Arc<PatientService>,
    practitioner_service: Arc<PractitionerService>,

    // Form state
    current_field: FormField,

    // Patient search
    patient_query: String,
    all_patients: Vec<Patient>,
    filtered_patients: Vec<Patient>,
    patient_list_state: ListState,
    selected_patient_id: Option<Uuid>,
    patient_search_active: bool,

    // Data for service calls
    practitioners: Vec<Practitioner>,
    appointment_types: Vec<AppointmentType>,

    // Form inputs - using tui-realm wrappers
    date_input: InputWrapper,
    time_input: InputWrapper,
    reason_input: InputWrapper,

    // Dropdown selections - using SelectWrapper
    practitioner_select: SelectWrapper,
    type_select: SelectWrapper,

    // Date picker widget
    date_calendar: MonthCalendar,

    // Time slot picker widget
    time_slot_picker: TimeSlotPicker,

    // Validation
    validation_errors: HashMap<FormField, String>,

    // Status
    error_message: Option<String>,
    is_submitting: bool,
}

impl AppointmentFormComponent {
    pub fn new(
        appointment_service: Arc<AppointmentService>,
        patient_service: Arc<PatientService>,
        practitioner_service: Arc<PractitionerService>,
    ) -> Self {
        let mut patient_list_state = ListState::default();
        patient_list_state.select(Some(0));

        let appointment_type_items: Vec<String> = vec![
            "Standard (15 min)".to_string(),
            "Long (30 min)".to_string(),
            "Brief (10 min)".to_string(),
            "New Patient (30 min)".to_string(),
            "Health Assessment (45 min)".to_string(),
            "Chronic Disease Review (30 min)".to_string(),
            "Mental Health Plan (60 min)".to_string(),
            "Immunisation (15 min)".to_string(),
            "Procedure (30 min)".to_string(),
            "Telephone (10 min)".to_string(),
            "Telehealth (15 min)".to_string(),
            "Home Visit (30 min)".to_string(),
            "Emergency (15 min)".to_string(),
        ];

        let appointment_types = vec![
            AppointmentType::Standard,
            AppointmentType::Long,
            AppointmentType::Brief,
            AppointmentType::NewPatient,
            AppointmentType::HealthAssessment,
            AppointmentType::ChronicDiseaseReview,
            AppointmentType::MentalHealthPlan,
            AppointmentType::Immunisation,
            AppointmentType::Procedure,
            AppointmentType::Telephone,
            AppointmentType::Telehealth,
            AppointmentType::HomeVisit,
            AppointmentType::Emergency,
        ];

        let now = Utc::now();
        let date_value = now.format("%Y-%m-%d").to_string();
        let time_value = "09:00".to_string();
        let initial_date = now.date_naive();

        Self {
            appointment_service,
            patient_service,
            practitioner_service,
            current_field: FormField::Patient,
            patient_query: String::new(),
            all_patients: Vec::new(),
            filtered_patients: Vec::new(),
            patient_list_state,
            selected_patient_id: None,
            patient_search_active: true,
            practitioners: Vec::new(),
            appointment_types,
            date_input: InputWrapper::new().init_value(&date_value),
            time_input: InputWrapper::new().init_value(&time_value),
            reason_input: InputWrapper::new(),
            practitioner_select: SelectWrapper::new(),
            type_select: SelectWrapper::new().items(appointment_type_items),
            date_calendar: MonthCalendar::new(initial_date),
            time_slot_picker: TimeSlotPicker::new(),
            validation_errors: HashMap::new(),
            error_message: None,
            is_submitting: false,
        }
    }

    fn apply_patient_filter(&mut self) {
        if self.patient_query.is_empty() {
            self.filtered_patients = self.all_patients.clone();
        } else {
            let query = &self.patient_query;

            let mut scored_patients: Vec<(Patient, isize)> = self
                .all_patients
                .iter()
                .filter_map(|p| {
                    let full_name = format!("{} {}", p.first_name, p.last_name);
                    let preferred = p.preferred_name.as_deref().unwrap_or("");
                    let medicare = p.medicare_number.as_deref().unwrap_or("");

                    let name_match = best_match(query, &full_name);
                    let preferred_match = best_match(query, preferred);
                    let medicare_match = best_match(query, medicare);

                    let best_score = [
                        name_match.map(|m| m.score()),
                        preferred_match.map(|m| m.score()),
                        medicare_match.map(|m| m.score()),
                    ]
                    .iter()
                    .filter_map(|&s| s)
                    .max()
                    .unwrap_or(0);

                    if best_score > 0 {
                        Some((p.clone(), best_score))
                    } else {
                        None
                    }
                })
                .collect();

            scored_patients.sort_by(|a, b| b.1.cmp(&a.1));

            self.filtered_patients = scored_patients.into_iter().map(|(p, _)| p).collect();
        }

        if !self.filtered_patients.is_empty() {
            self.patient_list_state.select(Some(0));
        } else {
            self.patient_list_state.select(None);
        }
    }

    fn validate_date(&self) -> Option<String> {
        let date_val = self.date_input.value();
        if date_val.is_empty() {
            return Some("Date is required".to_string());
        }

        match NaiveDate::parse_from_str(date_val, "%Y-%m-%d") {
            Ok(date) => {
                let today = Utc::now().date_naive();
                if date < today {
                    Some("Date cannot be in the past".to_string())
                } else {
                    None
                }
            }
            Err(_) => Some("Invalid date format (use YYYY-MM-DD)".to_string()),
        }
    }

    fn validate_time(&self) -> Option<String> {
        let time_val = self.time_input.value();
        if time_val.is_empty() {
            return Some("Time is required".to_string());
        }

        match NaiveTime::parse_from_str(time_val, "%H:%M") {
            Ok(time) => {
                let hour = time.hour();
                if !(8..18).contains(&hour) {
                    Some("Time must be between 08:00 and 18:00".to_string())
                } else {
                    None
                }
            }
            Err(_) => Some("Invalid time format (use HH:MM)".to_string()),
        }
    }

    fn validate_all_fields(&mut self) -> bool {
        self.validation_errors.clear();

        // Patient validation
        if self.selected_patient_id.is_none() {
            self.validation_errors
                .insert(FormField::Patient, "Please select a patient".to_string());
        }

        // Date validation
        if let Some(error) = self.validate_date() {
            self.validation_errors.insert(FormField::Date, error);
        }

        // Time validation
        if let Some(error) = self.validate_time() {
            self.validation_errors.insert(FormField::Time, error);
        }

        self.validation_errors.is_empty()
    }

    async fn fetch_availability(&mut self) -> Result<()> {
        let mut availability = vec![true; 40];

        if let Ok(date) = NaiveDate::parse_from_str(self.date_input.value(), "%Y-%m-%d") {
            let practitioner_idx = self.practitioner_select.selected_index().unwrap_or(0);
            if practitioner_idx < self.practitioners.len() {
                let practitioner_id = self.practitioners[practitioner_idx].id;

                let start_of_day = date.and_hms_opt(0, 0, 0).unwrap().and_utc();
                let end_of_day = date.and_hms_opt(23, 59, 59).unwrap().and_utc();

                let criteria = crate::domain::appointment::AppointmentSearchCriteria {
                    practitioner_id: Some(practitioner_id),
                    date_from: Some(start_of_day),
                    date_to: Some(end_of_day),
                    patient_id: None,
                    status: None,
                    appointment_type: None,
                    is_urgent: None,
                    confirmed: None,
                };

                match self
                    .appointment_service
                    .search_appointments(&criteria)
                    .await
                {
                    Ok(appointments) => {
                        for appointment in appointments {
                            let duration_minutes = (appointment.end_time - appointment.start_time)
                                .num_minutes()
                                as usize;
                            let slot_span = (duration_minutes / 15).max(1);

                            let start_hour = appointment.start_time.hour() as usize;
                            let start_minute = appointment.start_time.minute() as usize;
                            let start_slot = (start_hour - 8) * 4 + start_minute / 15;

                            for i in 0..slot_span {
                                if start_slot + i < 40 {
                                    availability[start_slot + i] = false;
                                }
                            }
                        }
                    }
                    Err(_) => {
                        tracing::warn!("Failed to fetch appointments for availability check");
                    }
                }
            }
        }

        self.time_slot_picker.set_availability(availability);
        Ok(())
    }

    async fn submit_form(&mut self) -> Result<()> {
        if !self.validate_all_fields() {
            return Ok(());
        }

        let date = NaiveDate::parse_from_str(self.date_input.value(), "%Y-%m-%d")
            .map_err(|e| crate::error::Error::App(format!("Invalid date: {}", e)))?;
        let time = NaiveTime::parse_from_str(self.time_input.value(), "%H:%M")
            .map_err(|e| crate::error::Error::App(format!("Invalid time: {}", e)))?;

        let start_time = date.and_time(time).and_utc();

        let practitioner_idx = self.practitioner_select.selected_index().unwrap_or(0);
        let practitioner_id = self.practitioners[practitioner_idx].id;

        let type_idx = self.type_select.selected_index().unwrap_or(0);
        let appointment_type = self.appointment_types[type_idx];
        let duration = Duration::minutes(appointment_type.default_duration_minutes());

        let data = NewAppointmentData {
            patient_id: self.selected_patient_id.unwrap(),
            practitioner_id,
            start_time,
            duration,
            appointment_type,
            reason: if self.reason_input.value().is_empty() {
                None
            } else {
                Some(self.reason_input.value().to_string())
            },
            is_urgent: false,
        };

        let user_id = Uuid::parse_str("a1b2c3d4-e5f6-4789-a1b2-c3d4e5f64789").unwrap();

        // Call service
        self.appointment_service
            .create_appointment(data, user_id)
            .await
            .map_err(|e| {
                crate::error::Error::App(format!("Failed to create appointment: {}", e))
            })?;

        Ok(())
    }

    fn get_selected_patient_display(&self) -> String {
        if let Some(patient_id) = self.selected_patient_id {
            if let Some(patient) = self.all_patients.iter().find(|p| p.id == patient_id) {
                return format!(
                    "{}, {} ({})",
                    patient.last_name,
                    patient
                        .preferred_name
                        .as_ref()
                        .unwrap_or(&patient.first_name),
                    patient.medicare_number.as_ref().unwrap_or(&"-".to_string())
                );
            }
        }
        self.patient_query.clone()
    }

    fn get_selected_practitioner_display(&self) -> String {
        let idx = self.practitioner_select.selected_index().unwrap_or(0);
        if idx < self.practitioners.len() {
            self.practitioners[idx].display_name()
        } else {
            "Select practitioner".to_string()
        }
    }

    fn get_selected_type_display(&self) -> String {
        if let Some(selected) = self.type_select.selected() {
            selected.clone()
        } else {
            "Select type".to_string()
        }
    }

    fn handle_patient_input(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Char(c) if key.modifiers.contains(KeyModifiers::CONTROL) => {
                if c == 'u' {
                    self.patient_query.clear();
                    self.selected_patient_id = None;
                    self.apply_patient_filter();
                    Action::Render
                } else {
                    Action::None
                }
            }
            KeyCode::Char(c) => {
                self.patient_search_active = true;
                self.patient_query.push(c);
                self.apply_patient_filter();
                self.validation_errors.remove(&FormField::Patient);
                Action::Render
            }
            KeyCode::Backspace => {
                if !self.patient_query.is_empty() {
                    self.patient_query.pop();
                    self.apply_patient_filter();
                    if self.patient_query.is_empty() {
                        self.selected_patient_id = None;
                    }
                }
                Action::Render
            }
            KeyCode::Esc => {
                if !self.patient_query.is_empty() {
                    self.patient_query.clear();
                    self.selected_patient_id = None;
                    self.apply_patient_filter();
                    Action::Render
                } else {
                    Action::None
                }
            }
            KeyCode::Down => {
                if self.patient_search_active && !self.filtered_patients.is_empty() {
                    let current = self.patient_list_state.selected().unwrap_or(0);
                    let next = (current + 1).min(self.filtered_patients.len() - 1);
                    self.patient_list_state.select(Some(next));
                }
                Action::Render
            }
            KeyCode::Up => {
                if self.patient_search_active && !self.filtered_patients.is_empty() {
                    let current = self.patient_list_state.selected().unwrap_or(0);
                    let prev = current.saturating_sub(1);
                    self.patient_list_state.select(Some(prev));
                }
                Action::Render
            }
            KeyCode::Enter => {
                if let Some(idx) = self.patient_list_state.selected() {
                    if idx < self.filtered_patients.len() {
                        let patient = &self.filtered_patients[idx];
                        self.selected_patient_id = Some(patient.id);
                        self.patient_query = format!(
                            "{}, {}",
                            patient.last_name,
                            patient
                                .preferred_name
                                .as_ref()
                                .unwrap_or(&patient.first_name)
                        );
                        self.patient_search_active = false;
                        self.validation_errors.remove(&FormField::Patient);
                    }
                }
                Action::Render
            }
            _ => Action::None,
        }
    }

    fn handle_practitioner_input(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.practitioner_select.toggle();
                Action::Render
            }
            KeyCode::Down => {
                if self.practitioner_select.is_open() {
                    self.practitioner_select.next();
                }
                Action::Render
            }
            KeyCode::Up => {
                if self.practitioner_select.is_open() {
                    self.practitioner_select.previous();
                }
                Action::Render
            }
            _ => Action::None,
        }
    }

    fn handle_date_input(&mut self, key: KeyEvent) -> Action {
        if self.date_calendar.handle_key_event(key) {
            self.sync_calendar_to_input();
            self.validation_errors.remove(&FormField::Date);
            return Action::Render;
        }

        match key.code {
            KeyCode::Enter => {
                self.validation_errors.remove(&FormField::Date);
                Action::Render
            }
            KeyCode::Char(c) if c.is_ascii_digit() || c == '-' => {
                if self.date_input.value().len() < 10 {
                    self.date_input.push_char(c);
                    self.validation_errors.remove(&FormField::Date);
                }
                Action::Render
            }
            KeyCode::Backspace => {
                self.date_input.pop_char();
                Action::Render
            }
            _ => Action::None,
        }
    }

    fn sync_calendar_to_input(&mut self) {
        let selected_date = self.date_calendar.selected_date();
        self.date_input.set_value(&selected_date.format("%Y-%m-%d").to_string());
    }

    fn handle_time_input(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Up => {
                self.time_slot_picker.prev();
                let selected = self.time_slot_picker.selected_time();
                self.time_input.set_value(selected);
                self.validation_errors.remove(&FormField::Time);
                Action::Render
            }
            KeyCode::Down => {
                self.time_slot_picker.next();
                let selected = self.time_slot_picker.selected_time();
                self.time_input.set_value(selected);
                self.validation_errors.remove(&FormField::Time);
                Action::Render
            }
            KeyCode::Enter => {
                self.validation_errors.remove(&FormField::Time);
                Action::Render
            }
            KeyCode::Char(c) if c.is_ascii_digit() || c == ':' => {
                if self.time_input.value().len() < 5 {
                    self.time_input.push_char(c);
                    self.validation_errors.remove(&FormField::Time);
                }
                Action::Render
            }
            KeyCode::Backspace => {
                self.time_input.pop_char();
                Action::Render
            }
            _ => Action::None,
        }
    }

    fn handle_type_input(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.type_select.toggle();
                Action::Render
            }
            KeyCode::Down => {
                if self.type_select.is_open() {
                    self.type_select.next();
                }
                Action::Render
            }
            KeyCode::Up => {
                if self.type_select.is_open() {
                    self.type_select.previous();
                }
                Action::Render
            }
            _ => Action::None,
        }
    }

    fn handle_reason_input(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Char(c) => {
                if self.reason_input.value().len() < 500 {
                    self.reason_input.push_char(c);
                }
                Action::Render
            }
            KeyCode::Backspace => {
                self.reason_input.pop_char();
                Action::Render
            }
            _ => Action::None,
        }
    }

    fn render_patient_field(&mut self, frame: &mut Frame, area: Rect) {
        let is_focused = self.current_field == FormField::Patient;
        let border_color = if is_focused {
            Color::Yellow
        } else if self.validation_errors.contains_key(&FormField::Patient) {
            Color::Red
        } else {
            Color::White
        };

        let display = self.get_selected_patient_display();
        let text = if self.patient_search_active && is_focused {
            format!("{}█", display)
        } else {
            display
        };

        let mut lines = vec![Line::from(text)];

        // Show validation error
        if let Some(error) = self.validation_errors.get(&FormField::Patient) {
            lines.push(Line::from(Span::styled(
                error.as_str(),
                Style::default().fg(Color::Red),
            )));
        }

        // Show search results if active
        if self.patient_search_active && is_focused {
            if self.filtered_patients.is_empty() {
                lines.push(Line::from(Span::styled(
                    "  No matching patients found",
                    Style::default()
                        .fg(Color::Gray)
                        .add_modifier(Modifier::ITALIC),
                )));
            } else {
                let max_results = 8;
                let total_count = self.filtered_patients.len();
                let visible_count = total_count.min(max_results);

                lines.push(Line::from(Span::styled(
                    format!("  {} of {} matches", visible_count, total_count),
                    Style::default().fg(Color::DarkGray),
                )));

                let results: Vec<String> = self
                    .filtered_patients
                    .iter()
                    .take(max_results)
                    .map(|p| {
                        format!(
                            "{}, {} ({})",
                            p.last_name,
                            p.preferred_name.as_ref().unwrap_or(&p.first_name),
                            p.medicare_number.as_ref().unwrap_or(&"-".to_string())
                        )
                    })
                    .collect();

                for (i, result) in results.iter().enumerate() {
                    let is_selected = Some(i) == self.patient_list_state.selected();
                    let style = if is_selected {
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::Gray)
                    };
                    lines.push(Line::from(Span::styled(format!("  {}", result), style)));
                }
            }
        }

        let paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Patient ")
                    .border_style(Style::default().fg(border_color)),
            )
            .wrap(Wrap { trim: false });

        frame.render_widget(paragraph, area);
    }

    fn render_practitioner_field(&mut self, frame: &mut Frame, area: Rect) {
        let is_focused = self.current_field == FormField::Practitioner;
        let border_color = if is_focused {
            Color::Yellow
        } else {
            Color::White
        };

        let display = self.get_selected_practitioner_display();

        if self.practitioner_select.is_open() && is_focused {
            let items: Vec<ListItem> = self
                .practitioners
                .iter()
                .map(|p| ListItem::new(p.display_name()))
                .collect();

            let list = List::new(items)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" Practitioner ")
                        .border_style(Style::default().fg(border_color)),
                )
                .highlight_style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol(">> ");

            let mut list_state = ListState::default();
            if let Some(idx) = self.practitioner_select.selected_index() {
                list_state.select(Some(idx));
            }
            frame.render_stateful_widget(list, area, &mut list_state);
        } else {
            let paragraph = Paragraph::new(display).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Practitioner ")
                    .border_style(Style::default().fg(border_color)),
            );
            frame.render_widget(paragraph, area);
        }
    }

    fn render_date_field(&mut self, frame: &mut Frame, area: Rect) {
        let is_focused = self.current_field == FormField::Date;

        if is_focused {
            self.date_calendar.render(frame, area, true);
            if let Some(error) = self.validation_errors.get(&FormField::Date) {
                let error_area = Rect {
                    y: area.y + area.height.saturating_sub(2),
                    height: 1,
                    ..area
                };
                let error_text = Paragraph::new(Span::styled(
                    error.as_str(),
                    Style::default().fg(Color::Red),
                ));
                frame.render_widget(error_text, error_area);
            }
        } else {
            let border_color = if self.validation_errors.contains_key(&FormField::Date) {
                Color::Red
            } else {
                Color::White
            };

            let display = self.date_input.value().to_string();
            let mut lines = vec![Line::from(display)];

            if let Some(error) = self.validation_errors.get(&FormField::Date) {
                lines.push(Line::from(Span::styled(
                    error.as_str(),
                    Style::default().fg(Color::Red),
                )));
            }

            let paragraph = Paragraph::new(lines).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Date (YYYY-MM-DD) ")
                    .border_style(Style::default().fg(border_color)),
            );

            frame.render_widget(paragraph, area);
        }
    }

    fn render_time_field(&mut self, frame: &mut Frame, area: Rect) {
        let is_focused = self.current_field == FormField::Time;

        if is_focused {
            self.time_slot_picker.render(frame, area);
            if let Some(error) = self.validation_errors.get(&FormField::Time) {
                let error_area = Rect {
                    y: area.y + area.height.saturating_sub(2),
                    height: 1,
                    ..area
                };
                let error_text = Paragraph::new(Span::styled(
                    error.as_str(),
                    Style::default().fg(Color::Red),
                ));
                frame.render_widget(error_text, error_area);
            }
        } else {
            let border_color = if self.validation_errors.contains_key(&FormField::Time) {
                Color::Red
            } else {
                Color::White
            };

            let display = self.time_input.value().to_string();
            let mut lines = vec![Line::from(display)];

            if let Some(error) = self.validation_errors.get(&FormField::Time) {
                lines.push(Line::from(Span::styled(
                    error.as_str(),
                    Style::default().fg(Color::Red),
                )));
            }

            let paragraph = Paragraph::new(lines).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Time (HH:MM) ")
                    .border_style(Style::default().fg(border_color)),
            );

            frame.render_widget(paragraph, area);
        }
    }

    fn render_type_field(&mut self, frame: &mut Frame, area: Rect) {
        let is_focused = self.current_field == FormField::Type;
        let border_color = if is_focused {
            Color::Yellow
        } else {
            Color::White
        };

        let display = self.get_selected_type_display();

        if self.type_select.is_open() && is_focused {
            let items: Vec<ListItem> = self
                .appointment_types
                .iter()
                .map(|t| ListItem::new(format!("{} ({} min)", t, t.default_duration_minutes())))
                .collect();

            let list = List::new(items)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" Appointment Type ")
                        .border_style(Style::default().fg(border_color)),
                )
                .highlight_style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol(">> ");

            let mut list_state = ListState::default();
            if let Some(idx) = self.type_select.selected_index() {
                list_state.select(Some(idx));
            }
            frame.render_stateful_widget(list, area, &mut list_state);
        } else {
            let paragraph = Paragraph::new(display).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Appointment Type ")
                    .border_style(Style::default().fg(border_color)),
            );
            frame.render_widget(paragraph, area);
        }
    }

    fn render_reason_field(&mut self, frame: &mut Frame, area: Rect) {
        let is_focused = self.current_field == FormField::Reason;
        let border_color = if is_focused {
            Color::Yellow
        } else {
            Color::White
        };

        let reason_val = self.reason_input.value();
        let display = if is_focused {
            format!("{}█", reason_val)
        } else {
            reason_val.to_string()
        };

        let paragraph = Paragraph::new(display)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Reason (Optional) ")
                    .border_style(Style::default().fg(border_color)),
            )
            .wrap(Wrap { trim: false });

        frame.render_widget(paragraph, area);
    }

    fn render_help_and_errors(&self, frame: &mut Frame, area: Rect) {
        let mut lines = vec![];

        if let Some(ref error) = self.error_message {
            lines.push(Line::from(Span::styled(
                format!("Error: {}", error),
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            )));
        }

        let field_indicator = format!("Field: {:?}", self.current_field);
        lines.push(Line::from(field_indicator));

        let paragraph = Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL).title(" Info "))
            .wrap(Wrap { trim: false });

        frame.render_widget(paragraph, area);
    }
}

#[async_trait]
impl Component for AppointmentFormComponent {
    async fn init(&mut self) -> Result<()> {
        match self.patient_service.list_active_patients().await {
            Ok(patients) => {
                self.all_patients = patients.clone();
                self.filtered_patients = patients;
            }
            Err(e) => {
                self.error_message = Some(format!("Failed to load patients: {}", e));
            }
        }

        match self.practitioner_service.get_active_practitioners().await {
            Ok(practitioners) => {
                self.practitioners = practitioners;
                if !self.practitioners.is_empty() {
                    self.practitioner_select.set_selected(0);
                }
            }
            Err(e) => {
                self.error_message = Some(format!("Failed to load practitioners: {}", e));
            }
        }

        Ok(())
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> Action {
        // Handle Esc to cancel
        if key.code == KeyCode::Esc {
            if self.patient_search_active {
                self.patient_search_active = false;
                return Action::Render;
            }
            if self.practitioner_select.is_open() {
                self.practitioner_select.toggle();
                return Action::Render;
            }
            if self.type_select.is_open() {
                self.type_select.toggle();
                return Action::Render;
            }
            return Action::AppointmentFormCancel;
        }

        // Handle Ctrl+S to submit
        if key.code == KeyCode::Char('s') && key.modifiers.contains(KeyModifiers::CONTROL) {
            self.is_submitting = true;
            return Action::AppointmentFormSubmit;
        }

        // Handle Tab navigation (forward)
        if key.code == KeyCode::Tab {
            self.current_field = self.current_field.next();

            self.practitioner_select.set_focus(self.current_field == FormField::Practitioner);
            self.type_select.set_focus(self.current_field == FormField::Type);

            let moved_to_patient_field = self.current_field == FormField::Patient;
            self.patient_search_active = moved_to_patient_field;

            if self.current_field == FormField::Date {
                if let Ok(date) = NaiveDate::parse_from_str(self.date_input.value(), "%Y-%m-%d") {
                    self.date_calendar = MonthCalendar::new(date);
                }
            }

            return Action::Render;
        }

        // Handle BackTab navigation (backward - Shift+Tab)
        if key.code == KeyCode::BackTab {
            self.current_field = self.current_field.previous();

            self.practitioner_select.set_focus(self.current_field == FormField::Practitioner);
            self.type_select.set_focus(self.current_field == FormField::Type);

            let moved_to_patient_field = self.current_field == FormField::Patient;
            self.patient_search_active = moved_to_patient_field;

            if self.current_field == FormField::Date {
                if let Ok(date) = NaiveDate::parse_from_str(self.date_input.value(), "%Y-%m-%d") {
                    self.date_calendar = MonthCalendar::new(date);
                }
            }

            return Action::Render;
        }

        // Field-specific handling
        match self.current_field {
            FormField::Patient => self.handle_patient_input(key),
            FormField::Practitioner => self.handle_practitioner_input(key),
            FormField::Date => self.handle_date_input(key),
            FormField::Time => self.handle_time_input(key),
            FormField::Type => self.handle_type_input(key),
            FormField::Reason => self.handle_reason_input(key),
        }
    }

    async fn update(&mut self, action: Action) -> Result<Option<Action>> {
        if action == Action::AppointmentFormSubmit && self.is_submitting {
            self.is_submitting = false;
            match self.submit_form().await {
                Ok(_) => {
                    return Ok(Some(Action::AppointmentFormSubmit));
                }
                Err(e) => {
                    self.error_message = Some(format!("Failed to create appointment: {}", e));
                    return Ok(Some(Action::Render));
                }
            }
        }

        if action == Action::Render && self.current_field == FormField::Time {
            let _ = self.fetch_availability().await;
        }

        Ok(None)
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let theme = Theme::default();
        let vertical = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(10),
                Constraint::Percentage(80),
                Constraint::Percentage(10),
            ])
            .split(area);

        let horizontal = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(15),
                Constraint::Percentage(70),
                Constraint::Percentage(15),
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

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(if self.current_field == FormField::Patient {
                    12
                } else {
                    3
                }),
                Constraint::Min(if self.current_field == FormField::Practitioner {
                    8
                } else {
                    3
                }),
                Constraint::Min(if self.current_field == FormField::Date {
                    15
                } else {
                    3
                }),
                Constraint::Min(if self.current_field == FormField::Time {
                    12
                } else {
                    3
                }),
                Constraint::Min(if self.current_field == FormField::Type {
                    16
                } else {
                    3
                }),
                Constraint::Min(if self.current_field == FormField::Reason {
                    8
                } else {
                    3
                }),
                Constraint::Length(3),
            ])
            .split(inner_area);

        // Title
        let title = Paragraph::new("Create New Appointment")
            .style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(title, chunks[0]);

        // Patient field
        self.render_patient_field(frame, chunks[1]);

        // Practitioner field
        self.render_practitioner_field(frame, chunks[2]);

        // Date field
        self.render_date_field(frame, chunks[3]);

        // Time field
        self.render_time_field(frame, chunks[4]);

        // Type field
        self.render_type_field(frame, chunks[5]);

        // Reason field
        self.render_reason_field(frame, chunks[6]);

        // Help text and errors
        self.render_help_and_errors(frame, chunks[7]);
    }
}

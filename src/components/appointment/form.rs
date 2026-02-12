use async_trait::async_trait;
use chrono::{Duration, NaiveDate, NaiveTime, Timelike, Utc};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};
use ratatui::Frame;
use std::collections::HashMap;
use std::sync::Arc;
use sublime_fuzzy::best_match;
use uuid::Uuid;

use crate::components::{Action, Component};
use crate::domain::appointment::{AppointmentService, AppointmentType, NewAppointmentData};
use crate::domain::patient::{Patient, PatientService};
use crate::domain::user::Practitioner;
use crate::error::Result;

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
    #[allow(dead_code)]
    patient_service: Arc<PatientService>,
    
    // Form state
    current_field: FormField,
    
    // Patient search
    patient_query: String,
    all_patients: Vec<Patient>,
    filtered_patients: Vec<Patient>,
    patient_list_state: ListState,
    selected_patient_id: Option<Uuid>,
    patient_search_active: bool,
    
    // Practitioner selection
    practitioners: Vec<Practitioner>,
    practitioner_list_state: ListState,
    practitioner_dropdown_open: bool,
    
    // Appointment type selection
    appointment_types: Vec<AppointmentType>,
    type_list_state: ListState,
    type_dropdown_open: bool,
    
    // Form inputs
    date_input: String,
    time_input: String,
    reason_input: String,
    
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
    ) -> Self {
        let mut patient_list_state = ListState::default();
        patient_list_state.select(Some(0));
        
        let mut practitioner_list_state = ListState::default();
        practitioner_list_state.select(Some(0));
        
        let mut type_list_state = ListState::default();
        type_list_state.select(Some(0));
        
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
        
        // Set default date to today and time to 9:00
        let now = Utc::now();
        let date_input = now.format("%Y-%m-%d").to_string();
        let time_input = "09:00".to_string();
        
        Self {
            appointment_service,
            patient_service,
            current_field: FormField::Patient,
            patient_query: String::new(),
            all_patients: Vec::new(),
            filtered_patients: Vec::new(),
            patient_list_state,
            selected_patient_id: None,
            patient_search_active: true,
            practitioners: Self::generate_mock_practitioners(),
            practitioner_list_state,
            practitioner_dropdown_open: false,
            appointment_types,
            type_list_state,
            type_dropdown_open: false,
            date_input,
            time_input,
            reason_input: String::new(),
            validation_errors: HashMap::new(),
            error_message: None,
            is_submitting: false,
        }
    }
    

    
    fn generate_mock_practitioners() -> Vec<Practitioner> {
        vec![
            Practitioner {
                id: Uuid::parse_str("a1b2c3d4-e5f6-4789-a1b2-c3d4e5f64789").unwrap(),
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
                id: Uuid::parse_str("b2c3d4e5-f6a7-89a1-b2c3-d4e5f6a789a1").unwrap(),
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
                id: Uuid::parse_str("c3d4e5f6-a789-a1b2-c3d4-e5f6a789a1b2").unwrap(),
                user_id: None,
                first_name: "Emily".to_string(),
                middle_name: None,
                last_name: "Williams".to_string(),
                title: "Dr".to_string(),
                hpi_i: Some("8003610000000002".to_string()),
                ahpra_registration: Some("MED0001234569".to_string()),
                prescriber_number: Some("345678".to_string()),
                provider_number: "345678C".to_string(),
                speciality: Some("General Practice".to_string()),
                qualifications: vec!["MBBS".to_string(), "FRACGP".to_string(), "FACRRM".to_string()],
                phone: Some("02 9876 5434".to_string()),
                email: Some("e.williams@clinic.com".to_string()),
                is_active: true,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
        ]
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
        if self.date_input.is_empty() {
            return Some("Date is required".to_string());
        }
        
        match NaiveDate::parse_from_str(&self.date_input, "%Y-%m-%d") {
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
        if self.time_input.is_empty() {
            return Some("Time is required".to_string());
        }
        
        match NaiveTime::parse_from_str(&self.time_input, "%H:%M") {
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
            self.validation_errors.insert(
                FormField::Patient,
                "Please select a patient".to_string(),
            );
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
    
    async fn submit_form(&mut self) -> Result<()> {
        if !self.validate_all_fields() {
            return Ok(());
        }
        
        // Parse date and time
        let date = NaiveDate::parse_from_str(&self.date_input, "%Y-%m-%d")
            .map_err(|e| crate::error::Error::App(format!("Invalid date: {}", e)))?;
        let time = NaiveTime::parse_from_str(&self.time_input, "%H:%M")
            .map_err(|e| crate::error::Error::App(format!("Invalid time: {}", e)))?;
        
        let start_time = date.and_time(time).and_utc();
        
        // Get selected practitioner
        let practitioner_idx = self.practitioner_list_state.selected().unwrap_or(0);
        let practitioner_id = self.practitioners[practitioner_idx].id;
        
        // Get selected appointment type
        let type_idx = self.type_list_state.selected().unwrap_or(0);
        let appointment_type = self.appointment_types[type_idx];
        let duration = Duration::minutes(appointment_type.default_duration_minutes());
        
        // Create appointment data
        let data = NewAppointmentData {
            patient_id: self.selected_patient_id.unwrap(),
            practitioner_id,
            start_time,
            duration,
            appointment_type,
            reason: if self.reason_input.is_empty() {
                None
            } else {
                Some(self.reason_input.clone())
            },
            is_urgent: false,
        };
        
        let user_id = Uuid::parse_str("a1b2c3d4-e5f6-4789-a1b2-c3d4e5f64789").unwrap();
        
        // Call service
        self.appointment_service
            .create_appointment(data, user_id)
            .await
            .map_err(|e| crate::error::Error::App(format!("Failed to create appointment: {}", e)))?;
        
        Ok(())
    }
    
    fn get_selected_patient_display(&self) -> String {
        if let Some(patient_id) = self.selected_patient_id {
            if let Some(patient) = self.all_patients.iter().find(|p| p.id == patient_id) {
                return format!(
                    "{}, {} ({})",
                    patient.last_name,
                    patient.preferred_name.as_ref().unwrap_or(&patient.first_name),
                    patient.medicare_number.as_ref().unwrap_or(&"-".to_string())
                );
            }
        }
        self.patient_query.clone()
    }
    
    fn get_selected_practitioner_display(&self) -> String {
        let idx = self.practitioner_list_state.selected().unwrap_or(0);
        if idx < self.practitioners.len() {
            self.practitioners[idx].display_name()
        } else {
            "Select practitioner".to_string()
        }
    }
    
    fn get_selected_type_display(&self) -> String {
        let idx = self.type_list_state.selected().unwrap_or(0);
        if idx < self.appointment_types.len() {
            let app_type = self.appointment_types[idx];
            format!("{} ({} min)", app_type, app_type.default_duration_minutes())
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
                            patient.preferred_name.as_ref().unwrap_or(&patient.first_name)
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
                self.practitioner_dropdown_open = !self.practitioner_dropdown_open;
                Action::Render
            }
            KeyCode::Down => {
                if self.practitioner_dropdown_open {
                    let current = self.practitioner_list_state.selected().unwrap_or(0);
                    let next = (current + 1).min(self.practitioners.len() - 1);
                    self.practitioner_list_state.select(Some(next));
                }
                Action::Render
            }
            KeyCode::Up => {
                if self.practitioner_dropdown_open {
                    let current = self.practitioner_list_state.selected().unwrap_or(0);
                    let prev = current.saturating_sub(1);
                    self.practitioner_list_state.select(Some(prev));
                }
                Action::Render
            }
            _ => Action::None,
        }
    }
    
    fn handle_date_input(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Char(c) if c.is_ascii_digit() || c == '-' => {
                if self.date_input.len() < 10 {
                    self.date_input.push(c);
                    self.validation_errors.remove(&FormField::Date);
                }
                Action::Render
            }
            KeyCode::Backspace => {
                self.date_input.pop();
                Action::Render
            }
            _ => Action::None,
        }
    }
    
    fn handle_time_input(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Char(c) if c.is_ascii_digit() || c == ':' => {
                if self.time_input.len() < 5 {
                    self.time_input.push(c);
                    self.validation_errors.remove(&FormField::Time);
                }
                Action::Render
            }
            KeyCode::Backspace => {
                self.time_input.pop();
                Action::Render
            }
            _ => Action::None,
        }
    }
    
    fn handle_type_input(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.type_dropdown_open = !self.type_dropdown_open;
                Action::Render
            }
            KeyCode::Down => {
                if self.type_dropdown_open {
                    let current = self.type_list_state.selected().unwrap_or(0);
                    let next = (current + 1).min(self.appointment_types.len() - 1);
                    self.type_list_state.select(Some(next));
                }
                Action::Render
            }
            KeyCode::Up => {
                if self.type_dropdown_open {
                    let current = self.type_list_state.selected().unwrap_or(0);
                    let prev = current.saturating_sub(1);
                    self.type_list_state.select(Some(prev));
                }
                Action::Render
            }
            _ => Action::None,
        }
    }
    
    fn handle_reason_input(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Char(c) => {
                if self.reason_input.len() < 500 {
                    self.reason_input.push(c);
                }
                Action::Render
            }
            KeyCode::Backspace => {
                self.reason_input.pop();
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
                    Style::default().fg(Color::Gray).add_modifier(Modifier::ITALIC),
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
                        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::Gray)
                    };
                    lines.push(Line::from(Span::styled(
                        format!("  {}", result),
                        style,
                    )));
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
        let border_color = if is_focused { Color::Yellow } else { Color::White };
        
        let display = self.get_selected_practitioner_display();
        
        if self.practitioner_dropdown_open && is_focused {
            // Show dropdown list
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
            
            frame.render_stateful_widget(list, area, &mut self.practitioner_list_state);
        } else {
            let paragraph = Paragraph::new(display)
                .block(
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
        let border_color = if is_focused {
            Color::Yellow
        } else if self.validation_errors.contains_key(&FormField::Date) {
            Color::Red
        } else {
            Color::White
        };
        
        let display = if is_focused {
            format!("{}█", self.date_input)
        } else {
            self.date_input.clone()
        };
        
        let mut lines = vec![Line::from(display)];
        
        if let Some(error) = self.validation_errors.get(&FormField::Date) {
            lines.push(Line::from(Span::styled(
                error.as_str(),
                Style::default().fg(Color::Red),
            )));
        }
        
        let paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Date (YYYY-MM-DD) ")
                    .border_style(Style::default().fg(border_color)),
            );
        
        frame.render_widget(paragraph, area);
    }
    
    fn render_time_field(&mut self, frame: &mut Frame, area: Rect) {
        let is_focused = self.current_field == FormField::Time;
        let border_color = if is_focused {
            Color::Yellow
        } else if self.validation_errors.contains_key(&FormField::Time) {
            Color::Red
        } else {
            Color::White
        };
        
        let display = if is_focused {
            format!("{}█", self.time_input)
        } else {
            self.time_input.clone()
        };
        
        let mut lines = vec![Line::from(display)];
        
        if let Some(error) = self.validation_errors.get(&FormField::Time) {
            lines.push(Line::from(Span::styled(
                error.as_str(),
                Style::default().fg(Color::Red),
            )));
        }
        
        let paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Time (HH:MM) ")
                    .border_style(Style::default().fg(border_color)),
            );
        
        frame.render_widget(paragraph, area);
    }
    
    fn render_type_field(&mut self, frame: &mut Frame, area: Rect) {
        let is_focused = self.current_field == FormField::Type;
        let border_color = if is_focused { Color::Yellow } else { Color::White };
        
        let display = self.get_selected_type_display();
        
        if self.type_dropdown_open && is_focused {
            // Show dropdown list
            let items: Vec<ListItem> = self
                .appointment_types
                .iter()
                .map(|t| {
                    ListItem::new(format!(
                        "{} ({} min)",
                        t,
                        t.default_duration_minutes()
                    ))
                })
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
            
            frame.render_stateful_widget(list, area, &mut self.type_list_state);
        } else {
            let paragraph = Paragraph::new(display)
                .block(
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
        let border_color = if is_focused { Color::Yellow } else { Color::White };
        
        let display = if is_focused {
            format!("{}█", self.reason_input)
        } else {
            self.reason_input.clone()
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
        
        if self.current_field == FormField::Patient {
            lines.push(Line::from(vec![
                Span::styled("[↑↓]", Style::default().fg(Color::Cyan)),
                Span::raw(" Navigate  "),
                Span::styled("[Esc]", Style::default().fg(Color::Cyan)),
                Span::raw(" Clear  "),
                Span::styled("[Ctrl+U]", Style::default().fg(Color::Cyan)),
                Span::raw(" Clear All  "),
                Span::styled("[Ctrl+S]", Style::default().fg(Color::Green)),
                Span::raw(" Submit"),
            ]));
        } else {
            lines.push(Line::from(vec![
                Span::styled("[Tab]", Style::default().fg(Color::Cyan)),
                Span::raw(" Next  "),
                Span::styled("[Shift+Tab]", Style::default().fg(Color::Cyan)),
                Span::raw(" Prev  "),
                Span::styled("[Ctrl+S]", Style::default().fg(Color::Green)),
                Span::raw(" Submit  "),
                Span::styled("[Esc]", Style::default().fg(Color::Red)),
                Span::raw(" Cancel"),
            ]));
        }
        
        let paragraph = Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL).title(" Help "))
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
        Ok(())
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> Action {
        // Handle Esc to cancel
        if key.code == KeyCode::Esc {
            if self.patient_search_active {
                self.patient_search_active = false;
                return Action::Render;
            }
            if self.practitioner_dropdown_open {
                self.practitioner_dropdown_open = false;
                return Action::Render;
            }
            if self.type_dropdown_open {
                self.type_dropdown_open = false;
                return Action::Render;
            }
            return Action::AppointmentFormCancel;
        }
        
        // Handle Ctrl+S to submit
        if key.code == KeyCode::Char('s') && key.modifiers.contains(KeyModifiers::CONTROL) {
            self.is_submitting = true;
            return Action::AppointmentFormSubmit;
        }
        
        // Handle Tab navigation
        if key.code == KeyCode::Tab {
            if key.modifiers.contains(KeyModifiers::SHIFT) {
                self.current_field = self.current_field.previous();
            } else {
                self.current_field = self.current_field.next();
            }
            self.practitioner_dropdown_open = false;
            self.type_dropdown_open = false;
            let moved_to_patient_field = self.current_field == FormField::Patient;
            self.patient_search_active = moved_to_patient_field;
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
                    // Form submitted successfully, return action to close and refresh
                    return Ok(Some(Action::AppointmentFormSubmit));
                }
                Err(e) => {
                    self.error_message = Some(format!("Failed to create appointment: {}", e));
                    return Ok(Some(Action::Render));
                }
            }
        }
        Ok(None)
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),   // Title
                Constraint::Length(12),  // Patient (increased for 8 results)
                Constraint::Length(3),   // Practitioner
                Constraint::Length(3),   // Date
                Constraint::Length(3),   // Time
                Constraint::Length(3),   // Type
                Constraint::Length(4),   // Reason
                Constraint::Length(4),   // Error/Help (fixed height, compact)
            ])
            .split(area);
        
        // Title
        let title = Paragraph::new("Create New Appointment")
            .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
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

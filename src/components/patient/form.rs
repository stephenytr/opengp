use async_trait::async_trait;
use chrono::NaiveDate;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph};
use ratatui::Frame;
use std::sync::Arc;

use crate::components::{Action, Component};
use crate::domain::patient::{Address, Gender, NewPatientData, PatientService};
use crate::error::Result;
use crate::ui::keybinds::{KeybindContext, KeybindRegistry};
use crate::ui::Theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FormField {
    FirstName,
    LastName,
    DateOfBirth,
    Gender,
    MedicareNumber,
    MedicareIrn,
    PhoneMobile,
    Email,
}

impl FormField {
    fn all() -> Vec<FormField> {
        vec![
            FormField::FirstName,
            FormField::LastName,
            FormField::DateOfBirth,
            FormField::Gender,
            FormField::MedicareNumber,
            FormField::MedicareIrn,
            FormField::PhoneMobile,
            FormField::Email,
        ]
    }

    fn label(&self) -> &str {
        match self {
            FormField::FirstName => "First Name *",
            FormField::LastName => "Last Name *",
            FormField::DateOfBirth => "Date of Birth * (DD/MM/YYYY)",
            FormField::Gender => "Gender *",
            FormField::MedicareNumber => "Medicare Number",
            FormField::MedicareIrn => "Medicare IRN (1-9)",
            FormField::PhoneMobile => "Mobile Phone",
            FormField::Email => "Email",
        }
    }
}

pub struct PatientFormComponent {
    patient_service: Arc<PatientService>,
    current_field: usize,
    scroll_offset: usize,
    first_name: String,
    last_name: String,
    date_of_birth: String,
    gender_index: usize,
    medicare_number: String,
    medicare_irn: String,
    phone_mobile: String,
    email: String,
    validation_errors: Vec<String>,
    is_submitting: bool,
}

impl PatientFormComponent {
    pub fn new(patient_service: Arc<PatientService>) -> Self {
        Self {
            patient_service,
            current_field: 0,
            scroll_offset: 0,
            first_name: String::new(),
            last_name: String::new(),
            date_of_birth: String::new(),
            gender_index: 0,
            medicare_number: String::new(),
            medicare_irn: String::new(),
            phone_mobile: String::new(),
            email: String::new(),
            validation_errors: Vec::new(),
            is_submitting: false,
        }
    }

    fn validate(&mut self) -> bool {
        self.validation_errors.clear();

        if self.first_name.trim().is_empty() {
            self.validation_errors
                .push("First name is required".to_string());
        }

        if self.last_name.trim().is_empty() {
            self.validation_errors
                .push("Last name is required".to_string());
        }

        if self.date_of_birth.trim().is_empty() {
            self.validation_errors
                .push("Date of birth is required".to_string());
        } else if self.parse_date().is_none() {
            self.validation_errors
                .push("Date of birth must be in DD/MM/YYYY format".to_string());
        }

        if !self.medicare_number.is_empty() && self.medicare_number.len() != 10 {
            self.validation_errors
                .push("Medicare number must be 10 digits".to_string());
        }

        if !self.medicare_irn.is_empty() {
            if let Ok(irn) = self.medicare_irn.parse::<u8>() {
                if !(1..=9).contains(&irn) {
                    self.validation_errors
                        .push("Medicare IRN must be between 1 and 9".to_string());
                }
            } else {
                self.validation_errors
                    .push("Medicare IRN must be a number".to_string());
            }
        }

        self.validation_errors.is_empty()
    }

    fn parse_date(&self) -> Option<NaiveDate> {
        let parts: Vec<&str> = self.date_of_birth.split('/').collect();
        if parts.len() != 3 {
            return None;
        }

        let day = parts[0].parse::<u32>().ok()?;
        let month = parts[1].parse::<u32>().ok()?;
        let year = parts[2].parse::<i32>().ok()?;

        NaiveDate::from_ymd_opt(year, month, day)
    }

    fn current_field(&self) -> FormField {
        FormField::all()[self.current_field]
    }

    fn get_field_value(&self, field: FormField) -> String {
        match field {
            FormField::FirstName => self.first_name.clone(),
            FormField::LastName => self.last_name.clone(),
            FormField::DateOfBirth => self.date_of_birth.clone(),
            FormField::Gender => self.gender_names()[self.gender_index].to_string(),
            FormField::MedicareNumber => self.medicare_number.clone(),
            FormField::MedicareIrn => self.medicare_irn.clone(),
            FormField::PhoneMobile => self.phone_mobile.clone(),
            FormField::Email => self.email.clone(),
        }
    }

    fn handle_input(&mut self, c: char) {
        match self.current_field() {
            FormField::FirstName => self.first_name.push(c),
            FormField::LastName => self.last_name.push(c),
            FormField::DateOfBirth => {
                if c.is_ascii_digit() || c == '/' {
                    self.date_of_birth.push(c);
                }
            }
            FormField::Gender => {}
            FormField::MedicareNumber => {
                if c.is_ascii_digit() && self.medicare_number.len() < 10 {
                    self.medicare_number.push(c);
                }
            }
            FormField::MedicareIrn => {
                if c.is_ascii_digit() && self.medicare_irn.is_empty() {
                    self.medicare_irn.push(c);
                }
            }
            FormField::PhoneMobile => {
                if c.is_ascii_digit() || c == ' ' || c == '+' {
                    self.phone_mobile.push(c);
                }
            }
            FormField::Email => self.email.push(c),
        }
    }

    fn handle_backspace(&mut self) {
        match self.current_field() {
            FormField::FirstName => {
                self.first_name.pop();
            }
            FormField::LastName => {
                self.last_name.pop();
            }
            FormField::DateOfBirth => {
                self.date_of_birth.pop();
            }
            FormField::Gender => {}
            FormField::MedicareNumber => {
                self.medicare_number.pop();
            }
            FormField::MedicareIrn => {
                self.medicare_irn.pop();
            }
            FormField::PhoneMobile => {
                self.phone_mobile.pop();
            }
            FormField::Email => {
                self.email.pop();
            }
        }
    }

    fn next_field(&mut self) {
        let fields = FormField::all();
        self.current_field = (self.current_field + 1) % fields.len();
        self.adjust_scroll();
    }

    fn prev_field(&mut self) {
        let fields = FormField::all();
        if self.current_field == 0 {
            self.current_field = fields.len() - 1;
        } else {
            self.current_field -= 1;
        }
        self.adjust_scroll();
    }

    fn adjust_scroll(&mut self) {
        const VISIBLE_FIELDS: usize = 6;

        if self.current_field < self.scroll_offset {
            self.scroll_offset = self.current_field;
        } else if self.current_field >= self.scroll_offset + VISIBLE_FIELDS {
            self.scroll_offset = self.current_field.saturating_sub(VISIBLE_FIELDS - 1);
        }
    }

    fn gender_names(&self) -> Vec<&str> {
        vec!["Male", "Female", "Other", "Prefer not to say"]
    }

    fn next_gender(&mut self) {
        let genders = self.gender_names();
        self.gender_index = (self.gender_index + 1) % genders.len();
    }

    fn prev_gender(&mut self) {
        let genders = self.gender_names();
        if self.gender_index == 0 {
            self.gender_index = genders.len() - 1;
        } else {
            self.gender_index -= 1;
        }
    }

    fn get_selected_gender(&self) -> Gender {
        match self.gender_index {
            0 => Gender::Male,
            1 => Gender::Female,
            2 => Gender::Other,
            _ => Gender::PreferNotToSay,
        }
    }

    async fn submit_form(&mut self) -> Result<Option<Action>> {
        if !self.validate() {
            return Ok(Some(Action::Render));
        }

        self.is_submitting = true;

        let dob = self.parse_date().unwrap();
        let medicare_irn = if self.medicare_irn.is_empty() {
            None
        } else {
            self.medicare_irn.parse::<u8>().ok()
        };

        let data = NewPatientData {
            ihi: None,
            medicare_number: if self.medicare_number.is_empty() {
                None
            } else {
                Some(self.medicare_number.clone())
            },
            medicare_irn,
            medicare_expiry: None,
            title: None,
            first_name: self.first_name.clone(),
            middle_name: None,
            last_name: self.last_name.clone(),
            preferred_name: None,
            date_of_birth: dob,
            gender: self.get_selected_gender(),
            address: Address::default(),
            phone_home: None,
            phone_mobile: if self.phone_mobile.is_empty() {
                None
            } else {
                Some(self.phone_mobile.clone())
            },
            email: if self.email.is_empty() {
                None
            } else {
                Some(self.email.clone())
            },
            emergency_contact: None,
            concession_type: None,
            concession_number: None,
            preferred_language: None,
            interpreter_required: None,
            aboriginal_torres_strait_islander: None,
        };

        match self.patient_service.register_patient(data).await {
            Ok(_patient) => {
                self.is_submitting = false;
                Ok(Some(Action::PatientFormSubmit))
            }
            Err(e) => {
                self.is_submitting = false;
                self.validation_errors
                    .push(format!("Failed to create patient: {}", e));
                Ok(Some(Action::Render))
            }
        }
    }
}

#[async_trait]
impl Component for PatientFormComponent {
    async fn init(&mut self) -> Result<()> {
        Ok(())
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> Action {
        if self.is_submitting {
            return Action::None;
        }

        match key.code {
            KeyCode::Esc => Action::PatientFormCancel,
            KeyCode::Enter | KeyCode::F(10) => Action::PatientFormSubmit,
            KeyCode::Tab => {
                self.next_field();
                Action::Render
            }
            KeyCode::BackTab => {
                self.prev_field();
                Action::Render
            }
            KeyCode::Up => {
                if matches!(self.current_field(), FormField::Gender) {
                    self.prev_gender();
                    Action::Render
                } else {
                    self.prev_field();
                    Action::Render
                }
            }
            KeyCode::Down => {
                if matches!(self.current_field(), FormField::Gender) {
                    self.next_gender();
                    Action::Render
                } else {
                    self.next_field();
                    Action::Render
                }
            }
            KeyCode::Char(c) => {
                self.handle_input(c);
                Action::Render
            }
            KeyCode::Backspace => {
                self.handle_backspace();
                Action::Render
            }
            _ => Action::None,
        }
    }

    async fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::PatientFormSubmit => self.submit_form().await,
            _ => Ok(None),
        }
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
            .constraints([Constraint::Min(0), Constraint::Length(3)])
            .split(inner_area);

        let available_height = chunks[0].height.saturating_sub(4);
        let max_visible_fields = (available_height / 3).min(8) as usize;

        let all_fields = FormField::all();
        let visible_fields: Vec<(usize, FormField)> = all_fields
            .iter()
            .enumerate()
            .skip(self.scroll_offset)
            .take(max_visible_fields)
            .map(|(idx, field)| (idx, *field))
            .collect();

        let form_chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints(
                visible_fields
                    .iter()
                    .map(|_| Constraint::Length(3))
                    .collect::<Vec<_>>(),
            )
            .split(chunks[0]);

        for (render_idx, (field_idx, field)) in visible_fields.iter().enumerate() {
            let is_current = *field_idx == self.current_field;
            let value = self.get_field_value(*field);

            let display_value = if is_current {
                format!("{} █", value)
            } else {
                value
            };

            let style = if is_current {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let paragraph = Paragraph::new(display_value)
                .style(style)
                .block(Block::default().borders(Borders::ALL).title(field.label()));

            frame.render_widget(paragraph, form_chunks[render_idx]);
        }

        let field_indicator = format!(
            "Field {}/{}",
            self.current_field + 1,
            FormField::all().len()
        );

        let help_text = if self.is_submitting {
            format!("Submitting... | {}", field_indicator)
        } else if !self.validation_errors.is_empty() {
            format!("Fix errors above | {}", field_indicator)
        } else {
            let help = KeybindRegistry::get_help_text(KeybindContext::PatientForm);
            format!("{} | {}", help, field_indicator)
        };

        let help_style = if !self.validation_errors.is_empty() {
            Style::default().fg(Color::Red)
        } else {
            Style::default().fg(Color::Green)
        };

        let help_widget = Paragraph::new(help_text)
            .style(help_style)
            .block(Block::default().borders(Borders::ALL).title(" Help "));

        frame.render_widget(help_widget, chunks[1]);

        if !self.validation_errors.is_empty() {
            let error_items: Vec<ListItem> = self
                .validation_errors
                .iter()
                .map(|e| ListItem::new(format!("• {}", e)))
                .collect();

            let error_list = List::new(error_items)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" Validation Errors ")
                        .border_style(Style::default().fg(Color::Red)),
                )
                .style(Style::default().fg(Color::Red));

            let error_width = modal_area.width.saturating_mul(2) / 3;
            let error_height =
                (self.validation_errors.len() as u16 + 2).min(modal_area.height.saturating_sub(4));
            let error_area = Rect {
                x: modal_area.x + modal_area.width.saturating_sub(error_width) / 2,
                y: modal_area.y + modal_area.height.saturating_sub(error_height) / 2,
                width: error_width,
                height: error_height,
            };

            frame.render_widget(error_list, error_area);
        }
    }
}

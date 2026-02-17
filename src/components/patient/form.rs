use async_trait::async_trait;
use chrono::NaiveDate;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph};
use ratatui::Frame;
use std::sync::Arc;
use uuid::Uuid;

use crate::components::{Action, Component};
use crate::domain::patient::{
    Address, Gender, NewPatientData, Patient, PatientService, UpdatePatientData,
};
use crate::error::Result;
use crate::ui::components::{InputWrapper, SelectWrapper};
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
    editing_patient_id: Option<Uuid>,
    current_field: usize,
    scroll_offset: usize,
    first_name: InputWrapper,
    last_name: InputWrapper,
    date_of_birth: InputWrapper,
    gender: SelectWrapper,
    medicare_number: InputWrapper,
    medicare_irn: InputWrapper,
    phone_mobile: InputWrapper,
    email: InputWrapper,
    validation_errors: Vec<String>,
    is_submitting: bool,
    form_area: Option<Rect>,
}

impl PatientFormComponent {
    pub fn new(patient_service: Arc<PatientService>) -> Self {
        Self {
            patient_service,
            editing_patient_id: None,
            current_field: 0,
            scroll_offset: 0,
            first_name: InputWrapper::new(),
            last_name: InputWrapper::new(),
            date_of_birth: InputWrapper::new(),
            gender: SelectWrapper::new().items(vec![
                "Male".to_string(),
                "Female".to_string(),
                "Other".to_string(),
                "Prefer not to say".to_string(),
            ]),
            medicare_number: InputWrapper::new(),
            medicare_irn: InputWrapper::new(),
            phone_mobile: InputWrapper::new(),
            email: InputWrapper::new(),
            validation_errors: Vec::new(),
            is_submitting: false,
            form_area: None,
        }
    }

    pub fn edit(patient_service: Arc<PatientService>, patient: Patient) -> Self {
        let gender_index = match patient.gender {
            Gender::Male => 0,
            Gender::Female => 1,
            Gender::Other => 2,
            Gender::PreferNotToSay => 3,
        };

        let mut gender = SelectWrapper::new().items(vec![
            "Male".to_string(),
            "Female".to_string(),
            "Other".to_string(),
            "Prefer not to say".to_string(),
        ]);
        gender.set_selected(gender_index);

        Self {
            patient_service,
            editing_patient_id: Some(patient.id),
            current_field: 0,
            scroll_offset: 0,
            first_name: InputWrapper::new().init_value(&patient.first_name),
            last_name: InputWrapper::new().init_value(&patient.last_name),
            date_of_birth: InputWrapper::new()
                .init_value(&patient.date_of_birth.format("%d/%m/%Y").to_string()),
            gender,
            medicare_number: InputWrapper::new()
                .init_value(&patient.medicare_number.unwrap_or_default()),
            medicare_irn: InputWrapper::new().init_value(
                &patient
                    .medicare_irn
                    .map(|i| i.to_string())
                    .unwrap_or_default(),
            ),
            phone_mobile: InputWrapper::new()
                .init_value(&patient.phone_mobile.unwrap_or_default()),
            email: InputWrapper::new().init_value(&patient.email.unwrap_or_default()),
            validation_errors: Vec::new(),
            is_submitting: false,
            form_area: None,
        }
    }

    pub fn is_edit_mode(&self) -> bool {
        self.editing_patient_id.is_some()
    }

    fn validate(&mut self) -> bool {
        self.validation_errors.clear();

        if self.first_name.value().trim().is_empty() {
            self.validation_errors
                .push("First name is required".to_string());
        }

        if self.last_name.value().trim().is_empty() {
            self.validation_errors
                .push("Last name is required".to_string());
        }

        let dob_str = self.date_of_birth.value();
        if dob_str.trim().is_empty() {
            self.validation_errors
                .push("Date of birth is required".to_string());
        } else if self.parse_date().is_none() {
            self.validation_errors
                .push("Date of birth must be in DD/MM/YYYY format".to_string());
        }

        let medicare = self.medicare_number.value();
        if !medicare.is_empty() && medicare.len() != 10 {
            self.validation_errors
                .push("Medicare number must be 10 digits".to_string());
        }

        let irn = self.medicare_irn.value();
        if !irn.is_empty() {
            if let Ok(irn_val) = irn.parse::<u8>() {
                if !(1..=9).contains(&irn_val) {
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
        let parts: Vec<&str> = self.date_of_birth.value().split('/').collect();
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

    fn next_field(&mut self) {
        let fields = FormField::all();
        self.current_field = (self.current_field + 1) % fields.len();
        self.update_focus();
    }

    fn prev_field(&mut self) {
        let fields = FormField::all();
        if self.current_field == 0 {
            self.current_field = fields.len() - 1;
        } else {
            self.current_field -= 1;
        }
        self.update_focus();
    }

    fn update_focus(&mut self) {
        self.adjust_scroll();
        self.first_name.set_focus(false);
        self.last_name.set_focus(false);
        self.date_of_birth.set_focus(false);
        self.gender.set_focus(false);
        self.medicare_number.set_focus(false);
        self.medicare_irn.set_focus(false);
        self.phone_mobile.set_focus(false);
        self.email.set_focus(false);

        match self.current_field() {
            FormField::FirstName => self.first_name.set_focus(true),
            FormField::LastName => self.last_name.set_focus(true),
            FormField::DateOfBirth => self.date_of_birth.set_focus(true),
            FormField::Gender => self.gender.set_focus(true),
            FormField::MedicareNumber => self.medicare_number.set_focus(true),
            FormField::MedicareIrn => self.medicare_irn.set_focus(true),
            FormField::PhoneMobile => self.phone_mobile.set_focus(true),
            FormField::Email => self.email.set_focus(true),
        }
    }

    fn adjust_scroll(&mut self) {
        const VISIBLE_FIELDS: usize = 6;

        if self.current_field < self.scroll_offset {
            self.scroll_offset = self.current_field;
        } else if self.current_field >= self.scroll_offset + VISIBLE_FIELDS {
            self.scroll_offset = self.current_field.saturating_sub(VISIBLE_FIELDS - 1);
        }
    }

    fn get_selected_gender(&self) -> Gender {
        match self.gender.selected_index() {
            Some(0) => Gender::Male,
            Some(1) => Gender::Female,
            Some(2) => Gender::Other,
            _ => Gender::PreferNotToSay,
        }
    }

    fn get_input_mut(&mut self, field: FormField) -> Option<&mut InputWrapper> {
        match field {
            FormField::FirstName => Some(&mut self.first_name),
            FormField::LastName => Some(&mut self.last_name),
            FormField::DateOfBirth => Some(&mut self.date_of_birth),
            FormField::MedicareNumber => Some(&mut self.medicare_number),
            FormField::MedicareIrn => Some(&mut self.medicare_irn),
            FormField::PhoneMobile => Some(&mut self.phone_mobile),
            FormField::Email => Some(&mut self.email),
            FormField::Gender => None,
        }
    }

    fn get_field_display_value(&self, field: FormField, is_current: bool) -> String {
        match field {
            FormField::FirstName => {
                let v = self.first_name.value();
                if is_current && !v.is_empty() {
                    format!("{} █", v)
                } else {
                    v.to_string()
                }
            }
            FormField::LastName => {
                let v = self.last_name.value();
                if is_current && !v.is_empty() {
                    format!("{} █", v)
                } else {
                    v.to_string()
                }
            }
            FormField::DateOfBirth => {
                let v = self.date_of_birth.value();
                if is_current && !v.is_empty() {
                    format!("{} █", v)
                } else {
                    v.to_string()
                }
            }
            FormField::Gender => self
                .gender
                .selected()
                .map(|s| s.to_string())
                .unwrap_or_else(|| "Select...".to_string()),
            FormField::MedicareNumber => {
                let v = self.medicare_number.value();
                if is_current && !v.is_empty() {
                    format!("{} █", v)
                } else {
                    v.to_string()
                }
            }
            FormField::MedicareIrn => {
                let v = self.medicare_irn.value();
                if is_current && !v.is_empty() {
                    format!("{} █", v)
                } else {
                    v.to_string()
                }
            }
            FormField::PhoneMobile => {
                let v = self.phone_mobile.value();
                if is_current && !v.is_empty() {
                    format!("{} █", v)
                } else {
                    v.to_string()
                }
            }
            FormField::Email => {
                let v = self.email.value();
                if is_current && !v.is_empty() {
                    format!("{} █", v)
                } else {
                    v.to_string()
                }
            }
        }
    }

    async fn submit_form(&mut self) -> Result<Option<Action>> {
        if !self.validate() {
            return Ok(Some(Action::Render));
        }

        self.is_submitting = true;

        let dob = self.parse_date().unwrap();
        let medicare_irn = if self.medicare_irn.value().is_empty() {
            None
        } else {
            self.medicare_irn.value().parse::<u8>().ok()
        };

        if let Some(patient_id) = self.editing_patient_id {
            let data = UpdatePatientData {
                ihi: None,
                medicare_number: if self.medicare_number.value().is_empty() {
                    None
                } else {
                    Some(self.medicare_number.value().to_string())
                },
                medicare_irn,
                medicare_expiry: None,
                title: None,
                first_name: Some(self.first_name.value().to_string()),
                middle_name: None,
                last_name: Some(self.last_name.value().to_string()),
                preferred_name: None,
                date_of_birth: Some(dob),
                gender: Some(self.get_selected_gender()),
                address: None,
                phone_home: None,
                phone_mobile: if self.phone_mobile.value().is_empty() {
                    None
                } else {
                    Some(self.phone_mobile.value().to_string())
                },
                email: if self.email.value().is_empty() {
                    None
                } else {
                    Some(self.email.value().to_string())
                },
                emergency_contact: None,
                concession_type: None,
                concession_number: None,
                preferred_language: None,
                interpreter_required: None,
                aboriginal_torres_strait_islander: None,
            };

            match self.patient_service.update_patient(patient_id, data).await {
                Ok(_patient) => {
                    self.is_submitting = false;
                    return Ok(Some(Action::PatientFormSubmit));
                }
                Err(e) => {
                    self.is_submitting = false;
                    self.validation_errors
                        .push(format!("Failed to update patient: {}", e));
                    return Ok(Some(Action::Render));
                }
            }
        }

        let data = NewPatientData {
            ihi: None,
            medicare_number: if self.medicare_number.value().is_empty() {
                None
            } else {
                Some(self.medicare_number.value().to_string())
            },
            medicare_irn,
            medicare_expiry: None,
            title: None,
            first_name: self.first_name.value().to_string(),
            middle_name: None,
            last_name: self.last_name.value().to_string(),
            preferred_name: None,
            date_of_birth: dob,
            gender: self.get_selected_gender(),
            address: Address::default(),
            phone_home: None,
            phone_mobile: if self.phone_mobile.value().is_empty() {
                None
            } else {
                Some(self.phone_mobile.value().to_string())
            },
            email: if self.email.value().is_empty() {
                None
            } else {
                Some(self.email.value().to_string())
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
        self.update_focus();
        Ok(())
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> Action {
        if self.is_submitting {
            return Action::None;
        }

        if (key.code == KeyCode::Char('s') && key.modifiers.contains(KeyModifiers::CONTROL))
            || key.code == KeyCode::F(10)
        {
            return Action::PatientFormSubmit;
        }

        match key.code {
            KeyCode::Esc => Action::PatientFormCancel,
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
                    self.gender.previous();
                }
                self.prev_field();
                Action::Render
            }
            KeyCode::Down => {
                if matches!(self.current_field(), FormField::Gender) {
                    self.gender.next();
                }
                self.next_field();
                Action::Render
            }
            KeyCode::Char(c) => {
                if let Some(input) = self.get_input_mut(self.current_field()) {
                    input.push_char(c);
                }
                Action::Render
            }
            KeyCode::Backspace => {
                if let Some(input) = self.get_input_mut(self.current_field()) {
                    input.pop_char();
                }
                Action::Render
            }
            _ => Action::None,
        }
    }

    fn handle_mouse_events(&mut self, mouse: MouseEvent) -> Action {
        use crate::ui::widgets::is_click;

        if !is_click(&mouse) {
            return Action::None;
        }

        let Some(area) = self.form_area else {
            return Action::None;
        };

        let col = mouse.column;
        let row = mouse.row;

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
        let modal_block = Block::default()
            .borders(Borders::ALL)
            .title(if self.is_edit_mode() {
                " Edit Patient "
            } else {
                " New Patient "
            })
            .border_style(Theme::default().normal);
        let inner_area = modal_block.inner(modal_area);

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

        for (render_idx, (field_idx, _)) in visible_fields.iter().enumerate() {
            if let Some(field_area) = form_chunks.get(render_idx) {
                if col >= field_area.x
                    && col < field_area.x + field_area.width
                    && row >= field_area.y
                    && row < field_area.y + field_area.height
                {
                    self.current_field = *field_idx;
                    self.update_focus();
                    return Action::Render;
                }
            }
        }

        Action::None
    }

    async fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::PatientFormSubmit => self.submit_form().await,
            _ => Ok(None),
        }
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        self.form_area = Some(area);

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

        let title = if self.is_edit_mode() {
            " Edit Patient "
        } else {
            " New Patient "
        };
        let modal_block = Block::default()
            .borders(Borders::ALL)
            .title(title)
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
            let value = self.get_field_display_value(*field, is_current);

            let style = if is_current {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let paragraph = Paragraph::new(value)
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
            field_indicator
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

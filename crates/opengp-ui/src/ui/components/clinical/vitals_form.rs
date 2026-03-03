//! Vital Signs Form Component
//!
//! Form for recording patient vital signs measurements.

use std::collections::HashMap;

use crossterm::event::{KeyEvent, KeyModifiers};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::{Block, Borders, Widget};

use opengp_domain::domain::clinical::VitalSigns;
use crate::ui::input::to_ratatui_key;
use crate::ui::layout::LABEL_WIDTH;
use crate::ui::theme::Theme;
use crate::ui::widgets::{HeightMode, ScrollableFormState, TextareaState, TextareaWidget};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VitalSignsFormField {
    SystolicBp,
    DiastolicBp,
    HeartRate,
    RespiratoryRate,
    Temperature,
    O2Saturation,
    Height,
    Weight,
    Notes,
}

impl VitalSignsFormField {
    pub fn all() -> Vec<VitalSignsFormField> {
        vec![
            VitalSignsFormField::SystolicBp,
            VitalSignsFormField::DiastolicBp,
            VitalSignsFormField::HeartRate,
            VitalSignsFormField::RespiratoryRate,
            VitalSignsFormField::Temperature,
            VitalSignsFormField::O2Saturation,
            VitalSignsFormField::Height,
            VitalSignsFormField::Weight,
            VitalSignsFormField::Notes,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            VitalSignsFormField::SystolicBp => "Systolic BP (mmHg)",
            VitalSignsFormField::DiastolicBp => "Diastolic BP (mmHg)",
            VitalSignsFormField::HeartRate => "Heart Rate (bpm)",
            VitalSignsFormField::RespiratoryRate => "Respiratory Rate",
            VitalSignsFormField::Temperature => "Temperature (C)",
            VitalSignsFormField::O2Saturation => "O2 Saturation (%)",
            VitalSignsFormField::Height => "Height (cm)",
            VitalSignsFormField::Weight => "Weight (kg)",
            VitalSignsFormField::Notes => "Notes",
        }
    }

    pub fn is_required(&self) -> bool {
        // No individual field is required - at least one measurement must be filled
        false
    }

    pub fn is_textarea(&self) -> bool {
        matches!(self, VitalSignsFormField::Notes)
    }
}

#[derive(Debug, Clone)]
pub enum VitalSignsFormAction {
    FocusChanged,
    ValueChanged,
    Submit,
    Cancel,
}

pub struct VitalSignsForm {
    pub systolic_bp: Option<u16>,
    pub diastolic_bp: Option<u16>,
    pub heart_rate: Option<u16>,
    pub respiratory_rate: Option<u16>,
    pub temperature: Option<f32>,
    pub o2_saturation: Option<u8>,
    pub height_cm: Option<u16>,
    pub weight_kg: Option<f32>,
    pub notes: TextareaState,
    pub focused_field: VitalSignsFormField,
    pub calculated_bmi: Option<f32>,
    systolic_bp_field: TextareaState,
    diastolic_bp_field: TextareaState,
    heart_rate_field: TextareaState,
    respiratory_rate_field: TextareaState,
    temperature_field: TextareaState,
    o2_saturation_field: TextareaState,
    height_cm_field: TextareaState,
    weight_kg_field: TextareaState,
    errors: HashMap<VitalSignsFormField, String>,
    theme: Theme,
    scroll: ScrollableFormState,
}

impl Clone for VitalSignsForm {
    fn clone(&self) -> Self {
        Self {
            systolic_bp: self.systolic_bp,
            diastolic_bp: self.diastolic_bp,
            heart_rate: self.heart_rate,
            respiratory_rate: self.respiratory_rate,
            temperature: self.temperature,
            o2_saturation: self.o2_saturation,
            height_cm: self.height_cm,
            weight_kg: self.weight_kg,
            notes: self.notes.clone(),
            focused_field: self.focused_field,
            calculated_bmi: self.calculated_bmi,
            systolic_bp_field: self.systolic_bp_field.clone(),
            diastolic_bp_field: self.diastolic_bp_field.clone(),
            heart_rate_field: self.heart_rate_field.clone(),
            respiratory_rate_field: self.respiratory_rate_field.clone(),
            temperature_field: self.temperature_field.clone(),
            o2_saturation_field: self.o2_saturation_field.clone(),
            height_cm_field: self.height_cm_field.clone(),
            weight_kg_field: self.weight_kg_field.clone(),
            errors: self.errors.clone(),
            theme: self.theme.clone(),
            scroll: self.scroll.clone(),
        }
    }
}

impl VitalSignsForm {
    pub fn new(theme: Theme) -> Self {
        Self {
            systolic_bp: None,
            diastolic_bp: None,
            heart_rate: None,
            respiratory_rate: None,
            temperature: None,
            o2_saturation: None,
            height_cm: None,
            weight_kg: None,
            notes: TextareaState::new("Notes").with_height_mode(HeightMode::FixedLines(4)),
            focused_field: VitalSignsFormField::SystolicBp,
            calculated_bmi: None,
            systolic_bp_field: TextareaState::new("Systolic BP (mmHg)")
                .with_height_mode(HeightMode::SingleLine)
                .max_length(3),
            diastolic_bp_field: TextareaState::new("Diastolic BP (mmHg)")
                .with_height_mode(HeightMode::SingleLine)
                .max_length(3),
            heart_rate_field: TextareaState::new("Heart Rate (bpm)")
                .with_height_mode(HeightMode::SingleLine)
                .max_length(3),
            respiratory_rate_field: TextareaState::new("Respiratory Rate")
                .with_height_mode(HeightMode::SingleLine)
                .max_length(2),
            temperature_field: TextareaState::new("Temperature (C)")
                .with_height_mode(HeightMode::SingleLine)
                .max_length(5),
            o2_saturation_field: TextareaState::new("O2 Saturation (%)")
                .with_height_mode(HeightMode::SingleLine)
                .max_length(3),
            height_cm_field: TextareaState::new("Height (cm)")
                .with_height_mode(HeightMode::SingleLine)
                .max_length(3),
            weight_kg_field: TextareaState::new("Weight (kg)")
                .with_height_mode(HeightMode::SingleLine)
                .max_length(6),
            errors: HashMap::new(),
            theme,
            scroll: ScrollableFormState::new(),
        }
    }

    pub fn focused_field(&self) -> VitalSignsFormField {
        self.focused_field
    }

    pub fn next_field(&mut self) {
        let fields = VitalSignsFormField::all();
        if let Some(current_idx) = fields.iter().position(|f| *f == self.focused_field) {
            let next_idx = (current_idx + 1) % fields.len();
            self.focused_field = fields[next_idx];
        }
    }

    pub fn prev_field(&mut self) {
        let fields = VitalSignsFormField::all();
        if let Some(current_idx) = fields.iter().position(|f| *f == self.focused_field) {
            let prev_idx = if current_idx == 0 {
                fields.len() - 1
            } else {
                current_idx - 1
            };
            self.focused_field = fields[prev_idx];
        }
    }

    pub fn get_value(&self, field: VitalSignsFormField) -> String {
        match field {
            VitalSignsFormField::SystolicBp => self.systolic_bp_field.value(),
            VitalSignsFormField::DiastolicBp => self.diastolic_bp_field.value(),
            VitalSignsFormField::HeartRate => self.heart_rate_field.value(),
            VitalSignsFormField::RespiratoryRate => self.respiratory_rate_field.value(),
            VitalSignsFormField::Temperature => self.temperature_field.value(),
            VitalSignsFormField::O2Saturation => self.o2_saturation_field.value(),
            VitalSignsFormField::Height => self.height_cm_field.value(),
            VitalSignsFormField::Weight => self.weight_kg_field.value(),
            VitalSignsFormField::Notes => self.notes.value(),
        }
    }

    pub fn set_value(&mut self, field: VitalSignsFormField, value: String) {
        match field {
            VitalSignsFormField::SystolicBp => {
                self.systolic_bp_field = TextareaState::new("Systolic BP (mmHg)")
                    .with_height_mode(HeightMode::SingleLine)
                    .max_length(3)
                    .with_value(value.clone());
                self.systolic_bp = value.parse().ok();
            }
            VitalSignsFormField::DiastolicBp => {
                self.diastolic_bp_field = TextareaState::new("Diastolic BP (mmHg)")
                    .with_height_mode(HeightMode::SingleLine)
                    .max_length(3)
                    .with_value(value.clone());
                self.diastolic_bp = value.parse().ok();
            }
            VitalSignsFormField::HeartRate => {
                self.heart_rate_field = TextareaState::new("Heart Rate (bpm)")
                    .with_height_mode(HeightMode::SingleLine)
                    .max_length(3)
                    .with_value(value.clone());
                self.heart_rate = value.parse().ok();
            }
            VitalSignsFormField::RespiratoryRate => {
                self.respiratory_rate_field = TextareaState::new("Respiratory Rate")
                    .with_height_mode(HeightMode::SingleLine)
                    .max_length(2)
                    .with_value(value.clone());
                self.respiratory_rate = value.parse().ok();
            }
            VitalSignsFormField::Temperature => {
                self.temperature_field = TextareaState::new("Temperature (C)")
                    .with_height_mode(HeightMode::SingleLine)
                    .max_length(5)
                    .with_value(value.clone());
                self.temperature = value.parse().ok();
            }
            VitalSignsFormField::O2Saturation => {
                self.o2_saturation_field = TextareaState::new("O2 Saturation (%)")
                    .with_height_mode(HeightMode::SingleLine)
                    .max_length(3)
                    .with_value(value.clone());
                self.o2_saturation = value.parse().ok();
            }
            VitalSignsFormField::Height => {
                self.height_cm_field = TextareaState::new("Height (cm)")
                    .with_height_mode(HeightMode::SingleLine)
                    .max_length(3)
                    .with_value(value.clone());
                self.height_cm = value.parse().ok();
            }
            VitalSignsFormField::Weight => {
                self.weight_kg_field = TextareaState::new("Weight (kg)")
                    .with_height_mode(HeightMode::SingleLine)
                    .max_length(6)
                    .with_value(value.clone());
                self.weight_kg = value.parse().ok();
            }
            VitalSignsFormField::Notes => {
                self.notes = TextareaState::new("Notes")
                    .with_height_mode(HeightMode::FixedLines(4))
                    .with_value(value);
            }
        }
        self.calculate_bmi();
        self.validate_field(&field);
    }

    pub fn calculate_bmi(&mut self) {
        if let (Some(height), Some(weight)) = (self.height_cm, self.weight_kg) {
            if height > 0 {
                let height_m = height as f32 / 100.0;
                self.calculated_bmi = Some(weight / (height_m * height_m));
            }
        } else {
            self.calculated_bmi = None;
        }
    }

    fn validate_field(&mut self, field: &VitalSignsFormField) {
        self.errors.remove(field);

        let value = self.get_value(*field);

        match field {
            VitalSignsFormField::SystolicBp => {
                if !value.is_empty() {
                    match value.parse::<u16>() {
                        Ok(v) if v < 50 || v > 300 => {
                            self.errors
                                .insert(*field, "Systolic BP must be 50-300 mmHg".to_string());
                        }
                        Err(_) => {
                            self.errors
                                .insert(*field, "Must be a whole number".to_string());
                        }
                        _ => {}
                    }
                }
            }
            VitalSignsFormField::DiastolicBp => {
                if !value.is_empty() {
                    match value.parse::<u16>() {
                        Ok(v) if v < 20 || v > 200 => {
                            self.errors
                                .insert(*field, "Diastolic BP must be 20-200 mmHg".to_string());
                        }
                        Err(_) => {
                            self.errors
                                .insert(*field, "Must be a whole number".to_string());
                        }
                        _ => {}
                    }
                }
            }
            VitalSignsFormField::HeartRate => {
                if !value.is_empty() {
                    match value.parse::<u16>() {
                        Ok(v) if v < 20 || v > 300 => {
                            self.errors
                                .insert(*field, "Heart rate must be 20-300 bpm".to_string());
                        }
                        Err(_) => {
                            self.errors
                                .insert(*field, "Must be a whole number".to_string());
                        }
                        _ => {}
                    }
                }
            }
            VitalSignsFormField::RespiratoryRate => {
                if !value.is_empty() {
                    match value.parse::<u16>() {
                        Ok(v) if v < 4 || v > 60 => {
                            self.errors
                                .insert(*field, "Respiratory rate must be 4-60 /min".to_string());
                        }
                        Err(_) => {
                            self.errors
                                .insert(*field, "Must be a whole number".to_string());
                        }
                        _ => {}
                    }
                }
            }
            VitalSignsFormField::Temperature => {
                if !value.is_empty() {
                    match value.parse::<f32>() {
                        Ok(v) if v < 30.0 || v > 45.0 => {
                            self.errors
                                .insert(*field, "Temperature must be 30-45 C".to_string());
                        }
                        Err(_) => {
                            self.errors
                                .insert(*field, "Must be a number (e.g. 37.2)".to_string());
                        }
                        _ => {}
                    }
                }
            }
            VitalSignsFormField::O2Saturation => {
                if !value.is_empty() {
                    match value.parse::<u8>() {
                        Ok(v) if v < 50 || v > 100 => {
                            self.errors
                                .insert(*field, "O2 saturation must be 50-100%".to_string());
                        }
                        Err(_) => {
                            self.errors
                                .insert(*field, "Must be a whole number".to_string());
                        }
                        _ => {}
                    }
                }
            }
            VitalSignsFormField::Height => {
                if !value.is_empty() {
                    match value.parse::<u16>() {
                        Ok(v) if v < 30 || v > 300 => {
                            self.errors
                                .insert(*field, "Height must be 30-300 cm".to_string());
                        }
                        Err(_) => {
                            self.errors
                                .insert(*field, "Must be a whole number".to_string());
                        }
                        _ => {}
                    }
                }
            }
            VitalSignsFormField::Weight => {
                if !value.is_empty() {
                    match value.parse::<f32>() {
                        Ok(v) if v < 0.5 || v > 700.0 => {
                            self.errors
                                .insert(*field, "Weight must be 0.5-700 kg".to_string());
                        }
                        Err(_) => {
                            self.errors
                                .insert(*field, "Must be a number (e.g. 72.5)".to_string());
                        }
                        _ => {}
                    }
                }
            }
            VitalSignsFormField::Notes => {}
        }
    }

    /// Validates the form. Returns true if at least one measurement is filled and no field errors.
    pub fn validate(&mut self) -> bool {
        self.errors.clear();

        for field in VitalSignsFormField::all() {
            self.validate_field(&field);
        }

        if self.errors.is_empty() && !self.has_any_measurement() {
            self.errors.insert(
                VitalSignsFormField::SystolicBp,
                "At least one measurement is required".to_string(),
            );
        }

        self.errors.is_empty()
    }

    /// Returns true if at least one numeric measurement field has a value.
    pub fn has_any_measurement(&self) -> bool {
        self.systolic_bp.is_some()
            || self.diastolic_bp.is_some()
            || self.heart_rate.is_some()
            || self.respiratory_rate.is_some()
            || self.temperature.is_some()
            || self.o2_saturation.is_some()
            || self.height_cm.is_some()
            || self.weight_kg.is_some()
    }

    pub fn error(&self, field: VitalSignsFormField) -> Option<&String> {
        self.errors.get(&field)
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Option<VitalSignsFormAction> {
        use crossterm::event::{KeyCode, KeyEventKind};

        if key.kind != KeyEventKind::Press {
            return None;
        }

        // Ctrl+Enter submits the form from any field.
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Enter {
            self.validate();
            return Some(VitalSignsFormAction::Submit);
        }

        let field = self.focused_field;
        let ratatui_key = to_ratatui_key(key);
        let consumed = match field {
            VitalSignsFormField::SystolicBp => self.systolic_bp_field.handle_key(ratatui_key),
            VitalSignsFormField::DiastolicBp => self.diastolic_bp_field.handle_key(ratatui_key),
            VitalSignsFormField::HeartRate => self.heart_rate_field.handle_key(ratatui_key),
            VitalSignsFormField::RespiratoryRate => {
                self.respiratory_rate_field.handle_key(ratatui_key)
            }
            VitalSignsFormField::Temperature => self.temperature_field.handle_key(ratatui_key),
            VitalSignsFormField::O2Saturation => self.o2_saturation_field.handle_key(ratatui_key),
            VitalSignsFormField::Height => self.height_cm_field.handle_key(ratatui_key),
            VitalSignsFormField::Weight => self.weight_kg_field.handle_key(ratatui_key),
            VitalSignsFormField::Notes => self.notes.handle_key(ratatui_key),
        };

        if consumed {
            let value = self.get_value(field);
            match field {
                VitalSignsFormField::SystolicBp => self.systolic_bp = value.parse().ok(),
                VitalSignsFormField::DiastolicBp => self.diastolic_bp = value.parse().ok(),
                VitalSignsFormField::HeartRate => self.heart_rate = value.parse().ok(),
                VitalSignsFormField::RespiratoryRate => self.respiratory_rate = value.parse().ok(),
                VitalSignsFormField::Temperature => self.temperature = value.parse().ok(),
                VitalSignsFormField::O2Saturation => self.o2_saturation = value.parse().ok(),
                VitalSignsFormField::Height => self.height_cm = value.parse().ok(),
                VitalSignsFormField::Weight => self.weight_kg = value.parse().ok(),
                VitalSignsFormField::Notes => {}
            }
            self.calculate_bmi();
            self.validate_field(&field);
            return Some(VitalSignsFormAction::ValueChanged);
        }

        // For textarea fields (all 9 fields now use TextareaState), delegate to TextareaState.
        let field = self.focused_field;
        if field.is_textarea() || !field.is_textarea() {
            // All fields are now TextareaState, delegate key handling
            let ratatui_key = to_ratatui_key(key);
            let consumed = match field {
                VitalSignsFormField::SystolicBp => self.systolic_bp_field.handle_key(ratatui_key),
                VitalSignsFormField::DiastolicBp => self.diastolic_bp_field.handle_key(ratatui_key),
                VitalSignsFormField::HeartRate => self.heart_rate_field.handle_key(ratatui_key),
                VitalSignsFormField::RespiratoryRate => {
                    self.respiratory_rate_field.handle_key(ratatui_key)
                }
                VitalSignsFormField::Temperature => self.temperature_field.handle_key(ratatui_key),
                VitalSignsFormField::O2Saturation => {
                    self.o2_saturation_field.handle_key(ratatui_key)
                }
                VitalSignsFormField::Height => self.height_cm_field.handle_key(ratatui_key),
                VitalSignsFormField::Weight => self.weight_kg_field.handle_key(ratatui_key),
                VitalSignsFormField::Notes => self.notes.handle_key(ratatui_key),
            };

            if consumed {
                // Update the parsed value from textarea
                let value = self.get_value(field);
                match field {
                    VitalSignsFormField::SystolicBp => self.systolic_bp = value.parse().ok(),
                    VitalSignsFormField::DiastolicBp => self.diastolic_bp = value.parse().ok(),
                    VitalSignsFormField::HeartRate => self.heart_rate = value.parse().ok(),
                    VitalSignsFormField::RespiratoryRate => {
                        self.respiratory_rate = value.parse().ok()
                    }
                    VitalSignsFormField::Temperature => self.temperature = value.parse().ok(),
                    VitalSignsFormField::O2Saturation => self.o2_saturation = value.parse().ok(),
                    VitalSignsFormField::Height => self.height_cm = value.parse().ok(),
                    VitalSignsFormField::Weight => self.weight_kg = value.parse().ok(),
                    VitalSignsFormField::Notes => {}
                }
                self.calculate_bmi();
                self.validate_field(&field);
                return Some(VitalSignsFormAction::ValueChanged);
            }
        }

        match key.code {
            KeyCode::Tab => {
                if key.modifiers.contains(KeyModifiers::SHIFT) {
                    self.prev_field();
                } else {
                    self.next_field();
                }
                Some(VitalSignsFormAction::FocusChanged)
            }
            KeyCode::BackTab => {
                self.prev_field();
                Some(VitalSignsFormAction::FocusChanged)
            }
            KeyCode::Up => {
                self.prev_field();
                Some(VitalSignsFormAction::FocusChanged)
            }
            KeyCode::Down => {
                self.next_field();
                Some(VitalSignsFormAction::FocusChanged)
            }
            KeyCode::PageUp => {
                self.scroll.scroll_up();
                Some(VitalSignsFormAction::FocusChanged)
            }
            KeyCode::PageDown => {
                self.scroll.scroll_down();
                Some(VitalSignsFormAction::FocusChanged)
            }
            KeyCode::Enter => {
                self.validate();
                Some(VitalSignsFormAction::Submit)
            }
            KeyCode::Esc => Some(VitalSignsFormAction::Cancel),
            _ => None,
        }
    }

    pub fn to_vital_signs(&self, patient_id: uuid::Uuid, created_by: uuid::Uuid) -> VitalSigns {
        VitalSigns {
            id: uuid::Uuid::new_v4(),
            patient_id,
            consultation_id: None,
            measured_at: chrono::Utc::now(),
            systolic_bp: self.systolic_bp,
            diastolic_bp: self.diastolic_bp,
            heart_rate: self.heart_rate,
            respiratory_rate: self.respiratory_rate,
            temperature: self.temperature,
            oxygen_saturation: self.o2_saturation,
            height_cm: self.height_cm,
            weight_kg: self.weight_kg,
            bmi: self.calculated_bmi,
            notes: Some(self.notes.value()).filter(|s: &String| !s.is_empty()),
            created_at: chrono::Utc::now(),
            created_by,
        }
    }
}

impl Widget for VitalSignsForm {
    fn render(mut self, area: Rect, buf: &mut Buffer) {
        if area.is_empty() {
            return;
        }

        let block = Block::default()
            .title(" Vital Signs ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.colors.border));

        block.clone().render(area, buf);

        let inner = block.inner(area);
        if inner.is_empty() {
            return;
        }

        let label_width = LABEL_WIDTH;
        let field_start = inner.x + label_width + 2;

        let fields = VitalSignsFormField::all();

        let mut y = inner.y + 1;
        let max_y = inner.y + inner.height - 2;

        for field in fields {
            if y > max_y {
                break;
            }

            let is_focused = field == self.focused_field;
            let field_height = match field {
                VitalSignsFormField::Notes => self.notes.height(),
                _ => {
                    let textarea_state = match field {
                        VitalSignsFormField::SystolicBp => &self.systolic_bp_field,
                        VitalSignsFormField::DiastolicBp => &self.diastolic_bp_field,
                        VitalSignsFormField::HeartRate => &self.heart_rate_field,
                        VitalSignsFormField::RespiratoryRate => &self.respiratory_rate_field,
                        VitalSignsFormField::Temperature => &self.temperature_field,
                        VitalSignsFormField::O2Saturation => &self.o2_saturation_field,
                        VitalSignsFormField::Height => &self.height_cm_field,
                        VitalSignsFormField::Weight => &self.weight_kg_field,
                        VitalSignsFormField::Notes => unreachable!(),
                    };
                    textarea_state.height()
                }
            };

            let field_area = Rect::new(inner.x + 1, y, inner.width - 2, field_height);

            match field {
                VitalSignsFormField::Notes => {
                    TextareaWidget::new(&self.notes, self.theme.clone())
                        .focused(is_focused)
                        .render(field_area, buf);
                }
                VitalSignsFormField::SystolicBp => {
                    let mut state = self.systolic_bp_field.clone();
                    if let Some(err) = self.error(field) {
                        state.set_error(Some(err.clone()));
                    }
                    TextareaWidget::new(&state, self.theme.clone())
                        .focused(is_focused)
                        .render(field_area, buf);
                }
                VitalSignsFormField::DiastolicBp => {
                    let mut state = self.diastolic_bp_field.clone();
                    if let Some(err) = self.error(field) {
                        state.set_error(Some(err.clone()));
                    }
                    TextareaWidget::new(&state, self.theme.clone())
                        .focused(is_focused)
                        .render(field_area, buf);
                }
                VitalSignsFormField::HeartRate => {
                    let mut state = self.heart_rate_field.clone();
                    if let Some(err) = self.error(field) {
                        state.set_error(Some(err.clone()));
                    }
                    TextareaWidget::new(&state, self.theme.clone())
                        .focused(is_focused)
                        .render(field_area, buf);
                }
                VitalSignsFormField::RespiratoryRate => {
                    let mut state = self.respiratory_rate_field.clone();
                    if let Some(err) = self.error(field) {
                        state.set_error(Some(err.clone()));
                    }
                    TextareaWidget::new(&state, self.theme.clone())
                        .focused(is_focused)
                        .render(field_area, buf);
                }
                VitalSignsFormField::Temperature => {
                    let mut state = self.temperature_field.clone();
                    if let Some(err) = self.error(field) {
                        state.set_error(Some(err.clone()));
                    }
                    TextareaWidget::new(&state, self.theme.clone())
                        .focused(is_focused)
                        .render(field_area, buf);
                }
                VitalSignsFormField::O2Saturation => {
                    let mut state = self.o2_saturation_field.clone();
                    if let Some(err) = self.error(field) {
                        state.set_error(Some(err.clone()));
                    }
                    TextareaWidget::new(&state, self.theme.clone())
                        .focused(is_focused)
                        .render(field_area, buf);
                }
                VitalSignsFormField::Height => {
                    let mut state = self.height_cm_field.clone();
                    if let Some(err) = self.error(field) {
                        state.set_error(Some(err.clone()));
                    }
                    TextareaWidget::new(&state, self.theme.clone())
                        .focused(is_focused)
                        .render(field_area, buf);
                }
                VitalSignsFormField::Weight => {
                    let mut state = self.weight_kg_field.clone();
                    if let Some(err) = self.error(field) {
                        state.set_error(Some(err.clone()));
                    }
                    TextareaWidget::new(&state, self.theme.clone())
                        .focused(is_focused)
                        .render(field_area, buf);
                }
            }

            y += field_height;
        }

        if let Some(bmi) = self.calculated_bmi {
            if y <= max_y {
                let bmi_label_style = Style::default().fg(self.theme.colors.info);
                buf.set_string(inner.x + 1, y, "Calculated BMI:", bmi_label_style);
                buf.set_string(
                    field_start,
                    y,
                    format!("{:.1}", bmi),
                    Style::default().fg(self.theme.colors.success),
                );
            }
        }

        self.scroll.render_scrollbar(inner, buf);

        let help_y = inner.y + inner.height - 1;
        buf.set_string(
            inner.x + 1,
            help_y,
            "Tab: Next | Ctrl+Enter: Submit | Esc: Cancel",
            Style::default().fg(self.theme.colors.disabled),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vitals_form_creation() {
        let theme = Theme::dark();
        let form = VitalSignsForm::new(theme);

        assert_eq!(form.focused_field(), VitalSignsFormField::SystolicBp);
        assert!(!form.has_errors());
        assert!(!form.has_any_measurement());
    }

    #[test]
    fn test_vitals_form_field_navigation() {
        let theme = Theme::dark();
        let mut form = VitalSignsForm::new(theme);

        assert_eq!(form.focused_field(), VitalSignsFormField::SystolicBp);
        form.next_field();
        assert_eq!(form.focused_field(), VitalSignsFormField::DiastolicBp);
        form.next_field();
        assert_eq!(form.focused_field(), VitalSignsFormField::HeartRate);
        form.prev_field();
        assert_eq!(form.focused_field(), VitalSignsFormField::DiastolicBp);
    }

    #[test]
    fn test_vitals_form_field_navigation_wraps() {
        let theme = Theme::dark();
        let mut form = VitalSignsForm::new(theme);

        form.prev_field();
        assert_eq!(form.focused_field(), VitalSignsFormField::Notes);

        form.focused_field = VitalSignsFormField::Notes;
        form.next_field();
        assert_eq!(form.focused_field(), VitalSignsFormField::SystolicBp);
    }

    #[test]
    fn test_vitals_form_validation_requires_at_least_one_measurement() {
        let theme = Theme::dark();
        let mut form = VitalSignsForm::new(theme);

        let valid = form.validate();
        assert!(!valid);
        assert!(form.error(VitalSignsFormField::SystolicBp).is_some());
    }

    #[test]
    fn test_vitals_form_validation_passes_with_one_measurement() {
        let theme = Theme::dark();
        let mut form = VitalSignsForm::new(theme);

        form.set_value(VitalSignsFormField::HeartRate, "72".to_string());
        let valid = form.validate();
        assert!(valid);
        assert!(!form.has_errors());
    }

    #[test]
    fn test_vitals_form_numeric_field_validation() {
        let theme = Theme::dark();
        let mut form = VitalSignsForm::new(theme);

        form.set_value(VitalSignsFormField::SystolicBp, "999".to_string());
        assert!(form.error(VitalSignsFormField::SystolicBp).is_some());

        form.set_value(VitalSignsFormField::SystolicBp, "120".to_string());
        assert!(form.error(VitalSignsFormField::SystolicBp).is_none());
    }

    #[test]
    fn test_vitals_form_temperature_validation() {
        let theme = Theme::dark();
        let mut form = VitalSignsForm::new(theme);

        form.set_value(VitalSignsFormField::Temperature, "99.0".to_string());
        assert!(form.error(VitalSignsFormField::Temperature).is_some());

        form.set_value(VitalSignsFormField::Temperature, "37.2".to_string());
        assert!(form.error(VitalSignsFormField::Temperature).is_none());
    }

    #[test]
    fn test_vitals_form_bmi_calculation() {
        let theme = Theme::dark();
        let mut form = VitalSignsForm::new(theme);

        form.set_value(VitalSignsFormField::Height, "170".to_string());
        form.set_value(VitalSignsFormField::Weight, "70.0".to_string());

        assert!(form.calculated_bmi.is_some());
        let bmi = form.calculated_bmi.unwrap();
        assert!((bmi - 24.22).abs() < 0.1);
    }

    #[test]
    fn test_vitals_form_all_fields_ordered() {
        let fields = VitalSignsFormField::all();
        assert_eq!(fields[0], VitalSignsFormField::SystolicBp);
        assert_eq!(fields[1], VitalSignsFormField::DiastolicBp);
        assert_eq!(fields[2], VitalSignsFormField::HeartRate);
        assert_eq!(fields[3], VitalSignsFormField::RespiratoryRate);
        assert_eq!(fields[4], VitalSignsFormField::Temperature);
        assert_eq!(fields[5], VitalSignsFormField::O2Saturation);
        assert_eq!(fields[6], VitalSignsFormField::Height);
        assert_eq!(fields[7], VitalSignsFormField::Weight);
        assert_eq!(fields[8], VitalSignsFormField::Notes);
    }

    #[test]
    fn test_vitals_form_numeric_field_accepts_input() {
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

        let theme = Theme::dark();
        let mut form = VitalSignsForm::new(theme);

        let key = KeyEvent::new(KeyCode::Char('1'), KeyModifiers::NONE);
        let action = form.handle_key(key);
        assert!(action.is_some());
        assert_eq!(form.get_value(VitalSignsFormField::SystolicBp), "1");

        let key = KeyEvent::new(KeyCode::Char('2'), KeyModifiers::NONE);
        let action = form.handle_key(key);
        assert!(action.is_some());
        assert_eq!(form.get_value(VitalSignsFormField::SystolicBp), "12");
    }

    #[test]
    fn test_vitals_form_decimal_field_allows_dot() {
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

        let theme = Theme::dark();
        let mut form = VitalSignsForm::new(theme);

        form.focused_field = VitalSignsFormField::Temperature;

        let key = KeyEvent::new(KeyCode::Char('3'), KeyModifiers::NONE);
        form.handle_key(key);
        let key = KeyEvent::new(KeyCode::Char('7'), KeyModifiers::NONE);
        form.handle_key(key);
        let key = KeyEvent::new(KeyCode::Char('.'), KeyModifiers::NONE);
        form.handle_key(key);
        let key = KeyEvent::new(KeyCode::Char('5'), KeyModifiers::NONE);
        form.handle_key(key);

        assert_eq!(form.get_value(VitalSignsFormField::Temperature), "37.5");
    }
}

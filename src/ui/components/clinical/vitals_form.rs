//! Vital Signs Form Component
//!
//! Form for recording patient vital signs measurements.

use std::collections::HashMap;

use crossterm::event::{KeyEvent, KeyModifiers};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Borders, Widget};

use crate::domain::clinical::VitalSigns;
use crate::ui::layout::LABEL_WIDTH;
use crate::ui::theme::Theme;

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

    /// Returns true if this field accepts only integer numeric input
    fn is_integer_field(&self) -> bool {
        matches!(
            self,
            VitalSignsFormField::SystolicBp
                | VitalSignsFormField::DiastolicBp
                | VitalSignsFormField::HeartRate
                | VitalSignsFormField::RespiratoryRate
                | VitalSignsFormField::O2Saturation
                | VitalSignsFormField::Height
        )
    }

    /// Returns true if this field accepts decimal numeric input
    fn is_decimal_field(&self) -> bool {
        matches!(
            self,
            VitalSignsFormField::Temperature | VitalSignsFormField::Weight
        )
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
    pub notes: String,
    pub focused_field: VitalSignsFormField,
    pub calculated_bmi: Option<f32>,
    // Raw string buffers for numeric fields during editing
    systolic_bp_buf: String,
    diastolic_bp_buf: String,
    heart_rate_buf: String,
    respiratory_rate_buf: String,
    temperature_buf: String,
    o2_saturation_buf: String,
    height_cm_buf: String,
    weight_kg_buf: String,
    errors: HashMap<VitalSignsFormField, String>,
    theme: Theme,
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
            systolic_bp_buf: self.systolic_bp_buf.clone(),
            diastolic_bp_buf: self.diastolic_bp_buf.clone(),
            heart_rate_buf: self.heart_rate_buf.clone(),
            respiratory_rate_buf: self.respiratory_rate_buf.clone(),
            temperature_buf: self.temperature_buf.clone(),
            o2_saturation_buf: self.o2_saturation_buf.clone(),
            height_cm_buf: self.height_cm_buf.clone(),
            weight_kg_buf: self.weight_kg_buf.clone(),
            errors: self.errors.clone(),
            theme: self.theme.clone(),
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
            notes: String::new(),
            focused_field: VitalSignsFormField::SystolicBp,
            calculated_bmi: None,
            systolic_bp_buf: String::new(),
            diastolic_bp_buf: String::new(),
            heart_rate_buf: String::new(),
            respiratory_rate_buf: String::new(),
            temperature_buf: String::new(),
            o2_saturation_buf: String::new(),
            height_cm_buf: String::new(),
            weight_kg_buf: String::new(),
            errors: HashMap::new(),
            theme,
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
            VitalSignsFormField::SystolicBp => self.systolic_bp_buf.clone(),
            VitalSignsFormField::DiastolicBp => self.diastolic_bp_buf.clone(),
            VitalSignsFormField::HeartRate => self.heart_rate_buf.clone(),
            VitalSignsFormField::RespiratoryRate => self.respiratory_rate_buf.clone(),
            VitalSignsFormField::Temperature => self.temperature_buf.clone(),
            VitalSignsFormField::O2Saturation => self.o2_saturation_buf.clone(),
            VitalSignsFormField::Height => self.height_cm_buf.clone(),
            VitalSignsFormField::Weight => self.weight_kg_buf.clone(),
            VitalSignsFormField::Notes => self.notes.clone(),
        }
    }

    pub fn set_value(&mut self, field: VitalSignsFormField, value: String) {
        match field {
            VitalSignsFormField::SystolicBp => {
                self.systolic_bp_buf = value.clone();
                self.systolic_bp = value.parse().ok();
            }
            VitalSignsFormField::DiastolicBp => {
                self.diastolic_bp_buf = value.clone();
                self.diastolic_bp = value.parse().ok();
            }
            VitalSignsFormField::HeartRate => {
                self.heart_rate_buf = value.clone();
                self.heart_rate = value.parse().ok();
            }
            VitalSignsFormField::RespiratoryRate => {
                self.respiratory_rate_buf = value.clone();
                self.respiratory_rate = value.parse().ok();
            }
            VitalSignsFormField::Temperature => {
                self.temperature_buf = value.clone();
                self.temperature = value.parse().ok();
            }
            VitalSignsFormField::O2Saturation => {
                self.o2_saturation_buf = value.clone();
                self.o2_saturation = value.parse().ok();
            }
            VitalSignsFormField::Height => {
                self.height_cm_buf = value.clone();
                self.height_cm = value.parse().ok();
            }
            VitalSignsFormField::Weight => {
                self.weight_kg_buf = value.clone();
                self.weight_kg = value.parse().ok();
            }
            VitalSignsFormField::Notes => {
                self.notes = value;
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
        use crossterm::event::KeyCode;

        match key.code {
            KeyCode::Tab => {
                if key.modifiers.contains(KeyModifiers::SHIFT) {
                    self.prev_field();
                } else {
                    self.next_field();
                }
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
            KeyCode::Enter => {
                self.validate();
                Some(VitalSignsFormAction::Submit)
            }
            KeyCode::Esc => Some(VitalSignsFormAction::Cancel),
            KeyCode::Char(c) => {
                let field = self.focused_field;
                if field.is_integer_field() {
                    if c.is_ascii_digit() {
                        let mut value = self.get_value(field);
                        value.push(c);
                        self.set_value(field, value);
                        Some(VitalSignsFormAction::ValueChanged)
                    } else {
                        None
                    }
                } else if field.is_decimal_field() {
                    if c.is_ascii_digit() || (c == '.' && !self.get_value(field).contains('.')) {
                        let mut value = self.get_value(field);
                        value.push(c);
                        self.set_value(field, value);
                        Some(VitalSignsFormAction::ValueChanged)
                    } else {
                        None
                    }
                } else {
                    let mut value = self.get_value(field);
                    value.push(c);
                    self.set_value(field, value);
                    Some(VitalSignsFormAction::ValueChanged)
                }
            }
            KeyCode::Backspace => {
                let field = self.focused_field;
                let mut value = self.get_value(field);
                value.pop();
                self.set_value(field, value);
                Some(VitalSignsFormAction::ValueChanged)
            }
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
            notes: Some(self.notes.clone()).filter(|s| !s.is_empty()),
            created_at: chrono::Utc::now(),
            created_by,
        }
    }
}

impl Widget for VitalSignsForm {
    fn render(self, area: Rect, buf: &mut Buffer) {
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
            let has_error = self.error(field).is_some();

            let label_style = if is_focused {
                Style::default()
                    .fg(self.theme.colors.primary)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(self.theme.colors.foreground)
            };

            buf.set_string(inner.x + 1, y, field.label(), label_style);

            if is_focused {
                buf.set_string(
                    field_start - 1,
                    y,
                    ">",
                    Style::default().fg(self.theme.colors.primary),
                );
            }

            let value = self.get_value(field);
            let value_style = if has_error {
                Style::default().fg(self.theme.colors.error)
            } else {
                Style::default().fg(self.theme.colors.foreground)
            };

            let max_value_width = inner.width.saturating_sub(label_width + 4);
            let display_value = if value.len() > max_value_width as usize {
                &value[value.len() - max_value_width as usize..]
            } else {
                &value
            };

            buf.set_string(field_start, y, display_value, value_style);

            if let Some(error_msg) = self.error(field) {
                let error_style = Style::default().fg(self.theme.colors.error);
                buf.set_string(field_start, y + 1, format!("  {}", error_msg), error_style);
                y += 1;
            }

            y += 2;
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

        let help_y = inner.y + inner.height - 1;
        buf.set_string(
            inner.x + 1,
            help_y,
            "Tab: Next | Enter: Submit | Esc: Cancel",
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
    fn test_vitals_form_integer_field_rejects_non_digits() {
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

        let theme = Theme::dark();
        let mut form = VitalSignsForm::new(theme);

        let key = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        let action = form.handle_key(key);
        assert!(action.is_none());

        let key = KeyEvent::new(KeyCode::Char('7'), KeyModifiers::NONE);
        let action = form.handle_key(key);
        assert!(action.is_some());
        assert_eq!(form.get_value(VitalSignsFormField::SystolicBp), "7");
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

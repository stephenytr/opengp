use crate::shared::{FormAction, FormMode};
use crate::ui::widgets::{
    FormField as FormStateField, FormFieldMeta, FormNavigation, FormRuleEngine, FormState,
    FormValidator, HeightMode, TextareaState, TextareaWidget,
};
use crate::ui::{input::to_ratatui_key, layout::LABEL_WIDTH, theme::Theme};
use crossterm::event::{KeyEvent, KeyModifiers};
use opengp_config::forms::{FormRule, FormRuleType, NumericRange, ValidationRules};
use opengp_domain::domain::clinical::VitalSigns;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    widgets::{Block, Borders, Widget},
};
use std::collections::HashMap;
use std::str::FromStr;
use uuid::Uuid;

const FIELD_SYSTOLIC_BP: &str = "systolic_bp";
const FIELD_DIASTOLIC_BP: &str = "diastolic_bp";
const FIELD_HEART_RATE: &str = "heart_rate";
const FIELD_RESPIRATORY_RATE: &str = "respiratory_rate";
const FIELD_TEMPERATURE: &str = "temperature";
const FIELD_OXYGEN_SATURATION: &str = "oxygen_saturation";
const FIELD_HEIGHT: &str = "height";
const FIELD_WEIGHT: &str = "weight";
const FIELD_NOTES: &str = "notes";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, strum::EnumIter, strum::IntoStaticStr)]
pub enum VitalSignsFormField {
    #[strum(to_string = "Systolic BP (mmHg)")]
    SystolicBp,
    #[strum(to_string = "Diastolic BP (mmHg)")]
    DiastolicBp,
    #[strum(to_string = "Heart Rate (bpm)")]
    HeartRate,
    #[strum(to_string = "Respiratory Rate")]
    RespiratoryRate,
    #[strum(to_string = "Temperature (C)")]
    Temperature,
    #[strum(to_string = "O2 Saturation (%)")]
    O2Saturation,
    #[strum(to_string = "Height (cm)")]
    Height,
    #[strum(to_string = "Weight (kg)")]
    Weight,
    #[strum(to_string = "Notes")]
    Notes,
}

impl FormStateField for VitalSignsFormField {
    fn all() -> Vec<Self> {
        use strum::IntoEnumIterator;
        Self::iter().collect()
    }
    fn label(&self) -> &'static str {
        (*self).into()
    }
    fn id(&self) -> &'static str {
        match self {
            Self::SystolicBp => FIELD_SYSTOLIC_BP,
            Self::DiastolicBp => FIELD_DIASTOLIC_BP,
            Self::HeartRate => FIELD_HEART_RATE,
            Self::RespiratoryRate => FIELD_RESPIRATORY_RATE,
            Self::Temperature => FIELD_TEMPERATURE,
            Self::O2Saturation => FIELD_OXYGEN_SATURATION,
            Self::Height => FIELD_HEIGHT,
            Self::Weight => FIELD_WEIGHT,
            Self::Notes => FIELD_NOTES,
        }
    }
    fn from_id(id: &str) -> Option<Self> {
        Some(match id {
            FIELD_SYSTOLIC_BP => Self::SystolicBp,
            FIELD_DIASTOLIC_BP => Self::DiastolicBp,
            FIELD_HEART_RATE => Self::HeartRate,
            FIELD_RESPIRATORY_RATE => Self::RespiratoryRate,
            FIELD_TEMPERATURE => Self::Temperature,
            FIELD_OXYGEN_SATURATION => Self::O2Saturation,
            FIELD_HEIGHT => Self::Height,
            FIELD_WEIGHT => Self::Weight,
            FIELD_NOTES => Self::Notes,
            _ => return None,
        })
    }
    fn is_required(&self) -> bool {
        false
    }
    fn is_textarea(&self) -> bool {
        true
    }
    fn is_dropdown(&self) -> bool {
        false
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
    mode: FormMode,
    state: FormState<VitalSignsFormField>,
    pub calculated_bmi: Option<f32>,
    validator: FormValidator,
    rule_engine: FormRuleEngine,
}

impl Clone for VitalSignsForm {
    fn clone(&self) -> Self {
        Self {
            mode: self.mode,
            state: self.state.clone(),
            calculated_bmi: self.calculated_bmi,
            validator: build_validator(),
            rule_engine: build_rule_engine(),
        }
    }
}

impl VitalSignsForm {
    pub fn new(theme: Theme) -> Self {
        let mut state = FormState::new(theme, VitalSignsFormField::SystolicBp);
        for field in <VitalSignsFormField as FormStateField>::all() {
            state
                .textareas
                .insert(field.id().to_string(), make_textarea_state(field));
        }
        Self {
            mode: FormMode::Create,
            state,
            calculated_bmi: None,
            validator: build_validator(),
            rule_engine: build_rule_engine(),
        }
    }

    pub fn is_edit_mode(&self) -> bool {
        matches!(self.mode, FormMode::Edit(_))
    }
    pub fn set_value(&mut self, field: VitalSignsFormField, value: String) {
        self.set_value_by_id(field.id(), value);
    }
    pub fn handle_key(&mut self, key: KeyEvent) -> Option<VitalSignsFormAction> {
        use crossterm::event::{KeyCode, KeyEventKind};
        if key.kind != KeyEventKind::Press {
            return None;
        }
        if key.modifiers.contains(KeyModifiers::CONTROL) && matches!(key.code, KeyCode::Char('s')) {
            self.validate_form();
            return Some(VitalSignsFormAction::Submit);
        }
        let field_id = self.state.focused_field.id().to_string();
        let consumed = self
            .state
            .textareas
            .get_mut(&field_id)
            .map(|t| t.handle_key(to_ratatui_key(key)))
            .unwrap_or(false);
        if consumed {
            self.calculate_bmi();
            self.validate_field(&field_id);
            return Some(VitalSignsFormAction::ValueChanged);
        }
        match key.code {
            KeyCode::PageUp => {
                self.state.scroll.scroll_up();
                return Some(VitalSignsFormAction::FocusChanged);
            }
            KeyCode::PageDown => {
                self.state.scroll.scroll_down();
                return Some(VitalSignsFormAction::FocusChanged);
            }
            KeyCode::Enter => return None,
            _ => {}
        }
        self.state.handle_navigation_key(key).map(|a| match a {
            FormAction::FocusChanged => VitalSignsFormAction::FocusChanged,
            FormAction::ValueChanged => VitalSignsFormAction::ValueChanged,
            FormAction::Submit => VitalSignsFormAction::Submit,
            FormAction::Cancel => VitalSignsFormAction::Cancel,
        })
    }

    pub fn to_vital_signs(&self, patient_id: Uuid, created_by: Uuid) -> VitalSigns {
        VitalSigns {
            id: Uuid::new_v4(),
            patient_id,
            consultation_id: None,
            measured_at: chrono::Utc::now(),
            systolic_bp: self.parse(VitalSignsFormField::SystolicBp),
            diastolic_bp: self.parse(VitalSignsFormField::DiastolicBp),
            heart_rate: self.parse(VitalSignsFormField::HeartRate),
            respiratory_rate: self.parse(VitalSignsFormField::RespiratoryRate),
            temperature: self.parse(VitalSignsFormField::Temperature),
            oxygen_saturation: self.parse(VitalSignsFormField::O2Saturation),
            height_cm: self.parse(VitalSignsFormField::Height),
            weight_kg: self.parse(VitalSignsFormField::Weight),
            bmi: self.calculated_bmi,
            notes: Some(self.state.get_value(VitalSignsFormField::Notes)).filter(|s| !s.is_empty()),
            created_at: chrono::Utc::now(),
            created_by,
        }
    }

    fn set_value_by_id(&mut self, field_id: &str, value: String) {
        self.state.set_value_by_id(field_id, value);
        self.calculate_bmi();
        self.validate_field(field_id);
    }

    fn validate_field(&mut self, field_id: &str) {
        self.state.errors.remove(field_id);
        let maybe_error = self
            .validator
            .validate(field_id, &self.state.get_value_by_id(field_id))
            .into_iter()
            .next();
        match maybe_error {
            Some(err) => {
                self.state.errors.insert(field_id.to_string(), err.clone());
                if let Some(t) = self.state.textareas.get_mut(field_id) {
                    t.set_error(Some(err));
                }
            }
            None => {
                if let Some(t) = self.state.textareas.get_mut(field_id) {
                    t.set_error(None);
                }
            }
        }
    }

    fn validate_form(&mut self) -> bool {
        self.state.errors.clear();
        for field in self.state.field_order.clone() {
            self.validate_field(field.id());
        }
        if self.state.errors.is_empty() {
            if let Some(first_error) = self
                .rule_engine
                .evaluate(|id| self.state.get_value_by_id(id))
                .into_iter()
                .next()
            {
                self.state
                    .errors
                    .insert(FIELD_SYSTOLIC_BP.to_string(), first_error);
            }
        }
        self.state.errors.is_empty()
    }

    fn calculate_bmi(&mut self) {
        self.calculated_bmi = match (
            self.parse::<u16>(VitalSignsFormField::Height),
            self.parse::<f32>(VitalSignsFormField::Weight),
        ) {
            (Some(height), Some(weight)) if height > 0 => {
                let m = height as f32 / 100.0;
                Some(weight / (m * m))
            }
            _ => None,
        };
    }

    fn parse<T: FromStr>(&self, field: VitalSignsFormField) -> Option<T> {
        self.state.get_value(field).trim().parse().ok()
    }
}

impl FormFieldMeta for VitalSignsFormField {
    fn label(&self) -> &'static str {
        FormStateField::label(self)
    }
    fn is_required(&self) -> bool {
        false
    }
}

impl FormNavigation for VitalSignsForm {
    type FormField = VitalSignsFormField;
    fn fields(&self) -> Vec<Self::FormField> {
        self.state.field_order.clone()
    }
    fn validate(&mut self) -> bool {
        self.validate_form()
    }
    fn current_field(&self) -> Self::FormField {
        self.state.focused_field
    }
    fn set_current_field(&mut self, field: Self::FormField) {
        self.state.focused_field = field;
    }
    fn get_error(&self, field: Self::FormField) -> Option<&str> {
        self.state.errors.get(field.id()).map(String::as_str)
    }
    fn set_error(&mut self, field: Self::FormField, error: Option<String>) {
        match error {
            Some(err) => {
                self.state
                    .errors
                    .insert(field.id().to_string(), err.clone());
                if let Some(t) = self.state.textareas.get_mut(field.id()) {
                    t.set_error(Some(err));
                }
            }
            None => {
                self.state.errors.remove(field.id());
                if let Some(t) = self.state.textareas.get_mut(field.id()) {
                    t.set_error(None);
                }
            }
        }
    }
}

impl Widget for VitalSignsForm {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.is_empty() {
            return;
        }
        let title = if self.is_edit_mode() {
            " Edit Vital Signs "
        } else {
            " New Vital Signs "
        };
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.state.theme.colors.border));
        block.clone().render(area, buf);
        let inner = block.inner(area);
        if inner.is_empty() {
            return;
        }

        let mut y = inner.y + 1;
        let max_y = inner.y + inner.height - 2;
        let field_start = inner.x + LABEL_WIDTH + 2;
        for field in self.state.field_order.clone() {
            if y > max_y {
                break;
            }
            let id = field.id();
            let Some(textarea) = self.state.textareas.get(id) else {
                continue;
            };
            let h = textarea.height();
            let area = Rect::new(inner.x + 1, y, inner.width - 2, h);
            let mut state = textarea.clone();
            if let Some(err) = self.state.errors.get(id) {
                state.set_error(Some(err.clone()));
            }
            TextareaWidget::new(&state, self.state.theme.clone())
                .focused(field == self.state.focused_field)
                .render(area, buf);
            y += h;
        }

        if let Some(bmi) = self.calculated_bmi {
            if y <= max_y {
                buf.set_string(
                    inner.x + 1,
                    y,
                    "Calculated BMI:",
                    Style::default().fg(self.state.theme.colors.info),
                );
                buf.set_string(
                    field_start,
                    y,
                    format!("{:.1}", bmi),
                    Style::default().fg(self.state.theme.colors.success),
                );
            }
        }

        self.state.scroll.render_scrollbar(inner, buf);
        buf.set_string(
            inner.x + 1,
            inner.y + inner.height - 1,
            "Tab: Next | Ctrl+S: Submit | Esc: Cancel",
            Style::default().fg(self.state.theme.colors.disabled),
        );
    }
}

fn make_textarea_state(field: VitalSignsFormField) -> TextareaState {
    match field {
        VitalSignsFormField::SystolicBp
        | VitalSignsFormField::DiastolicBp
        | VitalSignsFormField::HeartRate
        | VitalSignsFormField::O2Saturation
        | VitalSignsFormField::Height => TextareaState::new(FormStateField::label(&field))
            .with_height_mode(HeightMode::SingleLine)
            .max_length(3),
        VitalSignsFormField::RespiratoryRate => TextareaState::new(FormStateField::label(&field))
            .with_height_mode(HeightMode::SingleLine)
            .max_length(2),
        VitalSignsFormField::Temperature => TextareaState::new(FormStateField::label(&field))
            .with_height_mode(HeightMode::SingleLine)
            .max_length(5),
        VitalSignsFormField::Weight => TextareaState::new(FormStateField::label(&field))
            .with_height_mode(HeightMode::SingleLine)
            .max_length(6),
        VitalSignsFormField::Notes => TextareaState::new(FormStateField::label(&field))
            .with_height_mode(HeightMode::FixedLines(4)),
    }
}

fn build_validator() -> FormValidator {
    let mut rules = HashMap::new();
    for (id, min, max, integer_only) in [
        (FIELD_SYSTOLIC_BP, 50.0, 300.0, true),
        (FIELD_DIASTOLIC_BP, 20.0, 200.0, true),
        (FIELD_HEART_RATE, 20.0, 300.0, true),
        (FIELD_RESPIRATORY_RATE, 4.0, 60.0, true),
        (FIELD_TEMPERATURE, 30.0, 45.0, false),
        (FIELD_OXYGEN_SATURATION, 50.0, 100.0, true),
        (FIELD_HEIGHT, 30.0, 300.0, true),
        (FIELD_WEIGHT, 0.5, 700.0, false),
    ] {
        rules.insert(
            id.to_string(),
            ValidationRules {
                numeric_range: Some(NumericRange { min, max }),
                regex: integer_only.then_some(r"^\d+$".to_string()),
                ..ValidationRules::default()
            },
        );
    }
    rules.insert(FIELD_NOTES.to_string(), ValidationRules::default());
    FormValidator::new(&rules)
}

fn build_rule_engine() -> FormRuleEngine {
    FormRuleEngine::new(vec![FormRule {
        rule_type: FormRuleType::AnyNotEmpty,
        fields: vec![
            FIELD_SYSTOLIC_BP,
            FIELD_DIASTOLIC_BP,
            FIELD_HEART_RATE,
            FIELD_RESPIRATORY_RATE,
            FIELD_TEMPERATURE,
            FIELD_OXYGEN_SATURATION,
            FIELD_HEIGHT,
            FIELD_WEIGHT,
        ]
        .into_iter()
        .map(str::to_string)
        .collect(),
        message: "At least one measurement is required".to_string(),
    }])
}

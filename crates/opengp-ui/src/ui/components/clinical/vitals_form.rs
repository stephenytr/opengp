//! Vital Signs Form Component
//!
//! Form for recording patient vital signs measurements.

use std::collections::HashMap;

use crossterm::event::{KeyEvent, KeyModifiers};
use opengp_config::forms::{FormRule, FormRuleType, NumericRange, ValidationRules};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::{Block, Borders, Widget};
use uuid::Uuid;

use crate::ui::input::to_ratatui_key;
use crate::ui::layout::LABEL_WIDTH;
use crate::ui::theme::Theme;
use crate::ui::widgets::{
    DynamicForm, DynamicFormMeta, FormFieldMeta, FormNavigation, FormRuleEngine, FormValidator,
    HeightMode, ScrollableFormState, TextareaState, TextareaWidget,
};
use opengp_domain::domain::clinical::VitalSigns;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FormMode {
    #[default]
    Create,
    Edit(Uuid),
}

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

impl VitalSignsFormField {
    pub fn all() -> Vec<VitalSignsFormField> {
        use strum::IntoEnumIterator;
        VitalSignsFormField::iter().collect()
    }

    pub fn label(&self) -> &'static str {
        (*self).into()
    }

    pub fn id(&self) -> &'static str {
        match self {
            VitalSignsFormField::SystolicBp => FIELD_SYSTOLIC_BP,
            VitalSignsFormField::DiastolicBp => FIELD_DIASTOLIC_BP,
            VitalSignsFormField::HeartRate => FIELD_HEART_RATE,
            VitalSignsFormField::RespiratoryRate => FIELD_RESPIRATORY_RATE,
            VitalSignsFormField::Temperature => FIELD_TEMPERATURE,
            VitalSignsFormField::O2Saturation => FIELD_OXYGEN_SATURATION,
            VitalSignsFormField::Height => FIELD_HEIGHT,
            VitalSignsFormField::Weight => FIELD_WEIGHT,
            VitalSignsFormField::Notes => FIELD_NOTES,
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            FIELD_SYSTOLIC_BP => Some(VitalSignsFormField::SystolicBp),
            FIELD_DIASTOLIC_BP => Some(VitalSignsFormField::DiastolicBp),
            FIELD_HEART_RATE => Some(VitalSignsFormField::HeartRate),
            FIELD_RESPIRATORY_RATE => Some(VitalSignsFormField::RespiratoryRate),
            FIELD_TEMPERATURE => Some(VitalSignsFormField::Temperature),
            FIELD_OXYGEN_SATURATION => Some(VitalSignsFormField::O2Saturation),
            FIELD_HEIGHT => Some(VitalSignsFormField::Height),
            FIELD_WEIGHT => Some(VitalSignsFormField::Weight),
            FIELD_NOTES => Some(VitalSignsFormField::Notes),
            _ => None,
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

const FIELD_SYSTOLIC_BP: &str = "systolic_bp";
const FIELD_DIASTOLIC_BP: &str = "diastolic_bp";
const FIELD_HEART_RATE: &str = "heart_rate";
const FIELD_RESPIRATORY_RATE: &str = "respiratory_rate";
const FIELD_TEMPERATURE: &str = "temperature";
const FIELD_OXYGEN_SATURATION: &str = "oxygen_saturation";
const FIELD_HEIGHT: &str = "height";
const FIELD_WEIGHT: &str = "weight";
const FIELD_NOTES: &str = "notes";

pub struct VitalSignsForm {
    mode: FormMode,
    field_ids: Vec<String>,
    textareas: HashMap<String, TextareaState>,
    pub focused_field: VitalSignsFormField,
    pub calculated_bmi: Option<f32>,
    errors: HashMap<String, String>,
    validation_rules: HashMap<String, ValidationRules>,
    validator: FormValidator,
    rule_engine: FormRuleEngine,
    theme: Theme,
    scroll: ScrollableFormState,
}

impl Clone for VitalSignsForm {
    fn clone(&self) -> Self {
        Self {
            mode: self.mode,
            field_ids: self.field_ids.clone(),
            textareas: self.textareas.clone(),
            focused_field: self.focused_field,
            calculated_bmi: self.calculated_bmi,
            errors: self.errors.clone(),
            validation_rules: self.validation_rules.clone(),
            validator: FormValidator::new(&self.validation_rules),
            rule_engine: self.rule_engine.clone(),
            theme: self.theme.clone(),
            scroll: self.scroll.clone(),
        }
    }
}

impl VitalSignsForm {
    pub fn new(theme: Theme) -> Self {
        let field_ids = VitalSignsFormField::all()
            .into_iter()
            .map(|field| field.id().to_string())
            .collect::<Vec<_>>();

        let textareas = VitalSignsFormField::all()
            .into_iter()
            .map(|field| (field.id().to_string(), make_textarea_state(field, None)))
            .collect::<HashMap<_, _>>();

        let validation_rules = build_validation_rules();

        Self {
            mode: FormMode::Create,
            field_ids,
            textareas,
            focused_field: VitalSignsFormField::SystolicBp,
            calculated_bmi: None,
            errors: HashMap::new(),
            validator: FormValidator::new(&validation_rules),
            rule_engine: build_rule_engine(),
            validation_rules,
            theme,
            scroll: ScrollableFormState::new(),
        }
    }

    pub fn from_vitals(vitals: VitalSigns, theme: Theme) -> Self {
        let mut form = Self::new(theme);
        form.mode = FormMode::Edit(vitals.id);

        if let Some(systolic) = vitals.systolic_bp {
            form.set_value(VitalSignsFormField::SystolicBp, systolic.to_string());
        }

        if let Some(diastolic) = vitals.diastolic_bp {
            form.set_value(VitalSignsFormField::DiastolicBp, diastolic.to_string());
        }

        if let Some(heart_rate) = vitals.heart_rate {
            form.set_value(VitalSignsFormField::HeartRate, heart_rate.to_string());
        }

        if let Some(respiratory_rate) = vitals.respiratory_rate {
            form.set_value(
                VitalSignsFormField::RespiratoryRate,
                respiratory_rate.to_string(),
            );
        }

        if let Some(temperature) = vitals.temperature {
            form.set_value(VitalSignsFormField::Temperature, temperature.to_string());
        }

        if let Some(o2_sat) = vitals.oxygen_saturation {
            form.set_value(VitalSignsFormField::O2Saturation, o2_sat.to_string());
        }

        if let Some(height) = vitals.height_cm {
            form.set_value(VitalSignsFormField::Height, height.to_string());
        }

        if let Some(weight) = vitals.weight_kg {
            form.set_value(VitalSignsFormField::Weight, weight.to_string());
        }

        if let Some(notes) = vitals.notes {
            form.set_value(VitalSignsFormField::Notes, notes);
        }

        form
    }

    pub fn is_edit_mode(&self) -> bool {
        matches!(self.mode, FormMode::Edit(_))
    }

    pub fn vitals_id(&self) -> Option<Uuid> {
        match self.mode {
            FormMode::Edit(id) => Some(id),
            FormMode::Create => None,
        }
    }

    pub fn focused_field(&self) -> VitalSignsFormField {
        self.focused_field
    }

    pub fn get_value(&self, field: VitalSignsFormField) -> String {
        self.get_value_by_id(field.id())
    }

    pub fn set_value(&mut self, field: VitalSignsFormField, value: String) {
        self.set_value_by_id(field.id(), value);
    }

    fn get_value_by_id(&self, field_id: &str) -> String {
        self.textareas
            .get(field_id)
            .map(TextareaState::value)
            .unwrap_or_default()
    }

    fn set_value_by_id(&mut self, field_id: &str, value: String) {
        if let Some(field) = VitalSignsFormField::from_id(field_id) {
            let mut state = make_textarea_state(field, Some(value));
            state = state.focused(field == self.focused_field);
            self.textareas.insert(field_id.to_string(), state);
        }
        self.calculate_bmi();
        self.validate_field_by_id(field_id);
    }

    pub fn calculate_bmi(&mut self) {
        if let (Some(height), Some(weight)) = (self.height_cm(), self.weight_kg()) {
            if height > 0 {
                let height_m = height as f32 / 100.0;
                self.calculated_bmi = Some(weight / (height_m * height_m));
            }
        } else {
            self.calculated_bmi = None;
        }
    }

    fn validate_field_by_id(&mut self, field_id: &str) {
        self.errors.remove(field_id);

        let value = self.get_value_by_id(field_id);
        let validator_errors = self.validator.validate(field_id, &value);
        let error = validator_errors
            .into_iter()
            .next()
            .map(|msg| map_validation_error(field_id, msg));

        match error {
            Some(err) => {
                self.errors.insert(field_id.to_string(), err.clone());
                if let Some(textarea) = self.textareas.get_mut(field_id) {
                    textarea.set_error(Some(err));
                }
            }
            None => {
                if let Some(textarea) = self.textareas.get_mut(field_id) {
                    textarea.set_error(None);
                }
            }
        }
    }

    fn systolic_bp(&self) -> Option<u16> {
        self.get_value_by_id(FIELD_SYSTOLIC_BP).trim().parse().ok()
    }

    fn diastolic_bp(&self) -> Option<u16> {
        self.get_value_by_id(FIELD_DIASTOLIC_BP).trim().parse().ok()
    }

    fn heart_rate(&self) -> Option<u16> {
        self.get_value_by_id(FIELD_HEART_RATE).trim().parse().ok()
    }

    fn respiratory_rate(&self) -> Option<u16> {
        self.get_value_by_id(FIELD_RESPIRATORY_RATE)
            .trim()
            .parse()
            .ok()
    }

    fn temperature(&self) -> Option<f32> {
        self.get_value_by_id(FIELD_TEMPERATURE).trim().parse().ok()
    }

    fn oxygen_saturation(&self) -> Option<u8> {
        self.get_value_by_id(FIELD_OXYGEN_SATURATION)
            .trim()
            .parse()
            .ok()
    }

    fn height_cm(&self) -> Option<u16> {
        self.get_value_by_id(FIELD_HEIGHT).trim().parse().ok()
    }

    fn weight_kg(&self) -> Option<f32> {
        self.get_value_by_id(FIELD_WEIGHT).trim().parse().ok()
    }

    /// Returns true if at least one numeric measurement field has a value.
    pub fn has_any_measurement(&self) -> bool {
        self.systolic_bp().is_some()
            || self.diastolic_bp().is_some()
            || self.heart_rate().is_some()
            || self.respiratory_rate().is_some()
            || self.temperature().is_some()
            || self.oxygen_saturation().is_some()
            || self.height_cm().is_some()
            || self.weight_kg().is_some()
    }

    pub fn error(&self, field: VitalSignsFormField) -> Option<&String> {
        self.errors.get(field.id())
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Option<VitalSignsFormAction> {
        use crossterm::event::{KeyCode, KeyEventKind};

        if key.kind != KeyEventKind::Press {
            return None;
        }

        // Ctrl+S submits the form from any field
        if key.modifiers.contains(KeyModifiers::CONTROL) && matches!(key.code, KeyCode::Char('s')) {
            FormNavigation::validate(self);
            return Some(VitalSignsFormAction::Submit);
        }

        let field_id = self.focused_field.id().to_string();
        let ratatui_key = to_ratatui_key(key);
        let consumed = self
            .textareas
            .get_mut(&field_id)
            .map(|textarea| textarea.handle_key(ratatui_key))
            .unwrap_or(false);

        if consumed {
            self.calculate_bmi();
            self.validate_field_by_id(&field_id);
            return Some(VitalSignsFormAction::ValueChanged);
        }

        match key.code {
            KeyCode::Tab => {
                if key.modifiers.contains(KeyModifiers::SHIFT) {
                    FormNavigation::prev_field(self);
                } else {
                    FormNavigation::next_field(self);
                }
                Some(VitalSignsFormAction::FocusChanged)
            }
            KeyCode::BackTab => {
                FormNavigation::prev_field(self);
                Some(VitalSignsFormAction::FocusChanged)
            }
            KeyCode::Up => {
                FormNavigation::prev_field(self);
                Some(VitalSignsFormAction::FocusChanged)
            }
            KeyCode::Down => {
                FormNavigation::next_field(self);
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
            KeyCode::Enter => None,
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
            systolic_bp: self.systolic_bp(),
            diastolic_bp: self.diastolic_bp(),
            heart_rate: self.heart_rate(),
            respiratory_rate: self.respiratory_rate(),
            temperature: self.temperature(),
            oxygen_saturation: self.oxygen_saturation(),
            height_cm: self.height_cm(),
            weight_kg: self.weight_kg(),
            bmi: self.calculated_bmi,
            notes: Some(self.get_value_by_id(FIELD_NOTES)).filter(|s: &String| !s.is_empty()),
            created_at: chrono::Utc::now(),
            created_by,
        }
    }
}

impl DynamicFormMeta for VitalSignsForm {
    fn label(&self, field_id: &str) -> String {
        VitalSignsFormField::from_id(field_id)
            .map(|field| field.label().to_string())
            .unwrap_or_else(|| field_id.to_string())
    }

    fn is_required(&self, _field_id: &str) -> bool {
        false
    }

    fn field_type(&self, _field_id: &str) -> crate::ui::widgets::FieldType {
        crate::ui::widgets::FieldType::Text
    }
}

impl DynamicForm for VitalSignsForm {
    fn field_ids(&self) -> &[String] {
        &self.field_ids
    }

    fn current_field(&self) -> &str {
        self.focused_field.id()
    }

    fn set_current_field(&mut self, field_id: &str) {
        if let Some(field) = VitalSignsFormField::from_id(field_id) {
            self.focused_field = field;
        }
    }

    fn get_value(&self, field_id: &str) -> String {
        self.get_value_by_id(field_id)
    }

    fn set_value(&mut self, field_id: &str, value: String) {
        self.set_value_by_id(field_id, value);
    }

    fn validate(&mut self) -> bool {
        self.errors.clear();

        for field_id in self.field_ids.clone() {
            self.validate_field_by_id(&field_id);
        }

        if self.errors.is_empty() {
            let form_errors = self
                .rule_engine
                .evaluate(|field_id| self.get_value_by_id(field_id));
            if let Some(first_error) = form_errors.into_iter().next() {
                self.errors
                    .insert(FIELD_SYSTOLIC_BP.to_string(), first_error);
            }
        }

        self.errors.is_empty()
    }

    fn get_error(&self, field_id: &str) -> Option<&str> {
        self.errors.get(field_id).map(|s| s.as_str())
    }

    fn set_error(&mut self, field_id: &str, error: Option<String>) {
        match error {
            Some(msg) => {
                self.errors.insert(field_id.to_string(), msg);
            }
            None => {
                self.errors.remove(field_id);
            }
        }
    }
}

impl FormFieldMeta for VitalSignsFormField {
    fn label(&self) -> &'static str {
        VitalSignsFormField::label(self)
    }

    fn is_required(&self) -> bool {
        VitalSignsFormField::is_required(self)
    }
}

impl FormNavigation for VitalSignsForm {
    type FormField = VitalSignsFormField;

    fn get_error(&self, field: Self::FormField) -> Option<&str> {
        self.errors.get(field.id()).map(|s| s.as_str())
    }

    fn set_error(&mut self, field: Self::FormField, error: Option<String>) {
        <Self as DynamicForm>::set_error(self, field.id(), error);
    }

    fn validate(&mut self) -> bool {
        <Self as DynamicForm>::validate(self)
    }

    fn current_field(&self) -> Self::FormField {
        self.focused_field
    }

    fn fields(&self) -> Vec<Self::FormField> {
        VitalSignsFormField::all()
    }

    fn set_current_field(&mut self, field: Self::FormField) {
        self.focused_field = field;
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
            .border_style(Style::default().fg(self.theme.colors.border));

        block.clone().render(area, buf);

        let inner = block.inner(area);
        if inner.is_empty() {
            return;
        }

        let label_width = LABEL_WIDTH;
        let field_start = inner.x + label_width + 2;

        let fields = self.field_ids.clone();

        let mut y = inner.y + 1;
        let max_y = inner.y + inner.height - 2;

        for field_id in fields {
            if y > max_y {
                break;
            }

            let is_focused = field_id == self.focused_field.id();
            let Some(textarea) = self.textareas.get(&field_id) else {
                continue;
            };
            let field_height = textarea.height();

            let field_area = Rect::new(inner.x + 1, y, inner.width - 2, field_height);

            let mut state = textarea.clone();
            if let Some(err) = self.errors.get(&field_id) {
                state.set_error(Some(err.clone()));
            }
            TextareaWidget::new(&state, self.theme.clone())
                .focused(is_focused)
                .render(field_area, buf);

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

        self.scroll.render_scrollbar(inner, buf, &self.theme);

        let help_y = inner.y + inner.height - 1;
        buf.set_string(
            inner.x + 1,
            help_y,
            "Tab: Next | Ctrl+S: Submit | Esc: Cancel",
            Style::default().fg(self.theme.colors.disabled),
        );
    }
}

fn make_textarea_state(field: VitalSignsFormField, value: Option<String>) -> TextareaState {
    let mut state = match field {
        VitalSignsFormField::SystolicBp => TextareaState::new(field.label())
            .with_height_mode(HeightMode::SingleLine)
            .max_length(3),
        VitalSignsFormField::DiastolicBp => TextareaState::new(field.label())
            .with_height_mode(HeightMode::SingleLine)
            .max_length(3),
        VitalSignsFormField::HeartRate => TextareaState::new(field.label())
            .with_height_mode(HeightMode::SingleLine)
            .max_length(3),
        VitalSignsFormField::RespiratoryRate => TextareaState::new(field.label())
            .with_height_mode(HeightMode::SingleLine)
            .max_length(2),
        VitalSignsFormField::Temperature => TextareaState::new(field.label())
            .with_height_mode(HeightMode::SingleLine)
            .max_length(5),
        VitalSignsFormField::O2Saturation => TextareaState::new(field.label())
            .with_height_mode(HeightMode::SingleLine)
            .max_length(3),
        VitalSignsFormField::Height => TextareaState::new(field.label())
            .with_height_mode(HeightMode::SingleLine)
            .max_length(3),
        VitalSignsFormField::Weight => TextareaState::new(field.label())
            .with_height_mode(HeightMode::SingleLine)
            .max_length(6),
        VitalSignsFormField::Notes => {
            TextareaState::new(field.label()).with_height_mode(HeightMode::FixedLines(4))
        }
    };

    if let Some(value) = value {
        state = state.with_value(value);
    }

    state
}

fn build_validation_rules() -> HashMap<String, ValidationRules> {
    let mut rules = HashMap::new();

    rules.insert(
        FIELD_SYSTOLIC_BP.to_string(),
        ValidationRules {
            numeric_range: Some(NumericRange {
                min: 50.0,
                max: 300.0,
            }),
            regex: Some(r"^\d+$".to_string()),
            ..ValidationRules::default()
        },
    );

    rules.insert(
        FIELD_DIASTOLIC_BP.to_string(),
        ValidationRules {
            numeric_range: Some(NumericRange {
                min: 20.0,
                max: 200.0,
            }),
            regex: Some(r"^\d+$".to_string()),
            ..ValidationRules::default()
        },
    );

    rules.insert(
        FIELD_HEART_RATE.to_string(),
        ValidationRules {
            numeric_range: Some(NumericRange {
                min: 20.0,
                max: 300.0,
            }),
            regex: Some(r"^\d+$".to_string()),
            ..ValidationRules::default()
        },
    );

    rules.insert(
        FIELD_RESPIRATORY_RATE.to_string(),
        ValidationRules {
            numeric_range: Some(NumericRange {
                min: 4.0,
                max: 60.0,
            }),
            regex: Some(r"^\d+$".to_string()),
            ..ValidationRules::default()
        },
    );

    rules.insert(
        FIELD_TEMPERATURE.to_string(),
        ValidationRules {
            numeric_range: Some(NumericRange {
                min: 30.0,
                max: 45.0,
            }),
            ..ValidationRules::default()
        },
    );

    rules.insert(
        FIELD_OXYGEN_SATURATION.to_string(),
        ValidationRules {
            numeric_range: Some(NumericRange {
                min: 50.0,
                max: 100.0,
            }),
            regex: Some(r"^\d+$".to_string()),
            ..ValidationRules::default()
        },
    );

    rules.insert(
        FIELD_HEIGHT.to_string(),
        ValidationRules {
            numeric_range: Some(NumericRange {
                min: 30.0,
                max: 300.0,
            }),
            regex: Some(r"^\d+$".to_string()),
            ..ValidationRules::default()
        },
    );

    rules.insert(
        FIELD_WEIGHT.to_string(),
        ValidationRules {
            numeric_range: Some(NumericRange {
                min: 0.5,
                max: 700.0,
            }),
            ..ValidationRules::default()
        },
    );

    rules.insert(FIELD_NOTES.to_string(), ValidationRules::default());

    rules
}

fn build_rule_engine() -> FormRuleEngine {
    FormRuleEngine::new(vec![FormRule {
        rule_type: FormRuleType::AnyNotEmpty,
        fields: vec![
            FIELD_SYSTOLIC_BP.to_string(),
            FIELD_DIASTOLIC_BP.to_string(),
            FIELD_HEART_RATE.to_string(),
            FIELD_RESPIRATORY_RATE.to_string(),
            FIELD_TEMPERATURE.to_string(),
            FIELD_OXYGEN_SATURATION.to_string(),
            FIELD_HEIGHT.to_string(),
            FIELD_WEIGHT.to_string(),
        ],
        message: "At least one measurement is required".to_string(),
    }])
}

fn map_validation_error(field_id: &str, message: String) -> String {
    match message.as_str() {
        "Invalid number" => invalid_number_message(field_id).to_string(),
        "Invalid format" => {
            if is_integer_field(field_id) {
                "Must be a whole number".to_string()
            } else {
                message
            }
        }
        m if m.starts_with("Value must be between") => range_error_message(field_id).to_string(),
        _ => message,
    }
}

fn is_integer_field(field_id: &str) -> bool {
    matches!(
        field_id,
        FIELD_SYSTOLIC_BP
            | FIELD_DIASTOLIC_BP
            | FIELD_HEART_RATE
            | FIELD_RESPIRATORY_RATE
            | FIELD_OXYGEN_SATURATION
            | FIELD_HEIGHT
    )
}

fn invalid_number_message(field_id: &str) -> &'static str {
    match field_id {
        FIELD_TEMPERATURE => "Must be a number (e.g. 37.2)",
        FIELD_WEIGHT => "Must be a number (e.g. 72.5)",
        _ => "Must be a whole number",
    }
}

fn range_error_message(field_id: &str) -> &'static str {
    match field_id {
        FIELD_SYSTOLIC_BP => "Systolic BP must be 50-300 mmHg",
        FIELD_DIASTOLIC_BP => "Diastolic BP must be 20-200 mmHg",
        FIELD_HEART_RATE => "Heart rate must be 20-300 bpm",
        FIELD_RESPIRATORY_RATE => "Respiratory rate must be 4-60 /min",
        FIELD_TEMPERATURE => "Temperature must be 30-45 C",
        FIELD_OXYGEN_SATURATION => "O2 saturation must be 50-100%",
        FIELD_HEIGHT => "Height must be 30-300 cm",
        FIELD_WEIGHT => "Weight must be 0.5-700 kg",
        _ => "Invalid value",
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
        FormNavigation::next_field(&mut form);
        assert_eq!(form.focused_field(), VitalSignsFormField::DiastolicBp);
        FormNavigation::next_field(&mut form);
        assert_eq!(form.focused_field(), VitalSignsFormField::HeartRate);
        FormNavigation::prev_field(&mut form);
        assert_eq!(form.focused_field(), VitalSignsFormField::DiastolicBp);
    }

    #[test]
    fn test_vitals_form_field_navigation_wraps() {
        let theme = Theme::dark();
        let mut form = VitalSignsForm::new(theme);

        FormNavigation::prev_field(&mut form);
        assert_eq!(form.focused_field(), VitalSignsFormField::Notes);

        form.focused_field = VitalSignsFormField::Notes;
        FormNavigation::next_field(&mut form);
        assert_eq!(form.focused_field(), VitalSignsFormField::SystolicBp);
    }

    #[test]
    fn test_vitals_form_validation_requires_at_least_one_measurement() {
        let theme = Theme::dark();
        let mut form = VitalSignsForm::new(theme);

        let valid = FormNavigation::validate(&mut form);
        assert!(!valid);
        assert!(form.error(VitalSignsFormField::SystolicBp).is_some());
    }

    #[test]
    fn test_vitals_form_validation_passes_with_one_measurement() {
        let theme = Theme::dark();
        let mut form = VitalSignsForm::new(theme);

        form.set_value(VitalSignsFormField::HeartRate, "72".to_string());
        let valid = FormNavigation::validate(&mut form);
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

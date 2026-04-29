use std::collections::HashMap;

use crate::ui::input::to_ratatui_key;
use crate::ui::theme::Theme;
use crate::ui::widgets::{
    format_date, parse_date, DropdownOption, DropdownWidget, FormValidator, HeightMode,
    LoadingState, ScrollableFormState, TextareaState, TextareaWidget,
};
use crossterm::event::{Event, KeyEvent, KeyModifiers};
use rat_event::ct_event;
use opengp_config::{forms::ValidationRules, SocialHistoryConfig};
use opengp_domain::domain::clinical::{AlcoholStatus, ExerciseFrequency, SmokingStatus};
use rat_focus::{FocusBuilder, FocusFlag, HasFocus};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Widget};

#[derive(Clone)]
pub struct SocialHistoryData {
    pub smoking_status: SmokingStatus,
    pub cigarettes_per_day: Option<u8>,
    pub smoking_quit_date: Option<chrono::NaiveDate>,
    pub alcohol_status: AlcoholStatus,
    pub standard_drinks_per_week: Option<u8>,
    pub exercise_frequency: Option<ExerciseFrequency>,
    pub occupation: Option<String>,
    pub living_situation: Option<String>,
    pub support_network: Option<String>,
    pub notes: Option<String>,
}

const FIELD_SMOKING_STATUS: &str = "smoking_status";
const FIELD_CIGARETTES_PER_DAY: &str = "cigarettes_per_day";
const FIELD_QUIT_DATE: &str = "quit_date";
const FIELD_ALCOHOL_STATUS: &str = "alcohol_status";
const FIELD_DRINKS_PER_WEEK: &str = "drinks_per_week";
const FIELD_EXERCISE_FREQUENCY: &str = "exercise_frequency";
const FIELD_OCCUPATION: &str = "occupation";
const FIELD_LIVING_SITUATION: &str = "living_situation";
const FIELD_SUPPORT_NETWORK: &str = "support_network";
const FIELD_NOTES: &str = "notes";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, strum::EnumIter, strum::IntoStaticStr)]
pub enum SocialHistoryField {
    #[strum(to_string = "Smoking")]
    SmokingStatus,
    #[strum(to_string = "Cigarettes/day")]
    CigarettesPerDay,
    #[strum(to_string = "Quit date (dd/mm/yyyy)")]
    QuitDate,
    #[strum(to_string = "Alcohol")]
    AlcoholStatus,
    #[strum(to_string = "Drinks/week")]
    DrinksPerWeek,
    #[strum(to_string = "Exercise")]
    ExerciseFrequency,
    #[strum(to_string = "Occupation")]
    Occupation,
    #[strum(to_string = "Living situation")]
    LivingSituation,
    #[strum(to_string = "Support network")]
    SupportNetwork,
    #[strum(to_string = "Notes")]
    Notes,
}

impl SocialHistoryField {
    fn all() -> Vec<SocialHistoryField> {
        use strum::IntoEnumIterator;
        SocialHistoryField::iter().collect()
    }

    fn label(&self) -> &'static str {
        (*self).into()
    }

    fn id(&self) -> &'static str {
        match self {
            SocialHistoryField::SmokingStatus => FIELD_SMOKING_STATUS,
            SocialHistoryField::CigarettesPerDay => FIELD_CIGARETTES_PER_DAY,
            SocialHistoryField::QuitDate => FIELD_QUIT_DATE,
            SocialHistoryField::AlcoholStatus => FIELD_ALCOHOL_STATUS,
            SocialHistoryField::DrinksPerWeek => FIELD_DRINKS_PER_WEEK,
            SocialHistoryField::ExerciseFrequency => FIELD_EXERCISE_FREQUENCY,
            SocialHistoryField::Occupation => FIELD_OCCUPATION,
            SocialHistoryField::LivingSituation => FIELD_LIVING_SITUATION,
            SocialHistoryField::SupportNetwork => FIELD_SUPPORT_NETWORK,
            SocialHistoryField::Notes => FIELD_NOTES,
        }
    }

    fn from_id(field_id: &str) -> Option<Self> {
        match field_id {
            FIELD_SMOKING_STATUS => Some(SocialHistoryField::SmokingStatus),
            FIELD_CIGARETTES_PER_DAY => Some(SocialHistoryField::CigarettesPerDay),
            FIELD_QUIT_DATE => Some(SocialHistoryField::QuitDate),
            FIELD_ALCOHOL_STATUS => Some(SocialHistoryField::AlcoholStatus),
            FIELD_DRINKS_PER_WEEK => Some(SocialHistoryField::DrinksPerWeek),
            FIELD_EXERCISE_FREQUENCY => Some(SocialHistoryField::ExerciseFrequency),
            FIELD_OCCUPATION => Some(SocialHistoryField::Occupation),
            FIELD_LIVING_SITUATION => Some(SocialHistoryField::LivingSituation),
            FIELD_SUPPORT_NETWORK => Some(SocialHistoryField::SupportNetwork),
            FIELD_NOTES => Some(SocialHistoryField::Notes),
            _ => None,
        }
    }

    fn is_dropdown(&self) -> bool {
        matches!(
            self,
            SocialHistoryField::SmokingStatus
                | SocialHistoryField::AlcoholStatus
                | SocialHistoryField::ExerciseFrequency
        )
    }
}

pub struct SocialHistoryComponent {
    pub social_history: Option<SocialHistoryData>,
    pub is_editing: bool,
    pub focused_field: SocialHistoryField,
    pub loading: bool,
    loading_state: LoadingState,
    theme: Theme,
    field_ids: Vec<String>,
    errors: HashMap<String, String>,
    textareas: HashMap<String, TextareaState>,
    dropdowns: HashMap<String, DropdownWidget>,
    validator: FormValidator,
    scroll: ScrollableFormState,
    pub focus: FocusFlag,
}

impl Clone for SocialHistoryComponent {
    fn clone(&self) -> Self {
        Self {
            social_history: self.social_history.clone(),
            is_editing: self.is_editing,
            focused_field: self.focused_field,
            loading: self.loading,
            loading_state: self.loading_state.clone(),
            theme: self.theme.clone(),
            field_ids: self.field_ids.clone(),
            errors: self.errors.clone(),
            textareas: self.textareas.clone(),
            dropdowns: self.dropdowns.clone(),
            validator: build_validator(),
            scroll: self.scroll.clone(),
            focus: self.focus.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SocialHistoryAction {
    Edit,
    Save,
    Cancel,
    FieldChanged,
    FocusChanged,
}

impl SocialHistoryComponent {
    pub fn new(theme: Theme, config: &SocialHistoryConfig) -> Self {
        let field_ids: Vec<String> = SocialHistoryField::all()
            .iter()
            .map(|field| field.id().to_string())
            .collect();

        let mut textareas = HashMap::new();
        for field in SocialHistoryField::all() {
            if !field.is_dropdown() {
                textareas.insert(field.id().to_string(), make_textarea_state(field, None));
            }
        }

        Self {
            social_history: None,
            is_editing: false,
            focused_field: SocialHistoryField::SmokingStatus,
            loading: false,
            loading_state: LoadingState::new().message("Loading social history..."),
            theme: theme.clone(),
            field_ids,
            errors: HashMap::new(),
            textareas,
            dropdowns: build_dropdowns(theme, config),
            validator: build_validator(),
            scroll: ScrollableFormState::new(),
            focus: FocusFlag::default(),
        }
    }

    pub fn is_loading(&self) -> bool {
        self.loading
    }

    pub fn set_loading(&mut self, loading: bool) {
        self.loading = loading;
    }

    pub fn start_editing(&mut self) {
        if let Some(history) = self.social_history.clone() {
            self.set_field_value(
                &SocialHistoryField::CigarettesPerDay,
                history
                    .cigarettes_per_day
                    .map(|n| n.to_string())
                    .unwrap_or_default(),
            );
            self.set_field_value(
                &SocialHistoryField::QuitDate,
                history
                    .smoking_quit_date
                    .map(format_date)
                    .unwrap_or_default(),
            );
            self.set_field_value(
                &SocialHistoryField::DrinksPerWeek,
                history
                    .standard_drinks_per_week
                    .map(|n| n.to_string())
                    .unwrap_or_default(),
            );
            self.set_field_value(
                &SocialHistoryField::Occupation,
                history.occupation.clone().unwrap_or_default(),
            );
            self.set_field_value(
                &SocialHistoryField::LivingSituation,
                history.living_situation.clone().unwrap_or_default(),
            );
            self.set_field_value(
                &SocialHistoryField::SupportNetwork,
                history.support_network.clone().unwrap_or_default(),
            );
            self.set_field_value(
                &SocialHistoryField::Notes,
                history.notes.clone().unwrap_or_default(),
            );
            if let Some(dropdown) = self.dropdowns.get_mut(FIELD_SMOKING_STATUS) {
                dropdown.set_value(&format!("{:?}", history.smoking_status));
            }
            if let Some(dropdown) = self.dropdowns.get_mut(FIELD_ALCOHOL_STATUS) {
                dropdown.set_value(&format!("{:?}", history.alcohol_status));
            }
            if let Some(freq) = history.exercise_frequency {
                if let Some(dropdown) = self.dropdowns.get_mut(FIELD_EXERCISE_FREQUENCY) {
                    dropdown.set_value(&format!("{:?}", freq));
                }
            }
        }
        self.focused_field = SocialHistoryField::SmokingStatus;
        self.errors.clear();
        self.is_editing = true;
    }

    pub fn stop_editing(&mut self) {
        self.is_editing = false;
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Option<SocialHistoryAction> {
        use crossterm::event::KeyEventKind;

        if key.kind != KeyEventKind::Press {
            return None;
        }

        if !self.is_editing {
            let event = Event::Key(key);
            match &event {
                ct_event!(key press 'e') | ct_event!(key press 'n') => {
                    self.start_editing();
                    Some(SocialHistoryAction::Edit)
                }
                _ => None,
            }
        } else {
            // Ctrl+S saves the form
            let event = Event::Key(key);
            if key.modifiers.contains(KeyModifiers::CONTROL)
                && matches!(&event, ct_event!(key press CONTROL-'s'))
            {
                return Some(SocialHistoryAction::Save);
            }

            if self.focused_field.is_dropdown() {
                let focused_field_id = self.focused_field.id().to_string();
                let dropdown_consumed = self
                    .dropdowns
                    .get_mut(&focused_field_id)
                    .and_then(|dropdown| dropdown.handle_key(key));
                if let Some(_action) = dropdown_consumed {
                    let event = Event::Key(key);
                    match &event {
                        ct_event!(keycode press Tab) | ct_event!(keycode press BackTab) | ct_event!(keycode press Esc) => {}
                        _ => {
                            self.validate_field_by_id(&focused_field_id);
                            return Some(SocialHistoryAction::FieldChanged);
                        }
                    }
                }
            }

            if !self.focused_field.is_dropdown() {
                // Do not pass Tab/BackTab/Esc to textarea — those are navigation keys
                let is_nav_key = matches!(
                    key.code,
                    crossterm::event::KeyCode::Tab
                        | crossterm::event::KeyCode::BackTab
                        | crossterm::event::KeyCode::Esc
                );
                if !is_nav_key {
                    let focused_field_id = self.focused_field.id().to_string();
                    let ratatui_key = to_ratatui_key(key);
                    if let Some(textarea) = self.textareas.get_mut(&focused_field_id) {
                        let consumed = textarea.handle_key(ratatui_key);
                        if consumed {
                            self.validate_field_by_id(&focused_field_id);
                            return Some(SocialHistoryAction::FieldChanged);
                        }
                    }
                }
            }

            let event = Event::Key(key);
            match &event {
                ct_event!(keycode press Tab) | ct_event!(keycode press SHIFT-Tab) => {
                    if key.modifiers.contains(KeyModifiers::SHIFT) {
                        self.prev_field();
                    } else {
                        self.next_field();
                    }
                    Some(SocialHistoryAction::FocusChanged)
                }
                ct_event!(keycode press BackTab) | ct_event!(keycode press SHIFT-BackTab) => {
                    self.prev_field();
                    Some(SocialHistoryAction::FocusChanged)
                }
                ct_event!(keycode press Up) => {
                    self.prev_field();
                    Some(SocialHistoryAction::FocusChanged)
                }
                ct_event!(keycode press Down) => {
                    self.next_field();
                    Some(SocialHistoryAction::FocusChanged)
                }
                ct_event!(keycode press PageUp) => {
                    self.scroll.scroll_up();
                    Some(SocialHistoryAction::FocusChanged)
                }
                ct_event!(keycode press PageDown) => {
                    self.scroll.scroll_down();
                    Some(SocialHistoryAction::FocusChanged)
                }
                ct_event!(keycode press Enter) => None,
                ct_event!(keycode press Esc) => {
                    self.stop_editing();
                    Some(SocialHistoryAction::Cancel)
                }
                _ => None,
            }
        }
    }

    fn next_field(&mut self) {
        <Self as crate::ui::widgets::DynamicForm>::next_field(self);
        self.focused_field = SocialHistoryField::from_id(
            <Self as crate::ui::widgets::DynamicForm>::current_field(self),
        )
        .unwrap_or(SocialHistoryField::SmokingStatus);
    }

    fn prev_field(&mut self) {
        <Self as crate::ui::widgets::DynamicForm>::prev_field(self);
        self.focused_field = SocialHistoryField::from_id(
            <Self as crate::ui::widgets::DynamicForm>::current_field(self),
        )
        .unwrap_or(SocialHistoryField::SmokingStatus);
    }

    pub fn get_field_value(&self, field: &SocialHistoryField) -> String {
        self.get_value_by_id(field.id())
    }

    pub fn set_field_value(&mut self, field: &SocialHistoryField, value: String) {
        self.set_value_by_id(field.id(), value);
    }

    pub fn to_social_history(
        &self,
        _patient_id: uuid::Uuid,
        _updated_by: uuid::Uuid,
    ) -> SocialHistoryData {
        let smoking_status = self
            .dropdowns
            .get(FIELD_SMOKING_STATUS)
            .and_then(|dropdown| dropdown.selected_value())
            .and_then(|v: &str| v.parse::<SmokingStatus>().ok())
            .unwrap_or(SmokingStatus::NeverSmoked);

        let alcohol_status = self
            .dropdowns
            .get(FIELD_ALCOHOL_STATUS)
            .and_then(|dropdown| dropdown.selected_value())
            .and_then(|v: &str| v.parse::<AlcoholStatus>().ok())
            .unwrap_or(AlcoholStatus::None);

        let exercise_frequency = self
            .dropdowns
            .get(FIELD_EXERCISE_FREQUENCY)
            .and_then(|dropdown| dropdown.selected_value())
            .and_then(|v: &str| v.parse::<ExerciseFrequency>().ok());

        let cigarettes_value = self.get_value_by_id(FIELD_CIGARETTES_PER_DAY);
        let drinks_value = self.get_value_by_id(FIELD_DRINKS_PER_WEEK);
        let quit_date_value = self.get_value_by_id(FIELD_QUIT_DATE);
        let occupation_value = self.get_value_by_id(FIELD_OCCUPATION);
        let living_situation_value = self.get_value_by_id(FIELD_LIVING_SITUATION);
        let support_network_value = self.get_value_by_id(FIELD_SUPPORT_NETWORK);
        let notes_value = self.get_value_by_id(FIELD_NOTES);

        SocialHistoryData {
            smoking_status,
            cigarettes_per_day: cigarettes_value.parse::<u8>().ok(),
            smoking_quit_date: parse_date(&quit_date_value),
            alcohol_status,
            standard_drinks_per_week: drinks_value.parse::<u8>().ok(),
            exercise_frequency,
            occupation: Some(occupation_value).filter(|s| !s.is_empty()),
            living_situation: Some(living_situation_value).filter(|s| !s.is_empty()),
            support_network: Some(support_network_value).filter(|s| !s.is_empty()),
            notes: Some(notes_value).filter(|s| !s.is_empty()),
        }
    }

    fn get_value_by_id(&self, field_id: &str) -> String {
        if let Some(textarea) = self.textareas.get(field_id) {
            return textarea.value();
        }

        if let Some(dropdown) = self.dropdowns.get(field_id) {
            return dropdown.selected_label().unwrap_or("Select...").to_string();
        }

        String::new()
    }

    fn set_value_by_id(&mut self, field_id: &str, value: String) {
        if let Some(textarea) = self.textareas.get_mut(field_id) {
            let label = textarea.label.clone();
            let height_mode = textarea.height_mode.clone();
            let max_length = textarea.max_length;
            let focused = textarea.focused;

            let mut updated = TextareaState::new(label)
                .with_height_mode(height_mode)
                .with_value(value.clone())
                .focused(focused);
            if let Some(limit) = max_length {
                updated = updated.max_length(limit);
            }
            *textarea = updated;
            self.validate_field_by_id(field_id);
            return;
        }

        if let Some(dropdown) = self.dropdowns.get_mut(field_id) {
            dropdown.set_value(&value);
            self.validate_field_by_id(field_id);
        }
    }

    fn set_error_by_id(&mut self, field_id: &str, error: Option<String>) {
        match error {
            Some(msg) => {
                self.errors.insert(field_id.to_string(), msg);
            }
            None => {
                self.errors.remove(field_id);
            }
        }
    }

    fn validate_field_by_id(&mut self, field_id: &str) {
        self.errors.remove(field_id);

        let value = if let Some(textarea) = self.textareas.get(field_id) {
            textarea.value()
        } else if let Some(dropdown) = self.dropdowns.get(field_id) {
            dropdown.selected_value().unwrap_or("").to_string()
        } else {
            String::new()
        };

        let mut errors = self.validator.validate(field_id, &value);

        if matches!(field_id, FIELD_CIGARETTES_PER_DAY | FIELD_DRINKS_PER_WEEK)
            && !value.trim().is_empty()
            && value.parse::<u8>().is_err()
        {
            errors = vec!["Invalid number".to_string()];
        }

        if field_id == FIELD_QUIT_DATE && !value.trim().is_empty() && parse_date(&value).is_none() {
            errors = vec!["Use dd/mm/yyyy format".to_string()];
        }

        let error_msg = errors.into_iter().next();
        self.set_error_by_id(field_id, error_msg.clone());
        if let Some(textarea) = self.textareas.get_mut(field_id) {
            textarea.set_error(error_msg);
        }
    }

    fn get_field_height(&self, field: SocialHistoryField) -> u16 {
        if field.is_dropdown() {
            return 3;
        }

        self.textareas
            .get(field.id())
            .map(|textarea| textarea.height())
            .unwrap_or(1)
    }
}

impl crate::ui::widgets::DynamicFormMeta for SocialHistoryComponent {
    fn label(&self, field_id: &str) -> String {
        SocialHistoryField::from_id(field_id)
            .map(|field| field.label().to_string())
            .unwrap_or_else(|| field_id.to_string())
    }

    fn is_required(&self, _field_id: &str) -> bool {
        false
    }

    fn field_type(&self, field_id: &str) -> crate::ui::widgets::FieldType {
        match SocialHistoryField::from_id(field_id) {
            Some(SocialHistoryField::QuitDate) => crate::ui::widgets::FieldType::Date,
            Some(field) if field.is_dropdown() => crate::ui::widgets::FieldType::Select(vec![]),
            _ => crate::ui::widgets::FieldType::Text,
        }
    }
}

impl crate::ui::widgets::DynamicForm for SocialHistoryComponent {
    fn field_ids(&self) -> &[String] {
        &self.field_ids
    }

    fn current_field(&self) -> &str {
        self.focused_field.id()
    }

    fn set_current_field(&mut self, field_id: &str) {
        if let Some(field) = SocialHistoryField::from_id(field_id) {
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
        self.errors.is_empty()
    }

    fn get_error(&self, field_id: &str) -> Option<&str> {
        self.errors.get(field_id).map(|s| s.as_str())
    }

    fn set_error(&mut self, field_id: &str, error: Option<String>) {
        self.set_error_by_id(field_id, error);
    }
}

impl HasFocus for SocialHistoryComponent {
    fn build(&self, builder: &mut FocusBuilder) {
        builder.leaf_widget(self);
    }

    fn focus(&self) -> FocusFlag {
        self.focus.clone()
    }

    fn area(&self) -> ratatui::layout::Rect {
        ratatui::layout::Rect::default()
    }
}

fn make_textarea_state(field: SocialHistoryField, value: Option<String>) -> TextareaState {
    let mut state = TextareaState::new(field.label());
    state = match field {
        SocialHistoryField::Notes => state.with_height_mode(HeightMode::FixedLines(4)),
        _ => state.with_height_mode(HeightMode::SingleLine),
    };

    if matches!(
        field,
        SocialHistoryField::CigarettesPerDay | SocialHistoryField::DrinksPerWeek
    ) {
        state = state.max_length(3);
    }
    if field == SocialHistoryField::QuitDate {
        state = state.max_length(10);
    }

    if let Some(value) = value {
        state = state.with_value(value);
    }

    state
}

fn build_dropdowns(theme: Theme, config: &SocialHistoryConfig) -> HashMap<String, DropdownWidget> {
    let mut dropdowns = HashMap::new();

    // Build smoking options from config, filtering to enabled only
    let smoking_options: Vec<DropdownOption> = config
        .smoking_status
        .iter()
        .filter(|(_, opt)| opt.enabled)
        .map(|(key, opt): (&String, &opengp_config::EnumOption)| {
            DropdownOption::new(key.clone(), opt.label.clone())
        })
        .collect();

    // Build alcohol options from config, filtering to enabled only
    let alcohol_options: Vec<DropdownOption> = config
        .alcohol_status
        .iter()
        .filter(|(_, opt)| opt.enabled)
        .map(|(key, opt): (&String, &opengp_config::EnumOption)| {
            DropdownOption::new(key.clone(), opt.label.clone())
        })
        .collect();

    // Build exercise options from config, filtering to enabled only
    let exercise_options: Vec<DropdownOption> = config
        .exercise_frequency
        .iter()
        .filter(|(_, opt)| opt.enabled)
        .map(|(key, opt): (&String, &opengp_config::EnumOption)| {
            DropdownOption::new(key.clone(), opt.label.clone())
        })
        .collect();

    let mut smoking_dropdown = DropdownWidget::new("Smoking", smoking_options, theme.clone());
    smoking_dropdown.set_value("NeverSmoked");
    dropdowns.insert(FIELD_SMOKING_STATUS.to_string(), smoking_dropdown);

    let mut alcohol_dropdown = DropdownWidget::new("Alcohol", alcohol_options, theme.clone());
    alcohol_dropdown.set_value("None");
    dropdowns.insert(FIELD_ALCOHOL_STATUS.to_string(), alcohol_dropdown);

    let mut exercise_dropdown = DropdownWidget::new("Exercise", exercise_options, theme);
    exercise_dropdown.set_value("None");
    dropdowns.insert(FIELD_EXERCISE_FREQUENCY.to_string(), exercise_dropdown);

    dropdowns
}

fn build_validator() -> FormValidator {
    let mut rules = HashMap::new();
    rules.insert(
        FIELD_CIGARETTES_PER_DAY.to_string(),
        ValidationRules {
            max_length: Some(3),
            ..ValidationRules::default()
        },
    );
    rules.insert(
        FIELD_DRINKS_PER_WEEK.to_string(),
        ValidationRules {
            max_length: Some(3),
            ..ValidationRules::default()
        },
    );
    rules.insert(
        FIELD_QUIT_DATE.to_string(),
        ValidationRules {
            date_format: Some("dd/mm/yyyy".to_string()),
            ..ValidationRules::default()
        },
    );

    FormValidator::new(&rules)
}

fn format_smoking_status(status: SmokingStatus) -> String {
    match status {
        SmokingStatus::NeverSmoked => "Never smoked".to_string(),
        SmokingStatus::CurrentSmoker => "Current smoker".to_string(),
        SmokingStatus::ExSmoker => "Ex-smoker".to_string(),
    }
}

fn format_alcohol_status(status: AlcoholStatus) -> String {
    match status {
        AlcoholStatus::None => "None".to_string(),
        AlcoholStatus::Occasional => "Occasional".to_string(),
        AlcoholStatus::Moderate => "Moderate".to_string(),
        AlcoholStatus::Heavy => "Heavy".to_string(),
    }
}

fn format_exercise_frequency(freq: ExerciseFrequency) -> String {
    match freq {
        ExerciseFrequency::None => "None".to_string(),
        ExerciseFrequency::Rarely => "Rarely".to_string(),
        ExerciseFrequency::OnceOrTwicePerWeek => "1-2/week".to_string(),
        ExerciseFrequency::ThreeToFiveTimes => "3-5/week".to_string(),
        ExerciseFrequency::Daily => "Daily".to_string(),
    }
}

#[allow(dead_code)]
fn parse_smoking_status(value: &str) -> Option<SmokingStatus> {
    match value.to_lowercase().as_str() {
        "never" | "never smoked" | "neversmoker" => Some(SmokingStatus::NeverSmoked),
        "current" | "current smoker" | "currentsmoker" => Some(SmokingStatus::CurrentSmoker),
        "ex" | "ex-smoker" | "exsmoker" | "former" => Some(SmokingStatus::ExSmoker),
        _ => None,
    }
}

#[allow(dead_code)]
fn parse_alcohol_status(value: &str) -> Option<AlcoholStatus> {
    match value.to_lowercase().as_str() {
        "none" | "no" | "0" => Some(AlcoholStatus::None),
        "occasional" | "occ" => Some(AlcoholStatus::Occasional),
        "moderate" | "mod" => Some(AlcoholStatus::Moderate),
        "heavy" | "hvy" => Some(AlcoholStatus::Heavy),
        _ => None,
    }
}

#[allow(dead_code)]
fn parse_exercise_frequency(value: &str) -> Option<ExerciseFrequency> {
    match value.to_lowercase().as_str() {
        "none" | "no" | "0" => Some(ExerciseFrequency::None),
        "rarely" | "rare" => Some(ExerciseFrequency::Rarely),
        "1-2" | "1-2/week" | "once" | "twice" => Some(ExerciseFrequency::OnceOrTwicePerWeek),
        "3-5" | "3-5/week" | "threetofive" => Some(ExerciseFrequency::ThreeToFiveTimes),
        "daily" | "every day" => Some(ExerciseFrequency::Daily),
        _ => None,
    }
}

impl Widget for SocialHistoryComponent {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.is_empty() {
            return;
        }

        let title = if self.is_editing {
            " Social History [EDITING] "
        } else {
            " Social History "
        };

        let border_style = if self.is_editing {
            Style::default().fg(self.theme.colors.primary)
        } else {
            Style::default().fg(self.theme.colors.border)
        };

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(border_style);

        block.clone().render(area, buf);

        let inner = block.inner(area);
        if inner.is_empty() {
            return;
        }

        if self.loading {
            let mut loading_state = self.loading_state.clone();
            loading_state.tick();
            let indicator = loading_state.to_indicator(&self.theme);
            indicator.render(inner, buf);
            return;
        }

        if self.is_editing {
            render_edit_mode(&self, inner, buf);
        } else {
            render_view_mode(&self, inner, buf);
        }
    }
}

fn render_view_mode(component: &SocialHistoryComponent, inner: Rect, buf: &mut Buffer) {
    if let Some(ref history) = component.social_history {
        let mut y = inner.y + 1;
        let label_style = Style::default()
            .fg(component.theme.colors.primary)
            .add_modifier(Modifier::BOLD);
        let value_style = Style::default().fg(component.theme.colors.foreground);

        let lines = vec![
            Line::from(vec![
                Span::styled("Smoking: ", label_style),
                Span::styled(format_smoking_status(history.smoking_status), value_style),
            ]),
            Line::from(vec![
                Span::styled("Cigarettes/day: ", label_style),
                Span::styled(
                    history
                        .cigarettes_per_day
                        .map(|c| c.to_string())
                        .unwrap_or_else(|| "-".to_string()),
                    value_style,
                ),
            ]),
            Line::from(vec![
                Span::styled("Alcohol: ", label_style),
                Span::styled(format_alcohol_status(history.alcohol_status), value_style),
            ]),
            Line::from(vec![
                Span::styled("Drinks/week: ", label_style),
                Span::styled(
                    history
                        .standard_drinks_per_week
                        .map(|d| d.to_string())
                        .unwrap_or_else(|| "-".to_string()),
                    value_style,
                ),
            ]),
            Line::from(vec![
                Span::styled("Exercise: ", label_style),
                Span::styled(
                    history
                        .exercise_frequency
                        .map(format_exercise_frequency)
                        .unwrap_or_else(|| "-".to_string()),
                    value_style,
                ),
            ]),
            Line::from(vec![
                Span::styled("Occupation: ", label_style),
                Span::styled(history.occupation.as_deref().unwrap_or("-"), value_style),
            ]),
            Line::from(vec![
                Span::styled("Living situation: ", label_style),
                Span::styled(
                    history.living_situation.as_deref().unwrap_or("-"),
                    value_style,
                ),
            ]),
            Line::from(vec![
                Span::styled("Support network: ", label_style),
                Span::styled(
                    history.support_network.as_deref().unwrap_or("-"),
                    value_style,
                ),
            ]),
        ];

        for line in lines {
            if y < inner.y + inner.height {
                buf.set_line(inner.x + 1, y, &line, inner.width.saturating_sub(2));
                y += 1;
            }
        }

        let help_y = inner.y + inner.height.saturating_sub(1);
        if help_y > y {
            buf.set_string(
                inner.x + 1,
                help_y,
                "e: Edit",
                Style::default().fg(component.theme.colors.disabled),
            );
        }
    } else {
        let message = "No social history recorded. Press e to edit.";
        let text = Line::from(vec![Span::styled(
            message,
            Style::default().fg(component.theme.colors.disabled),
        )]);
        let x = inner.x + (inner.width.saturating_sub(message.len() as u16)) / 2;
        let y = inner.y + inner.height / 2;
        buf.set_line(x, y, &text, inner.width);
    }
}

fn render_edit_mode(component: &SocialHistoryComponent, inner: Rect, buf: &mut Buffer) {
    let fields = SocialHistoryField::all();
    let mut y = inner.y + 1;
    let max_y = inner.y + inner.height.saturating_sub(2);
    let mut open_dropdown: Option<(DropdownWidget, Rect)> = None;

    for field in &fields {
        if y > max_y {
            break;
        }

        let is_focused = field == &component.focused_field;
        if field.is_dropdown() {
            if let Some(dropdown) = component.dropdowns.get(field.id()) {
                let dropdown_area = Rect::new(inner.x + 1, y, inner.width.saturating_sub(2), 3);
                let focused_dropdown = dropdown.clone().focused(is_focused);

                if focused_dropdown.is_open() {
                    open_dropdown = Some((focused_dropdown, dropdown_area));
                } else {
                    focused_dropdown.render(dropdown_area, buf);
                }
                y += 3;
            }
        } else {
            if let Some(textarea) = component.textareas.get(field.id()) {
                let textarea_state = textarea.clone().focused(is_focused);
                let field_height = component.get_field_height(*field);
                let field_area =
                    Rect::new(inner.x + 1, y, inner.width.saturating_sub(2), field_height);
                TextareaWidget::new(&textarea_state, component.theme.clone())
                    .render(field_area, buf);
                y += field_height;
            }
        }
    }

    // Render open dropdown on top of subsequent fields
    if let Some((dropdown, dropdown_area)) = open_dropdown {
        dropdown.render(dropdown_area, buf);
    }

    let help_y = inner.y + inner.height.saturating_sub(1);
    buf.set_string(
        inner.x + 1,
        help_y,
        "Tab: Next  Ctrl+S: Save  Esc: Cancel",
        Style::default().fg(component.theme.colors.disabled),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyCode;

    #[test]
    fn test_component_construction_with_theme() {
        let theme = Theme::dark();
        let config = SocialHistoryConfig::default();
        let component = SocialHistoryComponent::new(theme.clone(), &config);

        assert!(!component.is_editing);
        assert!(component.social_history.is_none());
        assert!(!component.loading);
        assert_eq!(component.focused_field, SocialHistoryField::SmokingStatus);
    }

    #[test]
    fn test_form_state_field_values() {
        let theme = Theme::dark();
        let mut component = SocialHistoryComponent::new(theme, &SocialHistoryConfig::default());

        // Test getting default field values (config-driven forms start with placeholder)
        let smoking_val = component.get_field_value(&SocialHistoryField::SmokingStatus);
        assert_eq!(smoking_val, "Select...");

        let alcohol_val = component.get_field_value(&SocialHistoryField::AlcoholStatus);
        assert_eq!(alcohol_val, "Select...");

        let occupation_val = component.get_field_value(&SocialHistoryField::Occupation);
        assert_eq!(occupation_val, "");

        // Test setting field values
        component.set_field_value(&SocialHistoryField::Occupation, "Doctor".to_string());
        let occupation_val = component.get_field_value(&SocialHistoryField::Occupation);
        assert_eq!(occupation_val, "Doctor");
    }

    #[test]
    fn test_editing_mode_toggle() {
        let theme = Theme::dark();
        let mut component = SocialHistoryComponent::new(theme, &SocialHistoryConfig::default());

        assert!(!component.is_editing);

        // Start editing
        component.start_editing();
        assert!(component.is_editing);
        assert_eq!(component.focused_field, SocialHistoryField::SmokingStatus);

        // Stop editing
        component.stop_editing();
        assert!(!component.is_editing);
    }

    #[test]
    fn test_key_handling_tab_navigation() {
        let theme = Theme::dark();
        let mut component = SocialHistoryComponent::new(theme, &SocialHistoryConfig::default());

        component.start_editing();
        assert_eq!(component.focused_field, SocialHistoryField::SmokingStatus);

        // Tab should move to next field
        let key = KeyEvent::new(KeyCode::Tab, KeyModifiers::empty());
        let action = component.handle_key(key);
        assert_eq!(action, Some(SocialHistoryAction::FocusChanged));
        assert_eq!(
            component.focused_field,
            SocialHistoryField::CigarettesPerDay
        );

        // Shift+Tab should move to previous field
        let key = KeyEvent::new(KeyCode::BackTab, KeyModifiers::SHIFT);
        let action = component.handle_key(key);
        assert_eq!(action, Some(SocialHistoryAction::FocusChanged));
        assert_eq!(component.focused_field, SocialHistoryField::SmokingStatus);
    }

    #[test]
    fn test_key_handling_escape_cancels_editing() {
        let theme = Theme::dark();
        let mut component = SocialHistoryComponent::new(theme, &SocialHistoryConfig::default());

        component.start_editing();
        assert!(component.is_editing);

        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::empty());
        let action = component.handle_key(key);
        assert_eq!(action, Some(SocialHistoryAction::Cancel));
        assert!(!component.is_editing);
    }

    #[test]
    fn test_key_handling_ctrl_s_saves() {
        let theme = Theme::dark();
        let mut component = SocialHistoryComponent::new(theme, &SocialHistoryConfig::default());

        component.start_editing();

        let key = KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL);
        let action = component.handle_key(key);
        assert_eq!(action, Some(SocialHistoryAction::Save));
    }

    #[test]
    fn test_data_conversion_to_social_history() {
        let theme = Theme::dark();
        let mut component = SocialHistoryComponent::new(theme, &SocialHistoryConfig::default());

        // Set some field values
        component.set_field_value(&SocialHistoryField::Occupation, "Engineer".to_string());
        component.set_field_value(&SocialHistoryField::CigarettesPerDay, "5".to_string());
        component.set_field_value(&SocialHistoryField::DrinksPerWeek, "10".to_string());

        let patient_id = uuid::Uuid::new_v4();
        let updated_by = uuid::Uuid::new_v4();

        let data = component.to_social_history(patient_id, updated_by);

        assert_eq!(data.smoking_status, SmokingStatus::NeverSmoked);
        assert_eq!(data.cigarettes_per_day, Some(5));
        assert_eq!(data.standard_drinks_per_week, Some(10));
        assert_eq!(data.occupation, Some("Engineer".to_string()));
        assert_eq!(data.alcohol_status, AlcoholStatus::None);
    }

    #[test]
    fn test_start_editing_populates_fields_from_data() {
        let theme = Theme::dark();
        let mut component = SocialHistoryComponent::new(theme, &SocialHistoryConfig::default());

        // Set initial data
        let data = SocialHistoryData {
            smoking_status: SmokingStatus::CurrentSmoker,
            cigarettes_per_day: Some(20),
            smoking_quit_date: None,
            alcohol_status: AlcoholStatus::Moderate,
            standard_drinks_per_week: Some(15),
            exercise_frequency: Some(ExerciseFrequency::OnceOrTwicePerWeek),
            occupation: Some("Doctor".to_string()),
            living_situation: Some("House".to_string()),
            support_network: Some("Family".to_string()),
            notes: Some("Test notes".to_string()),
        };
        component.social_history = Some(data);

        // Start editing should populate fields
        component.start_editing();

        assert_eq!(
            component.get_field_value(&SocialHistoryField::CigarettesPerDay),
            "20"
        );
        assert_eq!(
            component.get_field_value(&SocialHistoryField::DrinksPerWeek),
            "15"
        );
        assert_eq!(
            component.get_field_value(&SocialHistoryField::Occupation),
            "Doctor"
        );
        assert_eq!(
            component.get_field_value(&SocialHistoryField::LivingSituation),
            "House"
        );
    }

    #[test]
    fn test_loading_state() {
        let theme = Theme::dark();
        let mut component = SocialHistoryComponent::new(theme, &SocialHistoryConfig::default());

        assert!(!component.is_loading());

        component.set_loading(true);
        assert!(component.is_loading());

        component.set_loading(false);
        assert!(!component.is_loading());
    }
}

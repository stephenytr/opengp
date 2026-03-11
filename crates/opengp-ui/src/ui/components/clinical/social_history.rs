use crate::ui::input::to_ratatui_key;
use crate::ui::theme::Theme;
use crate::ui::widgets::{
    format_date, parse_date, DropdownOption, DropdownWidget, HeightMode, LoadingState,
    ScrollableFormState, TextareaState, TextareaWidget,
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use opengp_domain::domain::clinical::{AlcoholStatus, ExerciseFrequency, SmokingStatus};
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::EnumIter, strum::IntoStaticStr)]
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
        use strum::IntoStaticStr;
        (*self).into()
    }

    fn hint(&self) -> &'static str {
        match self {
            SocialHistoryField::SmokingStatus => "never/current/ex",
            SocialHistoryField::CigarettesPerDay => "number or blank",
            SocialHistoryField::QuitDate => "dd/mm/yyyy or blank",
            SocialHistoryField::AlcoholStatus => "none/occasional/moderate/heavy",
            SocialHistoryField::DrinksPerWeek => "number or blank",
            SocialHistoryField::ExerciseFrequency => "none/rarely/1-2/3-5/daily",
            SocialHistoryField::Occupation => "free text",
            SocialHistoryField::LivingSituation => "free text",
            SocialHistoryField::SupportNetwork => "free text",
            SocialHistoryField::Notes => "free text",
        }
    }
}

pub struct SocialHistoryComponent {
    pub social_history: Option<SocialHistoryData>,
    pub is_editing: bool,
    pub occupation: TextareaState,
    pub cigarettes_per_day: TextareaState,
    pub living_situation: TextareaState,
    pub quit_date: TextareaState,
    pub drinks_per_week: TextareaState,
    pub support_network: TextareaState,
    pub notes: TextareaState,
    pub focused_field: SocialHistoryField,
    pub loading: bool,
    loading_state: LoadingState,
    theme: Theme,
    smoking_dropdown: DropdownWidget,
    alcohol_dropdown: DropdownWidget,
    exercise_dropdown: DropdownWidget,
    scroll: ScrollableFormState,
}

impl Clone for SocialHistoryComponent {
    fn clone(&self) -> Self {
        Self {
            social_history: self.social_history.clone(),
            is_editing: self.is_editing,
            occupation: self.occupation.clone(),
            cigarettes_per_day: self.cigarettes_per_day.clone(),
            living_situation: self.living_situation.clone(),
            quit_date: self.quit_date.clone(),
            drinks_per_week: self.drinks_per_week.clone(),
            support_network: self.support_network.clone(),
            notes: self.notes.clone(),
            focused_field: self.focused_field.clone(),
            loading: self.loading,
            loading_state: self.loading_state.clone(),
            theme: self.theme.clone(),
            smoking_dropdown: self.smoking_dropdown.clone(),
            alcohol_dropdown: self.alcohol_dropdown.clone(),
            exercise_dropdown: self.exercise_dropdown.clone(),
            scroll: self.scroll.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum SocialHistoryAction {
    Edit,
    Save,
    Cancel,
    FieldChanged,
    FocusChanged,
}

impl SocialHistoryComponent {
    pub fn new(theme: Theme) -> Self {
        let smoking_options = vec![
            DropdownOption::new("NeverSmoked", "Never smoked"),
            DropdownOption::new("CurrentSmoker", "Current smoker"),
            DropdownOption::new("ExSmoker", "Ex-smoker"),
        ];
        let alcohol_options = vec![
            DropdownOption::new("None", "None"),
            DropdownOption::new("Occasional", "Occasional"),
            DropdownOption::new("Moderate", "Moderate"),
            DropdownOption::new("Heavy", "Heavy"),
        ];
        let exercise_options = vec![
            DropdownOption::new("None", "None"),
            DropdownOption::new("Rarely", "Rarely"),
            DropdownOption::new("OnceOrTwicePerWeek", "1-2/week"),
            DropdownOption::new("ThreeToFiveTimes", "3-5/week"),
            DropdownOption::new("Daily", "Daily"),
        ];

        let mut smoking_dropdown = DropdownWidget::new("Smoking", smoking_options, theme.clone());
        smoking_dropdown.set_value("NeverSmoked");

        let mut alcohol_dropdown = DropdownWidget::new("Alcohol", alcohol_options, theme.clone());
        alcohol_dropdown.set_value("None");

        let mut exercise_dropdown =
            DropdownWidget::new("Exercise", exercise_options, theme.clone());
        exercise_dropdown.set_value("None");

        Self {
            social_history: None,
            is_editing: false,
            occupation: TextareaState::new("Occupation").with_height_mode(HeightMode::SingleLine),
            cigarettes_per_day: TextareaState::new("Cigarettes/day")
                .with_height_mode(HeightMode::SingleLine)
                .max_length(3),
            living_situation: TextareaState::new("Living situation")
                .with_height_mode(HeightMode::SingleLine),
            quit_date: TextareaState::new("Quit date (dd/mm/yyyy)")
                .with_height_mode(HeightMode::SingleLine)
                .max_length(10),
            drinks_per_week: TextareaState::new("Drinks/week")
                .with_height_mode(HeightMode::SingleLine)
                .max_length(3),
            support_network: TextareaState::new("Support network")
                .with_height_mode(HeightMode::SingleLine),
            notes: TextareaState::new("Notes").with_height_mode(HeightMode::FixedLines(4)),
            focused_field: SocialHistoryField::SmokingStatus,
            loading: false,
            loading_state: LoadingState::new().message("Loading social history..."),
            theme,
            smoking_dropdown,
            alcohol_dropdown,
            exercise_dropdown,
            scroll: ScrollableFormState::new(),
        }
    }

    pub fn is_loading(&self) -> bool {
        self.loading
    }

    pub fn set_loading(&mut self, loading: bool) {
        self.loading = loading;
    }

    pub fn start_editing(&mut self) {
        if let Some(ref history) = self.social_history {
            self.cigarettes_per_day = TextareaState::new("Cigarettes/day")
                .with_height_mode(HeightMode::SingleLine)
                .max_length(3)
                .with_value(
                    history
                        .cigarettes_per_day
                        .map(|n| n.to_string())
                        .unwrap_or_default(),
                );
            self.quit_date = TextareaState::new("Quit date (dd/mm/yyyy)")
                .with_height_mode(HeightMode::SingleLine)
                .max_length(10)
                .with_value(
                    history
                        .smoking_quit_date
                        .map(|d| format_date(d))
                        .unwrap_or_default(),
                );
            self.drinks_per_week = TextareaState::new("Drinks/week")
                .with_height_mode(HeightMode::SingleLine)
                .max_length(3)
                .with_value(
                    history
                        .standard_drinks_per_week
                        .map(|n| n.to_string())
                        .unwrap_or_default(),
                );
            self.occupation = TextareaState::new("Occupation")
                .with_height_mode(HeightMode::SingleLine)
                .with_value(history.occupation.clone().unwrap_or_default());
            self.living_situation = TextareaState::new("Living situation")
                .with_height_mode(HeightMode::SingleLine)
                .with_value(history.living_situation.clone().unwrap_or_default());
            self.support_network = TextareaState::new("Support network")
                .with_height_mode(HeightMode::SingleLine)
                .with_value(history.support_network.clone().unwrap_or_default());
            self.notes = TextareaState::new("Notes")
                .with_height_mode(HeightMode::FixedLines(4))
                .with_value(history.notes.clone().unwrap_or_default());
            self.smoking_dropdown
                .set_value(&format!("{:?}", history.smoking_status));
            self.alcohol_dropdown
                .set_value(&format!("{:?}", history.alcohol_status));
            if let Some(freq) = history.exercise_frequency {
                self.exercise_dropdown.set_value(&format!("{:?}", freq));
            }
        }
        self.focused_field = SocialHistoryField::SmokingStatus;
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
            match key.code {
                KeyCode::Char('e') | KeyCode::Char('n') => {
                    self.start_editing();
                    Some(SocialHistoryAction::Edit)
                }
                _ => None,
            }
        } else {
            // Ctrl+S saves the form
            if key.modifiers.contains(KeyModifiers::CONTROL)
                && matches!(key.code, KeyCode::Char('s'))
            {
                return Some(SocialHistoryAction::Save);
            }

            let is_dropdown_field = matches!(
                self.focused_field,
                SocialHistoryField::SmokingStatus
                    | SocialHistoryField::AlcoholStatus
                    | SocialHistoryField::ExerciseFrequency
            );
            if is_dropdown_field {
                let dropdown_consumed = match self.focused_field {
                    SocialHistoryField::SmokingStatus => self.smoking_dropdown.handle_key(key),
                    SocialHistoryField::AlcoholStatus => self.alcohol_dropdown.handle_key(key),
                    SocialHistoryField::ExerciseFrequency => self.exercise_dropdown.handle_key(key),
                    _ => None,
                };
                if let Some(_action) = dropdown_consumed {
                    match key.code {
                        KeyCode::Tab | KeyCode::BackTab | KeyCode::Esc => {}
                        _ => return Some(SocialHistoryAction::FieldChanged),
                    }
                }
            }

            match self.focused_field {
                SocialHistoryField::Occupation => {
                    let ratatui_key = to_ratatui_key(key);
                    let consumed = self.occupation.handle_key(ratatui_key);
                    if consumed {
                        return Some(SocialHistoryAction::FieldChanged);
                    }
                }
                SocialHistoryField::CigarettesPerDay => {
                    let ratatui_key = to_ratatui_key(key);
                    let consumed = self.cigarettes_per_day.handle_key(ratatui_key);
                    if consumed {
                        return Some(SocialHistoryAction::FieldChanged);
                    }
                }
                SocialHistoryField::LivingSituation => {
                    let ratatui_key = to_ratatui_key(key);
                    let consumed = self.living_situation.handle_key(ratatui_key);
                    if consumed {
                        return Some(SocialHistoryAction::FieldChanged);
                    }
                }
                SocialHistoryField::QuitDate => {
                    let ratatui_key = to_ratatui_key(key);
                    let consumed = self.quit_date.handle_key(ratatui_key);
                    if consumed {
                        return Some(SocialHistoryAction::FieldChanged);
                    }
                }
                SocialHistoryField::DrinksPerWeek => {
                    let ratatui_key = to_ratatui_key(key);
                    let consumed = self.drinks_per_week.handle_key(ratatui_key);
                    if consumed {
                        return Some(SocialHistoryAction::FieldChanged);
                    }
                }
                SocialHistoryField::SupportNetwork => {
                    let ratatui_key = to_ratatui_key(key);
                    let consumed = self.support_network.handle_key(ratatui_key);
                    if consumed {
                        return Some(SocialHistoryAction::FieldChanged);
                    }
                }
                SocialHistoryField::Notes => {
                    let ratatui_key = to_ratatui_key(key);
                    let consumed = self.notes.handle_key(ratatui_key);
                    if consumed {
                        return Some(SocialHistoryAction::FieldChanged);
                    }
                }
                _ => {}
            }

            match key.code {
                KeyCode::Tab => {
                    if key.modifiers.contains(KeyModifiers::SHIFT) {
                        self.prev_field();
                    } else {
                        self.next_field();
                    }
                    Some(SocialHistoryAction::FocusChanged)
                }
                KeyCode::BackTab => {
                    self.prev_field();
                    Some(SocialHistoryAction::FocusChanged)
                }
                KeyCode::Up => {
                    self.prev_field();
                    Some(SocialHistoryAction::FocusChanged)
                }
                KeyCode::Down => {
                    self.next_field();
                    Some(SocialHistoryAction::FocusChanged)
                }
                KeyCode::PageUp => {
                    self.scroll.scroll_up();
                    Some(SocialHistoryAction::FocusChanged)
                }
                KeyCode::PageDown => {
                    self.scroll.scroll_down();
                    Some(SocialHistoryAction::FocusChanged)
                }
                KeyCode::Enter => None,
                KeyCode::Esc => {
                    self.stop_editing();
                    Some(SocialHistoryAction::Cancel)
                }
                _ => None,
            }
        }
    }

    fn next_field(&mut self) {
        let fields = SocialHistoryField::all();
        let current = fields
            .iter()
            .position(|f| f == &self.focused_field)
            .unwrap_or(0);
        self.focused_field = fields[(current + 1) % fields.len()].clone();
    }

    fn prev_field(&mut self) {
        let fields = SocialHistoryField::all();
        let current = fields
            .iter()
            .position(|f| f == &self.focused_field)
            .unwrap_or(0);
        self.focused_field = fields[(current + fields.len() - 1) % fields.len()].clone();
    }

    pub fn get_field_value(&self, field: &SocialHistoryField) -> String {
        match field {
            SocialHistoryField::SmokingStatus => self
                .smoking_dropdown
                .selected_label()
                .unwrap_or("Select...")
                .to_string(),
            SocialHistoryField::CigarettesPerDay => self.cigarettes_per_day.value(),
            SocialHistoryField::QuitDate => self.quit_date.value(),
            SocialHistoryField::AlcoholStatus => self
                .alcohol_dropdown
                .selected_label()
                .unwrap_or("Select...")
                .to_string(),
            SocialHistoryField::DrinksPerWeek => self.drinks_per_week.value(),
            SocialHistoryField::ExerciseFrequency => self
                .exercise_dropdown
                .selected_label()
                .unwrap_or("Select...")
                .to_string(),
            SocialHistoryField::Occupation => self.occupation.value(),
            SocialHistoryField::LivingSituation => self.living_situation.value(),
            SocialHistoryField::SupportNetwork => self.support_network.value(),
            SocialHistoryField::Notes => self.notes.value(),
        }
    }

    pub fn set_field_value(&mut self, field: &SocialHistoryField, value: String) {
        match field {
            SocialHistoryField::SmokingStatus => {
                self.smoking_dropdown.set_value(&value);
            }
            SocialHistoryField::CigarettesPerDay => {
                self.cigarettes_per_day = TextareaState::new("Cigarettes/day")
                    .with_height_mode(HeightMode::SingleLine)
                    .max_length(3)
                    .with_value(value);
            }
            SocialHistoryField::QuitDate => {
                self.quit_date = TextareaState::new("Quit date (dd/mm/yyyy)")
                    .with_height_mode(HeightMode::SingleLine)
                    .max_length(10)
                    .with_value(value);
            }
            SocialHistoryField::AlcoholStatus => {
                self.alcohol_dropdown.set_value(&value);
            }
            SocialHistoryField::DrinksPerWeek => {
                self.drinks_per_week = TextareaState::new("Drinks/week")
                    .with_height_mode(HeightMode::SingleLine)
                    .max_length(3)
                    .with_value(value);
            }
            SocialHistoryField::ExerciseFrequency => {
                self.exercise_dropdown.set_value(&value);
            }
            SocialHistoryField::Occupation => {
                self.occupation = TextareaState::new("Occupation")
                    .with_height_mode(HeightMode::SingleLine)
                    .with_value(value);
            }
            SocialHistoryField::LivingSituation => {
                self.living_situation = TextareaState::new("Living situation")
                    .with_height_mode(HeightMode::SingleLine)
                    .with_value(value);
            }
            SocialHistoryField::SupportNetwork => {
                self.support_network = TextareaState::new("Support network")
                    .with_height_mode(HeightMode::SingleLine)
                    .with_value(value);
            }
            SocialHistoryField::Notes => {
                self.notes = TextareaState::new("Notes")
                    .with_height_mode(HeightMode::FixedLines(4))
                    .with_value(value);
            }
        }
    }

    pub fn to_social_history(
        &self,
        _patient_id: uuid::Uuid,
        _updated_by: uuid::Uuid,
    ) -> SocialHistoryData {
        let smoking_status = self
            .smoking_dropdown
            .selected_value()
            .and_then(|v: &str| v.parse::<SmokingStatus>().ok())
            .unwrap_or(SmokingStatus::NeverSmoked);

        let alcohol_status = self
            .alcohol_dropdown
            .selected_value()
            .and_then(|v: &str| v.parse::<AlcoholStatus>().ok())
            .unwrap_or(AlcoholStatus::None);

        let exercise_frequency = self
            .exercise_dropdown
            .selected_value()
            .and_then(|v: &str| v.parse::<ExerciseFrequency>().ok());

        let cigarettes_value = self.cigarettes_per_day.value();
        let drinks_value = self.drinks_per_week.value();
        let quit_date_value = self.quit_date.value();
        let occupation_value = self.occupation.value();
        let living_situation_value = self.living_situation.value();
        let support_network_value = self.support_network.value();
        let notes_value = self.notes.value();

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
            let indicator = loading_state.to_indicator(self.theme.clone());
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
                        .map(|e| format_exercise_frequency(e))
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
        let is_dropdown_field = matches!(
            field,
            SocialHistoryField::SmokingStatus
                | SocialHistoryField::AlcoholStatus
                | SocialHistoryField::ExerciseFrequency
        );

        if is_dropdown_field {
            let dropdown = match field {
                SocialHistoryField::SmokingStatus => &component.smoking_dropdown,
                SocialHistoryField::AlcoholStatus => &component.alcohol_dropdown,
                SocialHistoryField::ExerciseFrequency => &component.exercise_dropdown,
                _ => unreachable!(),
            };

            let dropdown_area = Rect::new(inner.x + 1, y, inner.width.saturating_sub(2), 3);
            let dropdown_clone = dropdown.clone();
            let focused_dropdown = dropdown_clone.focused(is_focused);

            if focused_dropdown.is_open() {
                open_dropdown = Some((focused_dropdown, dropdown_area));
            } else {
                focused_dropdown.render(dropdown_area, buf);
            }
            y += 3;
        } else {
            let (textarea_state, field_height) = match field {
                SocialHistoryField::Occupation => (
                    component.occupation.clone().focused(is_focused),
                    component.occupation.height(),
                ),
                SocialHistoryField::CigarettesPerDay => (
                    component.cigarettes_per_day.clone().focused(is_focused),
                    component.cigarettes_per_day.height(),
                ),
                SocialHistoryField::LivingSituation => (
                    component.living_situation.clone().focused(is_focused),
                    component.living_situation.height(),
                ),
                SocialHistoryField::QuitDate => (
                    component.quit_date.clone().focused(is_focused),
                    component.quit_date.height(),
                ),
                SocialHistoryField::DrinksPerWeek => (
                    component.drinks_per_week.clone().focused(is_focused),
                    component.drinks_per_week.height(),
                ),
                SocialHistoryField::SupportNetwork => (
                    component.support_network.clone().focused(is_focused),
                    component.support_network.height(),
                ),
                SocialHistoryField::Notes => (
                    component.notes.clone().focused(is_focused),
                    component.notes.height(),
                ),
                _ => unreachable!(),
            };

            let field_area = Rect::new(inner.x + 1, y, inner.width.saturating_sub(2), field_height);
            TextareaWidget::new(&textarea_state, component.theme.clone()).render(field_area, buf);
            y += field_height;
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

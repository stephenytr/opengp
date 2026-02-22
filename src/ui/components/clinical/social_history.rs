use crate::domain::clinical::{AlcoholStatus, ExerciseFrequency, SmokingStatus};
use crate::ui::theme::Theme;
use crate::ui::widgets::{DropdownOption, DropdownWidget, LoadingState};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
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

#[derive(Debug, Clone, PartialEq)]
pub enum SocialHistoryField {
    SmokingStatus,
    CigarettesPerDay,
    QuitDate,
    AlcoholStatus,
    DrinksPerWeek,
    ExerciseFrequency,
    Occupation,
    LivingSituation,
    SupportNetwork,
    Notes,
}

impl SocialHistoryField {
    fn all() -> Vec<SocialHistoryField> {
        vec![
            SocialHistoryField::SmokingStatus,
            SocialHistoryField::CigarettesPerDay,
            SocialHistoryField::QuitDate,
            SocialHistoryField::AlcoholStatus,
            SocialHistoryField::DrinksPerWeek,
            SocialHistoryField::ExerciseFrequency,
            SocialHistoryField::Occupation,
            SocialHistoryField::LivingSituation,
            SocialHistoryField::SupportNetwork,
            SocialHistoryField::Notes,
        ]
    }

    fn label(&self) -> &'static str {
        match self {
            SocialHistoryField::SmokingStatus => "Smoking",
            SocialHistoryField::CigarettesPerDay => "Cigarettes/day",
            SocialHistoryField::QuitDate => "Quit date",
            SocialHistoryField::AlcoholStatus => "Alcohol",
            SocialHistoryField::DrinksPerWeek => "Drinks/week",
            SocialHistoryField::ExerciseFrequency => "Exercise",
            SocialHistoryField::Occupation => "Occupation",
            SocialHistoryField::LivingSituation => "Living situation",
            SocialHistoryField::SupportNetwork => "Support network",
            SocialHistoryField::Notes => "Notes",
        }
    }

    fn hint(&self) -> &'static str {
        match self {
            SocialHistoryField::SmokingStatus => "never/current/ex",
            SocialHistoryField::CigarettesPerDay => "number or blank",
            SocialHistoryField::QuitDate => "YYYY-MM-DD or blank",
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
    pub cigarettes_per_day: Option<u8>,
    pub quit_date: Option<String>,
    pub drinks_per_week: Option<u8>,
    pub occupation: String,
    pub living_situation: String,
    pub support_network: String,
    pub notes: String,
    pub focused_field: SocialHistoryField,
    pub loading: bool,
    loading_state: LoadingState,
    theme: Theme,
    smoking_dropdown: DropdownWidget,
    alcohol_dropdown: DropdownWidget,
    exercise_dropdown: DropdownWidget,
}

impl Clone for SocialHistoryComponent {
    fn clone(&self) -> Self {
        Self {
            social_history: self.social_history.clone(),
            is_editing: self.is_editing,
            cigarettes_per_day: self.cigarettes_per_day,
            quit_date: self.quit_date.clone(),
            drinks_per_week: self.drinks_per_week,
            occupation: self.occupation.clone(),
            living_situation: self.living_situation.clone(),
            support_network: self.support_network.clone(),
            notes: self.notes.clone(),
            focused_field: self.focused_field.clone(),
            loading: self.loading,
            loading_state: self.loading_state.clone(),
            theme: self.theme.clone(),
            smoking_dropdown: self.smoking_dropdown.clone(),
            alcohol_dropdown: self.alcohol_dropdown.clone(),
            exercise_dropdown: self.exercise_dropdown.clone(),
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
            cigarettes_per_day: None,
            quit_date: None,
            drinks_per_week: None,
            occupation: String::new(),
            living_situation: String::new(),
            support_network: String::new(),
            notes: String::new(),
            focused_field: SocialHistoryField::SmokingStatus,
            loading: false,
            loading_state: LoadingState::new().message("Loading social history..."),
            theme,
            smoking_dropdown,
            alcohol_dropdown,
            exercise_dropdown,
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
            self.cigarettes_per_day = history.cigarettes_per_day;
            self.quit_date = history
                .smoking_quit_date
                .map(|d| d.format("%Y-%m-%d").to_string());
            self.drinks_per_week = history.standard_drinks_per_week;
            self.occupation = history.occupation.clone().unwrap_or_default();
            self.living_situation = history.living_situation.clone().unwrap_or_default();
            self.support_network = history.support_network.clone().unwrap_or_default();
            self.notes = history.notes.clone().unwrap_or_default();
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
        if !self.is_editing {
            match key.code {
                KeyCode::Char('e') | KeyCode::Char('n') => {
                    self.start_editing();
                    Some(SocialHistoryAction::Edit)
                }
                _ => None,
            }
        } else {
            let is_dropdown_field = matches!(
                self.focused_field,
                SocialHistoryField::SmokingStatus
                    | SocialHistoryField::AlcoholStatus
                    | SocialHistoryField::ExerciseFrequency
            );

            if is_dropdown_field {
                let dropdown = match self.focused_field {
                    SocialHistoryField::SmokingStatus => &mut self.smoking_dropdown,
                    SocialHistoryField::AlcoholStatus => &mut self.alcohol_dropdown,
                    SocialHistoryField::ExerciseFrequency => &mut self.exercise_dropdown,
                    _ => unreachable!(),
                };

                if let Some(action) = dropdown.handle_key(key) {
                    return Some(SocialHistoryAction::FieldChanged);
                }

                match key.code {
                    KeyCode::Tab => {
                        if key.modifiers.contains(KeyModifiers::SHIFT) {
                            self.prev_field();
                        } else {
                            self.next_field();
                        }
                        return Some(SocialHistoryAction::FocusChanged);
                    }
                    KeyCode::Up | KeyCode::Down => {
                        return Some(SocialHistoryAction::FocusChanged);
                    }
                    _ => {}
                }
                return None;
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
                KeyCode::Up => {
                    self.prev_field();
                    Some(SocialHistoryAction::FocusChanged)
                }
                KeyCode::Down => {
                    self.next_field();
                    Some(SocialHistoryAction::FocusChanged)
                }
                KeyCode::Enter => Some(SocialHistoryAction::Save),
                KeyCode::Esc => {
                    self.stop_editing();
                    Some(SocialHistoryAction::Cancel)
                }
                KeyCode::Char(c) => {
                    let mut value = self.get_field_value(&self.focused_field.clone());
                    value.push(c);
                    self.set_field_value(&self.focused_field.clone(), value);
                    Some(SocialHistoryAction::FieldChanged)
                }
                KeyCode::Backspace => {
                    let mut value = self.get_field_value(&self.focused_field.clone());
                    value.pop();
                    self.set_field_value(&self.focused_field.clone(), value);
                    Some(SocialHistoryAction::FieldChanged)
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
            SocialHistoryField::CigarettesPerDay => self
                .cigarettes_per_day
                .map(|n| n.to_string())
                .unwrap_or_default(),
            SocialHistoryField::QuitDate => self.quit_date.clone().unwrap_or_default(),
            SocialHistoryField::AlcoholStatus => self
                .alcohol_dropdown
                .selected_label()
                .unwrap_or("Select...")
                .to_string(),
            SocialHistoryField::DrinksPerWeek => self
                .drinks_per_week
                .map(|n| n.to_string())
                .unwrap_or_default(),
            SocialHistoryField::ExerciseFrequency => self
                .exercise_dropdown
                .selected_label()
                .unwrap_or("Select...")
                .to_string(),
            SocialHistoryField::Occupation => self.occupation.clone(),
            SocialHistoryField::LivingSituation => self.living_situation.clone(),
            SocialHistoryField::SupportNetwork => self.support_network.clone(),
            SocialHistoryField::Notes => self.notes.clone(),
        }
    }

    pub fn set_field_value(&mut self, field: &SocialHistoryField, value: String) {
        match field {
            SocialHistoryField::SmokingStatus => {
                self.smoking_dropdown.set_value(&value);
            }
            SocialHistoryField::CigarettesPerDay => {
                self.cigarettes_per_day = value.parse::<u8>().ok();
            }
            SocialHistoryField::QuitDate => {
                self.quit_date = if value.is_empty() { None } else { Some(value) };
            }
            SocialHistoryField::AlcoholStatus => {
                self.alcohol_dropdown.set_value(&value);
            }
            SocialHistoryField::DrinksPerWeek => {
                self.drinks_per_week = value.parse::<u8>().ok();
            }
            SocialHistoryField::ExerciseFrequency => {
                self.exercise_dropdown.set_value(&value);
            }
            SocialHistoryField::Occupation => {
                self.occupation = value;
            }
            SocialHistoryField::LivingSituation => {
                self.living_situation = value;
            }
            SocialHistoryField::SupportNetwork => {
                self.support_network = value;
            }
            SocialHistoryField::Notes => {
                self.notes = value;
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

        SocialHistoryData {
            smoking_status,
            cigarettes_per_day: self.cigarettes_per_day,
            smoking_quit_date: None,
            alcohol_status,
            standard_drinks_per_week: self.drinks_per_week,
            exercise_frequency,
            occupation: Some(self.occupation.clone()).filter(|s| !s.is_empty()),
            living_situation: Some(self.living_situation.clone()).filter(|s| !s.is_empty()),
            support_network: Some(self.support_network.clone()).filter(|s| !s.is_empty()),
            notes: Some(self.notes.clone()).filter(|s| !s.is_empty()),
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

fn parse_smoking_status(value: &str) -> Option<SmokingStatus> {
    match value.to_lowercase().as_str() {
        "never" | "never smoked" | "neversmoker" => Some(SmokingStatus::NeverSmoked),
        "current" | "current smoker" | "currentsmoker" => Some(SmokingStatus::CurrentSmoker),
        "ex" | "ex-smoker" | "exsmoker" | "former" => Some(SmokingStatus::ExSmoker),
        _ => None,
    }
}

fn parse_alcohol_status(value: &str) -> Option<AlcoholStatus> {
    match value.to_lowercase().as_str() {
        "none" | "no" | "0" => Some(AlcoholStatus::None),
        "occasional" | "occ" => Some(AlcoholStatus::Occasional),
        "moderate" | "mod" => Some(AlcoholStatus::Moderate),
        "heavy" | "hvy" => Some(AlcoholStatus::Heavy),
        _ => None,
    }
}

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
    const LABEL_WIDTH: u16 = 18;
    let field_start = inner.x + LABEL_WIDTH + 2;
    let max_value_width = inner.width.saturating_sub(LABEL_WIDTH + 4);

    let fields = SocialHistoryField::all();
    let mut y = inner.y + 1;
    let max_y = inner.y + inner.height.saturating_sub(2);

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

        let label_style = if is_focused {
            Style::default()
                .fg(component.theme.colors.primary)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(component.theme.colors.foreground)
        };

        buf.set_string(inner.x + 1, y, field.label(), label_style);

        if is_focused && !is_dropdown_field {
            buf.set_string(
                field_start - 1,
                y,
                ">",
                Style::default().fg(component.theme.colors.primary),
            );
        }

        if is_dropdown_field {
            let dropdown = match field {
                SocialHistoryField::SmokingStatus => &component.smoking_dropdown,
                SocialHistoryField::AlcoholStatus => &component.alcohol_dropdown,
                SocialHistoryField::ExerciseFrequency => &component.exercise_dropdown,
                _ => unreachable!(),
            };

            let dropdown_area = Rect::new(field_start, y, max_value_width + 1, 3);
            let mut dropdown_clone = dropdown.clone();
            dropdown_clone.render(dropdown_area, buf);
        } else {
            let value = component.get_field_value(field);
            let value_style = Style::default().fg(component.theme.colors.foreground);

            let display_value = if value.len() > max_value_width as usize {
                value[value.len() - max_value_width as usize..].to_string()
            } else {
                value.clone()
            };

            let display_with_cursor = if is_focused {
                format!("{}_", display_value)
            } else {
                display_value
            };

            buf.set_string(field_start, y, &display_with_cursor, value_style);

            if is_focused {
                let hint = field.hint();
                let hint_x = field_start + value.len() as u16 + 2;
                if hint_x < inner.x + inner.width {
                    buf.set_string(
                        hint_x,
                        y,
                        format!("({})", hint),
                        Style::default().fg(component.theme.colors.disabled),
                    );
                }
            }
        }

        y += 1;
    }

    let help_y = inner.y + inner.height.saturating_sub(1);
    buf.set_string(
        inner.x + 1,
        help_y,
        "Tab: Next  Enter: Save  Esc: Cancel",
        Style::default().fg(component.theme.colors.disabled),
    );
}

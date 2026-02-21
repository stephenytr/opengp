use crate::domain::clinical::{AlcoholStatus, ExerciseFrequency, SmokingStatus};
use crate::ui::theme::Theme;
use crate::ui::widgets::LoadingState;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::Stylize;
use ratatui::style::Style;
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

#[derive(Debug, Clone)]
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

pub struct SocialHistoryComponent {
    pub social_history: Option<SocialHistoryData>,
    pub is_editing: bool,
    pub smoking_status: Option<SmokingStatus>,
    pub cigarettes_per_day: Option<u8>,
    pub quit_date: Option<String>,
    pub alcohol_status: Option<AlcoholStatus>,
    pub drinks_per_week: Option<u8>,
    pub exercise_frequency: Option<ExerciseFrequency>,
    pub occupation: String,
    pub living_situation: String,
    pub support_network: String,
    pub notes: String,
    pub focused_field: SocialHistoryField,
    pub loading: bool,
    loading_state: LoadingState,
    theme: Theme,
}

impl Clone for SocialHistoryComponent {
    fn clone(&self) -> Self {
        Self {
            social_history: self.social_history.clone(),
            is_editing: self.is_editing,
            smoking_status: self.smoking_status,
            cigarettes_per_day: self.cigarettes_per_day,
            quit_date: self.quit_date.clone(),
            alcohol_status: self.alcohol_status,
            drinks_per_week: self.drinks_per_week,
            exercise_frequency: self.exercise_frequency,
            occupation: self.occupation.clone(),
            living_situation: self.living_situation.clone(),
            support_network: self.support_network.clone(),
            notes: self.notes.clone(),
            focused_field: self.focused_field.clone(),
            loading: self.loading,
            loading_state: self.loading_state.clone(),
            theme: self.theme.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum SocialHistoryAction {
    Edit,
    Save,
    Cancel,
}

impl SocialHistoryComponent {
    pub fn new(theme: Theme) -> Self {
        Self {
            social_history: None,
            is_editing: false,
            smoking_status: Some(SmokingStatus::NeverSmoked),
            cigarettes_per_day: None,
            quit_date: None,
            alcohol_status: Some(AlcoholStatus::None),
            drinks_per_week: None,
            exercise_frequency: Some(ExerciseFrequency::None),
            occupation: String::new(),
            living_situation: String::new(),
            support_network: String::new(),
            notes: String::new(),
            focused_field: SocialHistoryField::SmokingStatus,
            loading: false,
            loading_state: LoadingState::new().message("Loading social history..."),
            theme,
        }
    }

    pub fn is_loading(&self) -> bool {
        self.loading
    }

    pub fn set_loading(&mut self, loading: bool) {
        self.loading = loading;
    }

    pub fn start_editing(&mut self) {
        self.is_editing = true;
    }

    pub fn stop_editing(&mut self) {
        self.is_editing = false;
    }

    pub fn to_social_history(
        &self,
        patient_id: uuid::Uuid,
        updated_by: uuid::Uuid,
    ) -> SocialHistoryData {
        SocialHistoryData {
            smoking_status: self.smoking_status.unwrap_or(SmokingStatus::NeverSmoked),
            cigarettes_per_day: self.cigarettes_per_day,
            smoking_quit_date: None,
            alcohol_status: self.alcohol_status.unwrap_or(AlcoholStatus::None),
            standard_drinks_per_week: self.drinks_per_week,
            exercise_frequency: self.exercise_frequency,
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

impl Widget for SocialHistoryComponent {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.is_empty() {
            return;
        }

        let block = Block::default()
            .title(" Social History ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.colors.border));

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

        if let Some(ref history) = self.social_history {
            let mut y = inner.y + 1;
            let label_style = Style::default().fg(self.theme.colors.primary).bold();
            let value_style = Style::default().fg(self.theme.colors.foreground);

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
        } else {
            let message = "No social history recorded. Press e to edit.";
            let text = Line::from(vec![Span::styled(
                message,
                Style::default().fg(self.theme.colors.disabled),
            )]);
            let x = inner.x + (inner.width.saturating_sub(message.len() as u16)) / 2;
            let y = inner.y + inner.height / 2;
            buf.set_line(x, y, &text, inner.width);
        }
    }
}

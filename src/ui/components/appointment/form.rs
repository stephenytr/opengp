//! Appointment Creation Form Component
//!
//! Single-page form for creating new appointments.
//! Follows the PatientForm pattern with Tab/Shift+Tab navigation.

use std::collections::HashMap;

use chrono::NaiveTime;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Borders, Widget};
use uuid::Uuid;

use crate::domain::appointment::{AppointmentType, NewAppointmentData};
use crate::ui::input::to_ratatui_key;
use crate::ui::layout::LABEL_WIDTH;
use crate::ui::theme::Theme;
use crate::ui::view_models::{PatientListItem, PractitionerViewItem};
use crate::ui::widgets::{
    parse_date, DropdownAction, DropdownOption, DropdownWidget, HeightMode, SearchableListAction,
    SearchableListState, TextareaState, TextareaWidget,
};

/// All fields in the appointment creation form, in tab order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AppointmentFormField {
    /// Patient search/select (displays name, stores UUID)
    Patient,
    /// Practitioner dropdown (displays name, stores UUID)
    Practitioner,
    /// Appointment date (YYYY-MM-DD)
    Date,
    /// Start time (HH:MM, 24-hour)
    StartTime,
    /// Duration in minutes
    Duration,
    /// Appointment type (Standard, Long, Brief, etc.)
    AppointmentType,
    /// Reason for visit (optional)
    Reason,
    /// Internal notes (optional)
    Notes,
}

impl AppointmentFormField {
    pub fn all() -> Vec<AppointmentFormField> {
        vec![
            AppointmentFormField::Patient,
            AppointmentFormField::Practitioner,
            AppointmentFormField::Date,
            AppointmentFormField::StartTime,
            AppointmentFormField::Duration,
            AppointmentFormField::AppointmentType,
            AppointmentFormField::Reason,
            AppointmentFormField::Notes,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            AppointmentFormField::Patient => "Patient *",
            AppointmentFormField::Practitioner => "Practitioner *",
            AppointmentFormField::Date => "Date * (dd/mm/yyyy)",
            AppointmentFormField::StartTime => "Start Time * (HH:MM)",
            AppointmentFormField::Duration => "Duration (minutes)",
            AppointmentFormField::AppointmentType => "Type *",
            AppointmentFormField::Reason => "Reason",
            AppointmentFormField::Notes => "Notes",
        }
    }

    pub fn is_required(&self) -> bool {
        matches!(
            self,
            AppointmentFormField::Patient
                | AppointmentFormField::Practitioner
                | AppointmentFormField::Date
                | AppointmentFormField::StartTime
                | AppointmentFormField::AppointmentType
        )
    }

    pub fn is_textarea(&self) -> bool {
        matches!(
            self,
            AppointmentFormField::Date
                | AppointmentFormField::StartTime
                | AppointmentFormField::Reason
                | AppointmentFormField::Notes
        )
    }
}

/// Actions returned by the appointment form's key handler.
#[derive(Debug, Clone)]
pub enum AppointmentFormAction {
    /// Focus moved to a different field
    FocusChanged,
    /// A field value was edited
    ValueChanged,
    /// User pressed Enter — form should be validated and submitted
    Submit,
    /// User pressed Esc — form should be dismissed
    Cancel,
    /// Async save completed (set externally by the caller)
    SaveComplete,
}

#[derive(Debug, Clone)]
pub struct AppointmentFormData {
    pub patient_id: Option<Uuid>,
    pub patient_display: String,
    pub practitioner_id: Option<Uuid>,
    pub practitioner_display: String,
    pub duration: String,
    pub appointment_type: String,
}

impl AppointmentFormData {
    pub fn empty() -> Self {
        Self {
            patient_id: None,
            patient_display: String::new(),
            practitioner_id: None,
            practitioner_display: String::new(),
            duration: "15".to_string(),
            appointment_type: "Standard".to_string(),
        }
    }
}

/// Appointment creation form widget.
///
/// Mirrors the PatientForm pattern: Tab/Shift+Tab to navigate fields,
/// Enter to submit, Esc to cancel.  Validation runs on submit and
/// highlights required fields that are missing or malformed.
pub struct AppointmentForm {
    data: AppointmentFormData,
    date: TextareaState,
    start_time: TextareaState,
    reason: TextareaState,
    notes: TextareaState,
    errors: HashMap<AppointmentFormField, String>,
    focused_field: AppointmentFormField,
    saving: bool,
    theme: Theme,
    type_dropdown: DropdownWidget,
    patient_picker: SearchableListState<PatientListItem>,
    practitioner_picker: SearchableListState<PractitionerViewItem>,
}

impl Clone for AppointmentForm {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
            date: self.date.clone(),
            start_time: self.start_time.clone(),
            reason: self.reason.clone(),
            notes: self.notes.clone(),
            errors: self.errors.clone(),
            focused_field: self.focused_field,
            saving: self.saving,
            theme: self.theme.clone(),
            type_dropdown: self.type_dropdown.clone(),
            patient_picker: self.patient_picker.clone(),
            practitioner_picker: self.practitioner_picker.clone(),
        }
    }
}

impl AppointmentForm {
    pub fn new(theme: Theme) -> Self {
        let type_options = vec![
            DropdownOption::new("Standard", "Standard (15 min)"),
            DropdownOption::new("Long", "Long (30 min)"),
            DropdownOption::new("Brief", "Brief (10 min)"),
            DropdownOption::new("NewPatient", "New Patient (45 min)"),
            DropdownOption::new("HealthAssessment", "Health Assessment"),
            DropdownOption::new("ChronicDiseaseReview", "Chronic Disease Review"),
            DropdownOption::new("MentalHealthPlan", "Mental Health Plan"),
            DropdownOption::new("Immunisation", "Immunisation"),
            DropdownOption::new("Procedure", "Procedure"),
            DropdownOption::new("Telephone", "Telephone"),
            DropdownOption::new("Telehealth", "Telehealth"),
            DropdownOption::new("HomeVisit", "Home Visit"),
            DropdownOption::new("Emergency", "Emergency"),
        ];

        let mut type_dropdown = DropdownWidget::new("Type *", type_options, theme.clone());
        type_dropdown.set_value("Standard");

        Self {
            data: AppointmentFormData::empty(),
            date: TextareaState::new("Date * (YYYY-MM-DD)")
                .with_height_mode(HeightMode::SingleLine),
            start_time: TextareaState::new("Start Time * (HH:MM)")
                .with_height_mode(HeightMode::SingleLine),
            reason: TextareaState::new("Reason").with_height_mode(HeightMode::SingleLine),
            notes: TextareaState::new("Notes").with_height_mode(HeightMode::FixedLines(3)),
            errors: HashMap::new(),
            focused_field: AppointmentFormField::Patient,
            saving: false,
            theme: theme.clone(),
            type_dropdown,
            patient_picker: SearchableListState::new(Vec::new()),
            practitioner_picker: SearchableListState::new(Vec::new()),
        }
    }

    // ── Field accessors ──────────────────────────────────────────────────────

    pub fn focused_field(&self) -> AppointmentFormField {
        self.focused_field
    }

    pub fn set_focus(&mut self, field: AppointmentFormField) {
        self.focused_field = field;
    }

    pub fn is_saving(&self) -> bool {
        self.saving
    }

    pub fn set_saving(&mut self, saving: bool) {
        self.saving = saving;
    }

    /// Set the selected patient (called after patient search resolves).
    pub fn set_patient(&mut self, id: Uuid, display_name: String) {
        self.data.patient_id = Some(id);
        self.data.patient_display = display_name;
        self.errors.remove(&AppointmentFormField::Patient);
    }

    /// Set patients available for selection in the picker
    pub fn set_patients(&mut self, patients: Vec<PatientListItem>) {
        self.patient_picker.set_items(patients);
    }

    /// Check if patient picker is open
    pub fn is_patient_picker_open(&self) -> bool {
        self.patient_picker.is_open()
    }

    /// Set practitioners available for selection in the picker
    pub fn set_practitioners(&mut self, practitioners: Vec<PractitionerViewItem>) {
        self.practitioner_picker.set_items(practitioners);
    }

    /// Check if practitioner picker is open
    pub fn is_practitioner_picker_open(&self) -> bool {
        self.practitioner_picker.is_open()
    }

    /// Set the selected practitioner (called after practitioner list resolves).
    pub fn set_practitioner(&mut self, id: Uuid, display_name: String) {
        self.data.practitioner_id = Some(id);
        self.data.practitioner_display = display_name;
        self.errors.remove(&AppointmentFormField::Practitioner);
    }

    pub fn get_value(&self, field: AppointmentFormField) -> String {
        match field {
            AppointmentFormField::Patient => self.data.patient_display.clone(),
            AppointmentFormField::Practitioner => self.data.practitioner_display.clone(),
            AppointmentFormField::Date => self.date.value(),
            AppointmentFormField::StartTime => self.start_time.value(),
            AppointmentFormField::Duration => self.data.duration.clone(),
            AppointmentFormField::AppointmentType => self
                .type_dropdown
                .selected_value()
                .map(|s: &str| s.to_string())
                .unwrap_or_default(),
            AppointmentFormField::Reason => self.reason.value(),
            AppointmentFormField::Notes => self.notes.value(),
        }
    }

    pub fn set_value(&mut self, field: AppointmentFormField, value: String) {
        match field {
            AppointmentFormField::Patient => {
                self.data.patient_display = value;
                self.data.patient_id = None;
            }
            AppointmentFormField::Practitioner => {
                self.data.practitioner_display = value;
                self.data.practitioner_id = None;
            }
            AppointmentFormField::Date => {
                self.date = TextareaState::new("Date * (YYYY-MM-DD)")
                    .with_height_mode(HeightMode::SingleLine)
                    .with_value(value);
            }
            AppointmentFormField::StartTime => {
                self.start_time = TextareaState::new("Start Time * (HH:MM)")
                    .with_height_mode(HeightMode::SingleLine)
                    .with_value(value);
            }
            AppointmentFormField::Duration => self.data.duration = value,
            AppointmentFormField::AppointmentType => {
                if let Ok(apt_type) = value.parse::<AppointmentType>() {
                    let default_mins = apt_type.default_duration_minutes();
                    self.data.duration = default_mins.to_string();
                }
                self.type_dropdown.set_value(&value);
                self.data.appointment_type = value;
            }
            AppointmentFormField::Reason => {
                self.reason = TextareaState::new("Reason")
                    .with_height_mode(HeightMode::SingleLine)
                    .with_value(value);
            }
            AppointmentFormField::Notes => {
                self.notes = TextareaState::new("Notes")
                    .with_height_mode(HeightMode::FixedLines(3))
                    .with_value(value);
            }
        }
        self.validate_field(&field);
    }

    // ── Navigation ───────────────────────────────────────────────────────────

    pub fn next_field(&mut self) {
        let fields = AppointmentFormField::all();
        if let Some(idx) = fields.iter().position(|f| *f == self.focused_field) {
            self.focused_field = fields[(idx + 1) % fields.len()];
        }
    }

    pub fn prev_field(&mut self) {
        let fields = AppointmentFormField::all();
        if let Some(idx) = fields.iter().position(|f| *f == self.focused_field) {
            let prev = if idx == 0 { fields.len() - 1 } else { idx - 1 };
            self.focused_field = fields[prev];
        }
    }

    // ── Validation ───────────────────────────────────────────────────────────

    fn validate_field(&mut self, field: &AppointmentFormField) {
        self.errors.remove(field);

        match field {
            AppointmentFormField::Patient => {
                if self.data.patient_id.is_none() {
                    self.errors
                        .insert(*field, "Select a patient from the picker".to_string());
                }
            }
            AppointmentFormField::Practitioner => {
                if self.data.practitioner_id.is_none() {
                    self.errors
                        .insert(*field, "Select a practitioner from the picker".to_string());
                }
            }
            AppointmentFormField::Date => {
                let v = self.date.value();
                if v.is_empty() {
                    self.errors.insert(*field, "Date is required".to_string());
                } else if parse_date(&v).is_none() {
                    self.errors
                        .insert(*field, "Use dd/mm/yyyy format".to_string());
                }
            }
            AppointmentFormField::StartTime => {
                let v = self.start_time.value();
                if v.is_empty() {
                    self.errors
                        .insert(*field, "Start time is required".to_string());
                } else if NaiveTime::parse_from_str(&v, "%H:%M").is_err() {
                    self.errors
                        .insert(*field, "Use HH:MM format (24-hour)".to_string());
                }
            }
            AppointmentFormField::Duration => {
                let v = &self.data.duration;
                if !v.is_empty() {
                    match v.parse::<u32>() {
                        Ok(mins) if mins == 0 => {
                            self.errors
                                .insert(*field, "Duration must be greater than 0".to_string());
                        }
                        Ok(mins) if mins > 480 => {
                            self.errors
                                .insert(*field, "Duration cannot exceed 480 minutes".to_string());
                        }
                        Err(_) => {
                            self.errors
                                .insert(*field, "Duration must be a number".to_string());
                        }
                        _ => {}
                    }
                }
            }
            AppointmentFormField::AppointmentType => {
                let v = &self.data.appointment_type;
                if v.is_empty() {
                    self.errors
                        .insert(*field, "Appointment type is required".to_string());
                } else if v.parse::<AppointmentType>().is_err() {
                    self.errors.insert(
                        *field,
                        "Invalid type. Use: Standard, Long, Brief, NewPatient, etc.".to_string(),
                    );
                }
            }
            // Optional fields — no validation required
            AppointmentFormField::Reason | AppointmentFormField::Notes => {}
        }
    }

    /// Validate all fields and return true if the form is error-free.
    pub fn validate(&mut self) -> bool {
        self.errors.clear();
        for field in AppointmentFormField::all() {
            self.validate_field(&field);
        }
        self.errors.is_empty()
    }

    pub fn error(&self, field: AppointmentFormField) -> Option<&String> {
        self.errors.get(&field)
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    // ── Build DTO ────────────────────────────────────────────────────────────

    /// Validate and build a `NewAppointmentData` DTO ready for the service layer.
    ///
    /// Returns `None` if validation fails.
    pub fn to_new_appointment_data(&mut self) -> Option<NewAppointmentData> {
        if !self.validate() {
            return None;
        }

        let patient_id = self.data.patient_id?;
        let practitioner_id = self.data.practitioner_id?;

        let date_str = self.date.value();
        let time_str = self.start_time.value();
        let date = parse_date(&date_str)?;
        let time = NaiveTime::parse_from_str(&time_str, "%H:%M").ok()?;

        let naive_dt = date.and_time(time);
        let start_time = naive_dt
            .and_local_timezone(chrono::Local)
            .single()
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .unwrap_or_else(|| naive_dt.and_utc());

        let duration_mins: i64 = self.data.duration.parse().unwrap_or(15);
        let duration = chrono::Duration::minutes(duration_mins);

        let appointment_type = self.data.appointment_type.parse::<AppointmentType>().ok()?;

        let reason_str = self.reason.value();
        let reason = if reason_str.trim().is_empty() {
            None
        } else {
            Some(reason_str)
        };

        Some(NewAppointmentData {
            patient_id,
            practitioner_id,
            start_time,
            duration,
            appointment_type,
            reason,
            is_urgent: false,
        })
    }

    // ── Key handling ─────────────────────────────────────────────────────────

    pub fn handle_key(&mut self, key: KeyEvent) -> Option<AppointmentFormAction> {
        use crossterm::event::KeyEventKind;

        if key.kind != KeyEventKind::Press {
            return None;
        }

        if self.saving {
            return None;
        }

        if self.focused_field == AppointmentFormField::Patient && self.patient_picker.is_open() {
            use crate::ui::widgets::SearchableList;
            let mut picker = SearchableList::new(
                &mut self.patient_picker,
                &self.theme,
                "Select Patient",
                true,
            );
            let action = picker.handle_key(key);
            match action {
                SearchableListAction::Selected(id, name) => {
                    self.set_patient(id, name);
                    return Some(AppointmentFormAction::ValueChanged);
                }
                SearchableListAction::Cancelled => {
                    return Some(AppointmentFormAction::FocusChanged);
                }
                SearchableListAction::None => {
                    return Some(AppointmentFormAction::FocusChanged);
                }
            }
        }

        if self.focused_field == AppointmentFormField::Practitioner
            && self.practitioner_picker.is_open()
        {
            use crate::ui::widgets::SearchableList;
            let mut picker = SearchableList::new(
                &mut self.practitioner_picker,
                &self.theme,
                "Select Practitioner",
                false,
            );
            let action = picker.handle_key(key);
            match action {
                SearchableListAction::Selected(id, name) => {
                    self.set_practitioner(id, name);
                    return Some(AppointmentFormAction::ValueChanged);
                }
                SearchableListAction::Cancelled => {
                    return Some(AppointmentFormAction::FocusChanged);
                }
                SearchableListAction::None => {
                    return Some(AppointmentFormAction::FocusChanged);
                }
            }
        }

        if self.focused_field == AppointmentFormField::AppointmentType {
            if let Some(action) = self.type_dropdown.handle_key(key) {
                match action {
                    DropdownAction::Selected(_) => {
                        if let Some(value) = self.type_dropdown.selected_value() {
                            if let Ok(apt_type) = value.parse::<AppointmentType>() {
                                let default_mins: i64 = apt_type.default_duration_minutes();
                                self.data.duration = default_mins.to_string();
                            }
                            self.data.appointment_type = value.to_string();
                        }
                        self.validate_field(&AppointmentFormField::AppointmentType);
                        return Some(AppointmentFormAction::ValueChanged);
                    }
                    DropdownAction::Opened
                    | DropdownAction::Closed
                    | DropdownAction::FocusChanged => {
                        return Some(AppointmentFormAction::FocusChanged);
                    }
                }
            }
        }

        if self.focused_field.is_textarea() {
            let ratatui_key = to_ratatui_key(key);
            let textarea = match self.focused_field {
                AppointmentFormField::Date => &mut self.date,
                AppointmentFormField::StartTime => &mut self.start_time,
                AppointmentFormField::Reason => &mut self.reason,
                AppointmentFormField::Notes => &mut self.notes,
                _ => unreachable!(),
            };
            let consumed = textarea.handle_key(ratatui_key);
            if consumed {
                self.validate_field(&self.focused_field.clone());
                return Some(AppointmentFormAction::ValueChanged);
            }
        }

        match key.code {
            KeyCode::Tab => {
                if key.modifiers.contains(KeyModifiers::SHIFT) {
                    self.prev_field();
                } else {
                    self.next_field();
                }
                Some(AppointmentFormAction::FocusChanged)
            }
            KeyCode::BackTab => {
                self.prev_field();
                Some(AppointmentFormAction::FocusChanged)
            }
            KeyCode::Up => {
                self.prev_field();
                Some(AppointmentFormAction::FocusChanged)
            }
            KeyCode::Down => {
                self.next_field();
                Some(AppointmentFormAction::FocusChanged)
            }
            KeyCode::Enter => {
                if self.focused_field == AppointmentFormField::Patient
                    && !self.patient_picker.is_open()
                {
                    self.patient_picker.open();
                    return Some(AppointmentFormAction::FocusChanged);
                }
                if self.focused_field == AppointmentFormField::Practitioner
                    && !self.practitioner_picker.is_open()
                {
                    self.practitioner_picker.open();
                    return Some(AppointmentFormAction::FocusChanged);
                }
                self.validate();
                Some(AppointmentFormAction::Submit)
            }
            KeyCode::Esc => {
                if self.patient_picker.is_open() {
                    self.patient_picker.close();
                    return Some(AppointmentFormAction::FocusChanged);
                }
                if self.practitioner_picker.is_open() {
                    self.practitioner_picker.close();
                    return Some(AppointmentFormAction::FocusChanged);
                }
                Some(AppointmentFormAction::Cancel)
            }
            KeyCode::Char(c) => {
                let mut value = self.get_value(self.focused_field);
                value.push(c);
                self.set_value(self.focused_field, value);
                Some(AppointmentFormAction::ValueChanged)
            }
            KeyCode::Backspace => {
                let mut value = self.get_value(self.focused_field);
                value.pop();
                self.set_value(self.focused_field, value);
                Some(AppointmentFormAction::ValueChanged)
            }
            _ => None,
        }
    }
}

// ── Widget ───────────────────────────────────────────────────────────────────

impl Widget for AppointmentForm {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.is_empty() {
            return;
        }

        let block = Block::default()
            .title(" New Appointment ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.colors.border));

        block.clone().render(area, buf);

        let inner = block.inner(area);
        if inner.is_empty() {
            return;
        }

        let label_width = LABEL_WIDTH;
        let field_start = inner.x + label_width + 2;

        let fields = AppointmentFormField::all();
        let mut y = inner.y + 1;
        let max_y = inner.y + inner.height - 2;

        for field in fields {
            if y > max_y {
                break;
            }

            let is_focused = field == self.focused_field;

            if field.is_textarea() {
                let textarea_state = match field {
                    AppointmentFormField::Date => &self.date,
                    AppointmentFormField::StartTime => &self.start_time,
                    AppointmentFormField::Reason => &self.reason,
                    AppointmentFormField::Notes => &self.notes,
                    _ => unreachable!(),
                };
                let field_height = textarea_state.height();
                let textarea_area =
                    Rect::new(inner.x + 1, y, inner.width.saturating_sub(2), field_height);
                TextareaWidget::new(textarea_state, self.theme.clone())
                    .focused(is_focused)
                    .render(textarea_area, buf);

                if let Some(error_msg) = self.error(field) {
                    let error_style = Style::default().fg(self.theme.colors.error);
                    buf.set_string(
                        inner.x + 2,
                        y + field_height,
                        format!("  {}", error_msg),
                        error_style,
                    );
                    y += 1;
                }
                y += field_height + 1;
                continue;
            }

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

            if field == AppointmentFormField::AppointmentType {
                let dropdown_area = Rect::new(
                    field_start - 1,
                    y,
                    inner.width.saturating_sub(label_width + 2),
                    3,
                );
                let dropdown = self.type_dropdown.clone();
                dropdown.render(dropdown_area, buf);

                if let Some(error_msg) = self.error(field) {
                    let error_style = Style::default().fg(self.theme.colors.error);
                    buf.set_string(field_start, y + 3, format!("  {}", error_msg), error_style);
                    y += 1;
                }
                y += 4;
            } else {
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
        }

        if self.patient_picker.is_open() {
            use crate::ui::widgets::SearchableList;
            let mut picker_state = self.patient_picker.clone();
            let picker_area = Rect::new(
                inner.x + 1,
                inner.y + 1,
                inner.width.saturating_sub(2),
                inner.height.saturating_sub(2),
            );
            let picker =
                SearchableList::new(&mut picker_state, &self.theme, "Select Patient", true);
            picker.render(picker_area, buf);
        }

        if self.practitioner_picker.is_open() {
            use crate::ui::widgets::SearchableList;
            let mut picker_state = self.practitioner_picker.clone();
            let picker_area = Rect::new(
                inner.x + 1,
                inner.y + 1,
                inner.width.saturating_sub(2),
                inner.height.saturating_sub(2),
            );
            let picker =
                SearchableList::new(&mut picker_state, &self.theme, "Select Practitioner", false);
            picker.render(picker_area, buf);
        }

        // Help bar at the bottom
        let help_y = inner.y + inner.height - 1;
        buf.set_string(
            inner.x + 1,
            help_y,
            "Tab: Next | Shift+Tab: Prev | Enter: Submit | Esc: Cancel",
            Style::default().fg(self.theme.colors.disabled),
        );
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_form() -> AppointmentForm {
        AppointmentForm::new(Theme::dark())
    }

    #[test]
    fn test_form_creation() {
        let form = make_form();
        assert_eq!(form.focused_field(), AppointmentFormField::Patient);
        assert!(!form.has_errors());
        assert!(!form.is_saving());
    }

    #[test]
    fn test_tab_navigation_wraps() {
        let mut form = make_form();
        let fields = AppointmentFormField::all();
        // Tab through all fields and back to the first
        for _ in 0..fields.len() {
            form.next_field();
        }
        assert_eq!(form.focused_field(), AppointmentFormField::Patient);
    }

    #[test]
    fn test_shift_tab_navigation_wraps() {
        let mut form = make_form();
        form.prev_field();
        assert_eq!(form.focused_field(), AppointmentFormField::Notes);
    }

    #[test]
    fn test_validation_requires_patient() {
        let mut form = make_form();
        form.validate();
        assert!(form.error(AppointmentFormField::Patient).is_some());
    }

    #[test]
    fn test_validation_requires_practitioner() {
        let mut form = make_form();
        form.validate();
        assert!(form.error(AppointmentFormField::Practitioner).is_some());
    }

    #[test]
    fn test_validation_requires_date() {
        let mut form = make_form();
        form.validate();
        assert!(form.error(AppointmentFormField::Date).is_some());
    }

    #[test]
    fn test_validation_requires_start_time() {
        let mut form = make_form();
        form.validate();
        assert!(form.error(AppointmentFormField::StartTime).is_some());
    }

    #[test]
    fn test_validation_requires_appointment_type() {
        let mut form = make_form();
        // Clear the default type to trigger the error
        form.data.appointment_type = String::new();
        form.validate();
        assert!(form.error(AppointmentFormField::AppointmentType).is_some());
    }

    #[test]
    fn test_validation_date_format() {
        let mut form = make_form();
        form.set_value(AppointmentFormField::Date, "not-a-date".to_string());
        assert!(form.error(AppointmentFormField::Date).is_some());

        form.set_value(AppointmentFormField::Date, "15/03/2026".to_string());
        assert!(form.error(AppointmentFormField::Date).is_none());
    }

    #[test]
    fn test_validation_time_format() {
        let mut form = make_form();
        form.set_value(AppointmentFormField::StartTime, "9am".to_string());
        assert!(form.error(AppointmentFormField::StartTime).is_some());

        form.set_value(AppointmentFormField::StartTime, "09:00".to_string());
        assert!(form.error(AppointmentFormField::StartTime).is_none());
    }

    #[test]
    fn test_validation_duration_bounds() {
        let mut form = make_form();
        form.set_value(AppointmentFormField::Duration, "0".to_string());
        assert!(form.error(AppointmentFormField::Duration).is_some());

        form.set_value(AppointmentFormField::Duration, "481".to_string());
        assert!(form.error(AppointmentFormField::Duration).is_some());

        form.set_value(AppointmentFormField::Duration, "30".to_string());
        assert!(form.error(AppointmentFormField::Duration).is_none());
    }

    #[test]
    fn test_set_patient_clears_error() {
        let mut form = make_form();
        form.validate();
        assert!(form.error(AppointmentFormField::Patient).is_some());

        form.set_patient(Uuid::new_v4(), "Jane Doe".to_string());
        assert!(form.error(AppointmentFormField::Patient).is_none());
    }

    #[test]
    fn test_appointment_type_auto_fills_duration() {
        let mut form = make_form();
        form.set_value(AppointmentFormField::AppointmentType, "Long".to_string());
        assert_eq!(form.get_value(AppointmentFormField::Duration), "30");

        form.set_value(AppointmentFormField::AppointmentType, "Brief".to_string());
        assert_eq!(form.get_value(AppointmentFormField::Duration), "10");
    }

    #[test]
    fn test_to_new_appointment_data_returns_none_on_invalid() {
        let mut form = make_form();
        assert!(form.to_new_appointment_data().is_none());
    }

    #[test]
    fn test_to_new_appointment_data_returns_some_when_valid() {
        let mut form = make_form();
        form.set_patient(Uuid::new_v4(), "Jane Doe".to_string());
        form.set_practitioner(Uuid::new_v4(), "Dr. Smith".to_string());
        form.set_value(AppointmentFormField::Date, "15/03/2026".to_string());
        form.set_value(AppointmentFormField::StartTime, "09:00".to_string());
        form.set_value(
            AppointmentFormField::AppointmentType,
            "Standard".to_string(),
        );

        let dto = form.to_new_appointment_data();
        assert!(dto.is_some());
        let dto = dto.unwrap();
        assert_eq!(dto.appointment_type, AppointmentType::Standard);
    }
}

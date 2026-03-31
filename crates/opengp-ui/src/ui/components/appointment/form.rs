//! Appointment Creation/Editing Form Component
//!
//! Single-page form for creating and editing appointments.
//! Follows the PatientForm pattern with Tab/Shift+Tab navigation.

use std::collections::HashMap;

use chrono::NaiveTime;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Borders, Widget};
use uuid::Uuid;

use crate::ui::input::to_ratatui_key;
use crate::ui::layout::LABEL_WIDTH;
use crate::ui::theme::Theme;
use crate::ui::view_models::{PatientListItem, PractitionerViewItem};
use crate::ui::widgets::{
    format_date, parse_date, DatePickerAction, DatePickerPopup, DropdownAction, DropdownOption,
    DropdownWidget, DynamicForm, DynamicFormMeta, FieldType, FormFieldMeta, FormNavigation,
    HeightMode, ScrollableFormState, SearchableListAction, SearchableListState, TextareaState,
    TextareaWidget, TimePickerAction, TimePickerPopup,
};
use opengp_config::healthcare::HealthcareConfig;
use opengp_domain::domain::appointment::{
    Appointment, AppointmentType, NewAppointmentData, UpdateAppointmentData,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FormMode {
    #[default]
    Create,
    Edit(Uuid),
}

/// All fields in the appointment creation form, in tab order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, strum::EnumIter, strum::IntoStaticStr)]
pub enum AppointmentFormField {
    /// Patient search/select (displays name, stores UUID)
    #[strum(to_string = "Patient *")]
    Patient,
    /// Practitioner dropdown (displays name, stores UUID)
    #[strum(to_string = "Practitioner *")]
    Practitioner,
    /// Appointment date (YYYY-MM-DD)
    #[strum(to_string = "Date * (dd/mm/yyyy)")]
    Date,
    /// Start time (HH:MM, 24-hour)
    #[strum(to_string = "Start Time * (HH:MM)")]
    StartTime,
    /// Duration in minutes
    #[strum(to_string = "Duration (minutes)")]
    Duration,
    /// Appointment type (Standard, Long, Brief, etc.)
    #[strum(to_string = "Type *")]
    AppointmentType,
    /// Reason for visit (optional)
    #[strum(to_string = "Reason")]
    Reason,
    /// Internal notes (optional)
    #[strum(to_string = "Notes")]
    Notes,
}

const FIELD_PATIENT: &str = "patient";
const FIELD_PRACTITIONER: &str = "practitioner";
const FIELD_DATE: &str = "date";
const FIELD_START_TIME: &str = "start_time";
const FIELD_DURATION: &str = "duration";
const FIELD_APPOINTMENT_TYPE: &str = "appointment_type";
const FIELD_REASON: &str = "reason";
const FIELD_NOTES: &str = "notes";

impl AppointmentFormField {
    pub fn all() -> Vec<AppointmentFormField> {
        use strum::IntoEnumIterator;
        // Duration is skipped in Tab navigation but still displayed
        AppointmentFormField::iter()
            .filter(|f| !matches!(f, AppointmentFormField::Duration))
            .collect()
    }

    pub fn label(&self) -> &'static str {
        (*self).into()
    }

    pub fn id(&self) -> &'static str {
        match self {
            AppointmentFormField::Patient => FIELD_PATIENT,
            AppointmentFormField::Practitioner => FIELD_PRACTITIONER,
            AppointmentFormField::Date => FIELD_DATE,
            AppointmentFormField::StartTime => FIELD_START_TIME,
            AppointmentFormField::Duration => FIELD_DURATION,
            AppointmentFormField::AppointmentType => FIELD_APPOINTMENT_TYPE,
            AppointmentFormField::Reason => FIELD_REASON,
            AppointmentFormField::Notes => FIELD_NOTES,
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            FIELD_PATIENT => Some(AppointmentFormField::Patient),
            FIELD_PRACTITIONER => Some(AppointmentFormField::Practitioner),
            FIELD_DATE => Some(AppointmentFormField::Date),
            FIELD_START_TIME => Some(AppointmentFormField::StartTime),
            FIELD_DURATION => Some(AppointmentFormField::Duration),
            FIELD_APPOINTMENT_TYPE => Some(AppointmentFormField::AppointmentType),
            FIELD_REASON => Some(AppointmentFormField::Reason),
            FIELD_NOTES => Some(AppointmentFormField::Notes),
            _ => None,
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

    pub fn is_dropdown(&self) -> bool {
        matches!(self, AppointmentFormField::AppointmentType)
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
    /// Time picker should open with booked slots to be loaded
    OpenTimePicker {
        practitioner_id: Uuid,
        date: chrono::NaiveDate,
        duration: u32,
    },
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

/// Appointment creation/editing form widget.
///
/// Mirrors the PatientForm pattern: Tab/Shift+Tab to navigate fields,
/// Enter to submit, Esc to cancel.  Validation runs on submit and
/// highlights required fields that are missing or malformed.
pub struct AppointmentForm {
    mode: FormMode,
    data: AppointmentFormData,
    errors: HashMap<String, String>,
    focused_field: String,
    field_ids: Vec<String>,
    textareas: HashMap<String, TextareaState>,
    dropdowns: HashMap<String, DropdownWidget>,
    saving: bool,
    theme: Theme,
    healthcare_config: HealthcareConfig,
    scroll: ScrollableFormState,
    patient_picker: SearchableListState<PatientListItem>,
    practitioner_picker: SearchableListState<PractitionerViewItem>,
    date_picker: DatePickerPopup,
    time_picker: TimePickerPopup,
}

impl Clone for AppointmentForm {
    fn clone(&self) -> Self {
        Self {
            mode: self.mode,
            data: self.data.clone(),
            errors: self.errors.clone(),
            focused_field: self.focused_field.clone(),
            field_ids: self.field_ids.clone(),
            textareas: self.textareas.clone(),
            dropdowns: self.dropdowns.clone(),
            saving: self.saving,
            theme: self.theme.clone(),
            healthcare_config: self.healthcare_config.clone(),
            scroll: self.scroll.clone(),
            patient_picker: self.patient_picker.clone(),
            practitioner_picker: self.practitioner_picker.clone(),
            date_picker: self.date_picker.clone(),
            time_picker: self.time_picker.clone(),
        }
    }
}

impl AppointmentForm {
    pub fn new(theme: Theme, healthcare_config: HealthcareConfig) -> Self {
        let type_options = vec![
            DropdownOption::new(
                "Standard",
                format!(
                    "Standard ({} min)",
                    healthcare_config
                        .appointment_durations
                        .get("Standard")
                        .copied()
                        .unwrap_or(15)
                ),
            ),
            DropdownOption::new(
                "Long",
                format!(
                    "Long ({} min)",
                    healthcare_config
                        .appointment_durations
                        .get("Long")
                        .copied()
                        .unwrap_or(30)
                ),
            ),
            DropdownOption::new(
                "Brief",
                format!(
                    "Brief ({} min)",
                    healthcare_config
                        .appointment_durations
                        .get("Brief")
                        .copied()
                        .unwrap_or(10)
                ),
            ),
            DropdownOption::new(
                "NewPatient",
                format!(
                    "New Patient ({} min)",
                    healthcare_config
                        .appointment_durations
                        .get("NewPatient")
                        .copied()
                        .unwrap_or(45)
                ),
            ),
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

        let field_ids: Vec<String> = AppointmentFormField::all()
            .into_iter()
            .map(|field| field.id().to_string())
            .collect();

        let mut textareas = HashMap::new();
        textareas.insert(
            FIELD_DATE.to_string(),
            TextareaState::new("Date * (dd/mm/yyyy)").with_height_mode(HeightMode::SingleLine),
        );
        textareas.insert(
            FIELD_START_TIME.to_string(),
            TextareaState::new("Start Time * (HH:MM)").with_height_mode(HeightMode::SingleLine),
        );
        textareas.insert(
            FIELD_REASON.to_string(),
            TextareaState::new("Reason").with_height_mode(HeightMode::SingleLine),
        );
        textareas.insert(
            FIELD_NOTES.to_string(),
            TextareaState::new("Notes").with_height_mode(HeightMode::FixedLines(3)),
        );

        let mut dropdowns = HashMap::new();
        let mut appointment_type_dropdown =
            DropdownWidget::new("Type *", type_options, theme.clone());
        appointment_type_dropdown.set_value("Standard");
        dropdowns.insert(
            FIELD_APPOINTMENT_TYPE.to_string(),
            appointment_type_dropdown,
        );

        Self {
            mode: FormMode::Create,
            data: AppointmentFormData::empty(),
            errors: HashMap::new(),
            focused_field: FIELD_PATIENT.to_string(),
            field_ids,
            textareas,
            dropdowns,
            saving: false,
            theme: theme.clone(),
            healthcare_config,
            scroll: ScrollableFormState::new(),
            patient_picker: SearchableListState::new(Vec::new()),
            practitioner_picker: SearchableListState::new(Vec::new()),
            date_picker: DatePickerPopup::new(theme),
            time_picker: TimePickerPopup::new(),
        }
    }

    pub fn from_appointment(
        appointment: Appointment,
        theme: Theme,
        healthcare_config: HealthcareConfig,
    ) -> Self {
        let mut form = Self::new(theme, healthcare_config);
        form.mode = FormMode::Edit(appointment.id);

        form.data.patient_id = Some(appointment.patient_id);
        form.data.practitioner_id = Some(appointment.practitioner_id);

        form.set_value(
            AppointmentFormField::Date,
            format_date(appointment.start_time.date_naive()),
        );
        form.set_value(
            AppointmentFormField::StartTime,
            appointment.start_time.format("%H:%M").to_string(),
        );
        form.set_value(
            AppointmentFormField::Reason,
            appointment.reason.clone().unwrap_or_default(),
        );
        form.set_value(
            AppointmentFormField::Notes,
            appointment.notes.clone().unwrap_or_default(),
        );
        form.set_value(
            AppointmentFormField::AppointmentType,
            appointment.appointment_type.to_string(),
        );
        form.data.appointment_type = appointment.appointment_type.to_string();

        let duration_minutes = appointment.duration_minutes();
        form.data.duration = duration_minutes.to_string();

        form
    }

    pub fn is_edit_mode(&self) -> bool {
        matches!(self.mode, FormMode::Edit(_))
    }

    pub fn appointment_id(&self) -> Option<Uuid> {
        match self.mode {
            FormMode::Edit(id) => Some(id),
            FormMode::Create => None,
        }
    }

    // ── Field accessors ──────────────────────────────────────────────────────

    pub fn focused_field(&self) -> AppointmentFormField {
        AppointmentFormField::from_id(&self.focused_field).unwrap_or(AppointmentFormField::Patient)
    }

    pub fn set_focus(&mut self, field: AppointmentFormField) {
        self.focused_field = field.id().to_string();
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
        self.errors.remove(FIELD_PATIENT);
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
        self.errors.remove(FIELD_PRACTITIONER);
    }

    pub fn set_booked_slots(&mut self, booked_slots: Vec<NaiveTime>) {
        self.time_picker.set_booked_slots(booked_slots);
    }

    pub fn open_time_picker(
        &mut self,
        practitioner_id: i64,
        date: chrono::NaiveDate,
        duration: u32,
    ) {
        self.time_picker.open(practitioner_id, date, duration);
    }

    fn get_value_by_id(&self, field_id: &str) -> String {
        match field_id {
            FIELD_PATIENT => self.data.patient_display.clone(),
            FIELD_PRACTITIONER => self.data.practitioner_display.clone(),
            FIELD_DURATION => self.data.duration.clone(),
            FIELD_APPOINTMENT_TYPE => self
                .dropdowns
                .get(FIELD_APPOINTMENT_TYPE)
                .and_then(|dropdown| dropdown.selected_value())
                .map(|value| value.to_string())
                .unwrap_or_default(),
            _ => self
                .textareas
                .get(field_id)
                .map(|textarea| textarea.value())
                .unwrap_or_default(),
        }
    }

    fn set_value_by_id(&mut self, field_id: &str, value: String) {
        match field_id {
            FIELD_PATIENT => {
                self.data.patient_display = value;
                self.data.patient_id = None;
            }
            FIELD_PRACTITIONER => {
                self.data.practitioner_display = value;
                self.data.practitioner_id = None;
            }
            FIELD_DURATION => {
                self.data.duration = value;
            }
            FIELD_APPOINTMENT_TYPE => {
                if let Ok(apt_type) = value.parse::<AppointmentType>() {
                    let default_mins = self
                        .healthcare_config
                        .appointment_durations
                        .get(&value)
                        .copied()
                        .unwrap_or_else(|| apt_type.default_duration_minutes() as u32);
                    self.data.duration = default_mins.to_string();
                }
                if let Some(dropdown) = self.dropdowns.get_mut(FIELD_APPOINTMENT_TYPE) {
                    dropdown.set_value(&value);
                }
                self.data.appointment_type = value;
            }
            _ => {
                if let Some(existing) = self.textareas.get_mut(field_id) {
                    let label = existing.label.clone();
                    let height_mode = existing.height_mode.clone();
                    let max_length = existing.max_length;
                    let focused = existing.focused;

                    let mut updated = TextareaState::new(label)
                        .with_height_mode(height_mode)
                        .with_value(value)
                        .focused(focused);
                    if let Some(limit) = max_length {
                        updated = updated.max_length(limit);
                    }

                    *existing = updated;
                }
            }
        }

        self.validate_field_by_id(field_id);
    }

    fn focused_textarea_mut(&mut self) -> Option<&mut TextareaState> {
        self.textareas.get_mut(&self.focused_field)
    }

    fn textarea_for(&self, field_id: &str) -> Option<&TextareaState> {
        self.textareas.get(field_id)
    }

    pub fn get_value(&self, field: AppointmentFormField) -> String {
        self.get_value_by_id(field.id())
    }

    pub fn set_value(&mut self, field: AppointmentFormField, value: String) {
        self.set_value_by_id(field.id(), value);
    }

    // ── Validation ───────────────────────────────────────────────────────────

    fn validate_field_by_id(&mut self, field_id: &str) {
        self.errors.remove(field_id);

        match field_id {
            FIELD_PATIENT => {
                if self.data.patient_id.is_none() {
                    self.errors.insert(
                        field_id.to_string(),
                        "Select a patient from the picker".to_string(),
                    );
                }
            }
            FIELD_PRACTITIONER => {
                if self.data.practitioner_id.is_none() {
                    self.errors.insert(
                        field_id.to_string(),
                        "Select a practitioner from the picker".to_string(),
                    );
                }
            }
            FIELD_DATE => {
                let v = self.get_value_by_id(field_id);
                if v.is_empty() {
                    self.errors
                        .insert(field_id.to_string(), "Date is required".to_string());
                } else if parse_date(&v).is_none() {
                    self.errors
                        .insert(field_id.to_string(), "Use dd/mm/yyyy format".to_string());
                }
            }
            FIELD_START_TIME => {
                let v = self.get_value_by_id(field_id);
                if v.is_empty() {
                    self.errors
                        .insert(field_id.to_string(), "Start time is required".to_string());
                } else if NaiveTime::parse_from_str(&v, "%H:%M").is_err() {
                    self.errors.insert(
                        field_id.to_string(),
                        "Use HH:MM format (24-hour)".to_string(),
                    );
                }
            }
            FIELD_DURATION => {
                let v = &self.data.duration;
                if !v.is_empty() {
                    match v.parse::<u32>() {
                        Ok(0) => {
                            self.errors.insert(
                                field_id.to_string(),
                                "Duration must be greater than 0".to_string(),
                            );
                        }
                        Ok(mins) if mins > 480 => {
                            self.errors.insert(
                                field_id.to_string(),
                                "Duration cannot exceed 480 minutes".to_string(),
                            );
                        }
                        Err(_) => {
                            self.errors.insert(
                                field_id.to_string(),
                                "Duration must be a number".to_string(),
                            );
                        }
                        _ => {}
                    }
                }
            }
            FIELD_APPOINTMENT_TYPE => {
                let v = &self.data.appointment_type;
                if v.is_empty() {
                    self.errors.insert(
                        field_id.to_string(),
                        "Appointment type is required".to_string(),
                    );
                } else if v.parse::<AppointmentType>().is_err() {
                    self.errors.insert(
                        field_id.to_string(),
                        "Invalid type. Use: Standard, Long, Brief, NewPatient, etc.".to_string(),
                    );
                }
            }
            // Optional fields — no validation required
            FIELD_REASON | FIELD_NOTES => {}
            _ => {}
        }
    }

    pub fn error(&self, field: AppointmentFormField) -> Option<&String> {
        self.errors.get(field.id())
    }

    // ── Build DTO ────────────────────────────────────────────────────────────

    /// Validate and build a `NewAppointmentData` DTO ready for the service layer.
    ///
    /// Returns `None` if validation fails.
    pub fn to_new_appointment_data(&mut self) -> Option<NewAppointmentData> {
        if !FormNavigation::validate(self) {
            return None;
        }

        let patient_id = self.data.patient_id?;
        let practitioner_id = self.data.practitioner_id?;

        let date_str = self.get_value_by_id(FIELD_DATE);
        let time_str = self.get_value_by_id(FIELD_START_TIME);
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

        let reason_str = self.get_value_by_id(FIELD_REASON);
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

    pub fn to_update_appointment_data(&mut self) -> Option<(Uuid, UpdateAppointmentData)> {
        let appointment_id = self.appointment_id()?;

        if !FormNavigation::validate(self) {
            return None;
        }

        let appointment_type = self.data.appointment_type.parse::<AppointmentType>().ok();

        let reason_str = self.get_value_by_id(FIELD_REASON);
        let reason = if reason_str.trim().is_empty() {
            None
        } else {
            Some(reason_str)
        };

        let notes_str = self.get_value_by_id(FIELD_NOTES);
        let notes = if notes_str.trim().is_empty() {
            None
        } else {
            Some(notes_str)
        };

        let data = UpdateAppointmentData {
            patient_id: None,
            practitioner_id: None,
            start_time: None,
            duration: None,
            status: None,
            appointment_type,
            reason,
            notes,
            is_urgent: None,
            confirmed: None,
            reminder_sent: None,
            cancellation_reason: None,
        };

        Some((appointment_id, data))
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

        if self.focused_field == FIELD_PATIENT && self.patient_picker.is_open() {
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

        if self.focused_field == FIELD_PRACTITIONER && self.practitioner_picker.is_open() {
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

        if self.focused_field == FIELD_APPOINTMENT_TYPE {
            if let Some(action) = self
                .dropdowns
                .get_mut(FIELD_APPOINTMENT_TYPE)
                .and_then(|dropdown| dropdown.handle_key(key))
            {
                // Allow Tab/BackTab/Esc to pass through to form's navigation handler
                match key.code {
                    KeyCode::Tab | KeyCode::BackTab | KeyCode::Esc => return None,
                    _ => match action {
                        DropdownAction::Selected(_) => {
                            if let Some(value) = self
                                .dropdowns
                                .get(FIELD_APPOINTMENT_TYPE)
                                .and_then(|dropdown| dropdown.selected_value())
                            {
                                if let Ok(apt_type) = value.parse::<AppointmentType>() {
                                    let default_mins: i64 =
                                        self.healthcare_config
                                            .appointment_durations
                                            .get(value)
                                            .copied()
                                            .unwrap_or_else(|| {
                                                apt_type.default_duration_minutes() as u32
                                            }) as i64;
                                    self.data.duration = default_mins.to_string();
                                }
                                self.data.appointment_type = value.to_string();
                            }
                            self.validate_field_by_id(FIELD_APPOINTMENT_TYPE);
                            return Some(AppointmentFormAction::ValueChanged);
                        }
                        DropdownAction::Opened
                        | DropdownAction::Closed
                        | DropdownAction::FocusChanged => {
                            return Some(AppointmentFormAction::FocusChanged);
                        }
                    },
                }
            }
        }

        if self.date_picker.is_visible() {
            if let Some(action) = self.date_picker.handle_key(key) {
                match action {
                    DatePickerAction::Selected(date) => {
                        self.set_value_by_id(FIELD_DATE, format_date(date));
                        return Some(AppointmentFormAction::ValueChanged);
                    }
                    DatePickerAction::Dismissed => {
                        return Some(AppointmentFormAction::FocusChanged);
                    }
                }
            }
            return Some(AppointmentFormAction::FocusChanged);
        }

        if self.focused_field == FIELD_DATE
            && matches!(key.code, KeyCode::Enter | KeyCode::Char(' '))
        {
            let current_value = parse_date(&self.get_value_by_id(FIELD_DATE));
            self.date_picker.open(current_value);
            return Some(AppointmentFormAction::FocusChanged);
        }

        // Time picker handling
        if self.time_picker.is_visible() {
            if let Some(action) = self.time_picker.handle_key(key) {
                match action {
                    TimePickerAction::Selected(time) => {
                        self.set_value_by_id(FIELD_START_TIME, time.format("%H:%M").to_string());
                        return Some(AppointmentFormAction::ValueChanged);
                    }
                    TimePickerAction::Dismissed => {
                        return Some(AppointmentFormAction::FocusChanged);
                    }
                }
            }
            return Some(AppointmentFormAction::FocusChanged);
        }

        if self.focused_field == FIELD_START_TIME
            && matches!(key.code, KeyCode::Enter | KeyCode::Char(' '))
        {
            // Need practitioner_id, date, and duration to open time picker
            if let (Some(practitioner_id), Some(date), Ok(duration)) = (
                self.data.practitioner_id,
                parse_date(&self.get_value_by_id(FIELD_DATE)),
                self.data.duration.parse::<u32>(),
            ) {
                return Some(AppointmentFormAction::OpenTimePicker {
                    practitioner_id,
                    date,
                    duration,
                });
            }
        }

        // Ctrl+S submits the form from any field
        if key.modifiers.contains(KeyModifiers::CONTROL) && matches!(key.code, KeyCode::Char('s')) {
            FormNavigation::validate(self);
            return Some(AppointmentFormAction::Submit);
        }

        let focused_field = AppointmentFormField::from_id(&self.focused_field)
            .unwrap_or(AppointmentFormField::Patient);
        if focused_field.is_textarea() {
            let ratatui_key = to_ratatui_key(key);
            if let Some(textarea) = self.focused_textarea_mut() {
                let consumed = textarea.handle_key(ratatui_key);
                if consumed {
                    let field_id = self.focused_field.clone();
                    self.validate_field_by_id(&field_id);
                    return Some(AppointmentFormAction::ValueChanged);
                }
            }
        }

        match key.code {
            KeyCode::Tab => {
                if key.modifiers.contains(KeyModifiers::SHIFT) {
                    FormNavigation::prev_field(self);
                } else {
                    FormNavigation::next_field(self);
                }
                Some(AppointmentFormAction::FocusChanged)
            }
            KeyCode::BackTab => {
                FormNavigation::prev_field(self);
                Some(AppointmentFormAction::FocusChanged)
            }
            KeyCode::Up => {
                FormNavigation::prev_field(self);
                Some(AppointmentFormAction::FocusChanged)
            }
            KeyCode::Down => {
                FormNavigation::next_field(self);
                Some(AppointmentFormAction::FocusChanged)
            }
            KeyCode::PageUp => {
                self.scroll.scroll_up();
                Some(AppointmentFormAction::FocusChanged)
            }
            KeyCode::PageDown => {
                self.scroll.scroll_down();
                Some(AppointmentFormAction::FocusChanged)
            }
            KeyCode::Enter => {
                if self.focused_field == FIELD_PATIENT && !self.patient_picker.is_open() {
                    self.patient_picker.open();
                    return Some(AppointmentFormAction::FocusChanged);
                }
                if self.focused_field == FIELD_PRACTITIONER && !self.practitioner_picker.is_open() {
                    self.practitioner_picker.open();
                    return Some(AppointmentFormAction::FocusChanged);
                }
                None
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
                let field_id = self.focused_field.clone();
                let mut value = self.get_value_by_id(&field_id);
                value.push(c);
                self.set_value_by_id(&field_id, value);
                Some(AppointmentFormAction::ValueChanged)
            }
            KeyCode::Backspace => {
                let field_id = self.focused_field.clone();
                let mut value = self.get_value_by_id(&field_id);
                value.pop();
                self.set_value_by_id(&field_id, value);
                Some(AppointmentFormAction::ValueChanged)
            }
            _ => None,
        }
    }
}

// ── Widget ───────────────────────────────────────────────────────────────────

impl Widget for AppointmentForm {
    fn render(mut self, area: Rect, buf: &mut Buffer) {
        if area.is_empty() {
            return;
        }

        let title = if self.is_edit_mode() {
            " Edit Appointment "
        } else {
            " New Appointment "
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

        let fields = AppointmentFormField::all();

        let mut total_height: u16 = 0;
        for field in &fields {
            total_height += 2;
            if field == &AppointmentFormField::AppointmentType {
                total_height += 2;
            }
        }
        self.scroll.set_total_height(total_height);
        self.scroll.clamp_offset(inner.height.saturating_sub(2));

        let mut y: i32 = (inner.y as i32) + 1 - (self.scroll.scroll_offset as i32);
        let max_y = inner.y as i32 + inner.height as i32 - 2;

        let mut open_dropdown: Option<(DropdownWidget, Rect)> = None;

        for field in fields {
            let is_focused = field.id() == self.focused_field;

            // Special handling for Duration (display only, not in focus cycle)
            if field == AppointmentFormField::Duration {
                let field_height = 3i32;
                if y + field_height <= inner.y as i32 || y >= max_y {
                    y += field_height;
                    continue;
                }

                if y >= inner.y as i32 && y < max_y {
                    let duration_value = self.get_value(field);
                    let has_error = self.error(field).is_some();
                    let border_style = if has_error {
                        Style::default().fg(self.theme.colors.error)
                    } else {
                        Style::default().fg(self.theme.colors.border)
                    };

                    let block = Block::default()
                        .title(" Duration ")
                        .borders(Borders::ALL)
                        .border_style(border_style);

                    let block_area =
                        Rect::new(inner.x + 1, y as u16, inner.width.saturating_sub(2), 3);
                    block.clone().render(block_area, buf);

                    let inner_area = block.inner(block_area);
                    buf.set_string(
                        inner_area.x,
                        inner_area.y,
                        format!(" {} ", duration_value),
                        Style::default().fg(self.theme.colors.foreground),
                    );

                    if let Some(error_msg) = self.error(field) {
                        let error_style = Style::default().fg(self.theme.colors.error);
                        buf.set_string(
                            inner.x + 2,
                            (y as u16) + 3,
                            format!("  {}", error_msg),
                            error_style,
                        );
                    }
                }
                y += 4;
                continue;
            }

            let field_height = if field == AppointmentFormField::AppointmentType {
                4i32
            } else {
                2i32
            };

            if y + field_height <= inner.y as i32 || y >= max_y {
                y += field_height;
                continue;
            }

            if field.is_textarea() {
                let Some(textarea_state) = self.textarea_for(field.id()) else {
                    y += 2;
                    continue;
                };
                let field_height = textarea_state.height();
                if y >= inner.y as i32 && y < max_y {
                    let textarea_area = Rect::new(
                        inner.x + 1,
                        y as u16,
                        inner.width.saturating_sub(2),
                        field_height,
                    );
                    TextareaWidget::new(textarea_state, self.theme.clone())
                        .focused(is_focused)
                        .render(textarea_area, buf);

                    if let Some(error_msg) = self.error(field) {
                        let error_style = Style::default().fg(self.theme.colors.error);
                        buf.set_string(
                            inner.x + 2,
                            (y as u16) + field_height,
                            format!("  {}", error_msg),
                            error_style,
                        );
                    }
                }
                y += field_height as i32 + 1;
                continue;
            }

            let has_error = self.error(field).is_some();

            // Patient and Practitioner fields with bordered box style at rest
            if (field == AppointmentFormField::Patient
                || field == AppointmentFormField::Practitioner)
                && !self.patient_picker.is_open()
                && !self.practitioner_picker.is_open()
            {
                if y >= inner.y as i32 && y < max_y {
                    let value = self.get_value(field);
                    let border_style = if has_error {
                        Style::default().fg(self.theme.colors.error)
                    } else if is_focused {
                        Style::default()
                            .fg(self.theme.colors.primary)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(self.theme.colors.border)
                    };

                    let title_style = if is_focused {
                        Style::default()
                            .fg(self.theme.colors.primary)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(self.theme.colors.foreground)
                    };

                    let block = Block::default()
                        .title(ratatui::text::Span::styled(
                            format!(" {} ", field.label()),
                            title_style,
                        ))
                        .borders(Borders::ALL)
                        .border_style(border_style);

                    let block_area =
                        Rect::new(inner.x + 1, y as u16, inner.width.saturating_sub(2), 3);
                    block.clone().render(block_area, buf);

                    let inner_area = block.inner(block_area);
                    if !value.is_empty() {
                        let max_width = inner_area.width.saturating_sub(1) as usize;
                        let display_value = if value.len() > max_width {
                            &value[value.len() - max_width..]
                        } else {
                            &value
                        };
                        buf.set_string(
                            inner_area.x,
                            inner_area.y,
                            display_value,
                            Style::default().fg(self.theme.colors.foreground),
                        );
                    }

                    if let Some(error_msg) = self.error(field) {
                        let error_style = Style::default().fg(self.theme.colors.error);
                        buf.set_string(
                            inner.x + 2,
                            (y as u16) + 3,
                            format!("  {}", error_msg),
                            error_style,
                        );
                    }
                }
                y += 4;
                continue;
            }

            if y >= inner.y as i32 && y < max_y && !field.is_dropdown() {
                let label_style = if is_focused {
                    Style::default()
                        .fg(self.theme.colors.primary)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(self.theme.colors.foreground)
                };

                buf.set_string(inner.x + 1, y as u16, field.label(), label_style);

                if is_focused {
                    buf.set_string(
                        field_start - 1,
                        y as u16,
                        ">",
                        Style::default().fg(self.theme.colors.primary),
                    );
                }
            }

            if field == AppointmentFormField::AppointmentType {
                if y >= inner.y as i32 && y < max_y {
                    let dropdown_area = Rect::new(
                        field_start - 1,
                        y as u16,
                        inner.width.saturating_sub(label_width + 2),
                        3,
                    );
                    let Some(dropdown) = self.dropdowns.get(FIELD_APPOINTMENT_TYPE).cloned() else {
                        y += 4;
                        continue;
                    };
                    if dropdown.is_open() {
                        open_dropdown = Some((dropdown.clone(), dropdown_area));
                    }
                    dropdown.focused(is_focused).render(dropdown_area, buf);

                    if let Some(error_msg) = self.error(field) {
                        let error_style = Style::default().fg(self.theme.colors.error);
                        buf.set_string(
                            field_start,
                            (y as u16) + 3,
                            format!("  {}", error_msg),
                            error_style,
                        );
                    }
                }
                y += 4;
            } else {
                if y >= inner.y as i32 && y < max_y {
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

                    buf.set_string(field_start, y as u16, display_value, value_style);

                    if let Some(error_msg) = self.error(field) {
                        let error_style = Style::default().fg(self.theme.colors.error);
                        buf.set_string(
                            field_start,
                            (y as u16) + 1,
                            format!("  {}", error_msg),
                            error_style,
                        );
                    }
                }
                y += 2;
            }
        }

        if let Some((dropdown, dropdown_area)) = open_dropdown {
            dropdown.render(dropdown_area, buf);
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

        self.scroll.render_scrollbar(inner, buf, &self.theme);

        let help_y = inner.y + inner.height - 1;
        buf.set_string(
            inner.x + 1,
            help_y,
            "Tab: Next | Shift+Tab: Prev | Ctrl+S: Submit | Esc: Cancel",
            Style::default().fg(self.theme.colors.disabled),
        );

        if self.date_picker.is_visible() {
            self.date_picker.render(area, buf);
        }

        if self.time_picker.is_visible() {
            self.time_picker.render(area, buf);
        }
    }
}

impl FormFieldMeta for AppointmentFormField {
    fn label(&self) -> &'static str {
        AppointmentFormField::label(self)
    }

    fn is_required(&self) -> bool {
        AppointmentFormField::is_required(self)
    }
}

impl DynamicFormMeta for AppointmentForm {
    fn label(&self, field_id: &str) -> String {
        AppointmentFormField::from_id(field_id)
            .map(|field| field.label().to_string())
            .unwrap_or_else(|| field_id.to_string())
    }

    fn is_required(&self, field_id: &str) -> bool {
        AppointmentFormField::from_id(field_id)
            .map(|field| field.is_required())
            .unwrap_or(false)
    }

    fn field_type(&self, field_id: &str) -> FieldType {
        match AppointmentFormField::from_id(field_id) {
            Some(AppointmentFormField::Date) => FieldType::Date,
            Some(AppointmentFormField::AppointmentType) => FieldType::Select(vec![]),
            _ => FieldType::Text,
        }
    }
}

impl DynamicForm for AppointmentForm {
    fn field_ids(&self) -> &[String] {
        &self.field_ids
    }

    fn current_field(&self) -> &str {
        &self.focused_field
    }

    fn set_current_field(&mut self, field_id: &str) {
        if self.field_ids.iter().any(|id| id == field_id) {
            self.focused_field = field_id.to_string();
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

impl FormNavigation for AppointmentForm {
    type FormField = AppointmentFormField;

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
        self.focused_field()
    }

    fn fields(&self) -> Vec<Self::FormField> {
        self.field_ids
            .iter()
            .filter_map(|field_id| AppointmentFormField::from_id(field_id))
            .collect()
    }

    fn set_current_field(&mut self, field: Self::FormField) {
        self.focused_field = field.id().to_string();
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_form() -> AppointmentForm {
        AppointmentForm::new(Theme::dark(), HealthcareConfig::default())
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
            FormNavigation::next_field(&mut form);
        }
        assert_eq!(form.focused_field(), AppointmentFormField::Patient);
    }

    #[test]
    fn test_shift_tab_navigation_wraps() {
        let mut form = make_form();
        FormNavigation::prev_field(&mut form);
        assert_eq!(form.focused_field(), AppointmentFormField::Notes);
    }

    #[test]
    fn test_validation_requires_patient() {
        let mut form = make_form();
        FormNavigation::validate(&mut form);
        assert!(form.error(AppointmentFormField::Patient).is_some());
    }

    #[test]
    fn test_validation_requires_practitioner() {
        let mut form = make_form();
        FormNavigation::validate(&mut form);
        assert!(form.error(AppointmentFormField::Practitioner).is_some());
    }

    #[test]
    fn test_validation_requires_date() {
        let mut form = make_form();
        FormNavigation::validate(&mut form);
        assert!(form.error(AppointmentFormField::Date).is_some());
    }

    #[test]
    fn test_validation_requires_start_time() {
        let mut form = make_form();
        FormNavigation::validate(&mut form);
        assert!(form.error(AppointmentFormField::StartTime).is_some());
    }

    #[test]
    fn test_validation_requires_appointment_type() {
        let mut form = make_form();
        // Clear the default type to trigger the error
        form.data.appointment_type = String::new();
        FormNavigation::validate(&mut form);
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
        FormNavigation::validate(&mut form);
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

use chrono::NaiveDate;
use crossterm::event::{KeyEvent, MouseEvent, MouseEventKind};
use rat_focus::{FocusBuilder, FocusFlag, HasFocus};
use ratatui::layout::Rect;
use std::collections::HashMap;
use uuid::Uuid;

use opengp_config::CalendarConfig;
use opengp_domain::domain::appointment::{AppointmentType, CalendarAppointment, CalendarDayView};
use opengp_domain::domain::user::Practitioner;

use crate::ui::input::{DoubleClickDetector, HoverState};
use crate::ui::keybinds::{Action, KeyContext, KeybindRegistry};
use crate::ui::view_models::PractitionerViewItem;
use crate::ui::widgets::LoadingState;

use super::calendar::Calendar;
use super::schedule::{Schedule, ScheduleAction};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppointmentView {
    Calendar,
    Schedule,
}

#[derive(Clone)]
pub struct AppointmentState {
    pub current_view: AppointmentView,
    pub calendar: Calendar,
    pub schedule: Schedule,
    pub selected_date: Option<NaiveDate>,
    pub schedule_data: Option<CalendarDayView>,
    pub practitioners: Vec<Practitioner>,
    pub selected_practitioner: Option<Uuid>,
    pub selected_appointment: Option<Uuid>,
    pub loading_state: LoadingState,
    loading: bool,
    pub hidden_columns: Vec<Uuid>,
    pub practitioners_view: Vec<PractitionerViewItem>,
    pub selected_practitioner_index: usize,
    pub selected_time_slot: u8,
    pub viewport_start_hour: u8,
    pub viewport_end_hour: u8,
    pub last_inner_height: u16,
    pub focused: bool,
    pub config: CalendarConfig,
    pub debug_overlay_visible: bool,
    pub hovered_slot: HoverState<(usize, u8)>,
    pub schedule_double_click_detector: DoubleClickDetector,
    pub focus: FocusFlag,
}

impl std::fmt::Debug for AppointmentState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppointmentState")
            .field("current_view", &self.current_view)
            .field("calendar", &self.calendar)
            .field("schedule", &"<Schedule widget>")
            .field("selected_date", &self.selected_date)
            .field("schedule_data", &self.schedule_data)
            .field("practitioners", &self.practitioners)
            .field("selected_practitioner", &self.selected_practitioner)
            .field("selected_appointment", &self.selected_appointment)
            .field("loading_state", &self.loading_state)
            .field("loading", &self.loading)
            .field("hidden_columns", &self.hidden_columns)
            .field("practitioners_view", &self.practitioners_view)
            .field(
                "selected_practitioner_index",
                &self.selected_practitioner_index,
            )
            .field("selected_time_slot", &self.selected_time_slot)
            .field("viewport_start_hour", &self.viewport_start_hour)
            .field("viewport_end_hour", &self.viewport_end_hour)
            .field("last_inner_height", &self.last_inner_height)
            .field("focused", &self.focused)
            .field("config", &self.config)
            .field("debug_overlay_visible", &self.debug_overlay_visible)
            .field("hovered_slot", &"<HoverState>")
            .field("schedule_double_click_detector", &"<DoubleClickDetector>")
            .field("focus", &"<FocusFlag>")
            .finish()
    }
}

impl AppointmentState {
    pub fn new(theme: crate::ui::theme::Theme, config: CalendarConfig) -> Self {
        let abbreviations = opengp_config::load_appointment_config()
            .map(|appointment_config| {
                let mut map = HashMap::new();
                for apt_type in [
                    AppointmentType::Standard,
                    AppointmentType::Long,
                    AppointmentType::Brief,
                    AppointmentType::NewPatient,
                    AppointmentType::HealthAssessment,
                    AppointmentType::ChronicDiseaseReview,
                    AppointmentType::MentalHealthPlan,
                    AppointmentType::Immunisation,
                    AppointmentType::Procedure,
                    AppointmentType::Telephone,
                    AppointmentType::Telehealth,
                    AppointmentType::HomeVisit,
                    AppointmentType::Emergency,
                ] {
                    if let Some(option) = appointment_config
                        .types
                        .get(Self::appointment_type_config_key(apt_type))
                        .filter(|option| option.enabled)
                    {
                        map.insert(apt_type.to_string(), option.abbreviation.clone());
                    }
                }
                map
            })
            .unwrap_or_default();

        Self {
            current_view: AppointmentView::Schedule,
            calendar: Calendar::new(theme.clone()),
            schedule: Schedule::new(theme, config.clone()).with_abbreviations(abbreviations),
            selected_date: Some(chrono::Utc::now().date_naive()),
            schedule_data: None,
            practitioners: Vec::new(),
            selected_practitioner: None,
            selected_appointment: None,
            loading_state: LoadingState::new().message("Loading appointments..."),
            loading: false,
            hidden_columns: Vec::new(),
            practitioners_view: Vec::new(),
            selected_practitioner_index: 0,
            selected_time_slot: 0,
            viewport_start_hour: config.viewport_start_hour,
            viewport_end_hour: config.viewport_end_hour,
            last_inner_height: 0,
            focused: false,
            config,
            debug_overlay_visible: false,
            hovered_slot: HoverState::new(),
            schedule_double_click_detector: DoubleClickDetector::default(),
            focus: FocusFlag::default(),
        }
    }

    fn appointment_type_config_key(apt_type: AppointmentType) -> &'static str {
        match apt_type {
            AppointmentType::Standard => "standard",
            AppointmentType::Long => "long",
            AppointmentType::Brief => "brief",
            AppointmentType::NewPatient => "new_patient",
            AppointmentType::HealthAssessment => "health_assessment",
            AppointmentType::ChronicDiseaseReview => "chronic_disease_review",
            AppointmentType::MentalHealthPlan => "mental_health_plan",
            AppointmentType::Immunisation => "immunisation",
            AppointmentType::Procedure => "procedure",
            AppointmentType::Telephone => "telephone",
            AppointmentType::Telehealth => "telehealth",
            AppointmentType::HomeVisit => "home_visit",
            AppointmentType::Emergency => "emergency",
        }
    }

    /// Set the selected date
    pub fn set_selected_date(&mut self, date: Option<NaiveDate>) {
        self.selected_date = date;
    }

    /// Get the currently selected date
    pub fn selected_date(&self) -> Option<NaiveDate> {
        self.selected_date
    }

    /// Switch to a different view
    pub fn set_view(&mut self, view: AppointmentView) {
        self.current_view = view;
    }

    /// Check if loading
    pub fn is_loading(&self) -> bool {
        self.loading
    }

    /// Set loading state
    pub fn set_loading(&mut self, loading: bool) {
        self.loading = loading;
    }

    /// Set the selected practitioner
    pub fn set_selected_practitioner(&mut self, practitioner_id: Option<Uuid>) {
        self.selected_practitioner = practitioner_id;
    }

    /// Get the selected practitioner ID
    pub fn selected_practitioner(&self) -> Option<Uuid> {
        self.selected_practitioner
    }

    /// Set the selected appointment
    pub fn set_selected_appointment(&mut self, appointment_id: Option<Uuid>) {
        self.selected_appointment = appointment_id;
    }

    /// Get the selected appointment ID
    pub fn selected_appointment(&self) -> Option<Uuid> {
        self.selected_appointment
    }

    /// Clear all selections
    pub fn clear_selections(&mut self) {
        self.selected_date = None;
        self.selected_practitioner = None;
        self.selected_appointment = None;
        self.schedule_data = None;
        self.hidden_columns = Vec::new();
    }

    /// Toggle visibility of a practitioner column
    /// If hiding would leave 0 visible columns, this is a no-op (minimum 1 visible enforced)
    pub fn toggle_column(&mut self, practitioner_id: Uuid, total_practitioners: usize) {
        if let Some(pos) = self
            .hidden_columns
            .iter()
            .position(|&id| id == practitioner_id)
        {
            // Column is hidden, show it
            self.hidden_columns.remove(pos);
        } else {
            // Column is visible, hide it (but only if at least 1 will remain visible)
            if self.hidden_columns.len() < total_practitioners - 1 {
                self.hidden_columns.push(practitioner_id);
            }
        }
    }

    /// Check if a practitioner column is hidden
    pub fn is_column_hidden(&self, practitioner_id: Uuid) -> bool {
        self.hidden_columns.contains(&practitioner_id)
    }

    /// Get visible practitioners (excluding hidden columns)
    pub fn visible_practitioners(&self) -> Vec<&Practitioner> {
        self.practitioners
            .iter()
            .filter(|p| !self.is_column_hidden(p.id))
            .collect()
    }

    pub fn load_schedule_data(&mut self, data: CalendarDayView) {
        self.load_schedule_data_with_colours(data, &[]);
    }

    pub fn load_schedule_data_with_colours(
        &mut self,
        data: CalendarDayView,
        practitioner_colours: &[ratatui::style::Color],
    ) {
        self.schedule_data = Some(data.clone());
        self.practitioners_view.clear();

        for (idx, ps) in data.practitioners.iter().enumerate() {
            let colour = if practitioner_colours.is_empty() {
                ratatui::style::Color::White
            } else {
                practitioner_colours[idx % practitioner_colours.len()]
            };
            self.practitioners_view.push(PractitionerViewItem {
                id: ps.practitioner_id,
                display_name: ps.practitioner_name.clone(),
                colour,
            });
        }

        if self.selected_practitioner_index >= self.practitioners_view.len() {
            self.selected_practitioner_index = self.practitioners_view.len().saturating_sub(1);
        }
    }

    pub fn set_inner_height(&mut self, inner_height: u16) {
        self.last_inner_height = inner_height;
        self.fit_viewport_to_height();
    }

    fn visible_slots(&self) -> u8 {
        if self.last_inner_height < 2 {
            return 1;
        }
        ((self.last_inner_height.saturating_sub(1)) / 2).min(255) as u8
    }

    fn fit_viewport_to_height(&mut self) {
        let visible = self.visible_slots();
        let hours_needed = ((visible as u16).div_ceil(4) as u8).max(1);
        let new_end = (self.viewport_start_hour + hours_needed).min(self.config.max_hour);
        self.viewport_end_hour = new_end.max(self.viewport_start_hour + 1);
    }

    fn ensure_slot_visible(&mut self) {
        let visible = self.visible_slots();
        if visible == 0 {
            return;
        }
        let slot = self.selected_time_slot;
        let max_hour = self.config.max_hour;
        let min_hour = self.config.min_hour;
        let window = self.viewport_end_hour - self.viewport_start_hour;

        if slot >= visible {
            let abs_slot = self.viewport_start_hour as u16 * 4 + slot as u16;
            let new_start_slot = abs_slot.saturating_sub(visible as u16 - 1);
            let new_start_hour = ((new_start_slot / 4) as u8).min(max_hour - window);
            let new_start_hour = new_start_hour.max(min_hour);
            self.viewport_start_hour = new_start_hour;
            self.viewport_end_hour = (self.viewport_start_hour + window).min(max_hour);
            self.selected_time_slot = (abs_slot as u8).saturating_sub(self.viewport_start_hour * 4);
        }
    }

    fn scroll_viewport_up_if_needed(&mut self) {
        let visible = self.visible_slots();
        if visible == 0 {
            return;
        }
        let slot = self.selected_time_slot;
        let min_hour = self.config.min_hour;
        let window = self.viewport_end_hour - self.viewport_start_hour;

        if slot == 0 && self.viewport_start_hour > min_hour {
            let new_start = self.viewport_start_hour.saturating_sub(1).max(min_hour);
            self.viewport_start_hour = new_start;
            self.viewport_end_hour = (self.viewport_start_hour + window).min(self.config.max_hour);
            self.selected_time_slot = 4;
        }
    }

    fn scroll_viewport_to_show_selection(&mut self) {
        let window_hours = self.viewport_end_hour - self.viewport_start_hour;
        let selected_hour = self.viewport_start_hour + (self.selected_time_slot / 4);

        if selected_hour < self.viewport_start_hour {
            self.viewport_start_hour = selected_hour;
            self.viewport_end_hour = selected_hour + window_hours;
        } else if selected_hour >= self.viewport_end_hour {
            self.viewport_end_hour = selected_hour + 1;
            self.viewport_start_hour = self.viewport_end_hour.saturating_sub(window_hours);
        }
    }

    pub fn max_time_slot(&self) -> u8 {
        (self.config.max_hour - self.viewport_start_hour) * 4 - 1
    }

    pub fn time_to_slot(&self, time: chrono::DateTime<chrono::Utc>) -> Option<u8> {
        use chrono::Timelike;
        let local = time.with_timezone(&chrono::Local);
        let hour = local.hour() as u8;
        let minute = local.minute() as u8;
        if hour < self.viewport_start_hour {
            return None;
        }
        let hour_offset = hour - self.viewport_start_hour;
        let slot = hour_offset * 4 + minute / 15;
        Some(slot)
    }

    pub fn slot_to_time(&self, slot: u8) -> String {
        let total_minutes = (self.viewport_start_hour as u16 * 60) + (slot as u16 * 15);
        let hour = total_minutes / 60;
        let minute = total_minutes % 60;
        format!("{:02}:{:02}", hour, minute)
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Option<ScheduleAction> {
        use crossterm::event::{Event, KeyEventKind, KeyModifiers};
        use rat_event::ct_event;

        if key.kind != KeyEventKind::Press {
            return None;
        }

        let event = Event::Key(key);

        // Handle Ctrl+D to toggle debug overlay
        if matches!(&event, ct_event!(key press CONTROL-'d')) {
            #[cfg(debug_assertions)]
            {
                self.debug_overlay_visible = !self.debug_overlay_visible;
            }
            return None;
        }

        let registry = KeybindRegistry::global();

        if let Some(keybind) = registry.lookup(key, KeyContext::Schedule) {
            return match keybind.action {
                Action::PrevPractitioner => {
                    if self.selected_practitioner_index > 0 {
                        self.selected_practitioner_index -= 1;
                    }
                    self.practitioners_view
                        .get(self.selected_practitioner_index)
                        .map(|p| ScheduleAction::SelectPractitioner(p.id))
                }
                Action::NextPractitioner => {
                    if self.selected_practitioner_index
                        < self.practitioners_view.len().saturating_sub(1)
                    {
                        self.selected_practitioner_index += 1;
                    }
                    self.practitioners_view
                        .get(self.selected_practitioner_index)
                        .map(|p| ScheduleAction::SelectPractitioner(p.id))
                }
                Action::PrevTimeSlot => {
                    if self.selected_time_slot > 0 {
                        self.selected_time_slot -= 1;
                        self.scroll_viewport_up_if_needed();
                    }
                    Some(ScheduleAction::NavigateTimeSlot(-1))
                }
                Action::NextTimeSlot => {
                    let max_slot = self.max_time_slot();
                    if self.selected_time_slot < max_slot {
                        self.selected_time_slot += 1;
                        self.ensure_slot_visible();
                    }
                    Some(ScheduleAction::NavigateTimeSlot(1))
                }
                Action::TogglePractitionerColumn => Some(ScheduleAction::ToggleColumn),
                Action::Enter => {
                    if let Some(apt) = self.get_appointment_at_selection() {
                        Some(ScheduleAction::SelectAppointment(apt.id))
                    } else if let (Some(practitioner), Some(schedule_data)) = (
                        self.practitioners_view
                            .get(self.selected_practitioner_index),
                        &self.schedule_data,
                    ) {
                        Some(ScheduleAction::CreateAtSlot {
                            practitioner_id: practitioner.id,
                            date: schedule_data.date,
                            time: self.slot_to_time(self.selected_time_slot),
                        })
                    } else {
                        None
                    }
                }
                Action::ScrollViewportUp => {
                    let min_hour = self.config.min_hour;
                    if self.viewport_start_hour > min_hour {
                        let abs_hour = self.viewport_start_hour + (self.selected_time_slot / 4);
                        let abs_min_slot = self.selected_time_slot % 4;

                        let window = self.viewport_end_hour - self.viewport_start_hour;
                        self.viewport_start_hour =
                            self.viewport_start_hour.saturating_sub(2).max(min_hour);
                        self.viewport_end_hour = self.viewport_start_hour + window;

                        if abs_hour >= self.viewport_start_hour && abs_hour < self.viewport_end_hour
                        {
                            self.selected_time_slot =
                                (abs_hour - self.viewport_start_hour) * 4 + abs_min_slot;
                        } else {
                            self.selected_time_slot = 0;
                        }
                    }
                    None
                }
                Action::ScrollViewportDown => {
                    let max_hour = self.config.max_hour;
                    if self.viewport_end_hour < max_hour {
                        let abs_hour = self.viewport_start_hour + (self.selected_time_slot / 4);
                        let abs_min_slot = self.selected_time_slot % 4;

                        let window = self.viewport_end_hour - self.viewport_start_hour;
                        self.viewport_end_hour = (self.viewport_end_hour + 2).min(max_hour);
                        self.viewport_start_hour = self.viewport_end_hour - window;

                        if abs_hour >= self.viewport_start_hour && abs_hour < self.viewport_end_hour
                        {
                            self.selected_time_slot =
                                (abs_hour - self.viewport_start_hour) * 4 + abs_min_slot;
                        } else {
                            self.selected_time_slot = self.max_time_slot();
                        }
                    }
                    None
                }
                _ => None,
            };
        }
        None
    }

    pub fn get_appointment_at_selection(&self) -> Option<&CalendarAppointment> {
        self.get_appointment_at_slot_for_practitioner(
            self.selected_time_slot,
            self.selected_practitioner_index,
        )
    }

    fn get_appointment_at_slot_for_practitioner(
        &self,
        slot: u8,
        practitioner_index: usize,
    ) -> Option<&CalendarAppointment> {
        let schedule = self.schedule_data.as_ref()?;
        let practitioner = self.practitioners_view.get(practitioner_index)?;

        schedule
            .practitioners
            .iter()
            .find(|ps| ps.practitioner_id == practitioner.id)
            .and_then(|ps| {
                ps.appointments
                    .iter()
                    .find(|apt| self.is_appointment_at_slot(apt, slot))
            })
    }

    fn is_appointment_at_slot(&self, apt: &CalendarAppointment, slot: u8) -> bool {
        let Some(start_slot) = self.time_to_slot(apt.start_time) else {
            return false;
        };
        let end_slot = start_slot.saturating_add(apt.slot_span).saturating_sub(1);
        slot >= start_slot && slot <= end_slot
    }

    pub fn handle_mouse(&mut self, mouse: MouseEvent, area: Rect) -> Option<ScheduleAction> {
        use crate::ui::layout::TIME_COLUMN_WIDTH;

        match mouse.kind {
            MouseEventKind::Moved => {
                // Track which slot is hovered
                let time_column_width = TIME_COLUMN_WIDTH;
                let inner = area.inner(ratatui::layout::Margin {
                    horizontal: 1,
                    vertical: 1,
                });

                // Check if mouse is within schedule area
                if mouse.column < inner.x || mouse.row < inner.y {
                    self.hovered_slot.clear_hover();
                    return None;
                }

                let y = mouse.row.saturating_sub(inner.y);
                let slot = (y as u8 / 2).min(self.max_time_slot());

                // Only set hover if not in time column
                if mouse.column >= inner.x + time_column_width {
                    let col = mouse.column.saturating_sub(inner.x + time_column_width);
                    let practitioner_cols = inner.width.saturating_sub(time_column_width);

                    if practitioner_cols > 0 && !self.practitioners_view.is_empty() {
                        let col_width = practitioner_cols / self.practitioners_view.len() as u16;
                        if col_width > 0 {
                            let practitioner_index = (col / col_width) as usize;
                            if practitioner_index < self.practitioners_view.len() {
                                self.hovered_slot.set_hovered(
                                    (practitioner_index, slot),
                                    (mouse.column, mouse.row),
                                );
                                return None;
                            }
                        }
                    }
                }
                self.hovered_slot.clear_hover();
                None
            }
            MouseEventKind::Up(crossterm::event::MouseButton::Left) => {
                let time_column_width = TIME_COLUMN_WIDTH;
                let inner = area.inner(ratatui::layout::Margin {
                    horizontal: 1,
                    vertical: 1,
                });

                let y = mouse.row.saturating_sub(inner.y);
                let slot = (y as u8 / 2).min(self.max_time_slot());
                self.selected_time_slot = slot;

                if mouse.column < inner.x + time_column_width {
                    return Some(ScheduleAction::NavigateTimeSlot(0));
                }

                let col = mouse.column.saturating_sub(inner.x + time_column_width);
                let practitioner_cols = inner.width.saturating_sub(time_column_width);

                if practitioner_cols > 0 && !self.practitioners_view.is_empty() {
                    let col_width = practitioner_cols / self.practitioners_view.len() as u16;
                    if col_width > 0 {
                        let practitioner_index = (col / col_width) as usize;
                        if practitioner_index < self.practitioners_view.len() {
                            self.selected_practitioner_index = practitioner_index;

                            if let Some(apt) = self.get_appointment_at_slot_for_practitioner(
                                slot,
                                self.selected_practitioner_index,
                            ) {
                                return Some(ScheduleAction::SelectAppointment(apt.id));
                            }

                            return Some(ScheduleAction::SelectPractitioner(
                                self.practitioners_view[practitioner_index].id,
                            ));
                        }
                    }
                }
                None
            }
            MouseEventKind::Down(crossterm::event::MouseButton::Left) => {
                // Check for double-click
                if self
                    .schedule_double_click_detector
                    .check_double_click_now(&mouse)
                {
                    let time_column_width = TIME_COLUMN_WIDTH;
                    let inner = area.inner(ratatui::layout::Margin {
                        horizontal: 1,
                        vertical: 1,
                    });

                    let y = mouse.row.saturating_sub(inner.y);
                    let slot = (y as u8 / 2).min(self.max_time_slot());
                    self.selected_time_slot = slot;

                    if mouse.column >= inner.x + time_column_width {
                        let col = mouse.column.saturating_sub(inner.x + time_column_width);
                        let practitioner_cols = inner.width.saturating_sub(time_column_width);

                        if practitioner_cols > 0 && !self.practitioners_view.is_empty() {
                            let col_width =
                                practitioner_cols / self.practitioners_view.len() as u16;
                            if col_width > 0 {
                                let practitioner_index = (col / col_width) as usize;
                                if practitioner_index < self.practitioners_view.len() {
                                    self.selected_practitioner_index = practitioner_index;

                                    // Try to select existing appointment, or trigger new creation
                                    if let Some(apt) = self
                                        .get_appointment_at_slot_for_practitioner(
                                            slot,
                                            self.selected_practitioner_index,
                                        )
                                    {
                                        return Some(ScheduleAction::SelectAppointment(apt.id));
                                    } else {
                                        // Create new appointment at this slot
                                        let time_str = self.slot_to_time(slot);
                                        return Some(ScheduleAction::CreateAtSlot {
                                            practitioner_id: self.practitioners_view
                                                [practitioner_index]
                                                .id,
                                            date: self
                                                .selected_date
                                                .unwrap_or_else(|| chrono::Utc::now().date_naive()),
                                            time: time_str,
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
                None
            }
            MouseEventKind::Down(crossterm::event::MouseButton::Right) => {
                // Right-click context menu
                let time_column_width = TIME_COLUMN_WIDTH;
                let inner = area.inner(ratatui::layout::Margin {
                    horizontal: 1,
                    vertical: 1,
                });

                let y = mouse.row.saturating_sub(inner.y);
                let slot = (y as u8 / 2).min(self.max_time_slot());

                if mouse.column >= inner.x + time_column_width {
                    let col = mouse.column.saturating_sub(inner.x + time_column_width);
                    let practitioner_cols = inner.width.saturating_sub(time_column_width);

                    if practitioner_cols > 0 && !self.practitioners_view.is_empty() {
                        let col_width = practitioner_cols / self.practitioners_view.len() as u16;
                        if col_width > 0 {
                            let practitioner_index = (col / col_width) as usize;
                            if practitioner_index < self.practitioners_view.len() {
                                self.selected_practitioner_index = practitioner_index;
                                self.selected_time_slot = slot;

                                // For context menu, try to select appointment or prepare new creation
                                if let Some(apt) = self.get_appointment_at_slot_for_practitioner(
                                    slot,
                                    practitioner_index,
                                ) {
                                    return Some(ScheduleAction::SelectAppointment(apt.id));
                                } else {
                                    // Return new creation action for context menu
                                    let time_str = self.slot_to_time(slot);
                                    return Some(ScheduleAction::CreateAtSlot {
                                        practitioner_id: self.practitioners_view
                                            [practitioner_index]
                                            .id,
                                        date: self
                                            .selected_date
                                            .unwrap_or_else(|| chrono::Utc::now().date_naive()),
                                        time: time_str,
                                    });
                                }
                            }
                        }
                    }
                }
                None
            }
            MouseEventKind::ScrollUp => {
                let min_hour = self.config.min_hour;
                if self.viewport_start_hour > min_hour {
                    self.viewport_start_hour = self.viewport_start_hour.saturating_sub(1);
                    self.viewport_end_hour = self.viewport_end_hour.saturating_sub(1);
                    self.scroll_viewport_to_show_selection();
                }
                None
            }
            MouseEventKind::ScrollDown => {
                let max_hour = self.config.max_hour;
                let window_hours = self.viewport_end_hour - self.viewport_start_hour;
                if self.viewport_end_hour < max_hour {
                    self.viewport_start_hour =
                        (self.viewport_start_hour + 1).min(max_hour - window_hours);
                    self.viewport_end_hour = (self.viewport_end_hour + 1).min(max_hour);
                    self.scroll_viewport_to_show_selection();
                }
                None
            }
            _ => None,
        }
    }
}

impl HasFocus for AppointmentState {
    fn build(&self, builder: &mut FocusBuilder) {
        builder.leaf_widget(self);
    }

    fn focus(&self) -> FocusFlag {
        self.focus.clone()
    }

    fn area(&self) -> Rect {
        Rect::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::theme::Theme;
    use opengp_config::CalendarConfig;

    fn create_test_state() -> AppointmentState {
        AppointmentState::new(Theme::dark(), CalendarConfig::default())
    }

    #[test]
    fn test_appointment_state_construction() {
        let state = create_test_state();

        assert_eq!(state.current_view, AppointmentView::Schedule);
        assert!(state.selected_date.is_some());
        assert!(state.schedule_data.is_none());
        assert!(state.practitioners.is_empty());
        assert!(state.selected_practitioner.is_none());
        assert!(state.selected_appointment.is_none());
        assert!(!state.is_loading());
    }

    #[test]
    fn test_appointment_state_initial_date_is_today() {
        let state = create_test_state();
        let today = chrono::Utc::now().date_naive();

        assert_eq!(state.selected_date(), Some(today));
    }

    #[test]
    fn test_view_switching_to_calendar() {
        let mut state = create_test_state();
        assert_eq!(state.current_view, AppointmentView::Schedule);

        state.set_view(AppointmentView::Calendar);
        assert_eq!(state.current_view, AppointmentView::Calendar);
    }

    #[test]
    fn test_view_switching_to_schedule() {
        let mut state = create_test_state();
        state.set_view(AppointmentView::Calendar);
        assert_eq!(state.current_view, AppointmentView::Calendar);

        state.set_view(AppointmentView::Schedule);
        assert_eq!(state.current_view, AppointmentView::Schedule);
    }

    #[test]
    fn test_date_selection_set_and_get() {
        let mut state = create_test_state();
        let test_date = chrono::NaiveDate::from_ymd_opt(2025, 6, 15).unwrap();

        state.set_selected_date(Some(test_date));
        assert_eq!(state.selected_date(), Some(test_date));
    }

    #[test]
    fn test_date_selection_clear() {
        let mut state = create_test_state();
        assert!(state.selected_date().is_some());

        state.set_selected_date(None);
        assert_eq!(state.selected_date(), None);
    }

    #[test]
    fn test_practitioner_selection_lifecycle() {
        let mut state = create_test_state();
        let practitioner_id = Uuid::new_v4();

        assert!(state.selected_practitioner().is_none());

        state.set_selected_practitioner(Some(practitioner_id));
        assert_eq!(state.selected_practitioner(), Some(practitioner_id));

        state.set_selected_practitioner(None);
        assert!(state.selected_practitioner().is_none());
    }

    #[test]
    fn test_appointment_selection_lifecycle() {
        let mut state = create_test_state();
        let appointment_id = Uuid::new_v4();

        assert!(state.selected_appointment().is_none());

        state.set_selected_appointment(Some(appointment_id));
        assert_eq!(state.selected_appointment(), Some(appointment_id));

        state.set_selected_appointment(None);
        assert!(state.selected_appointment().is_none());
    }

    #[test]
    fn test_loading_state_management() {
        let mut state = create_test_state();
        assert!(!state.is_loading());

        state.set_loading(true);
        assert!(state.is_loading());

        state.set_loading(false);
        assert!(!state.is_loading());
    }

    #[test]
    fn test_clear_selections_resets_all_selections() {
        let mut state = create_test_state();
        let test_date = chrono::NaiveDate::from_ymd_opt(2025, 6, 15).unwrap();
        let practitioner_id = Uuid::new_v4();
        let appointment_id = Uuid::new_v4();

        state.set_selected_date(Some(test_date));
        state.set_selected_practitioner(Some(practitioner_id));
        state.set_selected_appointment(Some(appointment_id));

        assert!(state.selected_date().is_some());
        assert!(state.selected_practitioner().is_some());
        assert!(state.selected_appointment().is_some());

        state.clear_selections();

        assert!(state.selected_date().is_none());
        assert!(state.selected_practitioner().is_none());
        assert!(state.selected_appointment().is_none());
        assert!(state.schedule_data.is_none());
    }

    #[test]
    fn test_toggle_column_hides_then_shows() {
        let mut state = create_test_state();
        let practitioner_id = Uuid::new_v4();

        // Initially visible (not hidden)
        assert!(!state.is_column_hidden(practitioner_id));

        // Toggle once → hidden
        state.toggle_column(practitioner_id, 3);
        assert!(state.is_column_hidden(practitioner_id));

        // Toggle again → visible
        state.toggle_column(practitioner_id, 3);
        assert!(!state.is_column_hidden(practitioner_id));
    }

    #[test]
    fn test_toggle_last_visible_column_is_noop() {
        let mut state = create_test_state();
        let practitioner_id = Uuid::new_v4();

        // Only 1 practitioner, hiding it should be a no-op
        state.toggle_column(practitioner_id, 1);
        assert!(!state.is_column_hidden(practitioner_id)); // Still visible
    }

    #[test]
    fn test_clear_selections_resets_hidden_columns() {
        let mut state = create_test_state();
        let practitioner_id = Uuid::new_v4();

        state.toggle_column(practitioner_id, 3);
        assert!(state.is_column_hidden(practitioner_id));

        state.clear_selections();
        assert!(!state.is_column_hidden(practitioner_id)); // Reset
    }

    #[test]
    fn test_visible_practitioners_excludes_hidden() {
        let mut state = create_test_state();
        let p1 = Uuid::new_v4();
        let p2 = Uuid::new_v4();
        let p3 = Uuid::new_v4();

        use chrono::{DateTime, Utc};
        use opengp_domain::domain::user::Practitioner;

        let now = Utc::now();
        state.practitioners = vec![
            Practitioner {
                id: p1,
                user_id: None,
                first_name: "Alice".to_string(),
                middle_name: None,
                last_name: "Doctor".to_string(),
                title: "Dr".to_string(),
                hpi_i: None,
                ahpra_registration: None,
                prescriber_number: None,
                provider_number: "12345".to_string(),
                speciality: None,
                qualifications: vec![],
                phone: None,
                email: None,
                is_active: true,
                created_at: now,
                updated_at: now,
            },
            Practitioner {
                id: p2,
                user_id: None,
                first_name: "Bob".to_string(),
                middle_name: None,
                last_name: "Doctor".to_string(),
                title: "Dr".to_string(),
                hpi_i: None,
                ahpra_registration: None,
                prescriber_number: None,
                provider_number: "12346".to_string(),
                speciality: None,
                qualifications: vec![],
                phone: None,
                email: None,
                is_active: true,
                created_at: now,
                updated_at: now,
            },
            Practitioner {
                id: p3,
                user_id: None,
                first_name: "Carol".to_string(),
                middle_name: None,
                last_name: "Doctor".to_string(),
                title: "Dr".to_string(),
                hpi_i: None,
                ahpra_registration: None,
                prescriber_number: None,
                provider_number: "12347".to_string(),
                speciality: None,
                qualifications: vec![],
                phone: None,
                email: None,
                is_active: true,
                created_at: now,
                updated_at: now,
            },
        ];

        state.toggle_column(p2, 3);

        let visible: Vec<_> = state.visible_practitioners();
        assert_eq!(visible.len(), 2);
        assert!(!visible.iter().any(|p| p.id == p2));
    }
}

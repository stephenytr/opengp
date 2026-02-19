use chrono::NaiveDate;
use uuid::Uuid;

use crate::domain::appointment::CalendarDayView;
use crate::domain::user::Practitioner;

use super::calendar::Calendar;
use super::schedule::Schedule;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppointmentView {
    Calendar,
    Schedule,
}

#[derive(Debug, Clone)]
pub struct AppointmentState {
    pub current_view: AppointmentView,
    pub calendar: Calendar,
    pub schedule: Schedule,
    pub selected_date: Option<NaiveDate>,
    pub schedule_data: Option<CalendarDayView>,
    pub practitioners: Vec<Practitioner>,
    pub selected_practitioner: Option<Uuid>,
    pub selected_appointment: Option<Uuid>,
    pub is_loading: bool,
}

impl AppointmentState {
    pub fn new(theme: crate::ui::theme::Theme) -> Self {
        Self {
            current_view: AppointmentView::Calendar,
            calendar: Calendar::new(theme.clone()),
            schedule: Schedule::new(theme),
            selected_date: Some(chrono::Utc::now().date_naive()),
            schedule_data: None,
            practitioners: Vec::new(),
            selected_practitioner: None,
            selected_appointment: None,
            is_loading: false,
        }
    }
}

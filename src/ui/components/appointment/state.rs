use chrono::NaiveDate;

use super::calendar::Calendar;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppointmentView {
    Calendar,
    Schedule,
}

#[derive(Debug, Clone)]
pub struct AppointmentState {
    pub current_view: AppointmentView,
    pub calendar: Calendar,
    pub selected_date: Option<NaiveDate>,
}

impl AppointmentState {
    pub fn new(theme: crate::ui::theme::Theme) -> Self {
        Self {
            current_view: AppointmentView::Calendar,
            calendar: Calendar::new(theme),
            selected_date: None,
        }
    }
}

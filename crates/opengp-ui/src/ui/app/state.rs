mod api_polling;
mod appointment;
mod billing;
mod clinical;
mod patient;

use crate::ui::app::App;

impl App {
    pub fn set_status_error(&mut self, message: impl Into<String>) {
        self.status_bar.set_error(message);
    }

    pub fn set_status_success(&mut self, message: impl Into<String>) {
        self.status_bar.set_left(message);
    }
}

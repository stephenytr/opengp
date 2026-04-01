mod api_polling;
mod appointment;
#[cfg(feature = "billing")]
mod billing;
mod clinical;
mod patient;

use crate::ui::app::App;

impl App {
    pub fn set_status_error(&mut self, message: impl Into<String>) {
        self.status_bar.set_error(message);
    }
}

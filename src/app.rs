//! Main application orchestration
//!
//! The App struct coordinates all components, manages application state,
//! and handles the main event loop.

use crate::config::Config;
use crate::error::Result;

/// Main application struct
///
/// Coordinates all components and manages the application lifecycle.
pub struct App {
    config: Config,
    should_quit: bool,
}

impl App {
    /// Create a new App instance
    pub fn new(config: Config) -> Result<Self> {
        Ok(Self {
            config,
            should_quit: false,
        })
    }

    /// Run the application
    pub async fn run(&mut self) -> Result<()> {
        // TODO: Initialize TUI
        // TODO: Initialize components
        // TODO: Start event loop

        Ok(())
    }

    /// Signal the application to quit
    pub fn quit(&mut self) {
        self.should_quit = true;
    }
}

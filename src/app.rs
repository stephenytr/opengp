//! Main application orchestration
//!
//! The App struct coordinates all components, manages application state,
//! and handles the main event loop.

use crate::config::Config;
use crate::error::Result;
use sqlx::SqlitePool;
use tracing::info;

/// Main application struct
///
/// Coordinates all components and manages the application lifecycle.
pub struct App {
    config: Config,
    db_pool: SqlitePool,
    should_quit: bool,
}

impl App {
    /// Create a new App instance
    pub fn new(config: Config, db_pool: SqlitePool) -> Result<Self> {
        info!("Initializing application");
        
        Ok(Self {
            config,
            db_pool,
            should_quit: false,
        })
    }

    /// Run the application
    pub async fn run(&mut self) -> Result<()> {
        info!("Starting application main loop");

        Ok(())
    }

    /// Signal the application to quit
    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    /// Get reference to database pool
    pub fn db_pool(&self) -> &SqlitePool {
        &self.db_pool
    }
}

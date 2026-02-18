use color_eyre::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use opengp::domain::patient::PatientRepository;
use opengp::infrastructure::crypto::EncryptionService;
use opengp::infrastructure::database::{create_pool, run_migrations};
use opengp::infrastructure::database::repositories::patient::SqlxPatientRepository;
use opengp::ui::app::App;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use opengp::Config;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let config = Config::from_env()?;

    init_logging(&config.log_level);

    tracing::info!("Starting OpenGP");
    tracing::info!("Database URL: {}", config.database.url);

    let db_pool = create_pool(&config.database).await?;

    run_migrations(&db_pool).await?;

    tracing::info!("Database pool created with {} connection(s)", db_pool.size());

    let crypto = Arc::new(EncryptionService::new()?);
    let patient_repo = SqlxPatientRepository::new(db_pool.clone(), crypto);

    let patients: Vec<opengp::domain::patient::Patient> = patient_repo.list_active().await?;
    tracing::info!("Loaded {} patients from database", patients.len());

    run_tui(patients, patient_repo).await?;

    tracing::info!("OpenGP shutdown complete");

    Ok(())
}

async fn run_tui(
    patients: Vec<opengp::domain::patient::Patient>,
    patient_repo: SqlxPatientRepository,
) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    app.load_patients(patients);

    loop {
        terminal.draw(|frame| {
            app.render(frame);
        })?;

        // Check if there's pending patient data to save
        if let Some(pending) = app.take_pending_patient_data() {
            use opengp::domain::patient::Patient;
            match pending {
                opengp::ui::app::PendingPatientData::New(data) => {
                    let patient = Patient::from_dto(data)?;
                    patient_repo.create(patient).await?;
                    tracing::info!("Created new patient in database");
                }
                opengp::ui::app::PendingPatientData::Update { id, data } => {
                    let mut patient = patient_repo.find_by_id(id).await?.ok_or_else(|| color_eyre::eyre::eyre!("Patient not found"))?;
                    patient.update(data)?;
                    patient_repo.update(patient).await?;
                    tracing::info!("Updated patient in database");
                }
            }
            
            // Reload patients to update the list
            let patients = patient_repo.list_active().await?;
            app.load_patients(patients);
        }

        if let Ok(event) = crossterm::event::read() {
            match event {
                Event::Key(key) => {
                    let action = app.handle_key_event(key);

                    if action == opengp::ui::keybinds::Action::Quit || app.should_quit() {
                        break;
                    }
                }
                Event::Mouse(mouse) => {
                    let terminal_size = terminal.size().unwrap_or_default();
                    let terminal_rect = ratatui::layout::Rect::new(0, 0, terminal_size.width, terminal_size.height);
                    app.handle_mouse_event(mouse, terminal_rect);
                }
                Event::Resize(_, _) => {}
                _ => {}
            }
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn init_logging(level: &str) {
    let log_level = level.parse().unwrap_or(tracing::Level::INFO);

    std::fs::create_dir_all("logs").ok();

    let log_file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("logs/opengp.log")
        .expect("Failed to open log file");

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(std::sync::Arc::new(log_file))
                .with_target(true)
                .with_thread_ids(false)
                .with_line_number(true)
                .with_ansi(false),
        )
        .with(
            tracing_subscriber::filter::Targets::new()
                .with_target("opengp", log_level)
                .with_default(tracing::Level::WARN),
        )
        .init();
}

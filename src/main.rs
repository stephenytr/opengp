use color_eyre::Result;
use opengp::infrastructure::database::{create_pool, run_migrations};
use opengp::{App, Config};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let config = Config::from_env()?;

    init_logging(&config.log_level);

    tracing::info!("Starting OpenGP");
    tracing::info!("Database URL: {}", config.database.url);

    let db_pool = create_pool(&config.database).await?;

    run_migrations(&db_pool).await?;

    let mut app = App::new(config, db_pool)?;

    app.run().await?;

    tracing::info!("OpenGP shutdown complete");

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

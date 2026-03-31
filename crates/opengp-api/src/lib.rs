mod config;
mod error;
mod migrations;
mod routes;
mod services;
mod state;

use color_eyre::Result;
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub use config::ApiConfig;
pub use error::ApiError;
pub use routes::router;
pub use state::ApiState;

pub async fn run() -> Result<()> {
    let config = ApiConfig::from_env()?;
    init_tracing(&config.log_level);

    let state = ApiState::new(config).await?;
    let app = routes::router(state.clone());
    let bind_addr = state.config.bind_address();

    let listener = TcpListener::bind(&bind_addr).await?;
    info!("opengp-api listening on {}", bind_addr);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

fn init_tracing(level: &str) {
    let log_level = level.parse().unwrap_or(tracing::Level::INFO);

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(true)
                .with_line_number(true),
        )
        .with(
            tracing_subscriber::filter::Targets::new()
                .with_target("opengp_api", log_level)
                .with_default(tracing::Level::WARN),
        )
        .init();
}

async fn shutdown_signal() {
    let ctrl_c = async {
        if let Err(e) = tokio::signal::ctrl_c().await {
            tracing::error!("failed to install Ctrl+C signal handler: {}", e);
        }
    };

    #[cfg(unix)]
    let terminate = async {
        let signal = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate());
        match signal {
            Ok(mut s) => {
                s.recv().await;
            }
            Err(e) => {
                tracing::error!("failed to install SIGTERM signal handler: {}", e);
            }
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("received Ctrl+C, starting graceful shutdown");
        }
        _ = terminate => {
            info!("received SIGTERM, starting graceful shutdown");
        }
    }
}

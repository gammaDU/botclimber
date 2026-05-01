use anyhow::Result;
use std::sync::Arc;
use tracing::info;

mod api;
mod config;
mod db;
mod error;
mod game;
mod models;
mod state;

use crate::config::AppConfig;
use crate::db::Database;
use crate::state::AppState;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "botclimber=debug,tower_http=debug".into()),
        )
        .init();

    let cfg = AppConfig::load()?;
    info!("Configuration loaded");

    let db = Database::open(&cfg.database_path)?;
    info!("Database ready at {}", cfg.database_path);

    let state = Arc::new(AppState::new(cfg.clone(), db));

    let app = api::router(state.clone());

    let bind_addr = cfg.server_bind.parse::<std::net::SocketAddr>()?;
    info!("Listening on {bind_addr}");

    let listener = tokio::net::TcpListener::bind(bind_addr).await?;

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    info!("Server shut down cleanly");
    Ok(())
}

async fn shutdown_signal() {
    use tokio::signal;

    let ctrl_c = async {
        signal::ctrl_c().await.expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("Shutdown signal received");
}

use anyhow::Result;
use std::sync::Arc;
use tracing::info;
use std::env;

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

    let config_path = env::args().nth(1).unwrap_or_else(|| "server_config".to_string());
    let cfg = AppConfig::load(&config_path)?;
    info!("Configuration loaded");

    let db = Database::open(&cfg.database.path)?;
    info!("Database ready at {}", cfg.database.path);

    let state = Arc::new(AppState::new(cfg.clone(), db));

    game::tick::spawn(state.clone());

    let app = api::router(state.clone());

    let bind_addr = cfg.server.bind.parse::<std::net::SocketAddr>()?;
    info!("Server listening on http://{bind_addr}");

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

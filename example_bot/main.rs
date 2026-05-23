/// Example bot that connects to a running Botclimber server and jumps every 5 seconds.
///
/// Configuration is read from a `.env` file (or environment variables):
///   BOT_API_KEY   - (required) API key for the bot account
///   BOT_BASE_URL  - base URL of the server (default: http://localhost:8080)
///   BOT_NAME      - display name used in logs  (default: ExampleBot)
///
/// Obtain an API key by logging in to the web UI and generating one on the
/// account page, then put it in a `.env` file next to this crate:
///
///   BOT_API_KEY=your_api_key_here
///
/// The bot will:
///   1. Read the API key from the environment / .env file.
///   2. Join the game session.
///   3. Jump straight up every 5 seconds until interrupted (Ctrl-C).

use anyhow::{Context, Result};
use reqwest::Client;
use serde::Deserialize;
use tokio::time::{Duration, sleep};

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct JoinResponse {
    player_id: String,
    session_id: String,
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env file if present (silently ignore if missing)
    let _ = dotenvy::dotenv();

    let api_key = std::env::var("BOT_API_KEY")
        .context("BOT_API_KEY must be set (in .env or environment)")?;
    let base_url =
        std::env::var("BOT_BASE_URL").unwrap_or_else(|_| "http://localhost:8080".into());
    
    println!("Botclimber example bot");
    println!("  server  : {base_url}");
    println!("  bot name: {bot_name}");
    println!();

    let client = Client::new();

    // 1. Join the game
    let join_res = client
        .post(format!("{base_url}/api/v1/bot/join"))
        .header("X-API-Key", &api_key)
        .send()
        .await
        .context("join request failed")?;

    if !join_res.status().is_success() {
        let body = join_res.text().await.unwrap_or_default();
        anyhow::bail!("Join failed: {body}");
    }
    let JoinResponse { player_id, session_id } =
        join_res.json().await.context("parse join response")?;
    println!("Joined session {session_id} as player {player_id}.");
    println!();
    println!("Jumping every 5 seconds. Press Ctrl-C to stop.");

    // 2. Jump loop
    loop {
        sleep(Duration::from_secs(5)).await;

        let jump_res = client
            .post(format!("{base_url}/api/v1/bot/jump"))
            .header("X-API-Key", &api_key)
            .json(&serde_json::json!({
                "direction": { "angle_deg": -80 },
                "strength": 10.0
            }))
            .send()
            .await;

        match jump_res {
            Ok(r) if r.status().is_success() => println!("Jumped!"),
            Ok(r) => println!("Jump returned status {}", r.status()),
            Err(e) => println!("Jump request error: {e}"),
        }
    }
}

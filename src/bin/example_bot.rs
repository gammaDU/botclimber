/// Example bot that connects to a running Botclimber server and jumps every 5 seconds.
///
/// Usage:
///   cargo run --bin example-bot -- [base_url] [username] [password] [bot_name]
///
/// Defaults:
///   base_url  = http://localhost:8080
///   username  = example_bot
///   password  = example_bot_password
///   bot_name  = ExampleBot
///
/// The bot will:
///   1. Register a new account (ignores 409 Conflict if already registered).
///   2. Log in to obtain a JWT.
///   3. Generate / retrieve an API key.
///   4. Join the game session.
///   5. Jump straight up every 5 seconds until interrupted (Ctrl-C).

use anyhow::{Context, Result};
use reqwest::Client;
use serde::Deserialize;
use std::env;
use tokio::time::{Duration, sleep};

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct LoginResponse {
    token: String,
}

#[derive(Deserialize)]
struct ApiKeyResponse {
    api_key: String,
}

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
    let mut args = env::args().skip(1);
    let base_url = args.next().unwrap_or_else(|| "http://localhost:8080".into());
    let username = args.next().unwrap_or_else(|| "example_bot".into());
    let password = args.next().unwrap_or_else(|| "example_bot_password".into());
    let bot_name = args.next().unwrap_or_else(|| "ExampleBot".into());

    println!("Botclimber example bot");
    println!("  server  : {base_url}");
    println!("  username: {username}");
    println!("  bot name: {bot_name}");
    println!();

    let client = Client::new();

    // 1. Register (ignore 409 if account already exists)
    let reg_res = client
        .post(format!("{base_url}/api/v1/user/register"))
        .json(&serde_json::json!({ "username": username, "password": password }))
        .send()
        .await
        .context("register request failed")?;

    match reg_res.status().as_u16() {
        201 => println!("Registered new account."),
        409 => println!("Account already exists, continuing."),
        s => {
            let body = reg_res.text().await.unwrap_or_default();
            anyhow::bail!("Unexpected register status {s}: {body}");
        }
    }

    // 2. Login → JWT
    let login_res = client
        .post(format!("{base_url}/api/v1/user/login"))
        .json(&serde_json::json!({ "username": username, "password": password }))
        .send()
        .await
        .context("login request failed")?;

    if !login_res.status().is_success() {
        let body = login_res.text().await.unwrap_or_default();
        anyhow::bail!("Login failed: {body}");
    }
    let LoginResponse { token } = login_res.json().await.context("parse login response")?;
    println!("Logged in, JWT obtained.");

    // 3. Generate API key
    let apikey_res = client
        .post(format!("{base_url}/api/v1/user/apikey"))
        .bearer_auth(&token)
        .send()
        .await
        .context("apikey request failed")?;

    if !apikey_res.status().is_success() {
        let body = apikey_res.text().await.unwrap_or_default();
        anyhow::bail!("API key generation failed: {body}");
    }
    let ApiKeyResponse { api_key } = apikey_res.json().await.context("parse apikey response")?;
    println!("API key obtained.");

    // 4. Join the game
    let join_res = client
        .post(format!("{base_url}/api/v1/bot/join"))
        .header("X-API-Key", &api_key)
        .json(&serde_json::json!({ "name": bot_name }))
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

    // 5. Jump loop
    loop {
        sleep(Duration::from_secs(5)).await;

        let jump_res = client
            .post(format!("{base_url}/api/v1/bot/jump"))
            .header("X-API-Key", &api_key)
            .json(&serde_json::json!({
                "direction": { "angle_deg": -45 },
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

/// Example bot that connects to a running Botclimber server and jumps every 5 seconds.
///
/// Configuration is read from a `.env` file (or environment variables):
///   BOT_API_KEY   - (required) API key for the bot account
///   BOT_BASE_URL  - base URL of the server (default: http://localhost:8080)
///
/// Obtain an API key by logging in to the web UI and generating one on the
/// account page, then put it in a `.env` file next to this crate:
///
///   BOT_API_KEY=your_api_key_here
mod bot;

use anyhow::{Context, Result, bail};
use tokio::time::{Duration, sleep};

use crate::bot::Vector2;
use bot::{Bot, JoinResponse};

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env file from the example_bot directory (relative to this source file),
    // or fall back to the current working directory. Silently ignore if missing.
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("example_bot/.env");
    let _ = dotenvy::from_path(&manifest_dir).or_else(|_| dotenvy::dotenv().map(|_| ()));

    let api_key =
        std::env::var("BOT_API_KEY").context("BOT_API_KEY must be set (in .env or environment)")?;
    let base_url = std::env::var("BOT_BASE_URL").unwrap_or_else(|_| "http://localhost:8080".into());

    println!("Botclimber example bot");
    println!("  server  : {base_url}");
    println!();

    let bot = Bot::new(api_key, base_url);

    // 1. Join the game
    let JoinResponse {
        player_id,
        session_id,
    } = bot.join().await?;

    println!("Joined session {session_id} as player {player_id}.");
    println!("================");

    // 2. Jump loop
    let mut counter = 2;
    loop {
        sleep(Duration::from_secs(1)).await;

        let state = bot.state().await?;

        if state.state != "Landed" {
            continue;
        }

        let next_platform = state
            .surrounding_platforms
            .iter()
            .filter(|p| p.rect.y >= state.position.y)
            .min_by(|a, b| {
                a.rect
                    .y
                    .partial_cmp(&b.rect.y)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        let Some(next_platform) = next_platform else {
            bail!("No platforms found!");
        };
        let target = Vector2::new(
            next_platform.rect.x + next_platform.rect.width / 2.0,
            next_platform.rect.y + next_platform.rect.height,
        );

        let jump = calc_jump(state.position, target);
        if let Some((angle_deg, strength)) = jump {
            println!("Jumping to {target} (angle_deg={angle_deg}, strength={strength})");
            sleep(Duration::from_secs(1)).await;
            bot.jump(angle_deg, strength).await?;
        }
    }
}

const GRAVITY: f64 = -10.0;
const MAX_STRENGTH: f64 = 20.0;

/// Calculate (angle_deg, strength) to reach `target` from `origin`.
/// Returns None if the target is unreachable within max strength.
pub fn calc_jump(origin: Vector2, target: Vector2) -> Option<(f64, f64)> {
    let dx = target.x - origin.x;
    let dy = target.y - origin.y;

    // Try a range of flight times and pick the one with the lowest valid strength
    let mut best: Option<(f64, f64)> = None;

    let steps = 200;
    for i in 1..=steps {
        let t = i as f64 * 0.05; // try t from 0.05s to 10s

        let vx = dx / t;
        let vy = dy / t - 0.5 * GRAVITY * t;

        let strength = (vx * vx + vy * vy).sqrt();
        if strength > MAX_STRENGTH {
            continue;
        }

        // Prefer the solution with the smallest strength (most efficient)
        if best.is_none() || strength < best.unwrap().1 {
            // angle_deg: 0 = up (+y), clockwise → atan2(vx, vy)
            let angle_deg = vx.atan2(vy).to_degrees();
            let angle_deg = if angle_deg < 0.0 {
                angle_deg + 360.0
            } else {
                angle_deg
            };
            best = Some((angle_deg, strength));
        }
    }

    best
}

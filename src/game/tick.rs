use std::sync::Arc;
use std::time::Duration;
use chrono::Utc;
use serde_json::json;
use tokio::time;
use tracing::warn;

use crate::game::physics;
use crate::state::AppState;

/// Spawn the game loop as a background tokio task.
/// Ticks all players at `physics::TICK_RATE_HZ` and broadcasts `position_update` events.
pub fn spawn(state: Arc<AppState>) {
    tokio::spawn(async move {
        let interval_duration = Duration::from_secs_f64(1.0 / physics::TICK_RATE_HZ);
        let mut interval = time::interval(interval_duration);
        interval.set_missed_tick_behavior(time::MissedTickBehavior::Skip);

        loop {
            interval.tick().await;

            let mut game = state.game.write().await;
            game.world.tick = game.world.tick.wrapping_add(1);

            let platforms = game.world.platforms.clone();
            let timestamp = Utc::now().to_rfc3339();

            let mut dead_players: Vec<uuid::Uuid> = Vec::new();

            for (player_id, player) in game.players.iter_mut() {
                let died = physics::tick_player(player, &platforms);
                if died {
                    dead_players.push(*player_id);
                }

                let event = json!({
                    "type": "position_update",
                    "timestamp": timestamp,
                    "data": {
                        "player_id": player_id.to_string(),
                        "x": player.position.x,
                        "y": player.position.y,
                        "vx": player.velocity.x,
                        "vy": player.velocity.y,
                        "state": format!("{:?}", player.state).to_uppercase(),
                        "consecutive_bounces": player.consecutive_bounces,
                    }
                });

                if let Ok(json_str) = serde_json::to_string(&event) {
                    state.broadcast(json_str);
                } else {
                    warn!("Failed to serialize position_update event");
                }
            }

            // Reset dead players to their last landed position
            for player_id in dead_players {
                if let Some(player) = game.players.get_mut(&player_id) {
                    player.position = player.last_landed_position;
                    player.velocity = crate::models::Vector2::zero();
                    player.session_max_height = player.position.y;
                    player.consecutive_bounces = 0;
                    player.state = crate::models::PlayerMovementState::Landed;
                }
            }
        }
    });
}

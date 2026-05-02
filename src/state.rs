use std::cmp::PartialEq;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};
use uuid::Uuid;

use crate::config::AppConfig;
use crate::db::Database;
use crate::game::world;
use crate::models::{GameMode, PlayerMovementState, State, Vector2, WorldState};

/// Capacity of the broadcast channel for WebSocket events.
const BROADCAST_CAPACITY: usize = 256;

/// The shared application state passed to every Axum handler via `Arc<AppState>`.
pub struct AppState {
    pub config: AppConfig,
    pub db: Database,
    /// The live game state, protected by a read-write lock.
    pub game: RwLock<State>,
    /// Broadcast sender for WebSocket events.
    /// Handlers clone the sender; spectators subscribe via `sender.subscribe()`.
    pub ws_tx: broadcast::Sender<Arc<String>>,
    /// Maps api_key_hash → player_id for fast bot-endpoint lookups.
    pub api_key_map: RwLock<HashMap<String, Uuid>>,
}

impl AppState {
    pub fn new(config: AppConfig, db: Database) -> Self {
        let game_mode = config
            .game
            .default_game_mode
            .parse::<GameMode>()
            .unwrap_or(GameMode::Checkpoint);

        let initial_state = State {
            session_id: Uuid::new_v4(),
            game_mode,
            world: WorldState {
                platforms: world::initial_platforms(),
                tick: 0,
                seed: 0, // TODO: generate or load seed for deterministic world generation
            },
            players: HashMap::new(),
        };

        let (ws_tx, _) = broadcast::channel(BROADCAST_CAPACITY);

        Self {
            config,
            db,
            game: RwLock::new(initial_state),
            ws_tx,
            api_key_map: RwLock::new(HashMap::new()),
        }
    }

    pub fn broadcast_player_joined(&self, player_id: Uuid, display_name: String) {
        let event = serde_json::json!({
            "type": "player_joined",
            "timestamp": Utc::now().timestamp_millis(),
            "data": {
                "player_id": player_id.to_string(),
                "name": display_name,
            }
        });
        self.broadcast(BroadcastType::PlayerJoined, event.to_string());
    }

    pub fn broadcast_player_left(&self, player_id: Uuid) {
        let event = serde_json::json!({
            "type": "player_left",
            "timestamp": Utc::now().timestamp_millis(),
            "data" : {
                "player_id": player_id,
            }
        });
        self.broadcast(BroadcastType::PlayerLeft, event.to_string());
    }

    pub fn broadcast_position_update(
        &self,
        player_id: Uuid,
        position: Vector2,
        velocity: Vector2,
        state: PlayerMovementState,
    ) {
        let event = serde_json::json!({
            "type": "position_update",
            "timestamp": Utc::now().timestamp_millis(),
            "data": {
                "player_id": player_id.to_string(),
                "x": position.x,
                "y": position.y,
                "vx": velocity.x,
                "vy": velocity.y,
                "state": format!("{:?}", state).to_uppercase(),
            }
        });
        self.broadcast(BroadcastType::PositionUpdate, event.to_string());
    }

    pub fn broadcast_world_reset(&self, mode: GameMode) {
        let event = serde_json::json!({
            "type": "world_reset",
            "timestamp": Utc::now().timestamp_millis(),
            "data": {
                "game_mode": mode.to_string(),
            },
        });
        self.broadcast(BroadcastType::WorldReset, event.to_string());
    }

    /// Broadcast a JSON-serialised event string to all WebSocket subscribers.
    /// Silently ignores send errors (no active subscribers).
    fn broadcast(&self, broadcast_type: BroadcastType,event_json: String) {
        if broadcast_type != BroadcastType::PositionUpdate {
            tracing::info!(event = %event_json, "ws_event");
        }
        let _ = self.ws_tx.send(Arc::new(event_json));
    }
}

#[derive(PartialEq)]
enum BroadcastType {
    PlayerJoined,
    PlayerLeft,
    PositionUpdate,
    WorldReset,
}

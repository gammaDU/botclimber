use chrono::Utc;
use std::cmp::PartialEq;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};
use uuid::Uuid;

use crate::config::AppConfig;
use crate::db::Database;
use crate::game::world;
use crate::models::{GameModeConfig, State, WorldState, StageType};

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
        let session_id = Uuid::new_v4();
        let mut game_mode = GameModeConfig::default();

        // Allow overriding the default game mode from config.
        if config.game.default_game_mode == "Checkpoint" {
            game_mode.fall_mode = crate::models::FallMode::Checkpoint;
        }

        if let Ok(st) = config.game.default_session_type.parse() {
            game_mode.session_type = st;
        }

        if let Ok(st) = config.game.default_stage_type.parse() {
            game_mode.stage_type = st;
        }

        if let Some(ref stage_id) = config.game.default_stage_id {
            game_mode.fixed_stage_id = Some(stage_id.clone());
        }

        // Handle stage configuration
        let platforms = if game_mode.stage_type == StageType::Fixed {
            let stage = if let Some(ref sid) = game_mode.fixed_stage_id {
                config.fixed_stages.iter().find(|s| &s.id == sid)
            } else {
                config.fixed_stages.first()
            };

            if let Some(stage) = stage {
                game_mode.fixed_stage_id = Some(stage.id.clone());
                game_mode.goal_height = Some(stage.goal_height);
                world::platforms_from_fixed_stage(stage)
            } else {
                // Fallback if fixed stage not found
                game_mode.stage_type = StageType::Generated;
                world::initial_platforms()
            }
        } else {
            world::initial_platforms()
        };

        let initial_state = State {
            session_id,
            game_mode: game_mode.clone(),
            world: WorldState {
                platforms,
                tick: 0,
                seed: 0, // TODO: generate or load seed for deterministic world generation
            },
            players: HashMap::new(),
        };

        let (ws_tx, _) = broadcast::channel(BROADCAST_CAPACITY);

        let app_state = Self {
            config,
            db,
            game: RwLock::new(initial_state),
            ws_tx,
            api_key_map: RwLock::new(HashMap::new()),
        };

        app_state.persist_session(session_id, &game_mode);

        app_state
    }

    /// Persists a new session record to the database. Logs a warning on failure.
    pub fn persist_session(&self, session_id: Uuid, game_mode: &GameModeConfig) {
        let started_at = Utc::now().to_rfc3339();
        let fall_mode = game_mode.fall_mode.to_string();
        let session_type = game_mode.session_type.to_string();
        let stage_type = game_mode.stage_type.to_string();
        if let Err(e) = self.db.create_session(
            &session_id.to_string(),
            &fall_mode,
            &session_type,
            &stage_type,
            None,
            game_mode.goal_height,
            game_mode.fixed_stage_id.as_deref(),
            &started_at,
        ) {
            tracing::warn!(error = %e, session_id = %session_id, "failed to persist session record");
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
        position: crate::models::Vector2,
        velocity: crate::models::Vector2,
        state: crate::models::PlayerMovementState,
        session_start_time: chrono::DateTime<chrono::Utc>,
        best_time: Option<f64>,
    ) {
        let now = chrono::Utc::now();
        let current_time = (now - session_start_time).num_milliseconds() as f64 / 1000.0;
        let event = serde_json::json!({
            "type": "position_update",
            "timestamp": now.timestamp_millis(),
            "data": {
                "player_id": player_id.to_string(),
                "x": position.x,
                "y": position.y,
                "vx": velocity.x,
                "vy": velocity.y,
                "state": format!("{:?}", state).to_uppercase(),
                "current_time": current_time,
                "best_time": best_time,
            }
        });
        self.broadcast(BroadcastType::PositionUpdate, event.to_string());
    }

    pub fn broadcast_world_reset(&self, mode: &GameModeConfig) {
        let event = serde_json::json!({
            "type": "world_reset",
            "timestamp": Utc::now().timestamp_millis(),
            "data": {
                "game_mode": mode,
            },
        });
        self.broadcast(BroadcastType::WorldReset, event.to_string());
    }

    /// Broadcast a JSON-serialised event string to all WebSocket subscribers.
    /// Silently ignores send errors (no active subscribers).
    fn broadcast(&self, broadcast_type: BroadcastType, event_json: String) {
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

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

use crate::config::AppConfig;
use crate::db::Database;
use crate::models::{GameMode, State, WorldState};

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
                platforms: Vec::new(),
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
        }
    }

    /// Broadcast a JSON-serialised event string to all WebSocket subscribers.
    /// Silently ignores send errors (no active subscribers).
    pub fn broadcast(&self, event_json: String) {
        let _ = self.ws_tx.send(Arc::new(event_json));
    }
}

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Shared primitives
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Vector2 {
    pub x: f64,
    pub y: f64,
}

impl Vector2 {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    pub fn length(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    /// Returns a unit vector, or None if the vector is zero-length.
    pub fn normalize(&self) -> Option<Self> {
        let len = self.length();
        if len < f64::EPSILON {
            None
        } else {
            Some(Self { x: self.x / len, y: self.y / len })
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

// ---------------------------------------------------------------------------
// Game mode
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameMode {
    Checkpoint,
    Hardcore,
}

impl std::fmt::Display for GameMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameMode::Checkpoint => write!(f, "Checkpoint"),
            GameMode::Hardcore => write!(f, "Hardcore"),
        }
    }
}

impl std::str::FromStr for GameMode {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Checkpoint" => Ok(GameMode::Checkpoint),
            "Hardcore" => Ok(GameMode::Hardcore),
            other => anyhow::bail!("unknown game mode: {other}"),
        }
    }
}

// ---------------------------------------------------------------------------
// SQLite-persisted entities
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    /// Argon2id hash of the password.
    pub password_hash: String,
    pub api_key_hash: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: Uuid,
    pub user_id: Uuid,
    pub game_mode: GameMode,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighScore {
    pub id: Uuid,
    pub user_id: Uuid,
    /// FK → sessions.id; game mode is derived via this join.
    pub session_id: Uuid,
    pub max_height: f64,
    pub achieved_at: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// In-memory runtime state  (§2 of data_model.md)
// ---------------------------------------------------------------------------

/// Per-player runtime state (§2.1).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerState {
    pub session_id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub position: Vector2,
    pub velocity: Vector2,
    pub is_grounded: bool,
    pub jump_count: u32,
    pub session_max_height: f64,
    /// Rolling activity score used by the director cam.
    pub activity_rating: f64,
    pub last_heartbeat: DateTime<Utc>,
    pub last_landed_position: Vector2,
}

/// Top-level game state container (§2.2).
#[derive(Debug, Clone, Serialize)]
pub struct State {
    pub session_id: Uuid,
    pub game_mode: GameMode,
    pub world: WorldState,
    pub players: std::collections::HashMap<Uuid, PlayerState>,
}

/// Shared world state (§2.3).
#[derive(Debug, Clone, Serialize)]
pub struct WorldState {
    pub platforms: Vec<Platform>,
    pub tick: u64,
}

/// A single platform (§2.4).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Platform {
    pub id: Uuid,
    pub rect: Rect,
}

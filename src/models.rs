use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::ops::{Add, AddAssign, Mul, MulAssign};
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
            Some(Self {
                x: self.x / len,
                y: self.y / len,
            })
        }
    }
}

impl Add for Vector2 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl AddAssign for Vector2 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl Mul<f64> for Vector2 {
    type Output = Self;
    fn mul(self, scalar: f64) -> Self {
        Self {
            x: self.x * scalar,
            y: self.y * scalar,
        }
    }
}

impl MulAssign<f64> for Vector2 {
    fn mul_assign(&mut self, scalar: f64) {
        self.x *= scalar;
        self.y *= scalar;
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixedStagePlatform {
    pub bottom_left: [f64; 2],
    pub top_right: [f64; 2],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixedStage {
    #[serde(default)]
    pub id: String,
    pub name: String,
    pub goal_height: f64,
    pub platforms: Vec<FixedStagePlatform>,
}

// ---------------------------------------------------------------------------
// Game mode
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum FallMode {
    Checkpoint,
    Hardcore,
}

impl std::fmt::Display for FallMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FallMode::Checkpoint => write!(f, "Checkpoint"),
            FallMode::Hardcore => write!(f, "Hardcore"),
        }
    }
}

impl std::str::FromStr for FallMode {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Checkpoint" => Ok(FallMode::Checkpoint),
            "Hardcore" => Ok(FallMode::Hardcore),
            other => anyhow::bail!("unknown fall_mode: {other}"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum SessionType {
    Race,
    Timetrial,
    Infinite,
}

impl std::fmt::Display for SessionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SessionType::Race => write!(f, "Race"),
            SessionType::Timetrial => write!(f, "Timetrial"),
            SessionType::Infinite => write!(f, "Infinite"),
        }
    }
}

impl std::str::FromStr for SessionType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Race" => Ok(SessionType::Race),
            "Timetrial" => Ok(SessionType::Timetrial),
            "Infinite" => Ok(SessionType::Infinite),
            other => anyhow::bail!("unknown session_type: {other}"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum StageType {
    Generated,
    Fixed,
}

impl std::fmt::Display for StageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StageType::Generated => write!(f, "Generated"),
            StageType::Fixed => write!(f, "Fixed"),
        }
    }
}

impl std::str::FromStr for StageType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Generated" => Ok(StageType::Generated),
            "Fixed" => Ok(StageType::Fixed),
            other => anyhow::bail!("unknown stage_type: {other}"),
        }
    }
}

/// Full game mode configuration as per the OpenAPI `GameModeConfig` schema.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GameModeConfig {
    pub fall_mode: FallMode,
    pub session_type: SessionType,
    pub stage_type: StageType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fixed_stage_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goal_height: Option<f64>,
}

impl Default for GameModeConfig {
    fn default() -> Self {
        Self {
            fall_mode: FallMode::Checkpoint,
            session_type: SessionType::Infinite,
            stage_type: StageType::Generated,
            fixed_stage_id: None,
            goal_height: None,
        }
    }
}

/// Legacy alias kept for compatibility with config parsing.
pub type GameMode = FallMode;

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
    pub game_mode: FallMode,
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

/// Movement state of a player (§2.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlayerMovementState {
    Landed,
    Jumping,
}

/// Per-player runtime state (§2.1).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerState {
    pub session_id: Uuid,
    pub player_id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub position: Vector2,
    pub velocity: Vector2,
    pub state: PlayerMovementState,
    pub session_max_height: f64,
    /// Rolling activity score used by the director cam.
    pub activity_rating: f64,
    pub last_heartbeat: DateTime<Utc>,
    pub last_landed_position: Vector2,
    pub session_start_time: DateTime<Utc>,
    pub best_time: Option<f64>,
    pub last_goal_time: Option<f64>,
}

/// Top-level game state container (§2.2).
#[derive(Debug, Clone, Serialize)]
pub struct State {
    pub session_id: Uuid,
    pub game_mode: GameModeConfig,
    pub world: WorldState,
    pub players: std::collections::HashMap<Uuid, PlayerState>,
}

/// Shared world state (§2.3).
#[derive(Debug, Clone, Serialize)]
pub struct WorldState {
    pub platforms: Vec<Platform>,
    pub tick: u64,
    /// TODO: use seed for deterministic world generation.
    pub seed: u64,
}

/// A single platform (§2.4).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Platform {
    pub id: Uuid,
    pub rect: Rect,
}

#[cfg(test)]
mod vector2_tests {
    use super::Vector2;

    #[test]
    fn new_sets_components() {
        let v = Vector2::new(3.0, 4.0);
        assert_eq!(v.x, 3.0);
        assert_eq!(v.y, 4.0);
    }

    #[test]
    fn zero_returns_origin() {
        let v = Vector2::zero();
        assert_eq!(v.x, 0.0);
        assert_eq!(v.y, 0.0);
    }

    #[test]
    fn length_is_correct() {
        let v = Vector2::new(3.0, 4.0);
        assert!((v.length() - 5.0).abs() < 1e-10);
    }

    #[test]
    fn length_of_zero_vector_is_zero() {
        assert_eq!(Vector2::zero().length(), 0.0);
    }

    #[test]
    fn normalize_unit_vector_is_unchanged() {
        let v = Vector2::new(1.0, 0.0).normalize().unwrap();
        assert!((v.x - 1.0).abs() < 1e-10);
        assert!((v.y - 0.0).abs() < 1e-10);
    }

    #[test]
    fn normalize_gives_unit_length() {
        let v = Vector2::new(3.0, 4.0).normalize().unwrap();
        assert!((v.length() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn normalize_zero_vector_returns_none() {
        assert!(Vector2::zero().normalize().is_none());
    }

    #[test]
    fn add_sums_components() {
        let a = Vector2::new(1.0, 2.0);
        let b = Vector2::new(3.0, 4.0);
        let c = a + b;
        assert_eq!(c.x, 4.0);
        assert_eq!(c.y, 6.0);
    }

    #[test]
    fn add_assign_mutates_in_place() {
        let mut a = Vector2::new(1.0, 2.0);
        a += Vector2::new(3.0, 4.0);
        assert_eq!(a.x, 4.0);
        assert_eq!(a.y, 6.0);
    }

    #[test]
    fn mul_scalar() {
        let v = Vector2::new(3.0, 4.0);
        let result = v * 2.0;
        assert_eq!(result, Vector2::new(6.0, 8.0));
    }

    #[test]
    fn mul_assign_scales_in_place() {
        let mut v = Vector2::new(2.0, -5.0);
        v *= 3.0;
        assert_eq!(v.x, 6.0);
        assert_eq!(v.y, -15.0);
    }

    #[test]
    fn mul_assign_by_zero_zeroes_vector() {
        let mut v = Vector2::new(9.0, 9.0);
        v *= 0.0;
        assert_eq!(v.x, 0.0);
        assert_eq!(v.y, 0.0);
    }
}

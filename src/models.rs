use std::ops::{Add, AddAssign, Mul, MulAssign};
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

    pub fn scale(&self, scalar: f64) -> Self {
        Self { x: self.x * scalar, y: self.y * scalar }
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

impl Add for Vector2 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self { x: self.x + rhs.x, y: self.y + rhs.y }
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
        Self { x: self.x * scalar, y: self.y * scalar }
    }
}

impl MulAssign<f64> for Vector2 {
    fn mul_assign(&mut self, scalar: f64) {
        self.x *= scalar;
        self.y *= scalar;
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
    fn scale_multiplies_both_components() {
        let v = Vector2::new(2.0, -3.0).scale(4.0);
        assert_eq!(v.x, 8.0);
        assert_eq!(v.y, -12.0);
    }

    #[test]
    fn scale_by_zero_gives_zero_vector() {
        let v = Vector2::new(5.0, 7.0).scale(0.0);
        assert_eq!(v.x, 0.0);
        assert_eq!(v.y, 0.0);
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

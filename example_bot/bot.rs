use anyhow::{Context, Result, bail};
use reqwest::Client;
use serde::Deserialize;

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

#[derive(Deserialize, Debug)]
pub struct JoinResponse {
    pub player_id: String,
    pub session_id: String,
}

#[derive(Deserialize, Debug, Clone, Copy)]
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

    pub fn length(self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn normalize(self) -> Self {
        let len = self.length();
        if len == 0.0 {
            Self::zero()
        } else {
            Self { x: self.x / len, y: self.y / len }
        }
    }

    pub fn dot(self, other: Self) -> f64 {
        self.x * other.x + self.y * other.y
    }

    pub fn distance(self, other: Self) -> f64 {
        (self - other).length()
    }
}

impl std::ops::Add for Vector2 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self { x: self.x + rhs.x, y: self.y + rhs.y }
    }
}

impl std::ops::Sub for Vector2 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self { x: self.x - rhs.x, y: self.y - rhs.y }
    }
}

impl std::ops::Mul<f64> for Vector2 {
    type Output = Self;
    fn mul(self, scalar: f64) -> Self {
        Self { x: self.x * scalar, y: self.y * scalar }
    }
}

impl std::ops::Neg for Vector2 {
    type Output = Self;
    fn neg(self) -> Self {
        Self { x: -self.x, y: -self.y }
    }
}

impl std::fmt::Display for Vector2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({:.2}, {:.2})", self.x, self.y)
    }
}

#[derive(Deserialize, Debug)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Deserialize, Debug)]
pub struct Platform {
    pub id: String,
    pub rect: Rect,
}

#[derive(Deserialize, Debug)]
pub struct StateResponse {
    pub session_id: String,
    pub position: Vector2,
    pub velocity: Vector2,
    pub dimensions: Vector2,
    pub session_max_height: f64,
    pub state: String,
    pub surrounding_platforms: Vec<Platform>,
}

// ---------------------------------------------------------------------------
// Bot
// ---------------------------------------------------------------------------

pub struct Bot {
    client: Client,
    key: String,
    base_url: String,
}

impl Bot {
    pub fn new(api_key: String, base_url: String) -> Self {
        Self {
            client: Client::new(),
            key: api_key,
            base_url,
        }
    }

    pub async fn join(&self) -> Result<JoinResponse> {
        let join_res = self
            .client
            .post(format!("{}/api/v1/bot/join", self.base_url))
            .header("X-API-Key", &self.key)
            .send()
            .await
            .context("join request failed")?;

        if !join_res.status().is_success() {
            let body = join_res.text().await.unwrap_or_default();
            anyhow::bail!("Join failed: {body}");
        }
        let res = join_res.json().await.context("parse join response")?;
        Ok(res)
    }

    pub async fn leave(&self) -> Result<()> {
        let resp = self
            .client
            .post(format!("{}/api/v1/bot/leave", self.base_url))
            .header("X-API-Key", &self.key)
            .send()
            .await
            .context("leave request failed")?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Leave failed: {body}");
        }
        Ok(())
    }

    pub async fn reset(&self) -> Result<()> {
        let resp = self
            .client
            .post(format!("{}/api/v1/bot/reset", self.base_url))
            .header("X-API-Key", &self.key)
            .send()
            .await
            .context("reset request failed")?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Reset failed: {body}");
        }
        Ok(())
    }

    pub async fn state(&self) -> Result<StateResponse> {
        let state = self
            .client
            .get(format!("{}/api/v1/bot/state", self.base_url))
            .header("X-API-Key", &self.key)
            .send()
            .await
            .context("state request failed")?;

        let state = state.json().await.context("parse state response")?;
        Ok(state)
    }

    pub async fn jump(&self, angle_deg: f64, strength: f64) -> Result<()> {
        let jump_res = self
            .client
            .post(format!("{}/api/v1/bot/jump", self.base_url))
            .header("X-API-Key", &self.key)
            .json(&serde_json::json!({
                "direction": { "angle_deg": angle_deg },
                "strength": strength
            }))
            .send()
            .await;

        match jump_res {
            Ok(r) if r.status().is_success() => println!("Jumped!"),
            Ok(r) => println!("Jump returned status {}", r.status()),
            Err(e) => bail!("Jump failed: {}", e.to_string().trim_end_matches('.')),
        }
        Ok(())
    }
}

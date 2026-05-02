use std::sync::Arc;

use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    routing::{get, post},
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::api::auth::{BotApiKey, hash_api_key};
use crate::error::{ApiResult, AppError};
use crate::game::physics;
use crate::models::{PlayerState, Vector2};
use crate::state::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/join", post(join))
        .route("/jump", post(jump))
        .route("/state", get(state))
        .route("/statistics", get(statistics))
        .route("/leave", post(leave))
}

// ---------------------------------------------------------------------------
// POST /api/v1/bot/join
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct JoinRequest {
    name: String,
}

#[derive(Serialize)]
struct JoinResponse {
    player_id: Uuid,
    session_id: Uuid,
    spawn_point: Vector2,
    state: String,
}

async fn join(
    BotApiKey(api_key): BotApiKey,
    State(state): State<Arc<AppState>>,
    Json(body): Json<JoinRequest>,
) -> ApiResult<Json<JoinResponse>> {
    let key_hash = hash_api_key(&api_key);

    // Reject if this key already has an active player in the session.
    {
        let map = state.api_key_map.read().await;
        if map.contains_key(&key_hash) {
            return Err(AppError::Conflict("already_joined".into()));
        }
    }

    let player_id = Uuid::new_v4();
    let game = state.game.read().await;
    let session_id = game.session_id;
    drop(game);

    let player = PlayerState {
        session_id,
        user_id: player_id,
        name: body.name.clone(),
        position: Vector2::new(0.0, 0.0),
        velocity: Vector2::zero(),
        state: crate::models::PlayerMovementState::Jumping,
        consecutive_bounces: 0,
        session_max_height: 0.0,
        activity_rating: 0.0,
        last_heartbeat: Utc::now(),
        last_landed_position: Vector2::new(0.0, 0.0),
    };

    state.game.write().await.players.insert(player_id, player);
    state.api_key_map.write().await.insert(key_hash, player_id);

    let event = serde_json::json!({
        "type": "player_joined",
        "timestamp": Utc::now().timestamp_millis(),
        "player_id": player_id,
        "name": body.name,
    });
    state.broadcast(event.to_string());

    let spawn_point = Vector2::new(0.0, 0.0);
    Ok(Json(JoinResponse { player_id, session_id, spawn_point, state: "JUMPING".to_string() }))
}

// ---------------------------------------------------------------------------
// POST /api/v1/bot/jump
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct JumpRequest {
    direction: DirectionBody,
    strength: f64,
}

/// Accepts either a vector `{"x": .., "y": ..}` or an angle `{"angle_deg": ..}` (0=straight up, CW).
#[derive(Deserialize)]
#[serde(untagged)]
enum DirectionBody {
    Vector { x: f64, y: f64 },
    Angle { angle_deg: f64 },
}

async fn jump(
    BotApiKey(api_key): BotApiKey,
    State(state): State<Arc<AppState>>,
    Json(body): Json<JumpRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let key_hash = hash_api_key(&api_key);

    let direction = match body.direction {
        DirectionBody::Vector { x, y } => Vector2::new(x, y),
        DirectionBody::Angle { angle_deg } => {
            let rad = angle_deg.to_radians();
            // 0° = straight up: x = sin(rad), y = cos(rad)
            Vector2::new(rad.sin(), rad.cos())
        }
    };

    // Validate before acquiring the write lock.
    if direction.normalize().is_none() {
        return Err(AppError::BadRequest("direction must not be a zero vector".into()));
    }
    if body.strength < physics::JUMP_STRENGTH_MIN || body.strength > physics::JUMP_STRENGTH_MAX {
        return Err(AppError::BadRequest("strength out of range".into()));
    }

    let player_id = {
        let map = state.api_key_map.read().await;
        *map.get(&key_hash).ok_or(AppError::Conflict("not_joined".into()))?
    };

    let mut game = state.game.write().await;
    let player = game.players.get_mut(&player_id)
        .ok_or_else(|| AppError::Internal(anyhow::anyhow!("player not found in game state")))?;

    physics::apply_jump(player, direction, body.strength)?;

    Ok(Json(serde_json::json!({ "status": "ok" })))
}

// ---------------------------------------------------------------------------
// GET /api/v1/bot/state
// ---------------------------------------------------------------------------

async fn state(
    BotApiKey(api_key): BotApiKey,
    State(app): State<Arc<AppState>>,
) -> ApiResult<Json<serde_json::Value>> {
    let key_hash = hash_api_key(&api_key);

    let player_id = {
        let map = app.api_key_map.read().await;
        *map.get(&key_hash).ok_or(AppError::BadRequest("not joined; call /bot/join first".into()))?
    };

    let game = app.game.read().await;
    let player = game.players.get(&player_id)
        .ok_or_else(|| AppError::Internal(anyhow::anyhow!("player not found in game state")))?;

    let surrounding_platforms: Vec<_> = game.world.platforms.iter()
        .map(|p| serde_json::json!({ "id": p.id, "rect": { "x": p.rect.x, "y": p.rect.y, "width": p.rect.width, "height": p.rect.height } }))
        .collect();

    Ok(Json(serde_json::json!({
        "session_id": player.session_id,
        "position": { "x": player.position.x, "y": player.position.y },
        "velocity": { "x": player.velocity.x, "y": player.velocity.y },
        "dimensions": { "x": crate::game::physics::PLAYER_WIDTH, "y": crate::game::physics::PLAYER_HEIGHT },
        "session_max_height": player.session_max_height,
        "state": format!("{:?}", player.state),
        "consecutive_bounces": player.consecutive_bounces,
        "surrounding_platforms": surrounding_platforms,
    })))
}

// ---------------------------------------------------------------------------
// GET /api/v1/bot/statistics
// ---------------------------------------------------------------------------

async fn statistics(
    BotApiKey(_api_key): BotApiKey,
    State(_app): State<Arc<AppState>>,
) -> ApiResult<Json<serde_json::Value>> {
    // TODO: query DB for PlayerStatistics
    Ok(Json(serde_json::json!({ "status": "not implemented" })))
}

// ---------------------------------------------------------------------------
// POST /api/v1/bot/leave
// ---------------------------------------------------------------------------

async fn leave(
    BotApiKey(api_key): BotApiKey,
    State(state): State<Arc<AppState>>,
) -> ApiResult<StatusCode> {
    let key_hash = hash_api_key(&api_key);

    let player_id = {
        let mut map = state.api_key_map.write().await;
        map.remove(&key_hash).ok_or(AppError::Conflict("not_joined".into()))?
    };

    state.game.write().await.players.remove(&player_id);

    let event = serde_json::json!({
        "type": "player_left",
        "timestamp": Utc::now().timestamp_millis(),
        "player_id": player_id,
    });
    state.broadcast(event.to_string());

    Ok(StatusCode::NO_CONTENT)
}

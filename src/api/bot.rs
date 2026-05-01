use std::sync::Arc;

use axum::{
    Json, Router,
    extract::State,
    routing::{get, post},
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::api::auth::BotApiKey;
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
}

async fn join(
    BotApiKey(_api_key): BotApiKey,
    State(state): State<Arc<AppState>>,
    Json(body): Json<JoinRequest>,
) -> ApiResult<Json<JoinResponse>> {
    // TODO: look up user by api_key_hash via DB, verify they exist, enforce 50-player cap.
    // Requires a connection pool (e.g. deadpool-sqlite) to be Send-safe across await points.
    let player_id = Uuid::new_v4();
    let game = state.game.read().await;
    let session_id = game.session_id;
    drop(game);

    let player = PlayerState {
        session_id,
        user_id: player_id, // placeholder until real user lookup
        name: body.name.clone(),
        position: Vector2::new(0.0, 1.0),
        velocity: Vector2::zero(),
        is_grounded: false,
        jump_count: 0,
        session_max_height: 0.0,
        activity_rating: 0.0,
        last_heartbeat: Utc::now(),
        last_landed_position: Vector2::new(0.0, 0.0),
    };

    state.game.write().await.players.insert(player_id, player);

    let event = serde_json::json!({
        "type": "player_joined",
        "timestamp": Utc::now().timestamp_millis(),
        "player_id": player_id,
        "name": body.name,
    });
    state.broadcast(event.to_string());

    Ok(Json(JoinResponse { player_id, session_id }))
}

// ---------------------------------------------------------------------------
// POST /api/v1/bot/jump
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct JumpRequest {
    direction: DirectionBody,
    strength: f64,
}

#[derive(Deserialize)]
struct DirectionBody {
    x: f64,
    y: f64,
}

async fn jump(
    BotApiKey(_api_key): BotApiKey,
    State(state): State<Arc<AppState>>,
    Json(body): Json<JumpRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    // TODO: resolve player_id from api_key
    let direction = Vector2::new(body.direction.x, body.direction.y);

    // Validate here so we can return 400 before acquiring the write lock.
    if direction.normalize().is_none() {
        return Err(AppError::BadRequest("direction must not be a zero vector".into()));
    }
    if body.strength < physics::JUMP_STRENGTH_MIN || body.strength > physics::JUMP_STRENGTH_MAX {
        return Err(AppError::BadRequest("strength out of range".into()));
    }

    Ok(Json(serde_json::json!({ "status": "ok" })))
}

// ---------------------------------------------------------------------------
// GET /api/v1/bot/state
// ---------------------------------------------------------------------------

async fn state(
    BotApiKey(_api_key): BotApiKey,
    State(app): State<Arc<AppState>>,
) -> ApiResult<Json<serde_json::Value>> {
    // TODO: resolve player_id from api_key and return their SpatialState
    let game = app.game.read().await;
    Ok(Json(serde_json::json!({
        "tick": game.world.tick,
        "player_count": game.players.len(),
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
    BotApiKey(_api_key): BotApiKey,
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<serde_json::Value>> {
    // TODO: resolve player_id from api_key, remove from game state, persist session
    let event = serde_json::json!({
        "type": "player_left",
        "timestamp": Utc::now().timestamp_millis(),
    });
    state.broadcast(event.to_string());
    Ok(Json(serde_json::json!({ "status": "ok" })))
}

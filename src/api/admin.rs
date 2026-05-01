use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, State},
    routing::post,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::api::auth::AdminUser;
use crate::error::{ApiResult, AppError};
use crate::models::GameMode;
use crate::state::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/mode", post(set_mode))
        .route("/ratelimits", post(set_ratelimits))
        .route("/world/reset", post(world_reset))
        .route("/players/reset", post(players_reset))
        .route("/players/{id}/reset", post(player_reset))
}

// ---------------------------------------------------------------------------
// POST /api/v1/admin/mode
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct SetModeRequest {
    game_mode: String,
}

async fn set_mode(
    AdminUser(_claims): AdminUser,
    State(state): State<Arc<AppState>>,
    Json(body): Json<SetModeRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let mode: GameMode = body
        .game_mode
        .parse()
        .map_err(|_| AppError::BadRequest("invalid game_mode".into()))?;

    state.game.write().await.game_mode = mode;

    let event = serde_json::json!({
        "type": "world_reset",
        "timestamp": chrono::Utc::now().timestamp_millis(),
        "game_mode": mode.to_string(),
    });
    state.broadcast(event.to_string());

    Ok(Json(serde_json::json!({ "game_mode": mode.to_string() })))
}

// ---------------------------------------------------------------------------
// POST /api/v1/admin/ratelimits
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct SetRateLimitsRequest {
    tier: String,
    requests_per_second: u32,
    burst_size: u32,
}

async fn set_ratelimits(
    AdminUser(_claims): AdminUser,
    State(_state): State<Arc<AppState>>,
    Json(_body): Json<SetRateLimitsRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    // TODO: update governor rate limiters at runtime
    Ok(Json(serde_json::json!({ "status": "ok" })))
}

// ---------------------------------------------------------------------------
// POST /api/v1/admin/world/reset
// ---------------------------------------------------------------------------

async fn world_reset(
    AdminUser(_claims): AdminUser,
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<serde_json::Value>> {
    use crate::game::world::initial_platforms;

    let mut game = state.game.write().await;
    game.world.platforms = initial_platforms();
    game.world.tick = 0;
    game.world.seed = 0; // TODO: generate new seed on world reset
    game.players.clear();

    let event = serde_json::json!({
        "type": "world_reset",
        "timestamp": chrono::Utc::now().timestamp_millis(),
    });
    state.broadcast(event.to_string());

    Ok(Json(serde_json::json!({ "status": "ok" })))
}

// ---------------------------------------------------------------------------
// POST /api/v1/admin/players/reset
// ---------------------------------------------------------------------------

async fn players_reset(
    AdminUser(_claims): AdminUser,
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<serde_json::Value>> {
    state.game.write().await.players.clear();
    Ok(Json(serde_json::json!({ "status": "ok" })))
}

// ---------------------------------------------------------------------------
// POST /api/v1/admin/players/{id}/reset
// ---------------------------------------------------------------------------

async fn player_reset(
    AdminUser(_claims): AdminUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<serde_json::Value>> {
    let mut game = state.game.write().await;
    let player = game.players.get_mut(&id).ok_or(AppError::NotFound)?;
    player.position = player.last_landed_position;
    player.velocity = crate::models::Vector2::zero();
    Ok(Json(serde_json::json!({ "status": "ok" })))
}

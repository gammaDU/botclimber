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
use crate::models::{FallMode, GameModeConfig, SessionType, StageType};
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
    fall_mode: Option<String>,
    session_type: Option<String>,
    stage_type: Option<String>,
    fixed_stage_id: Option<String>,
    goal_height: Option<f64>,
}

async fn set_mode(
    AdminUser(_claims): AdminUser,
    State(state): State<Arc<AppState>>,
    Json(body): Json<SetModeRequest>,
) -> ApiResult<Json<GameModeConfig>> {
    let mut game = state.game.write().await;

    if let Some(ref s) = body.fall_mode {
        game.game_mode.fall_mode = s
            .parse::<FallMode>()
            .map_err(|_| AppError::BadRequest("invalid fall_mode".into()))?;
    }
    if let Some(ref s) = body.session_type {
        game.game_mode.session_type = s
            .parse::<SessionType>()
            .map_err(|_| AppError::BadRequest("invalid session_type".into()))?;
    }
    if let Some(ref s) = body.stage_type {
        game.game_mode.stage_type = s
            .parse::<StageType>()
            .map_err(|_| AppError::BadRequest("invalid stage_type".into()))?;
    }
    game.game_mode.fixed_stage_id = body.fixed_stage_id.clone();
    game.game_mode.goal_height = body.goal_height;

    let current_mode = game.game_mode.clone();
    drop(game);

    state.broadcast_world_reset(&current_mode);

    Ok(Json(current_mode))
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
    game.session_id = Uuid::new_v4();
    game.world.platforms = initial_platforms();
    game.world.tick = 0;
    game.world.seed = 0; // TODO: generate new seed on world reset
    game.players.clear();

    let new_session_id = game.session_id;
    let game_mode = game.game_mode.clone();
    drop(game);

    state.persist_session(new_session_id, &game_mode);
    state.broadcast_world_reset(&game_mode);

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

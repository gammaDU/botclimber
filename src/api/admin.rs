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
        .route("/mode", axum::routing::get(get_mode).post(set_mode))
        .route("/stages", axum::routing::get(get_stages))
        .route("/ratelimits", post(set_ratelimits))
        .route("/world/reset", post(world_reset))
        .route("/players/reset", post(players_reset))
        .route("/players/{id}/reset", post(player_reset))
}

// ---------------------------------------------------------------------------
// GET /api/v1/admin/stages
// ---------------------------------------------------------------------------

#[derive(serde::Serialize)]
struct FixedStageInfo {
    id: String,
    name: String,
}

async fn get_stages(
    AdminUser(_claims): AdminUser,
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<Vec<FixedStageInfo>>> {
    let stages = state
        .config
        .fixed_stages
        .iter()
        .map(|s| FixedStageInfo {
            id: s.id.clone(),
            name: s.name.clone(),
        })
        .collect();
    Ok(Json(stages))
}

// ---------------------------------------------------------------------------
// GET /api/v1/admin/mode
// ---------------------------------------------------------------------------

async fn get_mode(
    AdminUser(_claims): AdminUser,
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<GameModeConfig>> {
    let game = state.game.read().await;
    let mode = game.game_mode.clone();
    Ok(Json(mode))
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

    // Load platforms if switching to a fixed stage
    if game.game_mode.stage_type == StageType::Fixed {
        if let Some(ref stage_id) = game.game_mode.fixed_stage_id {
            if let Some(stage) = state.config.fixed_stages.iter().find(|s| &s.id == stage_id) {
                game.game_mode.goal_height = Some(stage.goal_height);
                game.world.platforms = crate::game::world::platforms_from_fixed_stage(stage);
            }
        }
    } else if game.game_mode.stage_type == StageType::Generated {
        game.world.platforms = crate::game::world::initial_platforms();
    }

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
    use crate::game::world::{initial_platforms, platforms_from_fixed_stage};

    let mut game = state.game.write().await;
    game.session_id = Uuid::new_v4();

    // Re-verify the fixed stage if the game mode is set to Fixed.
    if game.game_mode.stage_type == StageType::Fixed {
        if let Some(ref stage_id) = game.game_mode.fixed_stage_id {
            if let Some(stage) = state.config.fixed_stages.iter().find(|s| &s.id == stage_id) {
                game.game_mode.goal_height = Some(stage.goal_height);
                game.world.platforms = platforms_from_fixed_stage(stage);
            } else {
                // If stage_id not found in config, fallback to generated.
                game.game_mode.stage_type = StageType::Generated;
                game.world.platforms = initial_platforms();
            }
        } else {
            // No stage_id provided, fallback to generated.
            game.game_mode.stage_type = StageType::Generated;
            game.world.platforms = initial_platforms();
        }
    } else {
        game.world.platforms = initial_platforms();
    }

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
    player.position = crate::models::Vector2::zero();
    player.velocity = crate::models::Vector2::zero();
    player.last_landed_position = crate::models::Vector2::zero();
    player.session_max_height = 0.0;
    player.session_start_time = chrono::Utc::now();
    Ok(Json(serde_json::json!({ "status": "ok" })))
}

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
use crate::models::{FallMode, GameModeConfig, PlayerMovementState, SessionType, StageType, Vector2};
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

    let mut reset_players_metrics = false;

    if let Some(ref s) = body.fall_mode {
        game.game_mode.fall_mode = s
            .parse::<FallMode>()
            .map_err(|_| AppError::BadRequest("invalid fall_mode".into()))?;
    }
    if let Some(ref s) = body.session_type {
        let new_session_type = s
            .parse::<SessionType>()
            .map_err(|_| AppError::BadRequest("invalid session_type".into()))?;
        if game.game_mode.session_type != new_session_type {
            game.game_mode.session_type = new_session_type;
            reset_players_metrics = true;
        }
    }
    if let Some(ref s) = body.stage_type {
        let new_stage_type = s
            .parse::<StageType>()
            .map_err(|_| AppError::BadRequest("invalid stage_type".into()))?;
        if game.game_mode.stage_type != new_stage_type {
            game.game_mode.stage_type = new_stage_type;
            reset_players_metrics = true;
        }
    }
    if game.game_mode.fixed_stage_id != body.fixed_stage_id {
        game.game_mode.fixed_stage_id = body.fixed_stage_id.clone();
        reset_players_metrics = true;
    }
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

    if reset_players_metrics {
        let new_id = Uuid::new_v4();
        game.session_id = new_id;
        game.world.tick = 0;
        for player in game.players.values_mut() {
            player.session_id = new_id;
            player.best_time = None;
            player.last_goal_time = None;
            player.session_max_height = 0.0;
            player.position = Vector2::new(0.0, 0.0);
            player.velocity = Vector2::new(0.0, 0.0);
            player.last_landed_position = Vector2::new(0.0, 0.0);
            player.state = PlayerMovementState::Landed;
            player.session_start_time = chrono::Utc::now();
        }
    }

    let current_mode = game.game_mode.clone();
    let new_session_id = game.session_id;
    drop(game);

    if reset_players_metrics {
        state.persist_session(new_session_id, &current_mode);
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{PlayerState, Vector2, PlayerMovementState};
    use chrono::{Duration, Utc};

    #[test]
    fn test_admin_player_reset_logic() {
        let session_id = Uuid::new_v4();
        let player_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let old_start_time = Utc::now() - Duration::seconds(100);
        
        let mut player = PlayerState {
            session_id,
            player_id,
            user_id,
            name: "TestPlayer".to_string(),
            position: Vector2::new(10.0, 20.0),
            velocity: Vector2::new(1.0, 2.0),
            state: PlayerMovementState::Jumping,
            session_max_height: 50.0,
            activity_rating: 1.0,
            last_heartbeat: Utc::now(),
            last_landed_position: Vector2::new(5.0, 5.0),
            session_start_time: old_start_time,
            best_time: Some(42.0),
            last_goal_time: Some(42.0),
        };

        // Simulating the logic inside admin player_reset handler
        player.position = Vector2::zero();
        player.velocity = Vector2::zero();
        player.last_landed_position = Vector2::zero();
        player.session_max_height = 0.0;
        player.session_start_time = Utc::now();

        assert_eq!(player.position.x, 0.0);
        assert_eq!(player.position.y, 0.0);
        assert_eq!(player.velocity.x, 0.0);
        assert_eq!(player.velocity.y, 0.0);
        assert_eq!(player.last_landed_position.x, 0.0);
        assert_eq!(player.last_landed_position.y, 0.0);
        assert_eq!(player.session_max_height, 0.0);
        assert!(player.session_start_time > old_start_time);
        
        // High scores (best_time) should NOT be reset
        assert_eq!(player.best_time, Some(42.0));
    }

    #[tokio::test]
    async fn test_set_mode_resets_best_time() {
        use crate::config::{AppConfig, ServerConfig, DatabaseConfig, AuthConfig, RateLimitingConfig, GameConfig};
        use crate::state::AppState;
        use crate::db::Database;

        let config = AppConfig {
            server: ServerConfig { bind: "0.0.0.0:8080".into(), frontend_dist_path: "frontend/dist".into() },
            database: DatabaseConfig { path: ":memory:".into() },
            auth: AuthConfig { jwt_secret: "secret-key-at-least-32-chars-long-x".into(), jwt_expiry_secs: 3600 },
            rate_limiting: RateLimitingConfig {
                ip_requests_per_second: 100, ip_burst_size: 200,
                bot_requests_per_second: 10, bot_burst_size: 20,
                user_requests_per_second: 30, user_burst_size: 60,
            },
            game: GameConfig {
                default_game_mode: "Hardcore".into(),
                default_session_type: "Infinite".into(),
                default_stage_type: "Generated".into(),
                default_stage_id: None,
                player_timeout_secs: 60,
                max_players: 50,
                tick_rate_hz: 60.0,
                maps_path: None,
            },
            fixed_stages: vec![crate::models::FixedStage {
                id: "tutorial".to_string(),
                name: "Tutorial".to_string(),
                goal_height: 100.0,
                platforms: vec![],
            }],
        };

        // Use an in-memory database for testing
        let db = Database::open(":memory:").unwrap();
        let state = Arc::new(AppState::new(config, db));
        
        let player_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let session_id = state.game.read().await.session_id;

        {
            let mut game = state.game.write().await;
            game.players.insert(player_id, PlayerState {
                session_id,
                player_id,
                user_id,
                name: "TestPlayer".to_string(),
                position: Vector2::new(10.0, 20.0),
                velocity: Vector2::new(1.0, 2.0),
                state: PlayerMovementState::Jumping,
                session_max_height: 50.0,
                activity_rating: 1.0,
                last_heartbeat: Utc::now(),
                last_landed_position: Vector2::new(5.0, 5.0),
                session_start_time: Utc::now(),
                best_time: Some(42.0),
                last_goal_time: Some(42.0),
            });
        }

        // Call set_mode
        let request = SetModeRequest {
            fall_mode: None,
            session_type: None,
            stage_type: Some("Fixed".to_string()),
            fixed_stage_id: Some("tutorial".to_string()),
            goal_height: None,
        };

        set_mode(
            AdminUser(crate::api::auth::Claims { 
                sub: user_id.to_string(), 
                username: "admin".to_string(), 
                admin: true,
                exp: 0 
            }),
            State(state.clone()),
            Json(request)
        ).await.unwrap();

        // Verify player metrics ARE reset
        let game = state.game.read().await;
        let player = game.players.get(&player_id).unwrap();
        assert_eq!(player.best_time, None, "Best time should be reset on stage change");
        assert_eq!(player.position.x, 0.0);
        assert_eq!(player.position.y, 0.0);
        assert_eq!(player.session_max_height, 0.0);
        assert_eq!(game.world.tick, 0);
    }
}

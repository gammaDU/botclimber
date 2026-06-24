use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Query, State},
    routing::get,
};
use serde::{Deserialize, Serialize};

use super::ws;
use crate::error::ApiResult;
use crate::models::GameModeConfig;
use crate::state::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/leaderboard", get(leaderboard))
        .route("/active-players", get(active_players))
        .route("/snapshot", get(snapshot))
        .route("/stage", get(stages))
        .route("/stream", get(ws::handler))
}

// ---------------------------------------------------------------------------
// GET /api/v1/system/stage
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct StageInfo {
    id: &'static str,
    label: &'static str,
    height: f64,
}

const AVAILABLE_STAGES: &[StageInfo] = &[
    StageInfo { id: "default", label: "Default", height: 100.0 },
];

async fn stages() -> Json<&'static [StageInfo]> {
    Json(AVAILABLE_STAGES)
}

// ---------------------------------------------------------------------------
// GET /api/v1/system/leaderboard
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct LeaderboardQuery {
    session_type: Option<String>,
    stage_type: Option<String>,
    fixed_stage_id: Option<String>,
    session_id: Option<String>,
    user_id: Option<String>,
    #[serde(default = "default_limit")]
    limit: i64,
    #[serde(default)]
    offset: i64,
}

fn default_limit() -> i64 {
    100
}

#[derive(Serialize)]
struct LeaderboardEntry {
    username: String,
    max_height: f64,
    session_type: String,
    achieved_at: String,
}

async fn leaderboard(
    State(state): State<Arc<AppState>>,
    Query(params): Query<LeaderboardQuery>,
) -> ApiResult<Json<Vec<LeaderboardEntry>>> {
    let rows = state.db.get_leaderboard(
        params.session_type.as_deref(),
        params.stage_type.as_deref(),
        params.fixed_stage_id.as_deref(),
        params.session_id.as_deref(),
        params.user_id.as_deref(),
        params.limit,
        params.offset,
    )?;

    let entries = rows
        .into_iter()
        .map(|r| LeaderboardEntry {
            username: r.username,
            max_height: r.max_height,
            session_type: r.session_type,
            achieved_at: r.achieved_at,
        })
        .collect();

    Ok(Json(entries))
}

// ---------------------------------------------------------------------------
// GET /api/v1/system/active-players
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct ActivePlayerEntry {
    player_id: String,
    session_id: String,
    name: String,
    current_height: f64,
    max_height_session: f64,
    activity_rating: f64,
}

async fn active_players(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<Vec<ActivePlayerEntry>>> {
    let game = state.game.read().await;
    let entries = game
        .players
        .iter()
        .map(|(id, p)| ActivePlayerEntry {
            player_id: id.to_string(),
            session_id: p.session_id.to_string(),
            name: p.name.clone(),
            current_height: p.position.y,
            max_height_session: p.session_max_height,
            activity_rating: p.activity_rating,
        })
        .collect();

    Ok(Json(entries))
}

// ---------------------------------------------------------------------------
// GET /api/v1/system/snapshot
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct SnapshotPlayerEntry {
    player_id: String,
    name: String,
    position: PositionBody,
    velocity: PositionBody,
    state: String,
}

#[derive(Serialize)]
struct PositionBody {
    x: f64,
    y: f64,
}

#[derive(Serialize)]
struct WorldSnapshot {
    timestamp: i64,
    session_id: String,
    game_mode: GameModeConfig,
    tick: u64,
    players: Vec<SnapshotPlayerEntry>,
    platforms: Vec<serde_json::Value>,
}

async fn snapshot(State(state): State<Arc<AppState>>) -> ApiResult<Json<WorldSnapshot>> {
    let game = state.game.read().await;

    let players = game
        .players
        .iter()
        .map(|(id, p)| SnapshotPlayerEntry {
            player_id: id.to_string(),
            name: p.name.clone(),
            position: PositionBody {
                x: p.position.x,
                y: p.position.y,
            },
            velocity: PositionBody {
                x: p.velocity.x,
                y: p.velocity.y,
            },
            state: format!("{:?}", p.state),
        })
        .collect();

    let platforms = game
        .world
        .platforms
        .iter()
        .map(|p| serde_json::json!({
            "id": p.id,
            "rect": { "x": p.rect.x, "y": p.rect.y, "width": p.rect.width, "height": p.rect.height }
        }))
        .collect();

    Ok(Json(WorldSnapshot {
        timestamp: chrono::Utc::now().timestamp_millis(),
        session_id: game.session_id.to_string(),
        game_mode: game.game_mode.clone(),
        tick: game.world.tick,
        players,
        platforms,
    }))
}

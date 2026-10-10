# Botclimber – Project Summary

_Last updated: 2026-10-10_

## What Is This Project?

Botclimber is a competitive multiplayer platformer game where players write bots to navigate an industrial tower. Bots interact with the server via a REST API (join, jump, get state, leave); humans watch via a browser-based spectator frontend that receives live updates over WebSocket. The project features real-time visualization with a gritty, industrial aesthetic.

## Tech Stack

- **Backend:** Rust (Axum, Tokio async runtime), SQLite (database), JWT + Argon2id auth, rate limiting via `governor`, config via TOML.
- **Frontend:** Angular SPA, served as static files from `frontend/dist/browser/`.
- **Tooling:** `mise` for task running, `tracing` for structured logging to `logs/events.log`.

## ⚠️ Known Gaps / TODOs

> These are the items that are **not yet implemented** or **incomplete**. List every stub, missing feature, and open TODO found in the code.

| Area | Item | Status | Details |
|------|------|--------|---------|
| Bot API | `GET /bot/statistics` | ⚠️ Stub | Handler returns `{"status": "not implemented"}`; DB query for `PlayerStatistics` not written. |
| Admin API | `POST /admin/ratelimits` | ⚠️ Stub | Accepts and validates the request body but does not update the in-memory `governor` rate limiters at runtime. |
| World | Seed-based generation | ❌ Missing | `WorldState.seed` is always `0`; deterministic world generation from seed not implemented. |
| Testing | Integration Tests | ⚠️ Partial | Unit tests exist for physics and models, but broad integration tests for all namespaces are still planned. |

## Frontend

Angular SPA served from `frontend/dist/browser/`. 2 main pages, 4 services.

### Pages / Components

| Component | Route | Description |
|-----------|-------|-------------|
| `SpectateComponent` | `/spectate` | Live canvas renderer: opens WebSocket (`/api/v1/system/stream`), fetches initial snapshot via `GET /api/v1/system/snapshot`, then applies `position_update` / `player_joined` / `player_left` / `world_reset` events; supports click-to-follow a player. |
| `AccountComponent` | `/account` | User registration, login, profile management (display name, password), and API key generation. Includes Admin panel for authorized users. |

### Services

| Service | Purpose |
|---------|---------|
| `AuthService` | Handles user authentication, registration, profile updates, and JWT/API key management. |
| `AdminService` | Provides methods for administrators to change game modes, reset the world, or reset players. |
| `ApiService` | Base service for making HTTP requests to the backend API. |
| `GameStreamService` | Manages the WebSocket connection for real-time game event streaming. |

### ⚠️ Frontend–Backend Mismatches

No significant mismatches found between the current frontend implementation and backend API routes. The mismatches reported in previous versions have been resolved.

## Architecture Overview

- `src/main.rs` — entry point: loads config, opens DB, spawns game loop, starts Axum server.
- `src/state.rs` — `AppState`: holds `RwLock<State>` (live game), `broadcast::Sender` (WebSocket events), `api_key_map` (hash → player_id).
- `src/models.rs` — shared types: `Vector2`, `Rect`, `Platform`, `PlayerState`, `WorldState`, `State`, `GameModeConfig`, `HighScore`, `User`, `Session`.
- `src/game/physics.rs` — 60 Hz tick, gravity, AABB collision, bounce, death detection, `apply_jump`.
- `src/game/tick.rs` — background Tokio task: ticks all players, resets dead ones to last landed position, broadcasts `position_update`.
- `src/game/world.rs` — procedural platform generation and fixed stage loading.
- `src/api/` — Axum routers: `bot`, `user`, `admin`, `system`, `ws`.
- `src/db.rs` — SQLite database wrapper.

## API Endpoints

### Bot (`/api/v1/bot`) — authenticated via `X-Api-Key` header

| Method | Path | Status | Notes |
|--------|------|--------|-------|
| POST | `/join` | ✅ Implemented | Allows a bot to enter the current session. |
| POST | `/jump` | ✅ Implemented | Accepts direction (vector or angle) and strength. |
| GET | `/state` | ✅ Implemented | Returns current bot position, velocity, and surrounding platforms. |
| GET | `/statistics` | ⚠️ Stub | Placeholder for future player performance metrics. |
| POST | `/reset` | ✅ Implemented | Resets bot to starting position and clears session progress. |
| POST | `/leave` | ✅ Implemented | Removes bot from session and persists final session high score. |

### User (`/api/v1/user`) — authenticated via JWT

| Method | Path | Status | Notes |
|--------|------|--------|-------|
| POST | `/register` | ✅ Implemented | Creates a new user account and initial API key. |
| POST | `/login` | ✅ Implemented | Authenticates user and returns a JWT. |
| GET | `/profile` | ✅ Implemented | Retrieves current user profile details. |
| PATCH | `/profile` | ✅ Implemented | Updates display name or password. |
| POST | `/apikey` | ✅ Implemented | Regenerates the bot API key. |

### Admin (`/api/v1/admin`) — JWT with admin role

| Method | Path | Status | Notes |
|--------|------|--------|-------|
| GET | `/mode` | ✅ Implemented | Retrieves the current global game mode configuration. |
| POST | `/mode` | ✅ Implemented | Updates fall mode, session type, and stage configuration. |
| GET | `/stages` | ✅ Implemented | Lists available fixed stages from the configuration. |
| POST | `/ratelimits` | ⚠️ Stub | Placeholder for runtime rate limit adjustments. |
| POST | `/world/reset` | ✅ Implemented | Restarts the session, clears players, and regenerates world. |
| POST | `/players/reset` | ✅ Implemented | Resets all active players to the starting position. |
| POST | `/players/{id}/reset` | ✅ Implemented | Resets a specific player to the starting position. |

### System (`/api/v1/system`)

| Method | Path | Status | Notes |
|--------|------|--------|-------|
| GET | `/leaderboard` | ✅ Implemented | Returns high scores filtered by session/stage type. |
| GET | `/active-players` | ✅ Implemented | Lists currently connected bots and their heights. |
| GET | `/snapshot` | ✅ Implemented | Provides a full world state for initial spectator bootstrap. |
| GET | `/stage` | ✅ Implemented | Public endpoint for stage metadata. |
| GET | `/stream` | ✅ Implemented | WebSocket endpoint for real-time event streaming. |

## WebSocket Events

- `player_joined` — `{ player_id, name }`
- `player_left` — `{ player_id }`
- `position_update` — `{ player_id, x, y, vx, vy, state, current_time, best_time }`
- `world_reset` — `{ game_mode }`
- `platform_spawn` — `{ platform_id, rect }`

## Physics Constants

| Constant | Value | Description |
|----------|-------|-------------|
| `GRAVITY` | -10.0 | Downward acceleration per second. |
| `TICK_RATE_HZ` | 60.0 | Physics simulation frequency. |
| `AIR_RESISTANCE` | 1.0 | Velocity multiplier per tick. |
| `FRICTION` | 0.5 | Velocity bounce/ground multiplier. |
| `PLAYER_WIDTH` | 0.8 | Width of the bot's AABB. |
| `PLAYER_HEIGHT` | 0.8 | Height of the bot's AABB. |
| `JUMP_STRENGTH_MAX` | 20.0 | Maximum allowed jump impulse. |
| `DEATH_DROP` | 15.0 | Height drop below peak that triggers a reset. |

## Testing

- Unit tests for physics engine in `src/game/physics.rs`.
- `Vector2` unit tests in `src/models.rs`.
- Logic tests for API handlers in `src/api/bot.rs` and `src/api/admin.rs`.
- Run with: `mise run test`

## How to Run

```powershell
# Run migrations
mise run migrate

# Build frontend
mise run frontend-build

# Start server
mise run server
```

## Key Docs

See `docs/requirements.md` for an index of all available documentation.

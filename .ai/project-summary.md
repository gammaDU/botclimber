# Botclimber – Project Summary

_Last updated: 2026-05-15_

## What Is This Project?

Botclimber is a multiplayer bot-programming game where players write bots that compete to climb as high as possible on a procedurally generated platform tower. Bots interact with the server via a REST API (join, jump, get state, leave); humans watch via a browser-based spectator frontend that receives live updates over WebSocket.

## Tech Stack

- **Backend:** Rust (Axum 0.8, Tokio async runtime), SQLite (`rusqlite` bundled), JWT + Argon2id auth, rate limiting via `governor`, config via TOML (`config` crate)
- **Frontend:** Angular 21, served as static files from `frontend/dist/browser/`
- **Tooling:** `mise` for task running, `tracing` + `tracing-appender` for structured logging to `logs/events.log`

## ⚠️ Known Gaps / TODOs

> These are the items that are **not yet implemented** or **incomplete**. List every stub, missing feature, and open TODO found in the code.

| Area | Item | Status | Details |
|------|------|--------|---------|
| Bot API | `GET /bot/statistics` | ⚠️ Stub | Handler returns `{"status": "not implemented"}`; DB query for `PlayerStatistics` not written |
| Admin API | `POST /admin/ratelimits` | ⚠️ Partial | Accepts and validates the request body but does not update the in-memory `governor` rate limiters at runtime (`// TODO` in `admin.rs`) |
| World | Seed-based generation | ❌ Missing | `WorldState.seed` is always `0`; deterministic world generation from seed not implemented (`// TODO` in `state.rs` and `admin.rs`) |
| Rate limiting | Per-user limits | ❌ Missing | `governor` dependency is present and global limits exist, but per-user rate limiting is not wired up |
| Tests | Integration / API tests | ❌ Missing | No HTTP-level or integration tests exist; only physics and `Vector2` unit tests are present |
| Frontend | `GET /system/leaderboard`, `/snapshot`, `/players` | ❌ Missing | Three endpoints called by the frontend do not exist in the backend |
| Frontend | `PUT /user/profile` vs `PATCH` | ⚠️ Mismatch | Frontend uses `PUT`, backend only accepts `PATCH` |
| Frontend | `POST /user/api-key` vs `/user/apikey` | ⚠️ Mismatch | Frontend path differs from backend route |

## Frontend

Angular 21 SPA served from `frontend/dist/browser/`. Three pages, three services.

### Pages / Components

| Component | Route | Description |
|-----------|-------|-------------|
| `LeaderboardComponent` | `/leaderboard` (default) | Fetches and displays high-score table via `GET /api/v1/system/leaderboard`; supports filtering by game mode |
| `SpectateComponent` | `/spectate` | Live canvas renderer: opens WebSocket (`/api/v1/system/stream`), fetches initial snapshot via `GET /api/v1/system/snapshot`, then applies `position_update` / `player_joined` / `player_left` / `world_reset` events; supports click-to-follow a player |
| `AccountComponent` | `/account` | Register, login, logout, update username/password, generate API key |

### Services

| Service | Purpose |
|---------|---------|
| `AuthService` | JWT storage (`localStorage`), register/login/logout, `GET /user/profile`, `PUT /user/profile`, `POST /user/api-key` |
| `ApiService` | `GET /system/leaderboard`, `GET /system/snapshot`, `GET /system/players` |
| `GameStreamService` | WebSocket client with exponential-backoff reconnect (1 s → 2 s → 4 s → 8 s); exposes `events$` and `status$` observables |
| `authInterceptor` | Attaches `Authorization: Bearer <jwt>` to every outgoing HTTP request when logged in |

### ⚠️ Frontend–Backend Mismatches

| Area | Frontend calls | Backend has | Status |
|------|---------------|-------------|--------|
| Leaderboard | `GET /api/v1/system/leaderboard` | ❌ Not registered | ❌ Missing |
| World snapshot | `GET /api/v1/system/snapshot` | ❌ Not registered | ❌ Missing |
| Active players | `GET /api/v1/system/players` | ❌ Not registered | ❌ Missing |
| Update profile | `PUT /api/v1/user/profile` | `PATCH /api/v1/user/profile` | ⚠️ Method mismatch |
| Generate API key | `POST /api/v1/user/api-key` | `POST /api/v1/user/apikey` | ⚠️ Path mismatch |

## Architecture Overview

- `src/main.rs` — entry point: loads config, opens DB, spawns game loop, starts Axum server
- `src/state.rs` — `AppState`: holds `RwLock<State>` (live game), `broadcast::Sender` (WebSocket events), `api_key_map` (hash → player_id)
- `src/models.rs` — shared types: `Vector2`, `Rect`, `Platform`, `PlayerState`, `WorldState`, `State`, `GameMode`, `HighScore`, `User`, `Session`
- `src/game/physics.rs` — 60 Hz tick, gravity, AABB collision, bounce, death detection, `apply_jump`
- `src/game/tick.rs` — background Tokio task: ticks all players, resets dead ones to last landed position, broadcasts `position_update`
- `src/game/world.rs` — procedural platform generation (`initial_platforms`, `extend_platforms`)
- `src/api/` — Axum routers: `bot`, `user`, `admin`, `system`, `ws`
- `src/db.rs` — thread-safe SQLite wrapper (`Mutex<Connection>`) with `spawn_blocking` for all queries
- `migrations/migrate.rs` — standalone binary to apply DB migrations

## API Endpoints

### Bot (`/api/v1/bot`) — authenticated via `X-Api-Key` header

| Method | Path | Status | Notes |
|--------|------|--------|-------|
| POST | `/join` | ✅ Implemented | Resolves user from API key hash, evicts duplicate bot for same user, spawns player at origin |
| POST | `/jump` | ✅ Implemented | Accepts `{direction: {x,y} or {angle_deg}, strength}`, validates before acquiring write lock |
| GET | `/state` | ✅ Implemented | Returns position, velocity, dimensions, session_max_height, movement state, all platforms |
| POST | `/leave` | ✅ Implemented | Removes player from game state and `api_key_map`, broadcasts `player_left` |
| GET | `/statistics` | ⚠️ Stub | Returns `{"status": "not implemented"}` — DB query not written |

### User (`/api/v1/user`) — authenticated via JWT

| Method | Path | Status | Notes |
|--------|------|--------|-------|
| POST | `/register` | ✅ Implemented | Creates user with Argon2id password hash and API key |
| POST | `/login` | ✅ Implemented | Verifies password, returns JWT |
| GET | `/profile` | ✅ Implemented | Returns username, display name, created_at |
| PATCH | `/profile` | ✅ Implemented | Updates display name |
| POST | `/apikey` | ✅ Implemented | Regenerates API key, returns new plaintext key |

### Admin (`/api/v1/admin`) — JWT with admin role

| Method | Path | Status | Notes |
|--------|------|--------|-------|
| POST | `/mode` | ✅ Implemented | Sets game mode, resets world, broadcasts `world_reset` |
| POST | `/ratelimits` | ⚠️ Partial | Accepts request but does not update in-memory rate limiters at runtime |
| POST | `/world/reset` | ✅ Implemented | Resets platforms, tick, clears all players, broadcasts `world_reset`; seed always reset to `0` |
| POST | `/players/reset` | ✅ Implemented | Clears all players from game state |
| POST | `/players/{id}/reset` | ✅ Implemented | Resets a single player to their last landed position |

### System (`/api/v1/system`)

| Method | Path | Status | Notes |
|--------|------|--------|-------|
| GET | `/stream` | ✅ Implemented | WebSocket upgrade; broadcasts all game events to spectators |

## WebSocket Events

- `player_joined` — `{player_id, name}`
- `player_left` — `{player_id}`
- `position_update` — `{player_id, x, y, vx, vy, state}` (60 Hz, not logged to file)
- `world_reset` — `{game_mode}`

## Physics Constants

| Constant | Value |
|----------|-------|
| `TICK_RATE_HZ` | 60.0 |
| `GRAVITY` | -10.0 |
| `TOWER_WIDTH` | 20.0 |
| `PLAYER_WIDTH` | 0.8 |
| `PLAYER_HEIGHT` | 0.8 |
| `JUMP_STRENGTH_MIN` | 0.0 |
| `JUMP_STRENGTH_MAX` | 20.0 |
| `DEATH_DROP` | 15.0 |
| `FRICTION` | 0.5 |
| `AIR_RESISTANCE` | 1.0 |

## Testing

- Physics unit tests in `src/game/physics.rs`: gravity, jump normalisation, zero-vector rejection, bounce schedule, death detection, AABB overlap
- `Vector2` unit tests in `src/models.rs`: construction, arithmetic, normalisation, edge cases
- No integration or API-level tests exist
- Run with: `mise run test`

## How to Run

```powershell
cargo run --bin migrate -- .data/botclimber.db   # apply DB migrations
mise run frontend-build                           # build Angular frontend
mise run dev                                      # start dev server
```

## Key Docs

See `docs/requirements.md` for an index of all available documentation.

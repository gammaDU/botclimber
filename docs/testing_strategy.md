# Botclimber: Testing Strategy

This document defines the testing approach for the Botclimber server. It covers the test pyramid, the two primary test targets (API endpoints and the physics engine), and the concrete test cases required for each.

---

## 1. Test Pyramid

```
        ┌─────────────────────┐
        │   Integration Tests  │  ← API endpoint tests (HTTP + WebSocket)
        ├─────────────────────┤
        │     Unit Tests       │  ← Physics engine: collisions, bounces, deceleration
        └─────────────────────┘
```

- **Unit tests** cover the physics engine in isolation — no HTTP, no database, no async runtime required. They are fast, deterministic, and run on every `cargo test`.
- **Integration tests** spin up the full Axum server (with an in-memory SQLite database) and exercise the API over real HTTP and WebSocket connections. They verify that authentication, routing, rate limiting, and game state management work end-to-end.

All tests live inside the Rust crate under `src/` (unit tests as `#[cfg(test)]` modules co-located with the code) and `tests/` (integration tests as separate files).

---

## 2. Physics Engine Unit Tests

The physics engine is the most critical and most testable component. All expected values below are derived directly from the constants in [game_rules.md](game_rules.md).

### 2.1 Constants Under Test

| Constant | Value |
|---|---|
| Gravity | `-10.0 units/s²` |
| Tick rate | `60 Hz` (Δt = `1/60 s` per tick) |
| Player AABB | `0.8 × 0.8 units` |
| Tower X bounds | `-10.0` to `+10.0` |
| Jump strength range | `0` – `20.0` |
| Landing threshold | `|v_y| < 0.1 units/s` on top surface contact |
| Death threshold | `15.0 units` below session max height |

### 2.2 Gravity and Projectile Motion

| Test Case | Setup | Expected Result |
|---|---|---|
| Gravity applied per tick | Player at rest with `v_y = 0`, advance one tick | `v_y = -10.0 / 60 ≈ -0.1667 units/s` |
| Parabolic arc | Player jumps straight up with strength `20.0` (`v0 = (0, 20)`), advance N ticks | Position follows `y = v0·t - 5·t²`; peak at `t = 2.0 s` (`y = 20.0 units`) |
| Zero air resistance | Player with horizontal velocity `v_x = 5.0`, advance 10 ticks | `v_x` unchanged; only `v_y` changes due to gravity |

### 2.3 Collision Detection (AABB vs AABB)

| Test Case | Setup | Expected Result |
|---|---|---|
| Top surface landing | Player falling onto top face of a platform | Collision detected; player placed on top surface; `v_y` reflected |
| Bottom surface hit | Player moving upward into bottom face of a platform | Collision detected; `v_y` reflected (negated) |
| Left wall hit | Player moving right into left face of a platform | Collision detected; `v_x` reflected (negated) |
| Right wall hit | Player moving left into right face of a platform | Collision detected; `v_x` reflected (negated) |
| Tower left wall | Player at `x ≈ -10.0` moving left | `v_x` reflected; player kept within bounds |
| Tower right wall | Player at `x ≈ +10.0` moving right | `v_x` reflected; player kept within bounds |
| No false positive | Player AABB adjacent to but not overlapping a platform | No collision detected |
| Corner case | Player AABB corner grazes platform corner | Collision resolved against the axis of least penetration |

### 2.4 Bounce Deceleration

Each bounce applies a flat `FRICTION = 0.5` factor to the reflected velocity component.

| Test Case | Setup | Expected Result |
|---|---|---|
| Side bounce | Player hits a wall with `v_x = 10.0` | `v_x` reflected and multiplied by `0.5`; outgoing `|v_x| = 5.0` |
| Top/bottom bounce | Player hits a platform underside with `v_y = 10.0` | `v_y` reflected and multiplied by `0.5`; outgoing `|v_y| = 5.0` |
| Soft landing | Player hits top surface with `|v_y| < 0.1` | Player transitions to `LANDED`; velocity set to `(0, 0)` |

### 2.5 State Transitions

| Test Case | Setup | Expected Result |
|---|---|---|
| LANDED → JUMPING on jump | Player in `LANDED` state receives a valid jump command | State becomes `JUMPING`; velocity set to `direction_unit * strength` |
| Jump ignored in JUMPING state | Player in `JUMPING` state receives a jump command | Command ignored; state and velocity unchanged |
| JUMPING → LANDED on soft top landing | Player hits top surface with `|v_y| < 0.1` | State becomes `LANDED`; velocity set to `(0, 0)` |
| JUMPING stays JUMPING on hard top landing | Player hits top surface with `|v_y| ≥ 0.1` | State remains `JUMPING`; velocity reflected |

### 2.6 Death and Reset (Checkpoint Mode)

| Test Case | Setup | Expected Result |
|---|---|---|
| Death threshold triggered | Player's `y` drops more than `15.0` below `session_max_height` | Player is considered "fallen off" |
| Checkpoint reset | Player falls off; `last_landed_position` exists | Player teleported to `last_landed_position`; velocity set to `(0, 0)` |
| No checkpoint reset | Player falls off; no `last_landed_position` (first jump) | Player reset to `(0, 0)`; velocity set to `(0, 0)` |
| Hardcore — no reset | Player falls off in Hardcore mode | No teleport; player continues to fall under gravity |
| Max height tracking | Player reaches new peak `y` | `session_max_height` updated; `last_landed_position` updated on next landing |

### 2.7 Jump Input Validation

| Test Case | Setup | Expected Result |
|---|---|---|
| Strength below minimum | `strength = -1.0` | Request rejected with `400 Bad Request` |
| Strength above maximum | `strength = 25.0` | Request rejected with `400 Bad Request` |
| Non-unit direction vector | `direction = (3.0, 4.0)` (magnitude 5) | Server normalises to `(0.6, 0.8)` and accepts; resulting velocity is `(0.6, 0.8) * strength` |
| Zero direction vector | `direction = (0.0, 0.0)` | Request rejected with `400 Bad Request` |

---

## 3. API Endpoint Integration Tests

Integration tests start the full server with a temporary in-memory database. Each test group is independent; the database is reset between groups.

### 3.1 Bot API (`/api/v1/bot/`)

Authentication: `X-API-Key` header.

| Test Case | Method + Path | Setup | Expected Response |
|---|---|---|---|
| Join without API key | `POST /bot/join` | No header | `401 Unauthorized` |
| Join with invalid API key | `POST /bot/join` | `X-API-Key: invalid` | `401 Unauthorized` |
| Join successfully | `POST /bot/join` | Valid API key | `200 OK`; body contains `player_id`, `session_id`, `spawn_point (0,0)`, `state: LANDED` |
| Join twice (same key) | `POST /bot/join` × 2 | Valid API key | Second call returns `409 Conflict` (already in session) |
| Jump while LANDED | `POST /bot/jump` | Valid key, player LANDED, valid strength+direction | `200 OK` |
| Jump while JUMPING | `POST /bot/jump` | Valid key, player JUMPING | `409 Conflict` or `200 OK` with ignored confirmation (per spec) |
| Jump with invalid strength | `POST /bot/jump` | `strength = 999` | `400 Bad Request` |
| Get state | `GET /bot/state` | Valid key, player in session | `200 OK`; body contains position, velocity, state, surrounding platforms |
| Get state without joining | `GET /bot/state` | Valid key, not joined | `404 Not Found` |
| Get statistics | `GET /bot/statistics` | Valid key, player in session | `200 OK`; body contains `session_id`, `maximal_height_session`, `high_score_history` |
| Leave session | `POST /bot/leave` | Valid key, player in session | `204 No Content` |
| Leave without joining | `POST /bot/leave` | Valid key, not joined | `404 Not Found` |

### 3.2 Frontend API (`/api/v1/user/`)

Authentication: `Authorization: Bearer <JWT>`.

| Test Case | Method + Path | Setup | Expected Response |
|---|---|---|---|
| Register new user | `POST /user/register` | Valid username + password | `201 Created` |
| Register duplicate username | `POST /user/register` | Username already exists | `409 Conflict` |
| Register with short password | `POST /user/register` | Password too short (per policy) | `400 Bad Request` |
| Login with correct credentials | `POST /user/login` | Valid username + password | `200 OK`; body contains JWT |
| Login with wrong password | `POST /user/login` | Wrong password | `401 Unauthorized` |
| Login with unknown username | `POST /user/login` | Non-existent user | `401 Unauthorized` |
| Get profile (authenticated) | `GET /user/profile` | Valid JWT | `200 OK`; body contains `id`, `username`, `display_name` |
| Get profile (unauthenticated) | `GET /user/profile` | No JWT | `401 Unauthorized` |
| Update display name | `PUT /user/profile` | Valid JWT, new `display_name` | `200 OK` |
| Generate API key | `POST /user/apikey` | Valid JWT | `200 OK`; body contains new API key |
| Generate API key (unauthenticated) | `POST /user/apikey` | No JWT | `401 Unauthorized` |

### 3.3 Admin API (`/api/v1/admin/`)

Authentication: `Authorization: Bearer <JWT>` with `is_admin = true`.

| Test Case | Method + Path | Setup | Expected Response |
|---|---|---|---|
| Switch mode (admin) | `POST /admin/mode` | Admin JWT, `{"game_mode": "Hardcore"}` | `200 OK` |
| Switch mode (non-admin) | `POST /admin/mode` | Non-admin JWT | `403 Forbidden` |
| Switch mode (unauthenticated) | `POST /admin/mode` | No JWT | `401 Unauthorized` |
| Switch to invalid mode | `POST /admin/mode` | Admin JWT, `{"game_mode": "invalid"}` | `400 Bad Request` |
| Update rate limits | `POST /admin/ratelimits` | Admin JWT, valid tier + values | `200 OK` |
| Update rate limits (missing tier) | `POST /admin/ratelimits` | Admin JWT, no `tier` field | `400 Bad Request` |
| Reset world | `POST /admin/world/reset` | Admin JWT | `200 OK` |
| Reset all players | `POST /admin/players/reset` | Admin JWT | `200 OK` |
| Reset specific player | `POST /admin/players/{id}/reset` | Admin JWT, valid player UUID | `200 OK` |
| Reset non-existent player | `POST /admin/players/{id}/reset` | Admin JWT, unknown UUID | `404 Not Found` |

### 3.4 System API (`/api/v1/system/`)

No authentication required.

| Test Case | Method + Path | Setup | Expected Response |
|---|---|---|---|
| Get leaderboard (no filter) | `GET /system/leaderboard` | — | `200 OK`; array of `LeaderboardEntry` |
| Get leaderboard (mode filter) | `GET /system/leaderboard?game_mode=Hardcore` | — | `200 OK`; entries filtered to Hardcore only |
| Get active players | `GET /system/active-players` | One player joined | `200 OK`; array contains that player |
| Get snapshot | `GET /system/snapshot` | One player joined | `200 OK`; body contains `timestamp`, `players`, `platforms` |
| WebSocket connect | `WS /system/stream` | — | `101 Switching Protocols` |
| WebSocket receives position_update | `WS /system/stream` | Player jumps | `position_update` event received within one tick period |
| WebSocket receives player_joined | `WS /system/stream` | Second player joins after WS open | `player_joined` event received |
| WebSocket receives world_reset | `WS /system/stream` | Admin resets world | `world_reset` event received |

### 3.5 Rate Limiting

| Test Case | Setup | Expected Result |
|---|---|---|
| IP limit exceeded | Send > `ip_burst_size` requests from same IP with no auth | `429 Too Many Requests` with `Retry-After` header |
| Bot limit exceeded | Send > `bot_burst_size` requests with same API key | `429 Too Many Requests` with `RateLimit-*` headers |
| User limit exceeded | Send > `user_burst_size` requests with same JWT | `429 Too Many Requests` |
| Burst then recover | Exhaust bot bucket, wait for refill, send one request | `200 OK` (bucket partially refilled) |

---

## 4. Test Data and Fixtures

- **Test user**: Created fresh per integration test group via `POST /user/register`; API key generated via `POST /user/apikey`.
- **Test admin**: A seeded admin user inserted directly into the test database before the server starts.
- **Physics fixtures**: All unit tests use hard-coded input values and compare against analytically computed expected values (no randomness).
- **Floating-point comparisons**: Use an epsilon of `1e-4` for all physics assertions to account for floating-point rounding across tick accumulation.

---

## 5. Related Documents

- [Game Rules and Physics](game_rules.md) — source of all physics constants used in Section 2
- [OpenAPI Specification](openapi.yaml) — authoritative contract for all endpoint tests in Section 3
- [Technical Constraints](technical_constraints.md) — Axum/Tokio stack; `tokio::test` is the async test runtime
- [Architecture: Security Design](architecture/security_design.md) — rate limiting tiers and auth mechanisms tested in Sections 3.1–3.5

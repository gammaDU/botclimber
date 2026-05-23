# Botclimber: Server Configuration

This document describes all runtime configuration options for the Botclimber server. Configuration is loaded from a `config.toml` file at startup and can be overridden by environment variables. Environment variables always take precedence over file values.

---

## Configuration File Location

By default the server looks for `config.toml` in the current working directory. A different path can be specified with the `BOTCLIMBER_CONFIG` environment variable:

```
BOTCLIMBER_CONFIG=/etc/botclimber/config.toml ./botclimber
```

---

## Full `config.toml` Reference

```toml
# ── Server ────────────────────────────────────────────────────────────────────
[server]
# Host and port the HTTP server binds to.
# Default: "0.0.0.0:8080"
bind = "0.0.0.0:8080"

# ── Database ──────────────────────────────────────────────────────────────────
[database]
# Path to the SQLite database file.
# The file is created automatically on first run if it does not exist.
# Default: "botclimber.db"
path = "botclimber.db"

# ── Authentication ────────────────────────────────────────────────────────────
[auth]
# Secret key used to sign and verify JWT tokens (HMAC SHA-256).
# REQUIRED — the server will refuse to start if this value is empty.
# Must be at least 32 characters long.
# Never commit this value to source control; use the environment variable override.
jwt_secret = ""

# JWT token lifetime in seconds.
# Default: 3600 (1 hour)
jwt_expiry_seconds = 3600

# ── Rate Limiting ─────────────────────────────────────────────────────────────
[rate_limiting]
# --- Global IP-based limiter (applies to all unauthenticated requests) ---

# Sustained request rate allowed per IP address (requests per second).
# Default: 10
ip_requests_per_second = 10

# Maximum burst size for the IP bucket (token bucket capacity).
# Default: 20
ip_burst_size = 20

# --- Per-API-key limiter (applies to /bot/ endpoints) ---

# Sustained request rate allowed per API key (requests per second).
# Default: 10
bot_requests_per_second = 10

# Maximum burst size for the per-key bucket.
# Default: 30
bot_burst_size = 30

# --- Per-user limiter (applies to /user/ endpoints) ---

# Sustained request rate allowed per authenticated user (requests per second).
# Default: 5
user_requests_per_second = 5

# Maximum burst size for the per-user bucket.
# Default: 10
user_burst_size = 10

# ── Game ──────────────────────────────────────────────────────────────────────
[game]
# Maximum number of players allowed in the session simultaneously.
# Default: 50
max_players = 50

# Physics tick rate in Hz (how often the game state is updated per second).
# Default: 60
tick_rate_hz = 60

# Inactivity timeout in seconds before an idle player is automatically removed.
# Default: 300 (5 minutes)
player_timeout_seconds = 300

# Fall behaviour mode: what happens when a player falls off a platform.
# Valid values: "Checkpoint", "Hardcore"
# Default: "Checkpoint"
fall_mode = "Checkpoint"

# Session type: defines the win/end condition for a session.
# Valid values: "Race", "Timetrial", "Infinite"
# Default: "Race"
session_type = "Race"

# Stage type: how the world platforms are produced each session.
# Valid values: "Generated", "Fixed"
# Default: "Generated"
stage_type = "Generated"

# Fixed stage identifier: the name/ID of the hand-crafted stage layout to load.
# Only used when stage_type = "Fixed". Ignored for Generated stages.
# The value must match a stage file present in the stages directory.
# Default: "default"
fixed_stage_id = "default"

# Goal height: the Y coordinate (world units) a player must reach to finish.
# Used by Race and Timetrial session types on Generated stages.
# Ignored for Fixed stages (the goal is defined inside the stage layout file)
# and for Infinite sessions (which have no end condition).
# Default: 1000.0
goal_height = 1000.0

# ── Frontend ──────────────────────────────────────────────────────────────────
[frontend]
# Path to the compiled frontend dist/ directory served at /.
# Default: "frontend/dist"
dist_path = "frontend/dist"
```

---

## Environment Variable Overrides

Every configuration key can be overridden by an environment variable. The naming convention is:

```
BOTCLIMBER_<SECTION>_<KEY>   (all uppercase, dots replaced by underscores)
```

| Environment Variable | Overrides | Example |
|---|---|---|
| `BOTCLIMBER_SERVER_BIND` | `server.bind` | `0.0.0.0:9090` |
| `BOTCLIMBER_DATABASE_PATH` | `database.path` | `/var/lib/botclimber/data.db` |
| `BOTCLIMBER_AUTH_JWT_SECRET` | `auth.jwt_secret` | `s3cr3t-at-least-32-chars-long!!` |
| `BOTCLIMBER_AUTH_JWT_EXPIRY_SECONDS` | `auth.jwt_expiry_seconds` | `7200` |
| `BOTCLIMBER_RATE_LIMITING_IP_REQUESTS_PER_SECOND` | `rate_limiting.ip_requests_per_second` | `20` |
| `BOTCLIMBER_RATE_LIMITING_IP_BURST_SIZE` | `rate_limiting.ip_burst_size` | `40` |
| `BOTCLIMBER_RATE_LIMITING_BOT_REQUESTS_PER_SECOND` | `rate_limiting.bot_requests_per_second` | `15` |
| `BOTCLIMBER_RATE_LIMITING_BOT_BURST_SIZE` | `rate_limiting.bot_burst_size` | `50` |
| `BOTCLIMBER_RATE_LIMITING_USER_REQUESTS_PER_SECOND` | `rate_limiting.user_requests_per_second` | `5` |
| `BOTCLIMBER_RATE_LIMITING_USER_BURST_SIZE` | `rate_limiting.user_burst_size` | `10` |
| `BOTCLIMBER_GAME_MAX_PLAYERS` | `game.max_players` | `50` |
| `BOTCLIMBER_GAME_TICK_RATE_HZ` | `game.tick_rate_hz` | `60` |
| `BOTCLIMBER_GAME_PLAYER_TIMEOUT_SECONDS` | `game.player_timeout_seconds` | `120` |
| `BOTCLIMBER_GAME_FALL_MODE` | `game.fall_mode` | `Hardcore` |
| `BOTCLIMBER_GAME_SESSION_TYPE` | `game.session_type` | `Infinite` |
| `BOTCLIMBER_GAME_STAGE_TYPE` | `game.stage_type` | `Fixed` |
| `BOTCLIMBER_GAME_FIXED_STAGE_ID` | `game.fixed_stage_id` | `my-custom-stage` |
| `BOTCLIMBER_GAME_GOAL_HEIGHT` | `game.goal_height` | `2000.0` |
| `BOTCLIMBER_FRONTEND_DIST_PATH` | `frontend.dist_path` | `/srv/botclimber/dist` |

---

## Startup Validation

The server performs the following checks at startup and exits with a non-zero status code if any check fails:

| Check | Condition |
|---|---|
| JWT secret present | `auth.jwt_secret` must be non-empty and ≥ 32 characters |
| Bind address valid | `server.bind` must be a valid `host:port` string |
| Database path writable | The directory containing `database.path` must be writable |
| Frontend dist exists | `frontend.dist_path` must exist and contain `index.html` |
| Tick rate in range | `game.tick_rate_hz` must be between 1 and 120 |
| Max players in range | `game.max_players` must be between 1 and 50 |

---

## Minimal Production `config.toml`

The following is the smallest valid configuration for a production deployment. All other values fall back to their defaults:

```toml
[server]
bind = "0.0.0.0:8080"

[database]
path = "/var/lib/botclimber/botclimber.db"

[auth]
# Set via environment variable — do not put the real secret here
jwt_secret = ""
```

Set the secret via the environment:

```powershell
$env:BOTCLIMBER_AUTH_JWT_SECRET = "your-secret-here-at-least-32-chars"
.\botclimber.exe
```

```bash
BOTCLIMBER_AUTH_JWT_SECRET="your-secret-here-at-least-32-chars" ./botclimber
```

---

## Live Reload

Rate limiting parameters (`rate_limiting.*`) can be updated at runtime without restarting the server via the Admin API:

```
POST /api/v1/admin/ratelimits
```

All other configuration options require a server restart to take effect.

---

## Related Documents

- [Technical Constraints](technical_constraints.md) — crate choices and hard constraints for configuration loading
- [Security Design](architecture/security_design.md) — JWT and API key security requirements
- [openapi.yaml](openapi.yaml) — `POST /admin/ratelimits` endpoint for live rate limit updates

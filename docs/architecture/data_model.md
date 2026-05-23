# Botclimber: Data Model Specification

This document defines the data structures for the Botclimber application, covering both persistent storage (SQLite) and in-memory runtime state.

## 1. Persistent Storage (SQLite)

The persistence layer handles user accounts and all-time records.

### 1.1 `users` Table
Stores user credentials and profile information.

| Column | Type | Description |
| :--- | :--- | :--- |
| `id` | UUID | Primary key. |
| `username` | TEXT | Unique username used for login. |
| `password_hash` | TEXT | Argon2id hash of the user's password. |
| `display_name` | TEXT | The name shown on leaderboards. |
| `api_key` | TEXT | Unique, secret key used for Bot API authentication. Indexed. |
| `is_admin` | BOOLEAN | Flag indicating if the user has administrative privileges. |
| `created_at` | DATETIME | Timestamp of account creation. |

### 1.2 `sessions` Table
Tracks individual gameplay sessions. Each session has a fully specified game mode configuration across three independent dimensions.

| Column | Type | Description |
| :--- | :--- | :--- |
| `id` | UUID | Primary key. |
| `fall_mode` | TEXT | Fall behaviour: `Checkpoint` (reset to last landed platform) or `Hardcore` (fall freely until natural stop). |
| `session_type` | TEXT | Session format: `Race` (first to goal wins), `Timetrial` (best time to goal), or `Infinite` (no goal, climb forever). |
| `stage_type` | TEXT | Stage source: `Generated` (procedurally generated from seed) or `Fixed` (predefined stage). |
| `fixed_stage_id` | TEXT | Identifier of the fixed stage layout. Only relevant when `stage_type = Fixed`; NULL otherwise. |
| `goal_height` | REAL | The height (units) that constitutes reaching the goal. Only relevant for `Race` and `Timetrial` session types on `Generated` stages; NULL otherwise. |
| `started_at` | DATETIME | Timestamp when the session started. |
| `ended_at` | DATETIME | Timestamp when the session ended (NULL if active). |

### 1.3 `high_scores` Table
Stores the best result achieved by a player in a given gameplay session. The full game mode configuration is derived by joining with the `sessions` table via `session_id`.

| Column | Type | Description |
| :--- | :--- | :--- |
| `id` | UUID | Primary key. |
| `user_id` | UUID | Foreign key to `users.id`. Links the score to a persistent user. |
| `session_id` | UUID | Foreign key to `sessions.id`. Links the score to a specific gameplay session; the game mode configuration is obtained from the `sessions` table. |
| `max_height` | REAL | The maximal height reached (units). Relevant for all session types. |
| `completion_time_ms` | INTEGER | Time in milliseconds to reach `goal_height`. Only populated for `Timetrial` sessions; NULL otherwise. |
| `achieved_at` | DATETIME | Timestamp of the record. |

## 2. In-Memory Runtime State (Game Engine)

The Game Engine maintains a single `State` object in memory, which holds the current session ID, the world configuration, and the per-player `PlayerState` entries, enabling high-performance physics simulation.

### 2.1 `PlayerState`
Represents the runtime state of an active bot in the game world.

| Field | Type | Description |
| :--- | :--- | :--- |
| `user_id` | UUID | Links the player to a persistent user. |
| `position` | Vector2 (x, y) | Current coordinates in the tower. |
| `velocity` | Vector2 (vx, vy) | Current velocity vector. |
| `state` | Enum | `LANDED`, `JUMPING`. |
| `last_landed_position` | Vector2 (x, y) | The last position where the player was in the `LANDED` state. Used in `Checkpoint` fall mode to reset the player's position after a fall. |
| `session_max_height` | REAL | The highest point reached in the current session. |
| `timetrial_start_ms` | INTEGER | Timestamp (ms) when the player started the current timetrial run. NULL if not in a `Timetrial` session or before the first reset. |
| `activity_rating` | REAL | Rolling activity score used by the director cam to prioritize which players to focus on. Higher values indicate more recent or frequent actions. |
| `last_heartbeat` | DATETIME | Used to cleanup inactive sessions. |

### 2.2 `State`
The top-level in-memory state container held by the Game Engine. Owns the world state and all active player states.

| Field | Type | Description |
| :--- | :--- | :--- |
| `session_id` | UUID | Unique identifier for the current gameplay session. |
| `fall_mode` | Enum | Current fall behaviour: `Checkpoint` or `Hardcore`. |
| `session_type` | Enum | Current session format: `Race`, `Timetrial`, or `Infinite`. |
| `stage_type` | Enum | Current stage source: `Generated` or `Fixed`. |
| `fixed_stage_id` | TEXT | Identifier of the active fixed stage. NULL when `stage_type = Generated`. |
| `goal_height` | REAL | Target height for `Race` and `Timetrial` sessions on `Generated` stages. NULL otherwise. |
| `world` | `WorldState` | The current world configuration (platforms, seed). |
| `players` | Map<UUID, `PlayerState`> | Map from `user_id` to the corresponding `PlayerState` for every active player. |

### 2.3 `WorldState`
Maintains the global configuration of the tower.

| Field | Type | Description |
| :--- | :--- | :--- |
| `platforms` | List<Platform> | The list of platforms in the current world. For `Generated` stages this is a repeating pattern; for `Fixed` stages it is the full predefined layout. |
| `seed` | INTEGER | The seed used for world generation. NULL for `Fixed` stages. |

### 2.4 `Platform`
Defines a solid surface in the tower.

| Field | Type | Description |
| :--- | :--- | :--- |
| `id` | UUID | Unique identifier. |
| `rect.x` | REAL | Left edge X coordinate of the platform. |
| `rect.y` | REAL | Bottom edge Y coordinate of the platform. |
| `rect.width` | REAL | Width of the platform. |
| `rect.height` | REAL | Height (thickness) of the platform. |

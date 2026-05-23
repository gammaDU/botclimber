### US12: Game Modes
**As a** Player,
**I want** the server to support different game modes,
**So that** I can practice in a checkpoint-enabled environment or challenge myself in a hardcore session set by the administrator.

**Acceptance Criteria:**

#### Fall Behaviour Mode
Controls what happens when a player falls off a platform. Configured via `game.fall_mode`.

- **Checkpoint** *(default)*: Falling off resets the player to the last position where they were in the **LANDED** state.
- **Hardcore**: No automatic reset; players fall freely until they come to a natural stop.

#### Session Type
Controls the win/end condition for a session. Configured via `game.session_type`.

- **Race** *(default)*: The first player to reach the top of the tower wins; the session ends immediately when a winner is declared.
- **Timetrial**: Each player is timed individually from join to reaching the top; the session continues until the admin resets it and the best time wins.
- **Infinite**: No end condition; the session runs indefinitely until the administrator resets the world.

#### Stage Type
Controls how the world platforms are produced. Configured via `game.stage_type`.

- **Generated** *(default)*: Platforms are procedurally generated from a random seed at the start of each session. When using Race or Timetrial, the height a player must reach to finish is set via `game.goal_height`.
- **Fixed**: Platforms are loaded from a fixed, hand-crafted layout identified by `game.fixed_stage_id`. The goal is defined inside the stage layout file itself; `game.goal_height` is ignored.

#### General
- All three dimensions are independent and can be combined freely (e.g. Hardcore + Race + Generated).
- All mode settings are server-wide and controlled by the Administrator (US09).
- Game mode information (all three dimensions) is included in game state reports (US03) and leaderboard displays (US06).

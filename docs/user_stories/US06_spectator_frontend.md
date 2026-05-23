### US06: Web Frontend and Leaderboards
**As a** Spectator or Player,
**I want** a web-based frontend to watch the game, view high scores, and manage my account,
**So that** I can enjoy the competition, track my progress against others, and register/login without using external tools.

**Acceptance Criteria:**

#### General Frontend
- The server provides a web-based frontend (HTML/JS) accessible via a browser.
- The frontend visualizes the current game state, including the infinite tower, platforms, and all active players, by connecting to a WebSocket-based stream.
- The frontend provides a full account management UI (registration, login, profile editing, API key generation) as specified in [US02](US02_account_and_auth.md).
- The frontend receives real-time game state updates (player positions, new platforms) via a WebSocket connection under the System API namespace.

#### Camera & Spectating
- The system provides an automated "Director Cam" that automatically switches focus between different active players to highlight interesting gameplay.
- Spectators can manually override the director cam to follow a specific player of interest.

#### Session Rankings (Live)
- The frontend displays a real-time list of currently active players for the ongoing session.
- In **Race** and **Infinite** sessions, players are ranked by their current or maximal height reached in the current session.
- In **Timetrial** sessions, players are ranked by their best completion time in the current session (lower is better); players who have not yet finished are ranked below those who have, ordered by current height.
- The session ranking panel displays the active game mode configuration (fall behaviour, session type, stage type) so spectators understand the current rules.

#### Leaderboard (Persistent High Scores)
- The system persists high scores across server restarts.
- High scores are stored and displayed **per game mode combination** (fall behaviour × session type × stage type). Scores from different mode combinations are never mixed.
- In **Race** and **Infinite** modes, the high score metric is the **maximum height** reached by a player/bot. The leaderboard shows player/bot name and their record height.
- In **Timetrial** mode, the high score metric is the **best completion time**. The leaderboard shows player/bot name and their fastest time.
- For **Fixed** stages, high scores are additionally scoped to the `fixed_stage_id` so records for different hand-crafted stages are kept separate.
- The frontend displays the leaderboard filtered to the currently active game mode by default, with an option to browse other mode combinations.
- The system provides an API endpoint to retrieve high score data (for the frontend or external queries, see System API and [US07](US07_api_specification.md)).

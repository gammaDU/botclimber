### US06: Web Frontend and Leaderboards
**As a** Spectator or Player,
**I want** a web-based frontend to watch the game, view high scores, and manage my account,
**So that** I can enjoy the competition, track my progress against others, and register/login without using external tools.

**Acceptance Criteria:**
- The server provides a web-based frontend (HTML/JS) accessible via a browser.
- The frontend visualizes the current game state, including the infinite tower, platforms, and all active players, by connecting to a WebSocket-based stream.
- The frontend provides a full account management UI (registration, login, profile editing, API key generation) as specified in [US02](US02_account_and_auth.md).
- The frontend receives real-time game state updates (player positions, new platforms) via a WebSocket connection under the System API namespace.
- The system persists all-time maximal heights reached (high scores) even if the server restarts.
- The frontend displays a leaderboard of these persisted high scores, including the player/bot name and their respective record height.
- The frontend displays a real-time list of currently active players, ranked by their current or maximal height reached in the current session.
- High scores (all-time records) and active session rankings are separated or tagged by game mode (US12).
- The leaderboard and session statistics allow users to see all high scores associated with a specific gameplay session.
- The system provides an automated "Director Cam" that automatically switches focus between different active players to highlight interesting gameplay.
- Spectators can manually override the director cam to follow a specific player of interest.
- The system provides a way to retrieve the high score data (for the frontend or external queries, see System API).

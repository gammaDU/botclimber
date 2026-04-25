### US03: Game State and Statistics
**As a** Player,
**I want** to query the current state of the game, my position, and my historical performance,
**So that** my bot can make informed decisions about the next jump and I can monitor its success.

**Acceptance Criteria:**
- Bot interaction requires an `X-API-Key` header for authentication; commands without a valid key are rejected with `401 Unauthorized` (see US01).
- The system provides a spatial game state endpoint that returns the player's current X, Y coordinates, bounding box dimensions, and current state (`LANDED` or `JUMPING`).
- The spatial state includes the player's current vertical height and a list of surrounding platforms within a certain radius, including their positions and dimensions.
- The system provides a separate statistics endpoint that returns the player's session-specific metrics, uniquely identified by a `session_id`.
- The statistics include the maximal height reached in the current session and a history of high scores achieved during this session.
- Both endpoints reflect the player's current game mode (Checkpoint vs. Hardcore, see US12).

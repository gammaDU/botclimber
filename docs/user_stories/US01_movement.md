### US01: Basic Movement
**As a** Player,
**I want** to send a jump command with specific strength and direction after joining the game,
**So that** I can move my player figure higher up the infinite tower.

**Acceptance Criteria:**
- Bot interaction requires an `X-API-Key` header for authentication; commands without a valid key are rejected with `401 Unauthorized`.
- Each API key is uniquely mapped to one active player figure in the game session.
- A player must have joined the game session before being able to jump.
- A command to jump is available, but only executable when the player is in the `LANDED` state.
- Bot developers must poll the game state to ensure the player has reached the `LANDED` state before sending the next jump command.
- Request accepts `strength` (float/int) and `direction` (angle or vector); physics behaviour (velocity calculation, reflections, state transitions) is defined in [game_rules.md](../game_rules.md).
- The tower is infinite; there is no upper bound or "top" to reach.
- Returns the new estimated position or a confirmation of the command.

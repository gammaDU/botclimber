### US04: Multi-Player Competition
**As a** Player,
**I want** to compete with up to 49 other bots simultaneously,
**So that** we can have a real-time race to the top.

**Acceptance Criteria:**
- Bot interaction for joining the game requires an `X-API-Key` header for authentication.
- Commands sent without a valid API key are rejected with `401 Unauthorized`.
- Each API key is uniquely mapped to one active player figure.
- A player can send a "Join" command to spawn their figure in the game world.
- The server supports at least 50 concurrent active player figures.
- Each player can send commands independently.
- The server processes commands and updates the global game state in real-time.

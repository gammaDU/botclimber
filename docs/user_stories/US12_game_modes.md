### US12: Game Modes
**As a** Player,
**I want** the server to support different game modes,
**So that** I can practice in a checkpoint-enabled environment or challenge myself in a hardcore session set by the administrator.

**Acceptance Criteria:**
- The system supports multiple game modes: "Checkpoint" and "Hardcore".
- The game mode is a server-wide setting controlled by the Administrator (US09).
- In **Checkpoint Mode**, falling off results in a reset to the last position where the player was in the **LANDED** state.
- In **Hardcore Mode**, there is no automatic reset; players simply land on whatever surface they hit after falling.
- Game mode information is included in game state reports (US03) and leaderboard displays (US06).

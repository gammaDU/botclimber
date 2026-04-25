### US05: Platform Generation (Prototype)
**As a** Player,
**I want** the tower platforms to be generated in a predictable, repeating pattern for the prototype,
**So that** I can test my bot's movement logic against a stable environment.

**Acceptance Criteria:**
- Platforms follow a fixed, repeating vertical pattern (e.g., every 50 units the pattern repeats).
- The pattern ensures that it is always physically possible to reach the next platform in the sequence (within maximum jump strength).
- All players see the same platforms at the same heights.
- The pattern includes platforms of varying widths and horizontal positions within the tower boundaries.
- The game state API (US03) reports these platforms as the player moves.
- **Future Requirement:** The system should be designed to allow swapping this fixed pattern with a full procedural generation algorithm later.

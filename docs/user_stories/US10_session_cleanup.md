### US10: Session Management and Cleanup
**As an** Administrator,
**I want** the system to automatically handle player departures and clean up inactive sessions,
**So that** server resources are preserved and the game world remains populated only by active bots.

**Acceptance Criteria:**
- A player can send a "Leave" command to gracefully exit the game session and remove their figure.
- The system automatically detects inactive bots (e.g., no heartbeat or commands for a specified duration).
- Inactive player figures are removed from the game world after a configurable timeout period.
- The "Active Players" list in the frontend (US06) and system API reflects only currently active sessions.
- Removing a figure does not delete the player's account or their all-time high scores.
- The server logs session starts and ends for administrative monitoring.

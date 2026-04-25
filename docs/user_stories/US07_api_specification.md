### US07: API Specification
**As a** Core Developer,
**I want** a formal OpenAPI definition of all system endpoints,
**So that** I can ensure API consistency, facilitate automated testing, and provide clear documentation for bot developers.

**Acceptance Criteria:**
- The system includes a valid OpenAPI 3.0.x specification file (e.g., `openapi.yaml`).
- The specification clearly separates endpoints into namespaces for Bots (gameplay), the Web Frontend (account management), Administration (server settings), and the System (public data) under the `/api/v1` base path.
- The specification covers all gameplay and state endpoints (Join, Leave, Jump, Spatial Game State, Player Statistics).
- The specification includes administrative endpoints to manage the server (Change Game Mode, World Regeneration, Player Resets).
- The specification covers all user management endpoints (Registration, Login, Profile, API Key management).
- The specification covers all shared public data endpoints (Leaderboards, Active Players, and WebSocket Game Stream) under the System API namespace.
- All request and response schemas are clearly defined with data types and mandatory fields.
- Authentication mechanisms (API Key and Bearer tokens) are documented in the specification.
- The specification serves as the "source of truth" for the API's structure and behavior.

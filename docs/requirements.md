# Botclimber Requirements
This directory contains the user stories, architectural designs, and technical specifications for the Botclimber project.

## User Stories

### Player
- [US01: Basic Movement](user_stories/US01_movement.md) - Defines the jump command and state-based movement (LANDED, JUMPING).
- [US02: Account Management](user_stories/US02_account_and_auth.md) - Covers registration, login, and API key generation for bot authentication.
- [US03: Game State and Statistics](user_stories/US03_game_state.md) - Specifies the retrieval of spatial world data and session-linked performance metrics.
- [US04: Multiplayer](user_stories/US04_multiplayer.md) - Handles joining/leaving sessions and concurrent player interaction.
- [US05: Platform Generation (Prototype)](user_stories/US05_platform_generation.md) - Defines the 50m repeating pattern for the initial tower implementation.
- [US11: Game Rules and Physics Documentation](user_stories/US11_game_rules_documentation.md) - Requires clear technical documentation for bot developers; canonical content lives in [game_rules.md](game_rules.md).
- [US12: Game Modes](user_stories/US12_game_modes.md) - Defines Checkpoint and Hardcore modes and their effect on gameplay.

### Spectator
- [US06: Web Frontend and Leaderboards](user_stories/US06_spectator_frontend.md) - Covers real-time visualization, high scores, and the automated "Director Cam".

### Development
- [US07: API Specification](user_stories/US07_api_specification.md) - Mandates a formal OpenAPI definition with separated namespaces.
- [US08: Fair Play and Rate Limiting](user_stories/US08_rate_limiting.md) - Ensures server stability and competitive fairness through request throttling and flood protection.

### Administration
- [US09: Server Administration](user_stories/US09_administration.md) - Provides tools for world regeneration, player resets, and mode switching.
- [US10: Session Management and Cleanup](user_stories/US10_session_cleanup.md) - Ensures graceful departures and cleanup of inactive bot sessions.

## Project Context and Design
- [Stakeholders](stakeholders.md) - Identification of primary users (Players, Spectators) and their interests.
- [Game Rules and Physics](game_rules.md) - Technical specifications including coordinates, gravity, physics constants, and AABB collision math.
- [Technical Constraints](technical_constraints.md) - Language, concurrency limits, persistence, protocol, and API architecture overview.
- [Server Configuration](configuration.md) - Full `config.toml` reference, environment variable overrides, startup validation, and live-reload behaviour.
- [Architecture: Component Diagram](architecture/component_diagram.md) - Visual representation of the Server, Bot Ecosystem, and Web Frontend packages.
- [Architecture: Sequence Diagram](architecture/sequence_diagram.md) - Detailed flow of interactions for Account Management and Bot Gameplay cycles.
- [Architecture: Data Model](architecture/data_model.md) - Database schema (SQLite) for users, sessions, and high scores, plus in-memory runtime structures.
- [Architecture: Security Design](architecture/security_design.md) - Details on rate limiting (Token Bucket), authentication (JWT/API Keys), and flood protection.
- [Frontend Design](frontend_design.md) - Industrial/worn visual theme: color tokens, typography, surface textures, button animations, spectate canvas (platforms as metal grating, bots as industrial cubes with visible internals — cog, glowing core, spring coils — through a recessed viewport).
- [Architecture: WebSocket Protocol](architecture/websocket_protocol.md) - Full WebSocket stream specification: connection lifecycle, all event types, ping/pong, close codes, and reconnection behaviour.
- [API Namespace Overview](api_endpoints.md) - Summary of the four API namespaces and their authentication requirements.
- [OpenAPI Specification](openapi.yaml) - Formal YAML definition of the REST API (OpenAPI 3.0.3); authoritative source of truth for all endpoints.
- [Testing Strategy](testing_strategy.md) - Test pyramid, physics engine unit test cases (gravity, AABB collision, bounce deceleration, state transitions, death/reset), and API integration test cases for all four namespaces.

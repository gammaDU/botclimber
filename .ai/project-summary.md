# Botclimber — Project Summary
_Last updated: 2026-05-01_

## What Is This Project?

Botclimber is a multiplayer bot-programming game where players write bots that compete to climb as high as possible on a procedurally generated platform world. Bots interact via a REST/WebSocket API; humans watch via a browser-based spectator frontend.

## Tech Stack

Rust (Axum 0.8, tokio) backend with SQLite (`rusqlite`), JWT + Argon2 auth, and an Angular 21 frontend. Rate limiting via `governor`, config via TOML.

## Code State

The project is in **early/mid development**. Core infrastructure is in place (auth, config, DB, game loop, WebSocket broadcast), but several API endpoints are stubs or incomplete:

- `bot/join` does not yet resolve the player from the API key — this blocks `bot/state`, `bot/statistics`, and `bot/leave`
- `admin/ratelimits` accepts requests but does not update limiters at runtime
- `GET /user/profile` returns only partial data
- Per-user rate limiting is configured but not wired up
- World seed generation is not implemented

Physics (60 Hz tick, gravity, AABB, bounce schedule) is implemented and unit-tested. Auth (JWT + API key) is functional. The Angular frontend has Leaderboard, Spectate, and Account views.

## Key Docs

See `docs/requirements.md` for an index of available documentation.

## How to Run

```powershell
cargo run --bin migrate -- botclimber.db   # apply DB migrations
mise run frontend-build                    # build Angular frontend
cargo run                                  # start server at http://localhost:8080
```

## Testing

Physics unit tests in `src/game/physics.rs`. Run with `cargo test`. Integration/API tests not yet written.

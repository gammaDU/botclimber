# Botclimber: Technical Constraints

## Technical Requirements
- **Language**: Rust (Edition 2024)
- **Max Concurrent Players**: 50
- **Persistence**: High scores and user accounts (SQLite)
- **Protocol**: REST API (JSON) + WebSockets (State Streaming)
- **Base Path**: `/api/v1`

## API Architecture
The API is separated into distinct namespaces under the `/api/v1` base path to distinguish between bot interaction, frontend management, and public system data:

- **Bot API** (`/api/v1/bot/`): Public endpoints used by players to control their bots (Joining, Jumping, State).
- **Frontend API** (`/api/v1/user/`): Endpoints used by the web interface for account management and authentication.
- **Admin API** (`/api/v1/admin/`): Administrative endpoints for server management (Resetting world/players, mode switching).
- **System API** (`/api/v1/system/`): Shared endpoints for real-time data (Leaderboards, WebSocket Stream).

For the full endpoint listing see [api_endpoints.md](api_endpoints.md) and [openapi.yaml](openapi.yaml).

## Backend Technical Constraints

### Technology Stack
- **Language**: Rust (Edition 2024)
- **HTTP Framework**: [Axum](https://github.com/tokio-rs/axum) (type-safe, async, built on Tokio — integrates natively with the Tokio ecosystem)
- **Async Runtime**: [Tokio](https://tokio.rs/) (multi-threaded, work-stealing scheduler; handles concurrent bot connections and WebSocket streams)
- **WebSocket**: [`tokio-tungstenite`](https://github.com/snapview/tokio-tungstenite) via Axum's built-in WebSocket upgrade support
- **Database**: [SQLite](https://www.sqlite.org/) via [`rusqlite`](https://github.com/rusqlite/rusqlite) (embedded, zero-configuration, sufficient for ≤ 50 concurrent players)
- **Authentication**: [`jsonwebtoken`](https://github.com/Keats/jsonwebtoken) for JWT signing/verification (HMAC SHA-256); API keys generated with [`rand`](https://github.com/rust-random/rand) (CSPRNG)
- **Password Hashing**: [`argon2`](https://github.com/RustCrypto/password-hashes/tree/master/argon2) (Argon2id variant; memory-hard, resistant to GPU cracking)
- **Serialisation**: [`serde`](https://serde.rs/) + [`serde_json`](https://github.com/serde-rs/json) for all JSON request/response bodies
- **Static File Serving**: [`tower-http`](https://github.com/tower-rs/tower-http) `ServeDir` layer — serves the compiled `frontend/dist/` folder at `/`
- **Configuration**: [`config`](https://github.com/mehcode/config-rs) crate reading from a `config.toml` file and environment variable overrides
- **Rate Limiting**: [`governor`](https://github.com/antifuchs/governor) (GCRA / Token Bucket; keyed `DashMap`-backed limiter — one bucket per API key, one global bucket per IP; no external cache required)

### Rationale
| Concern | Decision |
|---|---|
| Async concurrency | Tokio + Axum handle up to 50 simultaneous bot connections and WebSocket streams without blocking threads |
| Type safety | Axum's extractor model enforces request validation at compile time, reducing runtime panics |
| Persistence | SQLite via `rusqlite` requires no separate database process; the single `.db` file is portable and sufficient for the load profile |
| Password security | Argon2id is the current OWASP-recommended algorithm; `argon2` crate is a pure-Rust implementation with no C dependency |
| JWT | `jsonwebtoken` is the de-facto standard Rust JWT crate; supports HMAC SHA-256 and expiry validation out of the box |
| API key entropy | `rand::thread_rng` uses the OS CSPRNG (via `getrandom`); keys are base64url-encoded 32-byte values |
| Static serving | `tower-http` `ServeDir` integrates directly into the Axum router — no separate web server (nginx/caddy) required in development or production |
| Serialisation | `serde` + `serde_json` are the Rust ecosystem standard; zero-copy deserialisation where possible |
| Rate limiting | `governor` implements GCRA (Token Bucket equivalent) natively in async Rust; `DashMap`-keyed limiters require no locks beyond the map's own sharding |

### Constraints
- The server **must** compile and run as a single self-contained binary; no external runtime dependencies (no JVM, no Node.js, no separate database server).
- All game state **must** be held in memory (Tokio `RwLock`-protected shared state); only high scores and user accounts are persisted to SQLite.
- The SQLite database file path **must** be configurable via `config.toml` and overridable by the `BOTCLIMBER_DATABASE_PATH` environment variable.
- The HTTP server **must** bind to a configurable host and port (default `0.0.0.0:8080`), overridable via `BOTCLIMBER_SERVER_BIND` environment variable.
- JWT secrets **must** be loaded from the environment variable `BOTCLIMBER_AUTH_JWT_SECRET` and **must never** be hardcoded or committed to source control.
- Rate limiting **must** be implemented as a Tokio-compatible in-process Token Bucket (one bucket per API key, one global bucket per IP); no external cache (Redis) is required at this scale.
- The WebSocket game stream endpoint **must** broadcast state updates to all connected spectators using a Tokio `broadcast` channel; individual bot endpoints are independent request/response.
- All blocking SQLite calls **must** be executed via `tokio::task::spawn_blocking` to avoid blocking the async executor.
- The server **must** gracefully handle shutdown (SIGTERM/SIGINT) by closing all WebSocket connections and flushing any pending writes before exit.
- Minimum Rust toolchain: **stable** channel, version ≥ 1.80 (required for Rust Edition 2024 features used in the codebase).

## Frontend Technical Constraints

### Technology Stack
- **Framework**: [Svelte](https://svelte.dev/) (compiled, no virtual DOM — minimal bundle size, suitable for serving as static files from the Rust backend)
- **Build Tool**: [Vite](https://vitejs.dev/) (fast HMR in development; produces a single optimised static bundle for production)
- **Language**: TypeScript (strict mode)
- **Styling**: Plain CSS with CSS custom properties (no external UI framework dependency)
- **WebSocket Client**: Native browser `WebSocket` API (no additional library required)
- **HTTP Client**: Native browser `fetch` API (no Axios or similar)
- **Rendering**: HTML `<canvas>` 2D Context API for the game view (no WebGL or third-party game engine); reactive SVG or DOM elements for static UI panels (leaderboard, account)

### Rationale
| Concern | Decision |
|---|---|
| Bundle size | Svelte compiles to vanilla JS — no runtime framework shipped to the client |
| Serving | The Rust server serves the compiled `dist/` folder as static files; no separate Node.js server in production |
| Dependency footprint | Only `svelte`, `vite`, and TypeScript as dev dependencies; zero runtime npm dependencies |
| Real-time updates | Native `WebSocket` connects to `/api/v1/system/stream`; no polling required |
| Auth token handling | JWT stored in `sessionStorage` (cleared on tab close); API keys displayed once and never stored client-side |
| Game rendering | Canvas 2D drives a `requestAnimationFrame` loop decoupled from the WebSocket message rate; camera offset handles the infinite tower viewport |

### Constraints
- The frontend **must not** require a separate server process in production; it is served as static assets by the Rust backend.
- All API calls **must** go through the same origin to avoid CORS issues (no external CDN calls at runtime).
- The compiled output **must** be a single `index.html` + hashed asset files, placed in a `frontend/dist/` directory that the Rust server mounts at `/`.
- The frontend **must** degrade gracefully when the WebSocket connection is lost: display a reconnecting indicator and resume automatically.
- The game view **must** use an HTML `<canvas>` element driven by a `requestAnimationFrame` loop; no WebGL, no third-party rendering or game-engine library is permitted.
- The canvas render loop **must** apply a camera transform (Y-axis offset) so the viewport follows the tracked player up the infinite tower; the Director Cam switches the tracked player automatically unless overridden by the spectator.
- No client-side routing library is required; the application is a single-page with tab-based navigation (Spectator view, Leaderboard, Account).
- Minimum supported browsers: the two most recent stable versions of Chrome, Firefox, and Safari (no IE11 support).
- The frontend source lives in `frontend/` at the project root; it is built separately (`npm run build`) and its `dist/` output is embedded or served by the Rust binary.

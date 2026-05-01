# Botclimber Frontend

Angular 17 single-page application for the Botclimber game server.

## Prerequisites

- [Node.js](https://nodejs.org/) 18+
- Angular CLI: `npm install -g @angular/cli`

## Setup

```bash
cd frontend
npm install
```

## Development

Start the Angular dev server with a proxy to the Rust backend (default `http://localhost:3000`):

```bash
npm start
# → http://localhost:4200
```

The proxy config (`proxy.conf.json`) forwards all `/api` requests to the backend, so
you can run `cargo run` and `npm start` side-by-side without CORS issues.

## Production build

```bash
npm run build
# Output → frontend/dist/
```

The Rust server serves `frontend/dist/` as static files via `tower-http`'s `ServeDir`,
so after building, simply start the backend and navigate to its address.

## Project structure

```
frontend/
├── src/
│   ├── main.ts                        # Bootstrap
│   ├── index.html
│   ├── styles.css                     # Global styles
│   └── app/
│       ├── app.component.ts           # Shell + nav
│       ├── app.config.ts              # Providers (router, HTTP + auth interceptor)
│       ├── app.routes.ts              # Lazy-loaded routes
│       ├── services/
│       │   ├── api.service.ts         # Leaderboard / snapshot / players
│       │   ├── auth.service.ts        # Register / login / profile / API key
│       │   ├── auth.interceptor.ts    # Attaches JWT to every request
│       │   └── game-stream.service.ts # WebSocket live game events
│       ├── leaderboard/
│       │   └── leaderboard.component.ts
│       ├── spectate/
│       │   └── spectate.component.ts  # Canvas renderer + player list
│       └── account/
│           └── account.component.ts   # Login / register / profile panel
├── angular.json
├── tsconfig.json
├── tsconfig.app.json
├── proxy.conf.json
└── package.json
```

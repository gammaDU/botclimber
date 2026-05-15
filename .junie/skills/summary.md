# Project Summary Skill

## Purpose
This is a **read-and-write snapshot skill** — it reads the entire codebase and produces a comprehensive, up-to-date summary of the application's current state.
Its output overwrites `.ai/project-summary.md` with accurate, current information.

## Process

1. **Read all source files**
   - backend in `src/`
   - frontend in `frontend/src/app/` — all components, services, routes

2. **Read all documentation sources**
   - `docs/requirements.md` — index of available docs
   - Any other files referenced there

3. **Identify the current state of each area**
   - Which endpoints are fully implemented, stubs, or missing
   - Which TODOs remain in the code
   - Which tests exist and where
   - Any known gaps between docs and implementation

4. **Write the summary**
   - Save to `.ai/project-summary.md`, overwriting any outdated content
   - Include the date of the last update at the top
   - Use the Report Format below

## Report Format

```markdown
# Botclimber – Project Summary

_Last updated: YYYY-MM-DD_

## What Is This Project?

<one-paragraph description of the game and its purpose>

## Tech Stack

- **Backend:** <runtime, framework, DB, auth, config>
- **Frontend:** <framework, how it is served>
- **Tooling:** <task runner, logging>

## ⚠️ Known Gaps / TODOs

> These are the items that are **not yet implemented** or **incomplete**. List
> every stub, missing feature, and open TODO found in the code.

| Area | Item | Status | Details |
|------|------|--------|---------|
| `<area>` | `<item>` | ⚠️ Stub / ❌ Missing | <what is missing and what would be needed to complete it> |

...

_If no gaps exist, write: "No known gaps — all documented features are
implemented."_

## Frontend

Angular SPA served from `frontend/dist/browser/`. <N> pages, <N> services.

### Pages / Components

| Component | Route | Description |
|-----------|-------|-------------|
| `<Name>Component` | `/<route>` | <what it shows and which API calls it makes> |

### Services

| Service | Purpose |
|---------|--------|
| `<Name>Service` | <what it does, which endpoints it calls> |

### ⚠️ Frontend–Backend Mismatches

| Area | Frontend calls | Backend has | Status |
|------|---------------|-------------|--------|
| <area> | `<method> <path>` | `<method> <path>` or ❌ Not registered | ⚠️ / ❌ |

## Architecture Overview
describe the architecture of the backend:

### Bot (`/api/v1/bot`) — authenticated via `X-Api-Key` header

| Method | Path | Status | Notes |
|--------|------|--------|-------|
| POST | `/join` | ✅ / ⚠️ / ❌ | <notes> |

...

### User (`/api/v1/user`) — authenticated via JWT

| Method | Path | Status |
|--------|------|--------|

...

### Admin (`/api/v1/admin`) — JWT with admin role

| Method | Path | Status | Notes |
|--------|------|--------|-------|

...

### System (`/api/v1/system`)

| Method | Path | Status |
|--------|------|--------|

...

## WebSocket Events

- `<event_name>` — `{<fields>}`
  ...

## Physics Constants

| Constant | Value |
|----------|-------|
| `<NAME>` | <value> |

...

## Testing

- <what tests exist, where, what they cover>
- Run with: `mise run test`

## How to Run

\`\`\`powershell
<commands to migrate, build frontend, start server>
\`\`\`

## Key Docs

See `docs/requirements.md` for an index of all available documentation.
```

## Status Classification
- ✅ Implemented — handler is complete and correct
- ⚠️ Stub / Partial — handler exists but returns placeholder or is incomplete
- ❌ Missing — route is not registered or handler does not exist

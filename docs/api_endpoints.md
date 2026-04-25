# Botclimber: API Namespace Overview

The API is organised into four distinct namespaces under the `/api/v1` base path. For the full technical specification of every endpoint, request/response schema, and authentication requirements, refer to [openapi.yaml](openapi.yaml).

| Namespace | Base Path | Auth | Purpose |
|---|---|---|---|
| **Bot API** | `/api/v1/bot/` | `X-API-Key` | Bot gameplay commands (join, jump, state, leave) |
| **Frontend API** | `/api/v1/user/` | Bearer JWT | Account management and API key generation |
| **Admin API** | `/api/v1/admin/` | Bearer JWT (admin) | Server management, mode switching, world/player resets |
| **System API** | `/api/v1/system/` | None (public) | Leaderboards, active players, WebSocket game stream |

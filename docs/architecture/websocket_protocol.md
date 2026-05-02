# Botclimber: WebSocket Protocol Specification

This document defines the WebSocket game stream protocol exposed at `/api/v1/system/stream`. It covers the connection lifecycle, message envelope, all event types, and reconnection behaviour.

---

## 1. Connection

### 1.1 Endpoint

```
ws://<host>/api/v1/system/stream
wss://<host>/api/v1/system/stream   ← required in production (TLS)
```

### 1.2 Upgrade Handshake

The client initiates a standard HTTP → WebSocket upgrade:

```
GET /api/v1/system/stream HTTP/1.1
Host: <host>
Upgrade: websocket
Connection: Upgrade
Sec-WebSocket-Key: <base64-nonce>
Sec-WebSocket-Version: 13
```

**Authentication is not required** to connect to the stream — it is a public, read-only feed. The server responds with `101 Switching Protocols` on success.

### 1.3 Subprotocol

No `Sec-WebSocket-Protocol` subprotocol header is used. All messages are UTF-8 encoded JSON text frames.

---

## 2. Message Envelope

Every message sent by the server is a JSON text frame conforming to the following envelope:

```json
{
  "type": "<event_type>",
  "timestamp": "<ISO-8601 UTC string>",
  "data": { }
}
```

| Field | Type | Description |
|---|---|---|
| `type` | `string` | Identifies the event type (see Section 3) |
| `timestamp` | `string` | ISO-8601 UTC timestamp of when the event was generated on the server (e.g. `"2024-06-01T12:00:00.123Z"`) |
| `data` | `object` | Event-specific payload (see Section 3) |

The client **must** ignore unknown `type` values to remain forward-compatible.

---

## 3. Event Types

### 3.1 `position_update`

Emitted whenever one or more players' positions or states change (on every physics tick).

```json
{
  "type": "position_update",
  "timestamp": "2024-06-01T12:00:00.123Z",
  "data": {
    "player_id": "string",
    "x": 12.5,
    "y": 340.0,
    "vx": 0.0,
    "vy": -4.2,
    "state": "JUMPING",
  }
}
```

| Field | Type | Description |
|---|---|---|
| `player_id` | `string` | Unique identifier of the player figure |
| `x` | `number` | Current X position (world units) |
| `y` | `number` | Current Y position (world units); increases upward |
| `vx` | `number` | Current horizontal velocity (world units/tick) |
| `vy` | `number` | Current vertical velocity (world units/tick) |
| `state` | `"LANDED"` \| `"JUMPING"` | Current movement state (see [game_rules.md](../game_rules.md)) |

### 3.2 `platform_spawn`

Emitted when a new platform is generated as a player climbs higher.

```json
{
  "type": "platform_spawn",
  "timestamp": "2024-06-01T12:00:00.456Z",
  "data": {
    "platform_id": "string",
    "rect": {
      "x": 50.0,
      "y": 400.0,
      "width": 80.0,
      "height": 10.0
    }
  }
}
```

| Field | Type | Description |
|---|---|---|
| `platform_id` | `string` | Unique identifier of the platform |
| `rect.x` | `number` | Left edge X position (world units) |
| `rect.y` | `number` | Bottom edge Y position (world units) |
| `rect.width` | `number` | Platform width (world units) |
| `rect.height` | `number` | Platform height (world units) |

### 3.3 `player_joined`

Emitted when a new player joins the active session.

```json
{
  "type": "player_joined",
  "timestamp": "2024-06-01T12:00:01.000Z",
  "data": {
    "player_id": "string",
    "name": "string",
    "game_mode": "Checkpoint"
  }
}
```

| Field | Type | Description |
|---|---|---|
| `player_id` | `string` | Unique identifier of the new player |
| `name` | `string` | Display name of the bot/player |
| `game_mode` | `"Checkpoint"` \| `"Hardcore"` | Game mode the player joined with (see US12) |

### 3.4 `player_left`

Emitted when a player leaves or is removed from the session.

```json
{
  "type": "player_left",
  "timestamp": "2024-06-01T12:00:02.000Z",
  "data": {
    "player_id": "string",
    "reason": "voluntary"
  }
}
```

| Field | Type | Description |
|---|---|---|
| `player_id` | `string` | Unique identifier of the departing player |
| `reason` | `"voluntary"` \| `"timeout"` \| `"admin_reset"` | Why the player was removed |

### 3.5 `world_reset`

Emitted when an administrator resets the world or all players. Clients should clear their local game state and treat the next `position_update` and `platform_spawn` events as a fresh start.

```json
{
  "type": "world_reset",
  "timestamp": "2024-06-01T12:01:00.000Z",
  "data": {}
}
```

---

## 4. Server → Client Only

The stream is **unidirectional**: the server pushes events; the client never sends messages. Any text or binary frame sent by the client will be ignored by the server.

---

## 5. Frontend Bootstrap Sequence

The frontend **must** open the WebSocket and begin buffering incoming events *before* fetching the snapshot. This avoids a race condition where a `platform_spawn` (or `player_joined`) event is emitted by the server after the snapshot is taken but before the WebSocket is open, which would leave the client with a permanently stale platform map.

```
Frontend loads
     │
     ▼
Open WebSocket /api/v1/system/stream   ← start buffering ALL incoming events immediately
     │
     ▼
GET /api/v1/system/snapshot            ← all player positions + all visible platforms
     │
     ▼
Apply snapshot to local state map      ← seed players and platforms
     │
     ▼
Replay buffered events on top          ← events that arrived during the HTTP fetch
     │                                    (position_update, platform_spawn, player_joined,
     │                                     player_left, world_reset handled normally)
     ▼
Start requestAnimationFrame render loop (canvas draws from local state map)
     │
     ├── Apply incoming position_update events  (overwrite snapshot player entries)
     ├── Apply incoming platform_spawn events   (add new platforms to map)
     ├── Apply player_joined / player_left      (add/remove from player map)
     └── Apply world_reset                      (clear both maps, re-fetch snapshot)
```

**Why this order matters:** The snapshot carries a `timestamp` field. Any buffered event whose `timestamp` is ≤ the snapshot timestamp is already reflected in the snapshot and **must be discarded**. Any buffered event whose `timestamp` is > the snapshot timestamp represents state that occurred after the snapshot and **must be applied** in order. This guarantees exactly-once application of every event with no gaps and no duplicates.

The render loop **must not** start until the snapshot has been received, parsed, and all buffered events have been replayed. Display a "Loading…" indicator until that point.

---

## 6. Connection Lifecycle

```
Client                          Server
  |                               |
  |--- HTTP Upgrade Request ----→ |
  |←-- 101 Switching Protocols -- |
  |                               |
  |←-- position_update ---------- |  (continuous, every physics tick)
  |←-- platform_spawn ----------- |  (on new platform generation)
  |←-- player_joined ------------ |  (on join)
  |←-- player_left -------------- |  (on leave/timeout)
  |←-- world_reset -------------- |  (on admin reset)
  |                               |
  |--- Close Frame (1000) ------→ |  (client voluntary disconnect)
  |←-- Close Frame (1000) ------- |
```

### 6.1 Ping / Pong

The server sends a WebSocket **Ping** frame every **30 seconds** to detect stale connections. The client browser automatically responds with a **Pong** frame (per the WebSocket spec). If no Pong is received within **10 seconds** of a Ping, the server closes the connection with code `1001 Going Away`.

### 6.2 Close Codes

| Code | Meaning |
|---|---|
| `1000` | Normal closure (client or server initiated graceful shutdown) |
| `1001` | Going Away — server is shutting down or client timed out |
| `1011` | Internal Error — unexpected server-side error; client should reconnect |

---

## 7. Reconnection Behaviour

The frontend **must** implement automatic reconnection with exponential backoff:

| Attempt | Delay before retry |
|---|---|
| 1st | 1 second |
| 2nd | 2 seconds |
| 3rd | 4 seconds |
| 4th+ | 8 seconds (cap) |

During reconnection the frontend **must** display a visible "Reconnecting…" indicator in the game view. On successful reconnect, the client **must** follow the same bootstrap sequence as on initial load (Section 5): open the WebSocket first and begin buffering events, then re-fetch `GET /api/v1/system/snapshot`, apply the snapshot, replay any buffered events whose `timestamp` is > the snapshot timestamp, and only then resume the render loop.

---

## 8. Broadcast Semantics

The server maintains a single Tokio `broadcast` channel (see [technical_constraints.md](../technical_constraints.md)). All connected spectators receive every event. There is no per-client filtering or subscription mechanism — clients receive the full world state and may choose which players to render.

---

## 9. Rate and Volume Expectations

| Metric | Expected value |
|---|---|
| Max concurrent spectator connections | Unbounded (limited only by server resources) |
| `position_update` frequency | One message per player per physics tick |
| `platform_spawn` frequency | Low; only when the world frontier advances |
| Max players generating updates | 50 (see technical constraints) |

At 50 players and a typical tick rate, clients should expect up to **50 `position_update` messages per tick**. The frontend render loop is decoupled from the WebSocket message rate via `requestAnimationFrame` — see [technical_constraints.md](../technical_constraints.md).

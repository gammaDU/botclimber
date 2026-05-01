/**
 * stream.js — WebSocket game stream client.
 *
 * Implements the bootstrap sequence from the WebSocket protocol spec:
 *   1. Open WebSocket and buffer all incoming events immediately.
 *   2. Fetch the HTTP snapshot.
 *   3. Apply the snapshot to local state.
 *   4. Replay buffered events whose timestamp > snapshot timestamp.
 *   5. Start the render loop.
 *
 * Exports a single `GameStream` class.
 */

import { getSnapshot } from './api.js';

const WS_PATH = '/api/v1/system/stream';
const RECONNECT_DELAY_MS = 3000;

export class GameStream {
  /**
   * @param {object} handlers
   * @param {(state: GameState) => void} handlers.onReady   - called once bootstrap is complete
   * @param {(event: object) => void}   handlers.onEvent    - called for each live event after bootstrap
   * @param {(status: string) => void}  handlers.onStatus   - 'connecting' | 'connected' | 'disconnected'
   */
  constructor(handlers) {
    this._onReady  = handlers.onReady  ?? (() => {});
    this._onEvent  = handlers.onEvent  ?? (() => {});
    this._onStatus = handlers.onStatus ?? (() => {});

    /** @type {GameState} */
    this.state = { players: {}, platforms: [], snapshotTimestamp: null };

    this._ws = null;
    this._buffer = [];       // events buffered before snapshot arrives
    this._bootstrapped = false;
    this._stopped = false;
  }

  /** Open the stream (or reconnect). */
  connect() {
    this._stopped = false;
    this._open();
  }

  /** Permanently close the stream (no reconnect). */
  disconnect() {
    this._stopped = true;
    if (this._ws) this._ws.close();
  }

  // ── Private ────────────────────────────────────────────────────────────────

  _open() {
    this._bootstrapped = false;
    this._buffer = [];
    this._onStatus('connecting');

    const proto = location.protocol === 'https:' ? 'wss' : 'ws';
    const url   = `${proto}://${location.host}${WS_PATH}`;
    const ws    = new WebSocket(url);
    this._ws    = ws;

    ws.addEventListener('open', () => {
      this._onStatus('connected');
      this._bootstrap();
    });

    ws.addEventListener('message', (ev) => {
      let msg;
      try { msg = JSON.parse(ev.data); } catch { return; }
      if (this._bootstrapped) {
        this._applyEvent(msg);
        this._onEvent(msg);
      } else {
        this._buffer.push(msg);
      }
    });

    ws.addEventListener('close', () => {
      this._onStatus('disconnected');
      if (!this._stopped) {
        setTimeout(() => this._open(), RECONNECT_DELAY_MS);
      }
    });

    ws.addEventListener('error', () => {
      ws.close();
    });
  }

  async _bootstrap() {
    let snapshot;
    try {
      snapshot = await getSnapshot();
    } catch (err) {
      console.error('[stream] snapshot fetch failed:', err);
      this._ws?.close();
      return;
    }

    const snapTs = new Date(snapshot.timestamp).getTime();
    this.state.snapshotTimestamp = snapTs;

    // Apply snapshot
    this.state.players   = {};
    this.state.platforms = [];

    for (const p of snapshot.players ?? []) {
      this.state.players[p.id] = p;
    }
    for (const pl of snapshot.platforms ?? []) {
      this.state.platforms.push(pl);
    }

    // Replay buffered events that arrived after the snapshot
    for (const ev of this._buffer) {
      if (new Date(ev.timestamp).getTime() > snapTs) {
        this._applyEvent(ev);
      }
    }
    this._buffer = [];
    this._bootstrapped = true;

    this._onReady(this.state);
  }

  /** Mutate local state according to event type. */
  _applyEvent(ev) {
    switch (ev.type) {
      case 'position_update': {
        const d = ev.data;
        if (!this.state.players[d.player_id]) {
          this.state.players[d.player_id] = { id: d.player_id, username: d.username };
        }
        Object.assign(this.state.players[d.player_id], {
          x: d.x, y: d.y,
          velocity_x: d.velocity_x,
          velocity_y: d.velocity_y,
          state: d.state,
          height: d.height,
        });
        break;
      }
      case 'platform_spawn': {
        this.state.platforms.push(ev.data);
        break;
      }
      case 'player_joined': {
        const d = ev.data;
        this.state.players[d.player_id] = { id: d.player_id, username: d.username, x: 0, y: 0, height: 0 };
        break;
      }
      case 'player_left': {
        delete this.state.players[ev.data.player_id];
        break;
      }
      case 'world_reset': {
        this.state.players   = {};
        this.state.platforms = [];
        break;
      }
      default:
        // Forward-compatible: ignore unknown event types.
        break;
    }
  }
}

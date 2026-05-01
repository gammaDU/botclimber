/**
 * renderer.js — Canvas-based game renderer with Director Cam support.
 *
 * The render loop is driven by requestAnimationFrame and reads the live
 * GameStream state object on every frame — it never blocks on WebSocket I/O.
 *
 * Director Cam logic: automatically follows the player with the greatest
 * current height. The spectator can override by clicking a player in the
 * session list; calling enableDirectorCam() restores automatic tracking.
 */

// ── Constants ─────────────────────────────────────────────────────────────────

const PLATFORM_COLOR  = '#4a9eff';
const PLAYER_COLOR    = '#e94560';
const FOLLOWED_COLOR  = '#ffd700';
const PLATFORM_H      = 8;   // px drawn height for a platform
const PLAYER_W        = 16;
const PLAYER_H        = 20;
const PIXELS_PER_M    = 8;   // world metres → canvas pixels

// ── Renderer class ────────────────────────────────────────────────────────────

export class Renderer {
  /**
   * @param {HTMLCanvasElement} canvas
   * @param {object} callbacks
   * @param {(playerId: string|null) => void} callbacks.onFollowChange
   */
  constructor(canvas, callbacks = {}) {
    this._canvas  = canvas;
    this._ctx     = canvas.getContext('2d');
    this._state   = null;   // reference to GameStream.state (set via setGameState)
    this._rafId   = null;

    this._directorCam  = true;
    this._followedId   = null;   // player id being followed (null = director cam)
    this._cameraY      = 0;      // world-Y of the bottom of the viewport (metres)

    this._onFollowChange = callbacks.onFollowChange ?? (() => {});
  }

  /** Attach the live state object from GameStream. */
  setGameState(state) { this._state = state; }

  /** Start the render loop. */
  start() {
    if (this._rafId !== null) return;
    this._loop();
  }

  /** Stop the render loop. */
  stop() {
    if (this._rafId !== null) cancelAnimationFrame(this._rafId);
    this._rafId = null;
  }

  enableDirectorCam() {
    this._directorCam = true;
    this._followedId  = null;
    this._onFollowChange(null);
  }

  followPlayer(playerId) {
    this._directorCam = false;
    this._followedId  = playerId;
    this._onFollowChange(playerId);
  }

  get followedId()   { return this._followedId; }
  get directorCam()  { return this._directorCam; }

  // ── Private ─────────────────────────────────────────────────────────────────

  _loop() {
    this._rafId = requestAnimationFrame(() => {
      this._draw();
      this._loop();
    });
  }

  _draw() {
    const ctx    = this._ctx;
    const W      = this._canvas.width;
    const H      = this._canvas.height;
    const state  = this._state;

    ctx.clearRect(0, 0, W, H);

    if (!state) {
      this._drawOverlay(ctx, W, H, 'Loading…');
      return;
    }

    const players   = Object.values(state.players);
    const platforms = state.platforms;

    // ── Director cam: track highest player ──────────────────────────────────
    if (this._directorCam) {
      let best = null;
      for (const p of players) {
        if (best === null || (p.height ?? 0) > (best.height ?? 0)) best = p;
      }
      this._followedId = best?.id ?? null;
    }

    const followed = this._followedId ? state.players[this._followedId] : null;

    // ── Camera: keep followed player vertically centred ──────────────────────
    if (followed) {
      const targetCamY = (followed.y ?? 0) - (H / 2) / PIXELS_PER_M;
      // Smooth camera: lerp 10 % per frame
      this._cameraY += (targetCamY - this._cameraY) * 0.1;
    }

    const camY = this._cameraY;

    // ── Background ───────────────────────────────────────────────────────────
    ctx.fillStyle = '#111122';
    ctx.fillRect(0, 0, W, H);

    // ── Platforms ────────────────────────────────────────────────────────────
    ctx.fillStyle = PLATFORM_COLOR;
    for (const pl of platforms) {
      const sx = pl.x * PIXELS_PER_M;
      const sy = H - (pl.y - camY) * PIXELS_PER_M;
      ctx.fillRect(sx, sy - PLATFORM_H, pl.width * PIXELS_PER_M, PLATFORM_H);
    }

    // ── Players ──────────────────────────────────────────────────────────────
    for (const p of players) {
      const isFollowed = p.id === this._followedId;
      const sx = (p.x ?? 0) * PIXELS_PER_M - PLAYER_W / 2;
      const sy = H - ((p.y ?? 0) - camY) * PIXELS_PER_M - PLAYER_H;

      ctx.fillStyle = isFollowed ? FOLLOWED_COLOR : PLAYER_COLOR;
      ctx.fillRect(sx, sy, PLAYER_W, PLAYER_H);

      // Username label
      ctx.fillStyle    = '#fff';
      ctx.font         = '10px sans-serif';
      ctx.textAlign    = 'center';
      ctx.fillText(p.username ?? p.id, sx + PLAYER_W / 2, sy - 3);
    }

    // ── Height ruler (right edge) ─────────────────────────────────────────────
    ctx.fillStyle   = 'rgba(255,255,255,0.15)';
    ctx.font        = '10px sans-serif';
    ctx.textAlign   = 'right';
    const stepM = 10;
    const firstMark = Math.ceil(camY / stepM) * stepM;
    for (let m = firstMark; m < camY + H / PIXELS_PER_M; m += stepM) {
      const sy = H - (m - camY) * PIXELS_PER_M;
      ctx.fillRect(W - 20, sy, 20, 1);
      ctx.fillText(`${m}m`, W - 22, sy - 2);
    }
  }

  _drawOverlay(ctx, W, H, text) {
    ctx.fillStyle = 'rgba(0,0,0,0.6)';
    ctx.fillRect(0, 0, W, H);
    ctx.fillStyle   = '#e0e0e0';
    ctx.font        = '18px sans-serif';
    ctx.textAlign   = 'center';
    ctx.fillText(text, W / 2, H / 2);
  }
}

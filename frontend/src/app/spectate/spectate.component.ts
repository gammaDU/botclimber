import {
  Component, OnInit, OnDestroy, ElementRef, ViewChild, AfterViewInit, ChangeDetectionStrategy, ChangeDetectorRef, HostListener,
} from '@angular/core';
import { CommonModule } from '@angular/common';
import { HttpClient } from '@angular/common/http';
import { Subscription } from 'rxjs';
import { GameStreamService, WsEnvelope } from '../services/game-stream.service';

// ---------------------------------------------------------------------------
// Domain types matching websocket_protocol.md
// ---------------------------------------------------------------------------

interface Platform { id: string; rect: { x: number; y: number; width: number; height: number }; }

interface PlayerState {
  player_id: string;
  name: string;
  x: number; y: number;
  vx: number; vy: number;
  state: 'LANDED' | 'JUMPING';
  current_time?: number;
  best_time?: number;
}

interface GameModeConfig {
  fall_mode: 'Checkpoint' | 'Hardcore';
  session_type: 'Race' | 'Timetrial' | 'Infinite';
  stage_type: 'Generated' | 'Fixed';
  fixed_stage_id?: string;
  goal_height?: number;
}

interface SnapshotResponse {
  timestamp: number; // epoch ms
  game_mode: GameModeConfig;
  tick: number;
  players: Array<{
    player_id: string; name: string;
    position: { x: number; y: number };
    velocity: { x: number; y: number };
    state: string;
    current_time: number;
    best_time?: number;
  }>;
  platforms: Array<{ id: string; rect: { x: number; y: number; width: number; height: number } }>;
}

// ---------------------------------------------------------------------------
// Canvas constants
// ---------------------------------------------------------------------------

const CANVAS_W = 640;
const CANVAS_H = 480;
const TOWER_HALF_WIDTH = 10;  // world units, tower spans [-10, 10]
const WORLD_SCALE = CANVAS_W / (TOWER_HALF_WIDTH * 2); // pixels per world unit — fits full tower width

// Bot accent colors cycled per player index
const BOT_COLORS = ['#5a9aaa', '#5a8a6a', '#8a9aaa', '#7a6aaa', '#9a8a5a', '#6a9a7a'];

// Seeded per-platform rust patch positions (deterministic from platform id hash)
function hashCode(s: string): number {
  let h = 0;
  for (let i = 0; i < s.length; i++) { h = (Math.imul(31, h) + s.charCodeAt(i)) | 0; }
  return Math.abs(h);
}

@Component({
  selector: 'app-spectate',
  standalone: true,
  imports: [CommonModule],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
      <div class="view">
          <div id="status-bar">
              @if (status !== 'ready') {
                  <span id="connection-status" [class]="statusClass">{{ statusLabel }}</span>
              } @else {
                  <span style="color:var(--text-muted);font-size:0.85rem">
                    {{ playerList.length }} active player{{ playerList.length !== 1 ? 's' : '' }}
                  </span>
              }
          </div>
          <div id="game-container">
              <canvas #canvas id="game-canvas" [width]="canvasW" [height]="canvasH"
                      (wheel)="onWheel($event)"></canvas>

              <div id="session-panel">
                  @if (currentGameMode) {
                      <div id="game-mode-config">
                          <div class="gmc-row"><span class="gmc-label">Fall</span><span class="gmc-value">{{ currentGameMode.fall_mode }}</span></div>
                          <div class="gmc-row"><span class="gmc-label">Session</span><span class="gmc-value">{{ currentGameMode.session_type }}</span></div>
                          <div class="gmc-row"><span class="gmc-label">Stage</span><span class="gmc-value">{{ currentGameMode.stage_type }}</span></div>
                          @if (currentGameMode.goal_height != null) {
                              <div class="gmc-row"><span class="gmc-label">Goal</span><span class="gmc-value">{{ currentGameMode.goal_height }} m</span></div>
                          }
                          @if (currentGameMode.fixed_stage_id) {
                              <div class="gmc-row"><span class="gmc-label">Stage ID</span><span class="gmc-value">{{ currentGameMode.fixed_stage_id }}</span></div>
                          }
                      </div>
                  }
                  <h2>Active Players</h2>
                  <ul id="session-list">
                      @for (p of playerList; track p.player_id) {
                          <li [class.following]="followId === p.player_id" (click)="follow(p.player_id)" style="flex-direction: column; align-items: stretch;">
                              <div style="display: flex; align-items: center; gap: 6px; width: 100%;">
                                  <span>{{ p.name }}</span>
                                  @if (followId === p.player_id) {
                                      <img src="assets/icons/visibility.svg" class="follow-icon" alt="following" />
                                  }
                              </div>
                              <div style="font-size: 0.8rem; color: var(--text-muted); display: flex; justify-content: space-between; align-items: center; margin-top: 2px;">
                                  <div style="display: flex; gap: 8px;">
                                      <span>{{ p.y | number:'1.1-1' }} m</span>
                                      @if (currentGameMode?.session_type === 'Timetrial') {
                                          <span>{{ p.current_time | number:'1.1-1' }}s</span>
                                      }
                                  </div>
                                  @if (currentGameMode?.session_type === 'Timetrial' && p.best_time != null) {
                                      <span>🏆 {{ p.best_time | number:'1.1-1' }}s</span>
                                  }
                              </div>
                          </li>
                      } @empty {
                          <li style="color:var(--text-muted)">No active players</li>
                      }
                  </ul>

                  <div id="cam-controls">
                      <button [class.active]="!directedCam && followId === null" (click)="follow(null)">Free cam</button>
                      <button [class.active]="directedCam" (click)="toggleDirectedCam()">Directed Cam</button>
                  </div>
              </div>
          </div>
      </div>
  `,
})
export class SpectateComponent implements OnInit, AfterViewInit, OnDestroy {
  @ViewChild('canvas') canvasRef!: ElementRef<HTMLCanvasElement>;

  canvasW = CANVAS_W;
  canvasH = CANVAS_H;

  @HostListener('window:resize')
  onResize(): void {
    this._updateCanvasSize();
  }

  private _updateCanvasSize(): void {
    if (this.canvasRef) {
      const canvas = this.canvasRef.nativeElement;
      const rect = canvas.getBoundingClientRect();
      this.canvasW = CANVAS_W; // Restore fixed width
      this.canvasH = Math.floor(rect.height);
      // We don't need to manually set canvas.width/height because [width] and [height] bindings handle it
    }
  }

  followId: string | null = null;
  directedCam = false;

  status: 'connecting' | 'connected' | 'loading' | 'ready' | 'reconnecting' | 'disconnected' = 'connecting';

  playerList: PlayerState[] = [];
  currentGameMode: GameModeConfig | null = null;

  private platforms = new Map<string, Platform>();
  private players = new Map<string, PlayerState>();
  private cogAngles = new Map<string, number>();
  private playerColorIndex = new Map<string, number>();
  private colorCounter = 0;

  // Directed Cam state
  private botActivity = new Map<string, {
    lastY: number,
    activityScore: number,
    boredom: number,
    lastJumpTime: number,
    lastFollowTime: number
  }>();
  private lastDirectedSwitch = 0;

  private _syncPlayerList(): void {
    this.playerList = Array.from(this.players.values()).sort((a, b) => {
      // Sort by best_time (ascending, lower is better)
      if (a.best_time != null && b.best_time != null) {
        return a.best_time - b.best_time;
      }
      if (a.best_time != null) return -1;
      if (b.best_time != null) return 1;

      // Fallback to height (descending, higher is better)
      return b.y - a.y;
    });
  }

  private _getMaxCamY(): number | null {
    const goal = this.currentGameMode?.goal_height;
    if (goal == null || goal <= 0) return null;

    // The scroll window should be restricted so the top of the viewport
    // doesn't go above goal_height + 10.
    // viewportHeight in world units = canvasH / WORLD_SCALE.
    const viewportHeight = this.canvasH / WORLD_SCALE;
    const maxVisibleY = goal + 10;
    const maxCamY = maxVisibleY - viewportHeight;

    // camY should not be less than 0, even if maxCamY is negative (though unlikely)
    return Math.max(0, maxCamY);
  }

  /** Events buffered before the snapshot arrives. */
  private eventBuffer: WsEnvelope[] = [];
  private snapshotTimestamp = 0;

  private camY = 0;
  private subs = new Subscription();
  private ctx!: CanvasRenderingContext2D;
  private rafId = 0;

  constructor(
    private stream: GameStreamService,
    private http: HttpClient,
    private cdr: ChangeDetectorRef,
  ) {}

  get statusClass(): string { return `status-${this.status}`; }

  get statusLabel(): string {
    return this.status.charAt(0).toUpperCase() + this.status.slice(1);
  }

  get followName(): string {
    return this.players.get(this.followId ?? '')?.name ?? '';
  }

  ngOnInit(): void {
    this.subs.add(this.stream.status$.subscribe(s => {
      if (s === 'reconnecting') {
        // Reset bootstrap state so we redo it on reconnect
        this.status = 'reconnecting';
        this.eventBuffer = [];
      } else if (s === 'connected' && this.status !== 'loading' && this.status !== 'ready') {
        this._fetchSnapshot();
      } else if (s !== 'connected') {
        this.status = s;
      }
      this.cdr.markForCheck();
    }));

    this.subs.add(this.stream.events$.subscribe(e => this._onEvent(e)));

    // Step 1: open WS first (start buffering), then snapshot is fetched on 'connected'
    this.stream.reset();
    this.stream.connect();
  }

  ngAfterViewInit(): void {
    this.ctx = this.canvasRef.nativeElement.getContext('2d')!;
    this._updateCanvasSize();
    this.cdr.detectChanges();
    this._scheduleFrame();
  }

  ngOnDestroy(): void {
    cancelAnimationFrame(this.rafId);
    this.stream.disconnect();
    this.subs.unsubscribe();
  }

  follow(id: string | null): void {
    this.followId = id;
    this.directedCam = false;
  }

  toggleDirectedCam(): void {
    this.directedCam = !this.directedCam;
    if (this.directedCam) {
      this._updateDirectedCam(true);
    }
  }

  onWheel(e: WheelEvent): void {
    if (this.followId !== null) return;
    e.preventDefault();
    // deltaY > 0 = scroll down → move camera down (lower camY)
    this.camY -= e.deltaY / WORLD_SCALE;

    this._clampCamera();
  }

  private _clampCamera(): void {
    // Clamping to 0 (bottom)
    if (this.camY < 0.0001) {
      this.camY = 0;
    }

    // Clamping to goal height + 10 (top)
    const maxCamY = this._getMaxCamY();
    if (maxCamY !== null && this.camY > maxCamY) {
      this.camY = maxCamY;
    }
  }

  // ---------------------------------------------------------------------------
  // Bootstrap: fetch snapshot, replay buffer
  // ---------------------------------------------------------------------------

  private _fetchSnapshot(): void {
    this.status = 'loading';
    this.cdr.markForCheck();
    this.http.get<SnapshotResponse>('/api/v1/system/snapshot').subscribe({
      next: snap => {
        this.snapshotTimestamp = snap.timestamp;

        // Seed state from snapshot
        this.currentGameMode = snap.game_mode ?? null;
        this.platforms.clear();
        for (const p of snap.platforms) {
          this.platforms.set(p.id, p);
        }
        this.players.clear();
        this.playerColorIndex.clear();
        this.colorCounter = 0;
        for (const p of snap.players) {
          this.players.set(p.player_id, {
            player_id: p.player_id,
            name: p.name,
            x: p.position.x,
            y: p.position.y,
            vx: p.velocity.x,
            vy: p.velocity.y,
            state: p.state as 'LANDED' | 'JUMPING',
            current_time: p.current_time,
            best_time: p.best_time,
          });
          if (!this.playerColorIndex.has(p.player_id)) {
            this.playerColorIndex.set(p.player_id, this.colorCounter++ % BOT_COLORS.length);
          }
        }
        this._syncPlayerList();

        // Replay buffered events that arrived after the snapshot
        for (const e of this.eventBuffer) {
          if (new Date(e.timestamp).getTime() > this.snapshotTimestamp) {
            this._applyEvent(e);
          }
        }
        this.eventBuffer = [];
        this.status = 'ready';
        this.cdr.markForCheck();
      },
      error: () => {
        // Retry snapshot after a short delay
        this.status = 'connected';
        this.cdr.markForCheck();
        setTimeout(() => {
          if (this.status === 'connected') this._fetchSnapshot();
        }, 2000);
      },
    });
  }

  // ---------------------------------------------------------------------------
  // Event routing
  // ---------------------------------------------------------------------------

  private _onEvent(e: WsEnvelope): void {
    if (this.status !== 'ready') {
      this.eventBuffer.push(e);
    } else {
      this._applyEvent(e);
    }
  }

  private _applyEvent(e: WsEnvelope): void {
    const d = e.data;
    switch (e.type) {
      case 'position_update': {
        const id = d['player_id'] as string;
        const existing = this.players.get(id);
        const y = d['y'] as number;
        const vy = d['vy'] as number;

        // --- Activity Scoring ---
        let act = this.botActivity.get(id);
        if (!act) {
          act = { lastY: y, activityScore: 0, boredom: 0, lastJumpTime: 0, lastFollowTime: 0 };
          this.botActivity.set(id, act);
        }

        const dy = Math.abs(y - act.lastY);
        // Bonus for movement
        act.activityScore += dy * 2.0;

        // Bonus for jumping (sudden upward velocity)
        if (vy > 5 && (performance.now() - act.lastJumpTime > 500)) {
          act.activityScore += 10;
          act.lastJumpTime = performance.now();
        }

        // Weight by height (favor bots higher up)
        // We'll multiply the final score by a height factor in _updateDirectedCam

        act.lastY = y;
        // -----------------------

        this.players.set(id, {
          player_id: id,
          name: existing?.name ?? id,
          x: d['x'] as number,
          y: y,
          vx: d['vx'] as number,
          vy: vy,
          state: d['state'] as 'LANDED' | 'JUMPING',
          current_time: d['current_time'] as number,
          best_time: d['best_time'] as number,
        });

        if (this.directedCam) {
          this._updateDirectedCam();
        }
        break;
      }
      case 'platform_spawn': {
        const id = d['platform_id'] as string;
        const rect = d['rect'] as Platform['rect'];
        this.platforms.set(id, { id, rect });
        break;
      }
      case 'player_joined': {
        console.log(e, d);
        const id = d['player_id'] as string;
        if (!this.players.has(id)) {
          console.log('player_joined', id);
          this.players.set(id, {
            player_id: id,
            name: d['name'] as string,
            x: 0, y: 0, vx: 0, vy: 0,
            state: 'LANDED',
            current_time: 0,
          });
          this.playerColorIndex.set(id, this.colorCounter++ % BOT_COLORS.length);
        }
        break;
      }
      case 'player_left': {
        console.log(e, d);
        const id = d['player_id'] as string;
        console.log('player_left', id);
        this.players.delete(id);
        this.botActivity.delete(id);
        if (this.followId === id) {
          this.followId = null;
          if (this.directedCam) this._updateDirectedCam(true);
        }
        break;
      }
      case 'world_reset': {
        this.players.clear();
        this.platforms.clear();
        this.botActivity.clear();
        this.currentGameMode = d['game_mode'] as GameModeConfig ?? null;
        this.status = 'connected';
        this.eventBuffer = [];
        this._fetchSnapshot();
        break;
      }
    }
    this._syncPlayerList();
    this.cdr.markForCheck();
  }

  private _updateDirectedCam(forceSwitch = false): void {
    const now = performance.now();
    const MIN_FOLLOW_TIME = 5000; // 5 seconds
    const SCORE_DECAY = 0.95;

    // 1. Decay all activities and update boredom
    for (const [id, act] of this.botActivity.entries()) {
      act.activityScore *= SCORE_DECAY;

      if (id === this.followId) {
        // Increase boredom for current bot
        act.boredom += 0.005; // Slightly slower boredom increase
        if (act.boredom > 1) act.boredom = 1;
        act.lastFollowTime = now;
      } else {
        // Slowly decrease boredom for others
        act.boredom *= 0.995;
      }
    }

    // 2. Check if we should switch
    if (!forceSwitch && this.followId && (now - this.lastDirectedSwitch < MIN_FOLLOW_TIME)) {
      return;
    }

    // 3. Calculate final scores and pick best
    let bestId: string | null = null;
    let maxScore = -1;

    for (const [id, player] of this.players.entries()) {
      const act = this.botActivity.get(id);
      if (!act) continue;

      // Final score components:
      // - Activity (movement/jumps)
      // - Height (favor bots higher up)
      // - Boredom penalty (massively reduce if watched recently)

      const heightFactor = 1 + (player.y / 100); // 1.0 at 0m, 2.0 at 100m
      const adjustedScore = (act.activityScore + 5) * heightFactor * (1 - act.boredom);

      if (adjustedScore > maxScore) {
        maxScore = adjustedScore;
        bestId = id;
      }
    }

    if (bestId && bestId !== this.followId) {
      this.followId = bestId;
      this.lastDirectedSwitch = now;
      // Reset boredom slightly when starting to follow
      const act = this.botActivity.get(bestId);
      if (act) act.boredom = 0.2; // Start with some boredom to prevent staying forever
      this.cdr.markForCheck();
    }
  }

  // ---------------------------------------------------------------------------
  // Render loop
  // ---------------------------------------------------------------------------

  private _scheduleFrame(): void {
    this.rafId = requestAnimationFrame(() => {
      this._draw();
      this._scheduleFrame();
    });
  }

  private _draw(): void {
    const ctx = this.ctx;
    const now = performance.now();

    // ── Background: dark industrial shaft ──
    ctx.fillStyle = '#0d1014';
    ctx.fillRect(0, 0, this.canvasW, this.canvasH);

    // Vertical depth gradient
    const grad = ctx.createLinearGradient(0, 0, 0, this.canvasH);
    grad.addColorStop(0,   'rgba(0,0,0,0.45)');
    grad.addColorStop(0.4, 'rgba(0,0,0,0)');
    grad.addColorStop(0.6, 'rgba(0,0,0,0)');
    grad.addColorStop(1,   'rgba(0,0,0,0.45)');
    ctx.fillStyle = grad;
    ctx.fillRect(0, 0, this.canvasW, this.canvasH);

    // Faint horizontal structural beams
    ctx.strokeStyle = 'rgba(255,255,255,0.02)';
    ctx.lineWidth = 1;
    for (let y = 0; y < this.canvasH; y += 80) {
      ctx.beginPath();
      ctx.moveTo(0, y);
      ctx.lineTo(this.canvasW, y);
      ctx.stroke();
    }

    if (this.status !== 'ready') return;

    // Update camera
    if (this.followId) {
      const ps = this.players.get(this.followId);
      if (ps) {
        this.camY = ps.y - (this.canvasH / 2) / WORLD_SCALE;
      }
    }

    this._clampCamera();

    const toScreenX = (wx: number) => wx * WORLD_SCALE + this.canvasW / 2;
    const toScreenY = (wy: number) => this.canvasH - (wy - this.camY) * WORLD_SCALE;

    // ── Platforms ──
    for (const pl of this.platforms.values()) {
      const r = pl.rect;
      const sx = toScreenX(r.x);
      const sy = toScreenY(r.y + r.height);
      const sw = r.width * WORLD_SCALE;
      const sh = r.height * WORLD_SCALE;
      if (sy + sh < 0 || sy > this.canvasH) continue;

      // Base fill
      ctx.fillStyle = '#2e3a44';
      ctx.fillRect(sx, sy, sw, sh);

      // Diagonal hatch texture
      ctx.save();
      ctx.beginPath();
      ctx.rect(sx, sy, sw, sh);
      ctx.clip();
      ctx.strokeStyle = 'rgba(0,0,0,0.2)';
      ctx.lineWidth = 1;
      const step = 6;
      for (let i = -sh; i < sw + sh; i += step) {
        ctx.beginPath();
        ctx.moveTo(sx + i, sy);
        ctx.lineTo(sx + i + sh, sy + sh);
        ctx.stroke();
      }
      ctx.restore();

      // Top highlight edge
      ctx.strokeStyle = '#5a7080';
      ctx.lineWidth = 1;
      ctx.beginPath();
      ctx.moveTo(sx, sy);
      ctx.lineTo(sx + sw, sy);
      ctx.stroke();

      // Bottom shadow
      ctx.fillStyle = 'rgba(0,0,0,0.5)';
      ctx.fillRect(sx, sy + sh, sw, 2);

      // Rivet circles at ends
      const rh = hashCode(pl.id);
      const rivetPositions = [
        { x: sx + 4,      y: sy + sh / 2 },
        { x: sx + sw - 4, y: sy + sh / 2 },
      ];
      ctx.fillStyle = '#8090a0';
      for (const rv of rivetPositions) {
        ctx.beginPath();
        ctx.arc(rv.x, rv.y, 2.5, 0, Math.PI * 2);
        ctx.fill();
        // Rivet highlight
        ctx.fillStyle = 'rgba(255,255,255,0.15)';
        ctx.beginPath();
        ctx.arc(rv.x - 0.5, rv.y - 0.5, 1, 0, Math.PI * 2);
        ctx.fill();
        ctx.fillStyle = '#8090a0';
      }

      // Rust patches (1–2 per platform, deterministic)
      const numRust = 1 + (rh % 2);
      ctx.fillStyle = 'rgba(80,100,120,0.28)';
      for (let i = 0; i < numRust; i++) {
        const rx = sx + ((rh * (i + 3)) % Math.max(1, sw - 8));
        const ry = sy + 1;
        const rw = 4 + (rh % 5);
        ctx.fillRect(rx, ry, rw, sh - 2);
      }
    }

    // ── Players (bots as industrial cubes) ──
    const BOT_W = 20;
    const BOT_H = 20;

    for (const [id, ps] of this.players) {
      const sx = Math.round(toScreenX(ps.x) - BOT_W / 2);
      const sy = Math.round(toScreenY(ps.y) - BOT_H);

      // Assign / advance cog angle
      const prevAngle = this.cogAngles.get(id) ?? 0;
      const speed = ps.state === 'JUMPING' ? 0.06 : 0.015;
      const newAngle = prevAngle + speed;
      this.cogAngles.set(id, newAngle);

      const accentColor = BOT_COLORS[this.playerColorIndex.get(id) ?? 0];

      // 1. Cube body
      ctx.fillStyle = '#2e3338';
      ctx.fillRect(sx, sy, BOT_W, BOT_H);

      // Right-face depth sliver (darker)
      ctx.fillStyle = 'rgba(0,0,0,0.3)';
      ctx.fillRect(sx + BOT_W - 3, sy + 2, 3, BOT_H - 2);

      // Top-face depth sliver (lighter)
      ctx.fillStyle = 'rgba(255,255,255,0.12)';
      ctx.fillRect(sx, sy, BOT_W - 3, 3);

      // Top edge highlight
      ctx.strokeStyle = 'rgba(255,255,255,0.18)';
      ctx.lineWidth = 1;
      ctx.beginPath();
      ctx.moveTo(sx, sy);
      ctx.lineTo(sx + BOT_W, sy);
      ctx.stroke();

      // 2. Rivet corners
      ctx.fillStyle = '#8a7a60';
      const corners = [
        { x: sx + 2,        y: sy + 2 },
        { x: sx + BOT_W - 3, y: sy + 2 },
        { x: sx + 2,        y: sy + BOT_H - 3 },
        { x: sx + BOT_W - 3, y: sy + BOT_H - 3 },
      ];
      for (const c of corners) {
        ctx.beginPath();
        ctx.arc(c.x, c.y, 1.5, 0, Math.PI * 2);
        ctx.fill();
      }

      // 3. Viewport bezel (inset 4px each side)
      const vx = sx + 4;
      const vy = sy + 4;
      const vw = BOT_W - 8;
      const vh = BOT_H - 8;
      ctx.fillStyle = 'rgba(0,0,0,0.55)';
      ctx.fillRect(vx, vy, vw, vh);
      ctx.strokeStyle = '#5a7080';
      ctx.lineWidth = 1;
      ctx.strokeRect(vx, vy, vw, vh);

      // 4. Cog (approximated: circle + 6 teeth)
      const cogX = vx + vw * 0.38;
      const cogY = vy + vh * 0.42;
      const cogR = 2.8;
      ctx.save();
      ctx.translate(cogX, cogY);
      ctx.rotate(newAngle);
      ctx.fillStyle = '#8090a0';
      ctx.beginPath();
      ctx.arc(0, 0, cogR, 0, Math.PI * 2);
      ctx.fill();
      // Teeth
      for (let t = 0; t < 6; t++) {
        const ta = (t / 6) * Math.PI * 2;
        ctx.save();
        ctx.rotate(ta);
        ctx.fillRect(-1, -(cogR + 2), 2, 2.5);
        ctx.restore();
      }
      // Hub
      ctx.fillStyle = '#161a1d';
      ctx.beginPath();
      ctx.arc(0, 0, 1.2, 0, Math.PI * 2);
      ctx.fill();
      ctx.restore();

      // 5. Glowing core
      const coreX = vx + vw * 0.72;
      const coreY = vy + vh * 0.42;
      const pulse = 0.7 + 0.3 * Math.sin(now / 1000 * Math.PI);
      ctx.save();
      ctx.shadowBlur = 6;
      ctx.shadowColor = accentColor;
      ctx.globalAlpha = ps.state === 'JUMPING' ? 1.0 : pulse;
      ctx.fillStyle = accentColor;
      ctx.beginPath();
      ctx.arc(coreX, coreY, 2.5, 0, Math.PI * 2);
      ctx.fill();
      ctx.restore();

      // 6. Spring coils (zigzag below cog inside viewport)
      const springExtend = ps.state === 'JUMPING' ? 1.5 : 0;
      const springY0 = vy + vh * 0.65;
      const springY1 = vy + vh - 1 + springExtend;
      ctx.strokeStyle = '#5a6470';
      ctx.lineWidth = 1;
      // Left spring
      ctx.beginPath();
      ctx.moveTo(vx + 2, springY0);
      ctx.lineTo(vx + 4, springY0 + (springY1 - springY0) * 0.33);
      ctx.lineTo(vx + 2, springY0 + (springY1 - springY0) * 0.66);
      ctx.lineTo(vx + 4, springY1);
      ctx.stroke();
      // Right spring
      ctx.beginPath();
      ctx.moveTo(vx + vw - 2, springY0);
      ctx.lineTo(vx + vw - 4, springY0 + (springY1 - springY0) * 0.33);
      ctx.lineTo(vx + vw - 2, springY0 + (springY1 - springY0) * 0.66);
      ctx.lineTo(vx + vw - 4, springY1);
      ctx.stroke();

      // 7. Wear scratch
      ctx.strokeStyle = 'rgba(255,255,255,0.06)';
      ctx.lineWidth = 1;
      ctx.beginPath();
      ctx.moveTo(sx + 5, sy + 6);
      ctx.lineTo(sx + 11, sy + 14);
      ctx.stroke();

      // Name label
      const label = ps.name;
      ctx.font = '10px "Share Tech Mono", Consolas, monospace';
      const tw = ctx.measureText(label).width;
      const lx = sx + BOT_W / 2 - tw / 2;
      const ly = sy - 6;
      ctx.fillStyle = 'rgba(0,0,0,0.6)';
      ctx.fillRect(lx - 2, ly - 9, tw + 4, 11);
      ctx.fillStyle = '#e8dcc8';
      ctx.fillText(label, lx, ly);
    }

    // Vignette overlay
    const vign = ctx.createRadialGradient(this.canvasW / 2, this.canvasH / 2, this.canvasH * 0.3, this.canvasW / 2, this.canvasH / 2, this.canvasH * 0.8);
    vign.addColorStop(0, 'rgba(0,0,0,0)');
    vign.addColorStop(1, 'rgba(0,0,0,0.4)');
    ctx.fillStyle = vign;
    ctx.fillRect(0, 0, this.canvasW, this.canvasH);

    // HUD: following label
    if (this.followId) {
      const name = this.followName;
      ctx.font = '13px "Oswald", Impact, sans-serif';
      ctx.fillStyle = 'rgba(0,0,0,0.55)';
      ctx.fillRect(8, 8, ctx.measureText('◈ ' + name).width + 10, 20);
      ctx.fillStyle = '#8a7a60';
      ctx.fillText('◈ ', 13, 23);
      ctx.fillStyle = '#e8dcc8';
      ctx.fillText('◈ ' + name, 13, 23);
    }
  }
}

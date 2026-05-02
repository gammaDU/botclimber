import {
  Component, OnInit, OnDestroy, ElementRef, ViewChild, AfterViewInit, Input, ChangeDetectionStrategy,
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
  consecutive_bounces: number;
}

interface SnapshotResponse {
  timestamp: number; // epoch ms
  game_mode: string;
  tick: number;
  players: Array<{
    player_id: string; name: string;
    position: { x: number; y: number };
    velocity: { x: number; y: number };
    state: string;
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

@Component({
  selector: 'app-spectate',
  standalone: true,
  imports: [CommonModule],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
      <div class="view">
          <div id="status-bar">
              <span id="connection-status" [class]="statusClass">{{ statusLabel }}</span>
              <span style="color:var(--text-muted);font-size:0.85rem">
          {{ playerList.length }} active player{{ playerList.length !== 1 ? 's' : '' }}
        </span>
          </div>

          <div id="game-container">
              <canvas #canvas id="game-canvas" [width]="canvasW" [height]="canvasH"></canvas>

              <div id="session-panel">
                  <h2>Active Players</h2>
                  <ul id="session-list">
                      @for (p of playerList; track p.player_id) {
                          <li [class.following]="followId === p.player_id" (click)="follow(p.player_id)">
                              <span>{{ p.name }}</span>
                              <span>{{ p.y | number:'1.1-1' }} m</span>
                          </li>
                      } @empty {
                          <li style="color:var(--text-muted)">No active players</li>
                      }
                  </ul>

                  <div id="cam-controls">
                      <button [class.active]="followId === null" (click)="follow(null)">Free cam</button>
                      @if (followId) {
                          <span id="following-label">
              Following: {{ followName }}
            </span>
                      }
                  </div>
              </div>
          </div>
      </div>
  `,
})
export class SpectateComponent implements OnInit, AfterViewInit, OnDestroy {
  @ViewChild('canvas') canvasRef!: ElementRef<HTMLCanvasElement>;

  readonly canvasW = CANVAS_W;
  readonly canvasH = CANVAS_H;

  /** True once snapshot has been applied and buffered events replayed. */
  ready = false;

  followId: string | null = null;

  @Input() status: 'connecting' | 'connected' | 'reconnecting' | 'disconnected' = 'connecting';

  // Derived list for the sidebar
  get playerList(): PlayerState[] {
    return Array.from(this.players.values()).sort((a, b) => b.y - a.y);
  }

  private platforms = new Map<string, Platform>();
  private players = new Map<string, PlayerState>();

  /** Events buffered before the snapshot arrives. */
  private eventBuffer: WsEnvelope[] = [];
  private snapshotTimestamp = 0;
  private snapshotFetched = false;

  private camY = 0;
  private subs = new Subscription();
  private ctx!: CanvasRenderingContext2D;
  private rafId = 0;

  constructor(
    private stream: GameStreamService,
    private http: HttpClient,
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
      this.status = s;
      if (s === 'reconnecting') {
        // Reset bootstrap state so we redo it on reconnect
        this.ready = false;
        this.snapshotFetched = false;
        this.eventBuffer = [];
      }
      if (s === 'connected' && !this.snapshotFetched) {
        this._fetchSnapshot();
      }
    }));

    this.subs.add(this.stream.events$.subscribe(e => this._onEvent(e)));

    // Step 1: open WS first (start buffering), then snapshot is fetched on 'connected'
    this.stream.reset();
    this.stream.connect();
  }

  ngAfterViewInit(): void {
    this.ctx = this.canvasRef.nativeElement.getContext('2d')!;
    this._scheduleFrame();
  }

  ngOnDestroy(): void {
    cancelAnimationFrame(this.rafId);
    this.stream.disconnect();
    this.subs.unsubscribe();
  }

  follow(id: string | null): void { this.followId = id; }

  // ---------------------------------------------------------------------------
  // Bootstrap: fetch snapshot, replay buffer
  // ---------------------------------------------------------------------------

  private _fetchSnapshot(): void {
    this.snapshotFetched = true;
    this.http.get<SnapshotResponse>('/api/v1/system/snapshot').subscribe({
      next: snap => {
        this.snapshotTimestamp = snap.timestamp;

        // Seed state from snapshot
        this.platforms.clear();
        for (const p of snap.platforms) {
          this.platforms.set(p.id, p);
        }
        this.players.clear();
        for (const p of snap.players) {
          this.players.set(p.player_id, {
            player_id: p.player_id,
            name: p.name,
            x: p.position.x,
            y: p.position.y,
            vx: p.velocity.x,
            vy: p.velocity.y,
            state: p.state as 'LANDED' | 'JUMPING',
            consecutive_bounces: 0,
          });
        }

        // Replay buffered events that arrived after the snapshot
        for (const e of this.eventBuffer) {
          if (new Date(e.timestamp).getTime() > this.snapshotTimestamp) {
            this._applyEvent(e);
          }
        }
        this.eventBuffer = [];
        this.ready = true;
        this.status = 'connected';
      },
      error: () => {
        // Retry snapshot after a short delay
        this.snapshotFetched = false;
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
    if (this.status !== 'connected') {
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
        this.players.set(id, {
          player_id: id,
          name: existing?.name ?? id,
          x: d['x'] as number,
          y: d['y'] as number,
          vx: d['vx'] as number,
          vy: d['vy'] as number,
          state: d['state'] as 'LANDED' | 'JUMPING',
          consecutive_bounces: d['consecutive_bounces'] as number,
        });
        break;
      }
      case 'platform_spawn': {
        const id = d['platform_id'] as string;
        const rect = d['rect'] as Platform['rect'];
        this.platforms.set(id, { id, rect });
        break;
      }
      case 'player_joined': {
        const id = d['player_id'] as string;
        if (!this.players.has(id)) {
          this.players.set(id, {
            player_id: id,
            name: d['name'] as string,
            x: 0, y: 0, vx: 0, vy: 0,
            state: 'LANDED',
            consecutive_bounces: 0,
          });
        }
        break;
      }
      case 'player_left': {
        const id = d['player_id'] as string;
        this.players.delete(id);
        if (this.followId === id) this.followId = null;
        break;
      }
      case 'world_reset': {
        this.players.clear();
        this.platforms.clear();
        this.ready = false;
        this.snapshotFetched = false;
        this.eventBuffer = [];
        if (this.status === 'connected') this._fetchSnapshot();
        break;
      }
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
    ctx.clearRect(0, 0, CANVAS_W, CANVAS_H);
    ctx.fillStyle = '#111122';
    ctx.fillRect(0, 0, CANVAS_W, CANVAS_H);

    if (!this.ready) return;

    // Update camera
    if (this.followId) {
      const ps = this.players.get(this.followId);
      if (ps) this.camY = ps.y - (CANVAS_H / 2) / WORLD_SCALE;
    }

    const toScreenX = (wx: number) => wx * WORLD_SCALE + CANVAS_W / 2;
    const toScreenY = (wy: number) => CANVAS_H - (wy - this.camY) * WORLD_SCALE;

    // Draw platforms
    ctx.fillStyle = '#3a5a8a';
    for (const pl of this.platforms.values()) {
      const r = pl.rect;
      ctx.fillRect(toScreenX(r.x), toScreenY(r.y + r.height), r.width * WORLD_SCALE, r.height * WORLD_SCALE);
    }

    // Draw players
    for (const [id, ps] of this.players) {
      const sx = toScreenX(ps.x) - 10;
      const sy = toScreenY(ps.y) - 20;
      ctx.fillStyle = id === this.followId ? '#e94560' : '#4caf50';
      ctx.fillRect(sx, sy, 20, 20);
      ctx.fillStyle = '#e0e0e0';
      ctx.font = '11px Consolas, monospace';
      ctx.fillText(ps.name, sx, sy - 4);
    }
  }
}

import {
  Component, OnInit, OnDestroy, ElementRef, ViewChild, AfterViewInit,
} from '@angular/core';
import { CommonModule } from '@angular/common';
import { Subscription } from 'rxjs';
import { GameStreamService, GameEvent } from '../services/game-stream.service';
import { ApiService, ActivePlayer } from '../services/api.service';

interface Platform { x: number; y: number; width: number; height: number; }
interface PlayerState { session_id: string; username: string; x: number; y: number; }

const CANVAS_W = 640;
const CANVAS_H = 480;
const WORLD_SCALE = 40; // pixels per world unit

@Component({
  selector: 'app-spectate',
  standalone: true,
  imports: [CommonModule],
  template: `
    <div class="view">
      <div id="status-bar">
        <span id="connection-status" [class]="statusClass">{{ statusLabel }}</span>
        <span style="color:var(--text-muted);font-size:0.85rem">
          {{ players.length }} active session{{ players.length !== 1 ? 's' : '' }}
        </span>
      </div>

      <div id="game-container">
        <canvas #canvas id="game-canvas" [width]="canvasW" [height]="canvasH"></canvas>

        <div id="session-panel">
          <h2>Active Players</h2>
          <ul id="session-list">
            <li
              *ngFor="let p of players"
              [class.following]="followId === p.session_id"
              (click)="follow(p.session_id)"
              style="cursor:pointer"
            >
              <span>{{ p.username }}</span>
              <span>{{ p.height | number:'1.1-1' }} m</span>
            </li>
            <li *ngIf="players.length === 0" style="color:var(--text-muted)">No active sessions</li>
          </ul>

          <div id="cam-controls">
            <button [class.active]="followId === null" (click)="follow(null)">Free cam</button>
            <span id="following-label" *ngIf="followId">
              Following: {{ followName }}
            </span>
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

  players: ActivePlayer[] = [];
  followId: string | null = null;

  status: 'connecting' | 'connected' | 'disconnected' = 'disconnected';

  private platforms: Platform[] = [];
  private playerStates = new Map<string, PlayerState>();
  private camY = 0;
  private subs = new Subscription();
  private ctx!: CanvasRenderingContext2D;
  private rafId = 0;

  constructor(
    private stream: GameStreamService,
    private api: ApiService,
  ) {}

  get statusClass(): string {
    return `status-${this.status}`;
  }

  get statusLabel(): string {
    return this.status.charAt(0).toUpperCase() + this.status.slice(1);
  }

  get followName(): string {
    return this.players.find(p => p.session_id === this.followId)?.username ?? '';
  }

  ngOnInit(): void {
    this.api.getActivePlayers().subscribe({ next: p => { this.players = p; } });

    this.subs.add(this.stream.status$.subscribe(s => { this.status = s; }));
    this.subs.add(this.stream.events$.subscribe(e => this.handleEvent(e)));
    this.stream.connect();
  }

  ngAfterViewInit(): void {
    this.ctx = this.canvasRef.nativeElement.getContext('2d')!;
    this.scheduleFrame();
  }

  ngOnDestroy(): void {
    cancelAnimationFrame(this.rafId);
    this.stream.disconnect();
    this.subs.unsubscribe();
  }

  follow(id: string | null): void {
    this.followId = id;
  }

  private handleEvent(e: GameEvent): void {
    switch (e['type']) {
      case 'snapshot': {
        const snap = e as unknown as {
          type: string;
          players: PlayerState[];
          platforms: Platform[];
        };
        this.platforms = snap.platforms ?? [];
        this.playerStates.clear();
        for (const p of snap.players ?? []) {
          this.playerStates.set(p.session_id, p);
        }
        this.players = (snap.players ?? []).map(p => ({
          session_id: p.session_id,
          username: p.username,
          height: p.y,
          game_mode: '',
        }));
        break;
      }
      case 'player_update': {
        const u = e as unknown as PlayerState & { type: string };
        this.playerStates.set(u.session_id, u);
        const idx = this.players.findIndex(p => p.session_id === u.session_id);
        if (idx >= 0) this.players[idx] = { ...this.players[idx], height: u.y };
        break;
      }
      case 'player_left': {
        const id = e['session_id'] as string;
        this.playerStates.delete(id);
        this.players = this.players.filter(p => p.session_id !== id);
        if (this.followId === id) this.followId = null;
        break;
      }
    }
  }

  private scheduleFrame(): void {
    this.rafId = requestAnimationFrame(() => {
      this.draw();
      this.scheduleFrame();
    });
  }

  private draw(): void {
    const ctx = this.ctx;
    ctx.clearRect(0, 0, CANVAS_W, CANVAS_H);
    ctx.fillStyle = '#111122';
    ctx.fillRect(0, 0, CANVAS_W, CANVAS_H);

    // Update camera
    if (this.followId) {
      const ps = this.playerStates.get(this.followId);
      if (ps) this.camY = ps.y - (CANVAS_H / 2) / WORLD_SCALE;
    }

    const toScreenX = (wx: number) => wx * WORLD_SCALE + CANVAS_W / 2;
    const toScreenY = (wy: number) => CANVAS_H - (wy - this.camY) * WORLD_SCALE;

    // Draw platforms
    ctx.fillStyle = '#3a5a8a';
    for (const pl of this.platforms) {
      const sx = toScreenX(pl.x);
      const sy = toScreenY(pl.y + pl.height);
      ctx.fillRect(sx, sy, pl.width * WORLD_SCALE, pl.height * WORLD_SCALE);
    }

    // Draw players
    for (const [id, ps] of this.playerStates) {
      const sx = toScreenX(ps.x) - 10;
      const sy = toScreenY(ps.y) - 20;
      ctx.fillStyle = id === this.followId ? '#e94560' : '#4caf50';
      ctx.fillRect(sx, sy, 20, 20);
      ctx.fillStyle = '#e0e0e0';
      ctx.font = '11px Consolas, monospace';
      ctx.fillText(ps.username, sx, sy - 4);
    }
  }
}

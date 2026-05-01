import { Injectable, OnDestroy } from '@angular/core';
import { Subject } from 'rxjs';

export interface GameEvent {
  type: string;
  [key: string]: unknown;
}

@Injectable({ providedIn: 'root' })
export class GameStreamService implements OnDestroy {
  private ws: WebSocket | null = null;

  readonly events$ = new Subject<GameEvent>();
  readonly status$ = new Subject<'connecting' | 'connected' | 'disconnected'>();

  connect(): void {
    if (this.ws) return;
    const proto = location.protocol === 'https:' ? 'wss' : 'ws';
    const url = `${proto}://${location.host}/api/v1/ws`;
    this.status$.next('connecting');
    this.ws = new WebSocket(url);

    this.ws.onopen = () => this.status$.next('connected');

    this.ws.onmessage = ({ data }) => {
      try {
        this.events$.next(JSON.parse(data) as GameEvent);
      } catch {
        // ignore malformed frames
      }
    };

    this.ws.onclose = () => {
      this.ws = null;
      this.status$.next('disconnected');
    };

    this.ws.onerror = () => {
      this.ws?.close();
    };
  }

  disconnect(): void {
    this.ws?.close();
    this.ws = null;
  }

  ngOnDestroy(): void {
    this.disconnect();
  }
}

import { Injectable, OnDestroy } from '@angular/core';
import { BehaviorSubject, Subject } from 'rxjs';

export interface WsEnvelope {
  type: string;
  timestamp: string;
  data: Record<string, unknown>;
}

@Injectable({ providedIn: 'root' })
export class GameStreamService implements OnDestroy {
  private ws: WebSocket | null = null;
  private reconnectAttempt = 0;
  private reconnectTimer: ReturnType<typeof setTimeout> | null = null;
  private destroyed = false;

  /** Raw parsed envelopes from the server. */
  readonly events$ = new Subject<WsEnvelope>();
  readonly status$ = new BehaviorSubject<'connecting' | 'connected' | 'reconnecting' | 'disconnected'>('disconnected');

  connect(): void {
    if (this.ws || this.destroyed) return;
    this._open();
  }

  disconnect(): void {
    this.destroyed = true;
    this._clearReconnect();
    this.ws?.close(1000);
    this.ws = null;
    this.status$.next('disconnected');
  }

  /** Reset the service so it can be reconnected after a disconnect. */
  reset(): void {
    this.destroyed = false;
    this.reconnectAttempt = 0;
    this.status$.next('disconnected');
  }

  ngOnDestroy(): void {
    this.disconnect();
  }

  private _open(): void {
    const proto = location.protocol === 'https:' ? 'wss' : 'ws';
    const url = `${proto}://${location.host}/api/v1/system/stream`;

    this.status$.next(this.reconnectAttempt === 0 ? 'connecting' : 'reconnecting');
    const ws = new WebSocket(url);
    this.ws = ws;

    ws.onopen = () => {
      this.reconnectAttempt = 0;
      this.status$.next('connected');
    };

    ws.onmessage = ({ data }) => {
      try {
        this.events$.next(JSON.parse(data as string) as WsEnvelope);
      } catch {
        // ignore malformed frames
      }
    };

    ws.onclose = () => {
      if (this.ws === ws) this.ws = null;
      if (!this.destroyed) {
        this._scheduleReconnect();
      } else {
        this.status$.next('disconnected');
      }
    };

    ws.onerror = () => {
      ws.close();
    };
  }

  private _scheduleReconnect(): void {
    const delays = [1000, 2000, 4000, 8000];
    const delay = delays[Math.min(this.reconnectAttempt, delays.length - 1)];
    this.reconnectAttempt++;
    this.status$.next('reconnecting');
    this.reconnectTimer = setTimeout(() => {
      if (!this.destroyed) this._open();
    }, delay);
  }

  private _clearReconnect(): void {
    if (this.reconnectTimer !== null) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }
  }
}

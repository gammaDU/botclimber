import { Injectable } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { Observable } from 'rxjs';

const BASE = '/api/v1';

export interface LeaderboardEntry {
  rank: number;
  username: string;
  max_height: number;
  achieved_at: string;
}

export interface ActivePlayer {
  session_id: string;
  username: string;
  height: number;
  game_mode: string;
}

export interface WorldSnapshot {
  players: ActivePlayer[];
  platforms: { x: number; y: number; width: number; height: number }[];
}

@Injectable({ providedIn: 'root' })
export class ApiService {
  constructor(private http: HttpClient) {}

  getLeaderboard(mode?: string): Observable<LeaderboardEntry[]> {
    const q = mode ? `?mode=${encodeURIComponent(mode)}` : '';
    return this.http.get<LeaderboardEntry[]>(`${BASE}/system/leaderboard${q}`);
  }

  getSnapshot(): Observable<WorldSnapshot> {
    return this.http.get<WorldSnapshot>(`${BASE}/system/snapshot`);
  }

  getActivePlayers(): Observable<ActivePlayer[]> {
    return this.http.get<ActivePlayer[]>(`${BASE}/system/players`);
  }
}

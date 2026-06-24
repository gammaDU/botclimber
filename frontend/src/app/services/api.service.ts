import { Injectable } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { Observable } from 'rxjs';

const BASE = '/api/v1';

export interface StageInfo {
  id: string;
  label: string;
  height: number;
}

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

  getLeaderboard(sessionType?: string, stageType?: string, fixedStageId?: string): Observable<LeaderboardEntry[]> {
    const params: string[] = [];
    if (sessionType) params.push(`session_type=${encodeURIComponent(sessionType)}`);
    if (stageType)   params.push(`stage_type=${encodeURIComponent(stageType)}`);
    if (fixedStageId) params.push(`fixed_stage_id=${encodeURIComponent(fixedStageId)}`);
    const q = params.length ? '?' + params.join('&') : '';
    return this.http.get<LeaderboardEntry[]>(`${BASE}/system/leaderboard${q}`);
  }

  getStages(): Observable<StageInfo[]> {
    return this.http.get<StageInfo[]>(`${BASE}/system/stage`);
  }

  getSnapshot(): Observable<WorldSnapshot> {
    return this.http.get<WorldSnapshot>(`${BASE}/system/snapshot`);
  }

  getActivePlayers(): Observable<ActivePlayer[]> {
    return this.http.get<ActivePlayer[]>(`${BASE}/system/players`);
  }
}

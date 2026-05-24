import { Injectable } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { Observable } from 'rxjs';

const BASE = '/api/v1/admin';

export interface GameModeConfig {
  fall_mode?: string;
  session_type?: string;
  stage_type?: string;
  fixed_stage_id?: string;
  goal_height?: number;
}

@Injectable({ providedIn: 'root' })
export class AdminService {
  constructor(private http: HttpClient) {}

  setMode(config: GameModeConfig): Observable<GameModeConfig> {
    return this.http.post<GameModeConfig>(`${BASE}/mode`, config);
  }

  resetWorld(): Observable<{ status: string }> {
    return this.http.post<{ status: string }>(`${BASE}/world/reset`, {});
  }

  resetAllPlayers(): Observable<{ status: string }> {
    return this.http.post<{ status: string }>(`${BASE}/players/reset`, {});
  }
}

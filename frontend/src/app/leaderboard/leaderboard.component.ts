import { Component, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { ApiService, LeaderboardEntry, StageInfo } from '../services/api.service';

type SessionType = 'Race' | 'Timetrial' | 'Infinite';

@Component({
  selector: 'app-leaderboard',
  standalone: true,
  imports: [CommonModule],
  template: `
    <div id="view-leaderboard" class="view">
      <h2>Leaderboard</h2>

      <div id="mode-tabs">
        <button class="mode-tab" [class.active]="sessionType === 'Race'"      (click)="setSessionType('Race')">Race</button>
        <button class="mode-tab" [class.active]="sessionType === 'Timetrial'" (click)="setSessionType('Timetrial')">Time Trial</button>
        <button class="mode-tab" [class.active]="sessionType === 'Infinite'"  (click)="setSessionType('Infinite')">Infinite</button>
      </div>

      <div id="stage-filter">
        <label for="stage-select">Stage:</label>
        <select id="stage-select" (change)="onStageChange($event)">
          <option value="__random__">Random (Generated)</option>
          <option *ngFor="let s of stages" [value]="s.id">{{ s.label }}</option>
        </select>
      </div>

      <table id="leaderboard-table">
        <thead>
          <tr>
            <th>#</th>
            <th>Player</th>
            <th>Max Height</th>
            <th>Date</th>
          </tr>
        </thead>
        <tbody>
          <tr *ngFor="let entry of entries; let i = index">
            <td>{{ i + 1 }}</td>
            <td>{{ entry.username }}</td>
            <td>{{ entry.max_height | number:'1.1-1' }} m</td>
            <td>{{ entry.achieved_at | date:'short' }}</td>
          </tr>
          <tr *ngIf="entries.length === 0 && !loading">
            <td colspan="4" style="text-align:center;color:var(--text-muted)">No scores yet.</td>
          </tr>
          <tr *ngIf="loading">
            <td colspan="4" style="text-align:center;color:var(--text-muted)">Loading…</td>
          </tr>
        </tbody>
      </table>

      <p *ngIf="error" class="form-error" style="margin-top:0.75rem">{{ error }}</p>
    </div>
  `,
})
export class LeaderboardComponent implements OnInit {
  entries: LeaderboardEntry[] = [];
  stages: StageInfo[] = [];

  sessionType: SessionType = 'Race';
  /** '__random__' means Generated; any other value is a fixed_stage_id */
  selectedStage = '__random__';

  loading = false;
  error = '';

  constructor(private api: ApiService) {}

  ngOnInit(): void {
    this.api.getStages().subscribe({
      next: data => { this.stages = data; },
      error: () => { /* non-critical, ignore */ },
    });
    this.load();
  }

  setSessionType(t: SessionType): void {
    this.sessionType = t;
    this.load();
  }

  onStageChange(event: Event): void {
    this.selectedStage = (event.target as HTMLSelectElement).value;
    this.load();
  }

  private load(): void {
    this.loading = true;
    this.error = '';

    const isRandom = this.selectedStage === '__random__';
    const stageType   = isRandom ? 'Generated' : 'Fixed';
    const fixedStageId = isRandom ? undefined : this.selectedStage;

    this.api.getLeaderboard(this.sessionType, stageType, fixedStageId).subscribe({
      next: data => { this.entries = data; this.loading = false; },
      error: err => { this.error = err.message ?? 'Failed to load'; this.loading = false; },
    });
  }
}

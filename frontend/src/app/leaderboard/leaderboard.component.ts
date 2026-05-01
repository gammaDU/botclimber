import { Component, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { ApiService, LeaderboardEntry } from '../services/api.service';

@Component({
  selector: 'app-leaderboard',
  standalone: true,
  imports: [CommonModule],
  template: `
    <div id="view-leaderboard" class="view">
      <h2>Leaderboard</h2>

      <div id="mode-tabs">
        <button class="mode-tab" [class.active]="mode === 'Checkpoint'" (click)="setMode('Checkpoint')">Checkpoint</button>
        <button class="mode-tab" [class.active]="mode === 'Hardcore'"   (click)="setMode('Hardcore')">Hardcore</button>
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
          <tr *ngFor="let entry of entries">
            <td>{{ entry.rank }}</td>
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
  mode: 'Checkpoint' | 'Hardcore' = 'Checkpoint';
  loading = false;
  error = '';

  constructor(private api: ApiService) {}

  ngOnInit(): void {
    this.load();
  }

  setMode(m: 'Checkpoint' | 'Hardcore'): void {
    this.mode = m;
    this.load();
  }

  private load(): void {
    this.loading = true;
    this.error = '';
    this.api.getLeaderboard(this.mode).subscribe({
      next: data => { this.entries = data; this.loading = false; },
      error: err => { this.error = err.message ?? 'Failed to load'; this.loading = false; },
    });
  }
}

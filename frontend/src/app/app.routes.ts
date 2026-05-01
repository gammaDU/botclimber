import { Routes } from '@angular/router';

export const routes: Routes = [
  { path: '', redirectTo: 'leaderboard', pathMatch: 'full' },
  {
    path: 'leaderboard',
    loadComponent: () =>
      import('./leaderboard/leaderboard.component').then(m => m.LeaderboardComponent),
  },
  {
    path: 'spectate',
    loadComponent: () =>
      import('./spectate/spectate.component').then(m => m.SpectateComponent),
  },
  {
    path: 'account',
    loadComponent: () =>
      import('./account/account.component').then(m => m.AccountComponent),
  },
  { path: '**', redirectTo: 'leaderboard' },
];

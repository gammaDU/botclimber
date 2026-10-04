import { Routes } from '@angular/router';

export const routes: Routes = [
  { path: '', redirectTo: 'spectate', pathMatch: 'full' },
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
  { path: '**', redirectTo: 'spectate' },
];

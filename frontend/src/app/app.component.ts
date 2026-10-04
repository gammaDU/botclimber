import { Component } from '@angular/core';
import { RouterLink, RouterLinkActive, RouterOutlet } from '@angular/router';

@Component({
  selector: 'app-root',
  standalone: true,
  imports: [RouterOutlet, RouterLink, RouterLinkActive],
  template: `
    <header>
      <h1>Botclimber</h1>
      <nav id="nav">
        <a routerLink="/spectate"    routerLinkActive="active" class="nav-btn">Spectate</a>
        <a routerLink="/account"     routerLinkActive="active" class="nav-btn">Account</a>
      </nav>
    </header>
    <main>
      <router-outlet />
    </main>
  `,
})
export class AppComponent {}

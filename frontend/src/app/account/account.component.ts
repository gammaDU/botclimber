import { Component, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { AuthService, UserProfile } from '../services/auth.service';

@Component({
  selector: 'app-account',
  standalone: true,
  imports: [CommonModule, FormsModule],
  template: `
    <!-- ── Auth forms (logged out) ── -->
    <div id="auth-forms" *ngIf="!auth.isLoggedIn()">

      <div class="auth-card">
        <h2>Register</h2>
        <form (ngSubmit)="register()">
          <label>
            Username
            <input type="text" [(ngModel)]="reg.username" name="reg_username" autocomplete="username" required />
          </label>
          <label>
            Password
            <input type="password" [(ngModel)]="reg.password" name="reg_password" autocomplete="new-password" required />
          </label>
          <button type="submit">Create account</button>
        </form>
        <p class="form-error">{{ regError }}</p>
        <p class="form-error" style="color:var(--success)">{{ regSuccess }}</p>
      </div>

      <div class="auth-card">
        <h2>Login</h2>
        <form (ngSubmit)="login()">
          <label>
            Username
            <input type="text" [(ngModel)]="log.username" name="log_username" autocomplete="username" required />
          </label>
          <label>
            Password
            <input type="password" [(ngModel)]="log.password" name="log_password" autocomplete="current-password" required />
          </label>
          <button type="submit">Sign in</button>
        </form>
        <p class="form-error">{{ logError }}</p>
      </div>

    </div>

    <!-- ── Profile panel (logged in) ── -->
    <div id="profile-panel" *ngIf="auth.isLoggedIn()">
      <h2>{{ profile?.username ?? '…' }}</h2>
      <p>Logged in</p>

      <!-- Change username -->
      <div class="profile-section">
        <h3>Change Username</h3>
        <form (ngSubmit)="changeUsername()">
          <label>
            New username
            <input type="text" [(ngModel)]="newUsername" name="new_username" required />
          </label>
          <button type="submit">Update</button>
        </form>
        <p class="form-error">{{ usernameError }}</p>
        <p class="form-error" style="color:var(--success)">{{ usernameSuccess }}</p>
      </div>

      <!-- Change password -->
      <div class="profile-section">
        <h3>Change Password</h3>
        <form (ngSubmit)="changePassword()">
          <label>
            New password
            <input type="password" [(ngModel)]="newPassword" name="new_password" autocomplete="new-password" required />
          </label>
          <button type="submit">Update</button>
        </form>
        <p class="form-error">{{ passwordError }}</p>
        <p class="form-error" style="color:var(--success)">{{ passwordSuccess }}</p>
      </div>

      <!-- API key -->
      <div class="profile-section">
        <h3>API Key</h3>
        <p class="mono">{{ apiKey ?? '(hidden — generate to reveal)' }}</p>
        <button (click)="generateApiKey()">Generate new key</button>
        <p class="form-error">{{ apiKeyError }}</p>
      </div>

      <button id="btn-logout" (click)="logout()">Log out</button>
    </div>
  `,
})
export class AccountComponent implements OnInit {
  profile: UserProfile | null = null;

  reg = { username: '', password: '' };
  log = { username: '', password: '' };

  regError = '';
  regSuccess = '';
  logError = '';

  newUsername = '';
  usernameError = '';
  usernameSuccess = '';

  newPassword = '';
  passwordError = '';
  passwordSuccess = '';

  apiKey: string | null = null;
  apiKeyError = '';

  constructor(readonly auth: AuthService) {}

  ngOnInit(): void {
    if (this.auth.isLoggedIn()) this.loadProfile();
  }

  register(): void {
    this.regError = '';
    this.regSuccess = '';
    this.auth.register(this.reg.username, this.reg.password).subscribe({
      next: () => {
        this.regSuccess = 'Account created — you can now log in.';
        this.reg = { username: '', password: '' };
      },
      error: err => { this.regError = err.error?.error ?? err.message ?? 'Registration failed'; },
    });
  }

  login(): void {
    this.logError = '';
    this.auth.login(this.log.username, this.log.password).subscribe({
      next: () => { this.loadProfile(); },
      error: err => { this.logError = err.error?.error ?? err.message ?? 'Login failed'; },
    });
  }

  logout(): void {
    this.auth.logout();
    this.profile = null;
    this.apiKey = null;
  }

  changeUsername(): void {
    this.usernameError = '';
    this.usernameSuccess = '';
    this.auth.updateProfile({ username: this.newUsername }).subscribe({
      next: () => {
        this.usernameSuccess = 'Username updated.';
        this.newUsername = '';
        this.loadProfile();
      },
      error: err => { this.usernameError = err.error?.error ?? err.message ?? 'Update failed'; },
    });
  }

  changePassword(): void {
    this.passwordError = '';
    this.passwordSuccess = '';
    this.auth.updateProfile({ password: this.newPassword }).subscribe({
      next: () => {
        this.passwordSuccess = 'Password updated.';
        this.newPassword = '';
      },
      error: err => { this.passwordError = err.error?.error ?? err.message ?? 'Update failed'; },
    });
  }

  generateApiKey(): void {
    this.apiKeyError = '';
    this.auth.generateApiKey().subscribe({
      next: res => { this.apiKey = res.api_key; },
      error: err => { this.apiKeyError = err.error?.error ?? err.message ?? 'Failed'; },
    });
  }

  private loadProfile(): void {
    this.auth.getProfile().subscribe({
      next: p => { this.profile = p; },
      error: () => { this.auth.logout(); },
    });
  }
}

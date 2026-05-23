import { ChangeDetectorRef, Component, OnInit } from '@angular/core';
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

      <div class="auth-card auth-card--centered" *ngIf="!showRegister">
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
          <button type="submit" [disabled]="logLoading">
            <span *ngIf="logLoading" class="btn-spinner"></span>
            <span *ngIf="!logLoading">Sign in</span>
          </button>
        </form>
        <p class="form-error">{{ logError }}</p>
        <p class="form-hint">
          No account? <a href="#" (click)="$event.preventDefault(); showRegister = true">Create one here</a>
        </p>
      </div>

      <div class="auth-card auth-card--centered" *ngIf="showRegister">
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
          <button type="submit" [disabled]="regLoading">
            <span *ngIf="regLoading" class="btn-spinner"></span>
            <span *ngIf="!regLoading">Create account</span>
          </button>
        </form>
        <p class="form-error">{{ regError }}</p>
        <p class="form-error" style="color:var(--success)">{{ regSuccess }}</p>
        <p class="form-hint">
          Already have an account? <a href="#" (click)="$event.preventDefault(); showRegister = false">Sign in</a>
        </p>
      </div>

    </div>

    <!-- ── Profile panel (logged in) ── -->
    <div id="profile-panel" *ngIf="auth.isLoggedIn()">
      <div id="profile-header">
        <h2>Hello {{ profile?.username ?? auth.usernameFromToken() ?? '…' }}</h2>
        <button id="btn-logout" (click)="logout()">Log out</button>
      </div>

      <!-- API key -->
      <div class="profile-section">
        <h3>API Key</h3>
        <div class="api-key-row">
          <span class="mono">{{ apiKey ?? '(hidden — generate to reveal)' }}</span>
          <button *ngIf="apiKey" (click)="copyApiKey()" type="button" class="icon-btn" [title]="apiKeyCopied ? 'Copied!' : 'Copy to clipboard'">
            <svg *ngIf="!apiKeyCopied" xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path></svg>
            <svg *ngIf="apiKeyCopied" xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"></polyline></svg>
          </button>
        </div>
        <div style="display:flex;gap:0.5rem;flex-wrap:wrap;margin-top:0.5rem">
          <button (click)="generateApiKey()">Generate new key</button>
        </div>
        <p class="form-error">{{ apiKeyError }}</p>
      </div>
      
      <!-- Change username -->
      <div class="profile-section">
        <h3>Change Displayname</h3>
        <form (ngSubmit)="changeUsername()">
          <label>
            New Displayname
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
      
    </div>
  `,
})
export class AccountComponent implements OnInit {
  profile: UserProfile | null = null;

  showRegister = false;

  logLoading = false;
  regLoading = false;

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
  apiKeyCopied = false;

  constructor(readonly auth: AuthService, private cd: ChangeDetectorRef) {}

  ngOnInit(): void {
    if (this.auth.isLoggedIn()) this.loadProfile();
  }

  register(): void {
    this.regError = '';
    this.regSuccess = '';
    this.regLoading = true;
    this.auth.register(this.reg.username, this.reg.password).subscribe({
      next: () => {
        this.regLoading = false;
        this.regSuccess = 'Account created — you can now log in.';
        this.reg = { username: '', password: '' };
        this.showRegister = false;
        this.cd.markForCheck();
      },
      error: err => { this.regLoading = false; this.setError('regError', err, 'Registration failed'); },
    });
  }

  login(): void {
    this.logError = '';
    this.logLoading = true;
    this.auth.login(this.log.username, this.log.password).subscribe({
      next: () => { this.logLoading = false; this.cd.markForCheck(); this.loadProfile(); },
      error: err => { this.logLoading = false; this.setError('logError', err, 'Login failed'); },
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
    this.auth.updateProfile({ display_name: this.newUsername }).subscribe({
      next: () => {
        this.usernameSuccess = 'Display name updated.';
        this.newUsername = '';
        this.loadProfile();
        this.cd.markForCheck();
      },
      error: err => { this.setError('usernameError', err, 'Update failed'); },
    });
  }

  changePassword(): void {
    this.passwordError = '';
    this.passwordSuccess = '';
    this.auth.updateProfile({ password: this.newPassword }).subscribe({
      next: () => {
        this.passwordSuccess = 'Password updated.';
        this.newPassword = '';
        this.cd.markForCheck();
      },
      error: err => { this.setError('passwordError', err, 'Update failed'); },
    });
  }

  generateApiKey(): void {
    this.apiKeyError = '';
    this.auth.generateApiKey().subscribe({
      next: res => { this.apiKey = res.api_key; this.apiKeyCopied = false; this.cd.markForCheck(); },
      error: err => { this.setError('apiKeyError', err, 'Failed'); },
    });
  }

  copyApiKey(): void {
    if (!this.apiKey) return;
    navigator.clipboard.writeText(this.apiKey).then(() => {
      this.apiKeyCopied = true;
      this.cd.markForCheck();
      setTimeout(() => { this.apiKeyCopied = false; this.cd.markForCheck(); }, 2000);
    });
  }

  private setError(target: 'logError' | 'regError' | 'usernameError' | 'passwordError' | 'apiKeyError', err: any, fallback: string): void {
    (this as any)[target] = this.extractError(err, fallback);
    this.cd.markForCheck();
  }

  private extractError(err: any, fallback: string): string {
    const raw: string = err?.error?.error
      ?? (typeof err?.error === 'string' ? err.error : null)
      ?? err?.message
      ?? fallback;

    switch (raw.toLowerCase().trim()) {
      case 'unauthorized': return 'Invalid username or password.';
      case 'forbidden':    return 'You do not have permission to do that.';
      case 'not found':    return 'The requested resource was not found.';
      case 'too many requests': return 'Too many attempts — please wait a moment.';
      case 'internal server error': return 'Something went wrong on the server. Please try again later.';
      default: return raw;
    }
  }

  private loadProfile(): void {
    this.auth.getProfile().subscribe({
      next: p => { this.profile = p; this.newUsername = p.display_name ?? ''; this.cd.markForCheck(); },
      error: () => { this.auth.logout(); },
    });
  }
}

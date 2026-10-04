import { ChangeDetectorRef, Component, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { AuthService, UserProfile } from '../services/auth.service';
import { AdminService, GameModeConfig, FixedStageInfo } from '../services/admin.service';

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

    <!-- ── Profile + Admin panels (logged in) ── -->
    <div id="account-panels" *ngIf="auth.isLoggedIn()">
    <div id="profile-panel">
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

    <!-- ── Admin control panel (logged in + is_admin) ── -->
    <div id="admin-panel" *ngIf="profile?.is_admin">
      <h2>Admin Control Panel</h2>

      <!-- Game mode -->
      <div class="profile-section">
        <h3>Game Mode</h3>
        <form (ngSubmit)="applyMode()">
          <label>
            Fall Mode
            <select [(ngModel)]="adminMode.fall_mode" name="fall_mode">
              <option value="Checkpoint">Checkpoint</option>
              <option value="Hardcore">Hardcore</option>
            </select>
          </label>
          <label>
            Session Type
            <select [(ngModel)]="adminMode.session_type" name="session_type">
              <option value="Race">Race</option>
              <option value="Timetrial">Timetrial</option>
              <option value="Infinite">Infinite</option>
            </select>
          </label>
          <label>
            Stage Type
            <select [(ngModel)]="adminMode.stage_type" name="stage_type">
              <option value="Generated">Generated</option>
              <option value="Fixed">Fixed</option>
            </select>
          </label>
          <label *ngIf="adminMode.stage_type === 'Fixed'">
            Fixed Stage ID
            <select [(ngModel)]="adminMode.fixed_stage_id" name="fixed_stage_id">
              <option *ngFor="let stage of availableStages" [value]="stage.id">
                {{ stage.name }} ({{ stage.id }})
              </option>
            </select>
          </label>
          <label *ngIf="adminMode.stage_type === 'Generated' && adminMode.session_type !== 'Infinite'">
            Goal Height
            <input type="number" [(ngModel)]="adminMode.goal_height" name="goal_height" step="any" />
          </label>
          <button type="submit">Apply</button>
        </form>
        <p class="form-error">{{ adminModeError }}</p>
        <p class="form-error" style="color:var(--success)">{{ adminModeSuccess }}</p>
      </div>

      <!-- Session -->
      <div class="profile-section">
        <h3>Session</h3>
        <button (click)="restartSession()">Restart Session</button>
        <p class="form-error">{{ adminSessionError }}</p>
        <p class="form-error" style="color:var(--success)">{{ adminSessionSuccess }}</p>
      </div>

      <!-- Players -->
      <div class="profile-section">
        <h3>Players</h3>
        <button (click)="resetPlayers()">Reset All Players</button>
        <p class="form-error">{{ adminPlayersError }}</p>
        <p class="form-error" style="color:var(--success)">{{ adminPlayersSuccess }}</p>
      </div>
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

  adminMode: GameModeConfig = { fall_mode: 'Checkpoint', session_type: 'Infinite', stage_type: 'Generated' };
  adminModeError = '';
  adminModeSuccess = '';
  adminSessionError = '';
  adminSessionSuccess = '';
  adminPlayersError = '';
  adminPlayersSuccess = '';

  availableStages: FixedStageInfo[] = [];

  constructor(readonly auth: AuthService, private admin: AdminService, private cd: ChangeDetectorRef) {}

  ngOnInit(): void {
    if (this.auth.isLoggedIn()) {
      this.loadProfile();
      this.loadAvailableStages();
      this.loadCurrentMode();
    }
  }

  loadCurrentMode(): void {
    this.admin.getMode().subscribe({
      next: (mode) => {
        this.adminMode = { ...this.adminMode, ...mode };
        this.cd.markForCheck();
      },
      error: (err) => {
        console.error('Failed to load current game mode', err);
      }
    });
  }

  loadAvailableStages(): void {
    this.admin.getStages().subscribe({
      next: (stages) => {
        this.availableStages = stages;
        if (stages.length > 0 && !this.adminMode.fixed_stage_id) {
          this.adminMode.fixed_stage_id = stages[0].id;
        }
      },
      error: (err) => {
        console.error('Failed to load stages', err);
      }
    });
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

  private setError(target: 'logError' | 'regError' | 'usernameError' | 'passwordError' | 'apiKeyError' | 'adminModeError' | 'adminSessionError' | 'adminPlayersError', err: any, fallback: string): void {
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

  applyMode(): void {
    this.adminModeError = '';
    this.adminModeSuccess = '';
    this.admin.setMode(this.adminMode).subscribe({
      next: () => { this.adminModeSuccess = 'Game mode updated.'; this.cd.markForCheck(); },
      error: err => { this.setError('adminModeError', err, 'Failed to update mode'); },
    });
  }

  restartSession(): void {
    this.adminSessionError = '';
    this.adminSessionSuccess = '';
    this.admin.resetWorld().subscribe({
      next: () => { this.adminSessionSuccess = 'Session restarted.'; this.cd.markForCheck(); },
      error: err => { this.setError('adminSessionError', err, 'Failed to restart session'); },
    });
  }

  resetPlayers(): void {
    this.adminPlayersError = '';
    this.adminPlayersSuccess = '';
    this.admin.resetAllPlayers().subscribe({
      next: () => { this.adminPlayersSuccess = 'All players reset.'; this.cd.markForCheck(); },
      error: err => { this.setError('adminPlayersError', err, 'Failed to reset players'); },
    });
  }

  private loadProfile(): void {
    this.auth.getProfile().subscribe({
      next: p => { this.profile = p; this.newUsername = p.display_name ?? ''; this.cd.markForCheck(); },
      error: () => { this.auth.logout(); },
    });
  }
}

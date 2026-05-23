import { Injectable, signal } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { tap } from 'rxjs/operators';
import { Observable } from 'rxjs';

const BASE = '/api/v1';
const JWT_KEY = 'jwt';

export interface UserProfile {
  id: string;
  username: string;
  display_name: string;
  is_admin: boolean;
  created_at: string;
}

@Injectable({ providedIn: 'root' })
export class AuthService {
  private _token = signal<string | null>(localStorage.getItem(JWT_KEY));

  readonly token = this._token.asReadonly();

  constructor(private http: HttpClient) {}

  isLoggedIn(): boolean {
    return this._token() !== null;
  }

  /** Decode the username from the stored JWT without a network call. */
  usernameFromToken(): string | null {
    const token = this._token();
    if (!token) return null;
    try {
      const payload = JSON.parse(atob(token.split('.')[1]));
      return payload.username ?? null;
    } catch {
      return null;
    }
  }

  register(username: string, password: string): Observable<void> {
    return this.http.post<void>(`${BASE}/user/register`, { username, password });
  }

  login(username: string, password: string): Observable<{ token: string }> {
    return this.http.post<{ token: string }>(`${BASE}/user/login`, { username, password }).pipe(
      tap(res => this.setToken(res.token)),
    );
  }

  logout(): void {
    this.setToken(null);
  }

  getProfile(): Observable<UserProfile> {
    return this.http.get<UserProfile>(`${BASE}/user/profile`);
  }

  updateProfile(fields: Partial<{ display_name: string; password: string }>): Observable<void> {
    return this.http.patch<void>(`${BASE}/user/profile`, fields);
  }

  generateApiKey(): Observable<{ api_key: string }> {
    return this.http.post<{ api_key: string }>(`${BASE}/user/apikey`, {});
  }

  private setToken(token: string | null): void {
    this._token.set(token);
    if (token) localStorage.setItem(JWT_KEY, token);
    else localStorage.removeItem(JWT_KEY);
  }
}

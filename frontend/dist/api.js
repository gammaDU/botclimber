/**
 * api.js — thin wrappers around the Botclimber REST API.
 *
 * All functions return the parsed JSON body on success and throw an Error
 * (with message set to the server's error text) on failure.
 */

const BASE = '/api/v1';

// ── Auth token storage ────────────────────────────────────────────────────────

let _token = localStorage.getItem('jwt') ?? null;

export function getToken() { return _token; }

export function setToken(token) {
  _token = token;
  if (token) localStorage.setItem('jwt', token);
  else        localStorage.removeItem('jwt');
}

function authHeaders() {
  const h = { 'Content-Type': 'application/json' };
  if (_token) h['Authorization'] = `Bearer ${_token}`;
  return h;
}

async function request(method, path, body) {
  const res = await fetch(`${BASE}${path}`, {
    method,
    headers: authHeaders(),
    body: body !== undefined ? JSON.stringify(body) : undefined,
  });
  const text = await res.text();
  let data;
  try { data = JSON.parse(text); } catch { data = text; }
  if (!res.ok) throw new Error(data?.error ?? data ?? res.statusText);
  return data;
}

// ── User / Auth namespace (/api/v1/user) ──────────────────────────────────────

/** POST /api/v1/user/register */
export function register(username, password) {
  return request('POST', '/user/register', { username, password });
}

/** POST /api/v1/user/login  → returns { token } */
export function login(username, password) {
  return request('POST', '/user/login', { username, password });
}

/** GET /api/v1/user/profile */
export function getProfile() {
  return request('GET', '/user/profile');
}

/** PUT /api/v1/user/profile */
export function updateProfile(fields) {
  return request('PUT', '/user/profile', fields);
}

/** POST /api/v1/user/api-key  → returns { api_key } */
export function generateApiKey() {
  return request('POST', '/user/api-key');
}

// ── System namespace (/api/v1/system) ────────────────────────────────────────

/** GET /api/v1/system/leaderboard?mode=<mode> */
export function getLeaderboard(mode) {
  const q = mode ? `?mode=${encodeURIComponent(mode)}` : '';
  return request('GET', `/system/leaderboard${q}`);
}

/** GET /api/v1/system/snapshot  → full world snapshot */
export function getSnapshot() {
  return request('GET', '/system/snapshot');
}

/** GET /api/v1/system/players  → active players */
export function getActivePlayers() {
  return request('GET', '/system/players');
}

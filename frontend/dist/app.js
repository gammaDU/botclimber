/**
 * app.js — main entry point.
 *
 * Wires together:
 *  - Navigation (Leaderboard / Spectate / Account tabs)
 *  - Leaderboard view (fetches high scores, mode tabs)
 *  - Spectator view (GameStream + Renderer + session list + director cam)
 *  - Account view (login, register, profile, API key, change password)
 */

import { GameStream } from './stream.js';
import { Renderer }   from './renderer.js';
import {
  getToken, setToken,
  login, register, getProfile, updateProfile, generateApiKey,
  getLeaderboard,
} from './api.js';

// ── DOM refs ──────────────────────────────────────────────────────────────────

const views = {
  leaderboard: document.getElementById('view-leaderboard'),
  spectate:    document.getElementById('view-spectate'),
  account:     document.getElementById('view-account'),
};

const navBtns = {
  leaderboard: document.getElementById('btn-leaderboard'),
  spectate:    document.getElementById('btn-spectate'),
  account:     document.getElementById('btn-account'),
};

// Spectate
const connectionStatus  = document.getElementById('connection-status');
const activePlayersCount = document.getElementById('active-players-count');
const sessionList       = document.getElementById('session-list');
const followingLabel    = document.getElementById('following-label');
const btnDirectorCam    = document.getElementById('btn-director-cam');
const canvas            = document.getElementById('game-canvas');

// Leaderboard
const leaderboardBody   = document.getElementById('leaderboard-body');
const modeTabs          = document.querySelectorAll('.mode-tab');

// Account
const authForms         = document.getElementById('auth-forms');
const profilePanel      = document.getElementById('profile-panel');
const formLogin         = document.getElementById('form-login');
const formRegister      = document.getElementById('form-register');
const loginForm         = document.getElementById('login-form');
const registerForm      = document.getElementById('register-form');
const loginError        = document.getElementById('login-error');
const regError          = document.getElementById('reg-error');
const linkToRegister    = document.getElementById('link-to-register');
const linkToLogin       = document.getElementById('link-to-login');
const profileUsername   = document.getElementById('profile-username');
const apiKeyDisplay     = document.getElementById('api-key-display');
const btnGenerateKey    = document.getElementById('btn-generate-key');
const changePasswordForm = document.getElementById('change-password-form');
const pwError           = document.getElementById('pw-error');
const btnLogout         = document.getElementById('btn-logout');

// ── Navigation ────────────────────────────────────────────────────────────────

let activeView = 'leaderboard';

function showView(name) {
  for (const [k, el] of Object.entries(views))  el.classList.toggle('hidden', k !== name);
  for (const [k, el] of Object.entries(navBtns)) el.classList.toggle('active', k === name);
  activeView = name;

  if (name === 'leaderboard') loadLeaderboard(currentMode);
  if (name === 'spectate')    ensureStreamRunning();
  if (name === 'account')     refreshAccountView();
}

for (const [name, btn] of Object.entries(navBtns)) {
  btn.addEventListener('click', () => showView(name));
}

// ── Leaderboard ───────────────────────────────────────────────────────────────

let currentMode = 'checkpoint';

modeTabs.forEach(tab => {
  tab.addEventListener('click', () => {
    modeTabs.forEach(t => t.classList.remove('active'));
    tab.classList.add('active');
    currentMode = tab.dataset.mode;
    loadLeaderboard(currentMode);
  });
});

async function loadLeaderboard(mode) {
  leaderboardBody.innerHTML = '<tr><td colspan="4">Loading…</td></tr>';
  try {
    const data = await getLeaderboard(mode);
    const entries = Array.isArray(data) ? data : (data.entries ?? []);
    if (entries.length === 0) {
      leaderboardBody.innerHTML = '<tr><td colspan="4">No scores yet.</td></tr>';
      return;
    }
    leaderboardBody.innerHTML = entries.map((e, i) => `
      <tr>
        <td>${i + 1}</td>
        <td>${escHtml(e.username ?? e.player_name ?? '—')}</td>
        <td>${(e.height ?? e.max_height ?? 0).toFixed(1)}</td>
        <td>${escHtml(e.mode ?? mode)}</td>
      </tr>`).join('');
  } catch (err) {
    leaderboardBody.innerHTML = `<tr><td colspan="4">Error: ${escHtml(err.message)}</td></tr>`;
  }
}

// ── Spectator / Stream ────────────────────────────────────────────────────────

const renderer = new Renderer(canvas, {
  onFollowChange(playerId) {
    updateFollowingLabel(playerId);
    updateSessionListHighlight(playerId);
    btnDirectorCam.classList.toggle('active', renderer.directorCam);
  },
});

const stream = new GameStream({
  onStatus(status) {
    connectionStatus.className = `status-${status}`;
    connectionStatus.textContent =
      status === 'connected'    ? 'Connected'    :
      status === 'connecting'   ? 'Connecting…'  : 'Disconnected';
  },
  onReady(state) {
    renderer.setGameState(state);
    renderer.start();
    updateSessionList(state);
  },
  onEvent(ev) {
    updateSessionList(stream.state);
    activePlayersCount.textContent = `Players: ${Object.keys(stream.state.players).length}`;
  },
});

let streamStarted = false;

function ensureStreamRunning() {
  if (!streamStarted) {
    streamStarted = true;
    stream.connect();
  }
}

// Director cam button
btnDirectorCam.addEventListener('click', () => {
  renderer.enableDirectorCam();
  btnDirectorCam.classList.add('active');
});

function updateFollowingLabel(playerId) {
  if (!playerId || renderer.directorCam) {
    followingLabel.textContent = 'Director Cam';
  } else {
    const p = stream.state.players[playerId];
    followingLabel.textContent = p ? `Following: ${p.username ?? playerId}` : '';
  }
}

function updateSessionList(state) {
  const players = Object.values(state.players)
    .sort((a, b) => (b.height ?? 0) - (a.height ?? 0));

  activePlayersCount.textContent = `Players: ${players.length}`;

  sessionList.innerHTML = players.map(p => `
    <li class="${p.id === renderer.followedId ? 'following' : ''}"
        data-id="${escHtml(p.id)}"
        style="cursor:pointer">
      <span>${escHtml(p.username ?? p.id)}</span>
      <span>${(p.height ?? 0).toFixed(1)} m</span>
    </li>`).join('') || '<li style="color:var(--text-muted)">No active players</li>';

  sessionList.querySelectorAll('li[data-id]').forEach(li => {
    li.addEventListener('click', () => {
      renderer.followPlayer(li.dataset.id);
      btnDirectorCam.classList.remove('active');
    });
  });
}

function updateSessionListHighlight(playerId) {
  sessionList.querySelectorAll('li[data-id]').forEach(li => {
    li.classList.toggle('following', li.dataset.id === playerId);
  });
}

// ── Account ───────────────────────────────────────────────────────────────────

linkToRegister.addEventListener('click', e => {
  e.preventDefault();
  formLogin.classList.add('hidden');
  formRegister.classList.remove('hidden');
});

linkToLogin.addEventListener('click', e => {
  e.preventDefault();
  formRegister.classList.add('hidden');
  formLogin.classList.remove('hidden');
});

loginForm.addEventListener('submit', async e => {
  e.preventDefault();
  loginError.textContent = '';
  const username = document.getElementById('login-username').value.trim();
  const password = document.getElementById('login-password').value;
  try {
    const res = await login(username, password);
    setToken(res.token);
    refreshAccountView();
  } catch (err) {
    loginError.textContent = err.message;
  }
});

registerForm.addEventListener('submit', async e => {
  e.preventDefault();
  regError.textContent = '';
  const username = document.getElementById('reg-username').value.trim();
  const password = document.getElementById('reg-password').value;
  try {
    await register(username, password);
    // Auto-login after registration
    const res = await login(username, password);
    setToken(res.token);
    refreshAccountView();
  } catch (err) {
    regError.textContent = err.message;
  }
});

btnGenerateKey.addEventListener('click', async () => {
  try {
    const res = await generateApiKey();
    apiKeyDisplay.textContent = res.api_key ?? '—';
  } catch (err) {
    apiKeyDisplay.textContent = `Error: ${err.message}`;
  }
});

changePasswordForm.addEventListener('submit', async e => {
  e.preventDefault();
  pwError.textContent = '';
  const newPassword = document.getElementById('new-password').value;
  try {
    await updateProfile({ password: newPassword });
    pwError.style.color = 'var(--success)';
    pwError.textContent = 'Password updated.';
  } catch (err) {
    pwError.style.color = 'var(--error)';
    pwError.textContent = err.message;
  }
});

btnLogout.addEventListener('click', () => {
  setToken(null);
  refreshAccountView();
});

async function refreshAccountView() {
  if (!getToken()) {
    authForms.classList.remove('hidden');
    profilePanel.classList.add('hidden');
    return;
  }
  try {
    const profile = await getProfile();
    authForms.classList.add('hidden');
    profilePanel.classList.remove('hidden');
    profileUsername.textContent = profile.username ?? '—';
    apiKeyDisplay.textContent   = profile.api_key  ?? '(none — click Generate)';
  } catch {
    // Token invalid/expired
    setToken(null);
    authForms.classList.remove('hidden');
    profilePanel.classList.add('hidden');
  }
}

// ── Utilities ─────────────────────────────────────────────────────────────────

function escHtml(str) {
  return String(str)
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;');
}

// ── Bootstrap ─────────────────────────────────────────────────────────────────

showView('leaderboard');

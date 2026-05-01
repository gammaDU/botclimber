# Frontend Color Scheme

All colors are defined as CSS custom properties in `:root` inside `frontend/dist/style.css`.

## Color Tokens

| Token          | Value     | Role                                              |
|----------------|-----------|---------------------------------------------------|
| `--bg`         | `#0d0d0d` | Page background — near-black                      |
| `--surface`    | `#1a1a2e` | Card / header / panel background — dark navy      |
| `--accent`     | `#e94560` | Primary accent — vivid crimson-red (buttons, highlights, active states) |
| `--accent2`    | `#0f3460` | Secondary accent — deep navy-blue                 |
| `--text`       | `#e0e0e0` | Primary body text — light grey                    |
| `--text-muted` | `#888888` | Secondary / placeholder text — mid grey           |
| `--border`     | `#2a2a4a` | Borders and dividers — dark blue-grey             |
| `--success`    | `#4caf50` | Success state — green                             |
| `--error`      | `#e94560` | Error state — same crimson-red as `--accent`      |

## Typography Tokens

| Token    | Value                                  | Role                  |
|----------|----------------------------------------|-----------------------|
| `--font` | `'Segoe UI', system-ui, sans-serif`    | Default UI font stack |
| `--mono` | `'Consolas', 'Courier New', monospace` | Monospace / code font |

## Usage Highlights

- **Header** — `--surface` background, `--accent` title color, `--border` bottom border.
- **Nav buttons** — transparent with `--border` outline; hover shifts to `--accent` border; active state fills with `--accent`.
- **Canvas / game area** — sits on `--bg`, framed by `--border`.
- **Tables** — `--surface` header rows, alternating `--bg` / `--surface` body rows, `--border` cell borders.
- **Forms & inputs** — `--bg` fill, `--border` outline, focus ring switches to `--accent`.
- **Status indicators** — connected: `--success`; disconnected: `--error` (`--accent`).
- **Leaderboard rank badges** — gold `#ffd700` (1st), silver `#c0c0c0` (2nd), bronze `#cd7f32` (3rd); these are the only hard-coded colors outside the token system.

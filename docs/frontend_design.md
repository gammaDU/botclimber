# Frontend Design: Industrial Rework

This document defines the new visual design language for the Botclimber frontend — both the general application UI and the spectate/game canvas. The theme is **industrial, worn, and mechanical**: heavy metal textures, riveted panels, rust stains, steam-punk typography, and small kinetic animations that reward interaction.

---

## 1. Design Philosophy

| Principle | Description |
|-----------|-------------|
| **Industrial & Used** | Everything looks like it has been bolted together in a factory and run hard for years. Surfaces show wear: scratches, rust streaks, oil stains, uneven paint. |
| **Gamey, not Minimal** | Buttons have weight. Hover states animate. Fonts are expressive. The UI should feel like a physical control panel, not a SaaS dashboard. |
| **Kinetic Feedback** | Small CSS animations on interactive elements (button press, hover shimmer, gear spin) make the interface feel alive without being distracting. |
| **Readable First** | Despite the heavy aesthetic, text contrast must remain WCAG AA compliant. Decorative noise never obscures functional content. |

---

## 2. Color Tokens

Replace the existing tokens in `:root` inside `frontend/src/styles.css`:

| Token | Value | Role |
|-------|-------|------|
| `--bg` | `#1a1208` | Page background — dark, oily brown-black |
| `--surface` | `#2b1f0e` | Panel / card background — aged dark wood / iron |
| `--surface-alt` | `#221a0a` | Alternate row / inset surface |
| `--plate` | `#3d3020` | Raised metal plate — slightly lighter than surface |
| `--rust` | `#b5451b` | Primary accent — burnt rust orange (replaces crimson) |
| `--rust-light` | `#d4622a` | Hover / highlight variant of rust |
| `--iron` | `#6b6560` | Secondary accent — weathered iron grey |
| `--rivet` | `#8a7a60` | Rivet / bolt highlight — dull brass |
| `--text` | `#e8dcc8` | Primary text — aged parchment / warm off-white |
| `--text-muted` | `#9a8a70` | Secondary text — faded ink |
| `--border` | `#4a3820` | Borders — dark, scorched wood / iron seam |
| `--border-bright` | `#7a6040` | Active / focused border — polished edge |
| `--success` | `#5a8a3a` | Success — military olive green |
| `--error` | `#b5451b` | Error — same as `--rust` |
| `--warning` | `#c8922a` | Warning — amber / hot metal |
| `--steam` | `rgba(200,190,170,0.08)` | Subtle steam/fog overlay for depth |

### Leaderboard Rank Badges (unchanged concept, restyled)

| Rank | Color | Label |
|------|-------|-------|
| 1st | `#c8922a` (hot brass) | Gold → Brass |
| 2nd | `#8a9aaa` (cold steel) | Silver → Steel |
| 3rd | `#7a5030` (dark copper) | Bronze → Copper |

---

## 3. Typography

| Token | Value | Role |
|-------|-------|------|
| `--font-display` | `'Oswald', 'Impact', sans-serif` | Headings, nav labels, button text |
| `--font-body` | `'Special Elite', 'Courier New', serif` | Body text, descriptions — typewriter feel |
| `--font-mono` | `'Share Tech Mono', 'Consolas', monospace` | Code, coordinates, numeric readouts |

**Loading:** Import from Google Fonts:
```
Oswald:wght@400;600;700
Special Elite
Share Tech Mono
```

**Scale:**

| Use | Size | Weight | Transform |
|-----|------|--------|-----------|
| Page title | `2.4rem` | 700 | `uppercase`, `letter-spacing: 0.12em` |
| Section heading | `1.4rem` | 600 | `uppercase`, `letter-spacing: 0.08em` |
| Nav label | `1rem` | 600 | `uppercase`, `letter-spacing: 0.1em` |
| Body | `0.95rem` | 400 | none |
| Muted / caption | `0.8rem` | 400 | none |
| Numeric readout | `1.1rem` | 400 | `font-variant-numeric: tabular-nums` |

---

## 4. Surface & Texture Treatment

Textures are achieved with CSS only (no external image assets required for the base implementation):

### Metal Plate Panel
```css
background-color: var(--plate);
background-image:
  repeating-linear-gradient(
    0deg,
    transparent,
    transparent 2px,
    rgba(0,0,0,0.15) 2px,
    rgba(0,0,0,0.15) 3px
  );
box-shadow:
  inset 0 1px 0 rgba(255,255,255,0.06),
  inset 0 -1px 0 rgba(0,0,0,0.4),
  2px 2px 6px rgba(0,0,0,0.5);
border: 1px solid var(--border);
```

### Rust Streak (decorative pseudo-element)
```css
.panel::after {
  content: '';
  position: absolute;
  top: 0; right: 12px;
  width: 3px; height: 60%;
  background: linear-gradient(to bottom, var(--rust), transparent);
  opacity: 0.25;
  pointer-events: none;
}
```

### Rivet Corners
Add four small circles (4–6 px diameter, `background: var(--rivet)`, `border-radius: 50%`) at panel corners via `::before`/`::after` or a `.rivets` wrapper element.

---

## 5. Interactive Elements

### Buttons

**Default state:**
- Background: `var(--plate)`
- Border: `2px solid var(--border-bright)`
- Text: `var(--text)`, `font-family: var(--font-display)`, `uppercase`, `letter-spacing: 0.1em`
- Box shadow: `2px 2px 0 var(--border)` (hard offset — stamped metal look)
- Border-radius: `2px` (nearly square, industrial)

**Hover state (CSS transition: 120ms ease):**
- Border color → `var(--rust)`
- Text color → `var(--rust-light)`
- Box shadow → `3px 3px 0 var(--rust)` (shadow shifts, button "lifts")
- `transform: translate(-1px, -1px)`
- Add a subtle shimmer: `background-image: linear-gradient(135deg, transparent 40%, rgba(181,69,27,0.12) 50%, transparent 60%)` animated via `background-position` shift

**Active / pressed state:**
- `transform: translate(1px, 1px)`
- Box shadow → `0 0 0 var(--border)` (button "sinks")

**Primary / CTA button variant:**
- Background: `var(--rust)`
- Text: `#fff`
- Hover: background → `var(--rust-light)`, same lift animation

### Nav Tabs
- Inactive: `var(--surface)` background, `var(--border)` bottom border, `var(--text-muted)` text
- Active: `var(--plate)` background, `3px solid var(--rust)` bottom border, `var(--text)` text, small gear icon (SVG or `◈`) prepended
- Hover: border-bottom color → `var(--rust-light)`, text → `var(--text)`

### Form Inputs
- Background: `var(--surface-alt)`
- Border: `1px solid var(--border)`
- Font: `var(--font-mono)`
- Focus: border → `var(--rust)`, `box-shadow: 0 0 0 2px rgba(181,69,27,0.25)`
- Placeholder: `var(--text-muted)`

---

## 6. Animations

All animations respect `prefers-reduced-motion: reduce` — wrap in `@media (prefers-reduced-motion: no-preference)`.

| Name | Trigger | Effect | Duration |
|------|---------|--------|----------|
| `btn-lift` | Button hover | `translate(-1px,-1px)` + shadow shift | 120ms ease |
| `btn-press` | Button active | `translate(1px,1px)` + shadow collapse | 80ms ease |
| `shimmer` | Button hover | Diagonal highlight sweeps across | 400ms ease |
| `gear-spin` | Nav active tab | Small gear icon rotates 360° | 800ms ease-in-out, once |
| `rust-pulse` | Status indicator (connected) | Opacity pulses 0.6→1.0 | 2s ease-in-out, infinite |
| `steam-rise` | Canvas background | Faint vertical gradient drifts upward | 8s linear, infinite |
| `panel-flicker` | Page load | Surface panels fade in with slight brightness flicker | 300ms |

---

## 7. Header & Navigation

```
┌─────────────────────────────────────────────────────────────┐
│  ◈ BOTCLIMBER          [SPECTATE] [LEADERBOARD] [ACCOUNT]   │
│  ══════════════════════════════════════════════════════════  │
│  ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░  │
└─────────────────────────────────────────────────────────────┘
```

- Header background: `var(--surface)` with horizontal scan-line texture
- Title font: `var(--font-display)`, `2rem`, `var(--rust)`, `letter-spacing: 0.15em`
- Title prefix glyph `◈` in `var(--rivet)` — acts as a logo stand-in
- Bottom border: `3px solid var(--border)` with a `1px` inner highlight line in `rgba(255,255,255,0.04)`
- Nav buttons: see §5 Nav Tabs

---

## 8. Spectate Canvas — Game World

### 8.1 Background

The canvas background represents the interior of an industrial tower shaft:

- Base fill: `#0e0b06` (near-black, oily)
- Vertical gradient overlay: dark at top and bottom, slightly lighter in the middle (depth illusion)
- Faint repeating horizontal lines every ~80px in `rgba(255,255,255,0.02)` (structural beams)
- Optional: slow upward-drifting steam particles (small white ellipses, low opacity, CSS or canvas animation)

### 8.2 Platforms

Platforms are rendered as **industrial metal grating / walkways**:

| Property | Value |
|----------|-------|
| Fill | `#4a3820` (dark iron) |
| Top surface highlight | `1px` line in `#7a6040` (polished edge) |
| Bottom shadow | `2px` in `#1a1208` |
| Texture | Repeating diagonal hatch pattern in `rgba(0,0,0,0.2)` (grating) |
| Rust patches | Small `var(--rust)` rectangles at random positions, opacity 0.3 |
| Bolts | 4–6 px circles in `var(--rivet)` at platform ends |
| Moving platforms | Add a faint amber `var(--warning)` glow on edges + slow oscillation trail |

**Canvas drawing approach (2D context):**
```
1. fillRect with base iron color
2. strokeRect with highlight on top edge
3. Draw hatch lines across the surface
4. Draw rivet circles at corners/ends
5. Optionally scatter rust patches
```

### 8.3 Players (Bots)

Each player is rendered as a **small industrial cube** — a boxy machine casing with a transparent or grated front panel that reveals the mechanical internals (cogs, springs, a glowing core):

#### Anatomy (drawn in canvas 2D, ~20×20 px footprint)

The bot is fundamentally a **square cube** with an isometric-lite perspective hint (top face slightly lighter, right face slightly darker). The front face is a recessed viewport showing the internals.

```
  ┌────────────┐   ← top edge highlight (1px, rgba(255,255,255,0.18))
  │╔══════════╗│   ← outer casing (var(--plate), 20×20px rect)
  │║  ⚙  ◎  ║│   ← internals viewport: cog (left) + glowing core (right)
  │║ ╱╲  ╱╲  ║│   ← spring coils below cog (zigzag, var(--iron))
  │╚══════════╝│   ← inner bezel / viewport frame (var(--border-bright))
  └────────────┘   ← bottom shadow (2px, rgba(0,0,0,0.5))
```

**Layered drawing order (Canvas 2D):**

1. **Cube body** — `fillRect` 20×20 px with `var(--plate)`; right-face sliver (3px wide) filled at 70% brightness for depth; top-face sliver (3px tall) filled at 130% brightness
2. **Outer rivet corners** — four 3px circles in `var(--rivet)` at each corner of the face
3. **Viewport bezel** — inset `strokeRect` (2px inset on each side) in `var(--border-bright)`, 1px stroke; fill interior with `rgba(0,0,0,0.55)` (dark glass / grating)
4. **Cog** — 6px gear shape (or approximated with a circle + 6 small radial rects as teeth) in `var(--rivet)`, centered left of viewport; rotates continuously
5. **Glowing core** — 4px filled circle in the player's unique accent color, right of cog; surrounded by a soft radial glow (`shadowBlur: 6`, same color at 60% opacity)
6. **Spring coils** — two short zigzag lines (3 zigs, 2px wide) in `var(--iron)` below the cog, representing internal suspension; height varies by bot state
7. **Wear details** — one or two 1px scratch lines diagonally across the face in `rgba(255,255,255,0.06)`; optional small rust patch (`var(--rust)`, opacity 0.2) in a corner

#### Per-player color coding
- Each player gets a unique **accent color** for their glowing core and a matching tint on the viewport glow
- Suggested palette (cycled): `#d4622a`, `#5a8a3a`, `#c8922a`, `#4a7aaa`, `#9a5aaa`, `#aa5a5a`
- The cube body (`var(--plate)`) is the same for all bots; only the core color and glow differ

#### Animations
| State | Animation |
|-------|-----------|
| Idle / landed | Cog rotates slowly (1 RPM); core glow pulses gently (opacity 0.6→1.0, 2s); springs at rest |
| Jumping (ascending) | Cube squishes vertically (scaleY 0.85) on launch frame; cog spins faster; core brightens |
| Falling | Cube stretches slightly (scaleY 1.1); spring coils extend (drawn taller); core dims |
| Landing impact | Cube squishes hard (scaleY 0.75) then bounces back with ease-out-elastic over 200ms |
| Death / reset | Core flashes white then cuts to `var(--error)` red; cube shakes (±2px translate, 3 cycles); fades out over 400ms |

#### Name label
- Rendered above the bot in `var(--font-mono)`, `10px`, `var(--text)` with a dark background pill (`rgba(0,0,0,0.6)`)

### 8.4 Camera & Viewport

- The canvas has a dark vignette overlay (radial gradient, transparent center → `rgba(0,0,0,0.4)` edges) to frame the action
- Director Cam transitions: smooth lerp over 600ms, not instant cut
- A small **HUD overlay** in the top-left corner shows current followed player name in `var(--font-display)` with a `◈` prefix

---

## 9. Leaderboard & Tables

- Table container: `var(--surface)` background, `var(--border)` border, `2px` corner radius
- Header row: `var(--plate)` background, `var(--rust)` text, `var(--font-display)`, `uppercase`
- Body rows: alternating `var(--surface)` / `var(--surface-alt)`
- Row hover: `background: rgba(181,69,27,0.08)` — subtle rust tint
- Rank badge: small pill with brass/steel/copper colors (see §2), `var(--font-display)`, bold
- Height / time values: `var(--font-mono)`, right-aligned, `var(--text)`

---

## 10. Status Indicators

| State | Color | Animation |
|-------|-------|-----------|
| Connected (WebSocket) | `var(--success)` | `rust-pulse` (slow opacity pulse) |
| Disconnected | `var(--error)` | Static, no animation |
| Reconnecting | `var(--warning)` | Fast blink (0.5s) |

Indicator shape: small circle (8px) with a `2px` border in the same color at 50% opacity.

---

## 11. Responsive Considerations

- The spectate canvas scales to fill available width while maintaining aspect ratio
- On narrow viewports (<768px): nav collapses to icon-only tabs; leaderboard moves below canvas
- Font sizes scale down by ~15% on mobile; rivet decorations are hidden on very small screens

---

## 12. Implementation Notes

1. **CSS custom properties** — all tokens defined in `:root` in `frontend/src/styles.css`, replacing the existing set
2. **Google Fonts** — add `<link>` preconnect + stylesheet in `index.html`
3. **Canvas rendering** — bot anatomy and platform textures implemented in `spectate.component.ts` using the Canvas 2D API; no external sprite assets required for the base implementation
4. **Reduced motion** — all CSS `@keyframes` animations wrapped in `@media (prefers-reduced-motion: no-preference)`
5. **No framework dependency** — all visual effects achieved with CSS and Canvas 2D; no additional JS animation libraries needed
6. **Existing color token names** — where possible, keep token names (`--bg`, `--surface`, `--accent`, etc.) and only change values, to minimise refactoring across component stylesheets; new tokens (`--plate`, `--rust`, `--rivet`, etc.) are additive

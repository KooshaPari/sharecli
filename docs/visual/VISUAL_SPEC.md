# Visual specification — sharecli

Acceptance contract for visual polish (C10 L107). Surfaces: CLI splash, TUI, dashboard HTML, tray icons, docs.

## 1. Palette (Backbone-2)

| Token | Hex | Role |
|-------|-----|------|
| `--bb2-graphite` | `#0a0d12` | Page / terminal ground |
| `--bb2-panel` | `#161b22` | Inset panels |
| `--bb2-pulse-green` | `#3fb950` | Primary accent / healthy |
| `--bb2-sync-violet` | `#a371f7` | Sync / secondary accent |
| `--bb2-warm-amber` | `#d29922` | Warn / cooldown |

SoT: `assets/tokens.css`. Do not invent one-off hexes on new surfaces; mirror via `src/theme.rs`.

## 2. Typography

See [typography.md](typography.md). Dashboard/CLI prefer intentional mono stacks (JetBrains Mono → Fira Code → Cascadia Code → system mono).

## 3. Empty / zero-data

CLI and dashboard should state *what is missing* and the next command (e.g. start a process), not a blank panel.

## 4. Loading

Prefer content-shaped placeholders or pulse-green activity indicators; avoid generic browser spinners on branded surfaces.

## 5. Error / failure

Human-readable status + recovery hint; avoid raw red walls of stack traces in operator UI. Map severity to pulse-green (ok) / amber (warn) / distinct failure tone.

## 6. Major views (reference)

| View | Reference |
|------|-----------|
| Brand mark | `assets/brand/sharecli-icon.svg` |
| Motion mark | `assets/brand/sharecli-icon-animated.svg` |
| Identity demo | `docs/assets/identity/demo.svg` |
| CLI splash | textual regression in `tests/integration_cli.rs` |
| Tokens | `assets/tokens.css` |

## 7. Golden / regression (soft)

Today: splash string assertions. Follow-up: `tests/golden/` CLI snapshots and/or Playwright screenshots for dashboard.

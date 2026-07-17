# High contrast — forced-colors & prefers-contrast (soft)

Audit-v38 **C10 L104** (theming) + **L105** (brand cohesion) — documents current behavior, token pairs, and dashboard hex drift audit steps. Implementation of a dedicated high-contrast theme remains a soft follow-up.

## Media-query posture

| Query | Status | Where |
|-------|--------|-------|
| `prefers-color-scheme: light` | **Implemented** | `assets/tokens.css:32-39` — system-follow light pair when `data-theme` unset |
| `prefers-reduced-motion: reduce` | **Implemented** | `assets/tokens.css:42-47` — motion tokens collapse to `0ms` |
| `prefers-contrast: more` | **Not implemented** | Rubric gap: `audit/rubric/audit-30-pillar/audit-30-pillar-L17.md` |
| `forced-colors: active` | **Not implemented** | Windows High Contrast / `forced-colors` users get default dashboard hex |

### Expected behavior (soft goal)

When `prefers-contrast: more` or `forced-colors: active` is detected:

1. **Surfaces** — rely on `Canvas`, `CanvasText`, `ButtonText`, `Highlight` system colors instead of hard-coded dashboard hex (`src/dashboard.html`).
2. **Borders** — ensure focus rings and table dividers use `currentColor` or `ButtonBorder` so they survive palette inversion.
3. **Status** — connection dot and thermal badges keep text labels (already non-color-only); avoid `background` alone for state.
4. **CLI** — `NO_COLOR=1` path already strips ANSI; document as the terminal high-contrast mode.

Proposed CSS hook (not yet in repo):

```css
@media (forced-colors: active) {
  :root { color-scheme: light dark; }
  /* map dashboard chrome to system colors; wire data-theme=high-contrast */
}

@media (prefers-contrast: more) {
  :root[data-theme="high-contrast"],
  :root:not([data-theme]) {
    /* bump accent luminance; see token pair table below */
  }
}
```

## Token pairs (`assets/tokens.css`)

Dark and light pairs are the shipped themes. High-contrast pair is documented for audit traceability only.

### Dark (default `:root`)

| Token | Hex | Rust mirror (`src/theme.rs`) |
|-------|-----|------------------------------|
| `--bb2-graphite` | `#0a0d12` | `Tokens::BACKBONE2.graphite` |
| `--bb2-panel` | `#161b22` | `Tokens::BACKBONE2.panel` |
| `--bb2-pulse-green` | `#3fb950` | `Tokens::BACKBONE2.pulse_green` |
| `--bb2-sync-violet` | `#a371f7` | `Tokens::BACKBONE2.sync_violet` |
| `--bb2-warm-amber` | `#d29922` | `Tokens::BACKBONE2.warm_amber` |

### Light (`[data-theme="light"]` + `prefers-color-scheme: light`)

| Token | Hex | Rust mirror |
|-------|-----|---------------|
| `--bb2-graphite` | `#f6f8fa` | `Tokens::BACKBONE2_LIGHT.graphite` |
| `--bb2-panel` | `#ffffff` | `Tokens::BACKBONE2_LIGHT.panel` |
| `--bb2-pulse-green` | `#1a7f37` | `Tokens::BACKBONE2_LIGHT.pulse_green` |
| `--bb2-sync-violet` | `#8250df` | `Tokens::BACKBONE2_LIGHT.sync_violet` |
| `--bb2-warm-amber` | `#9a6700` | `Tokens::BACKBONE2_LIGHT.warm_amber` |

CLI: `sharecli --theme light version` (aliases: `bb2-light`, `light`). See `docs/visual/theming.md`.

### High-contrast pair (proposed — soft)

| Token | Proposed hex | Rationale |
|-------|--------------|-----------|
| `--bb2-graphite` | `#000000` | Maximum ground contrast |
| `--bb2-panel` | `#1a1a1a` | Separates chrome from ground |
| `--bb2-pulse-green` | `#00ff00` | System-HC-friendly accent (verify under `forced-colors`) |
| `--bb2-sync-violet` | `#e0b0ff` | Lighter violet for `prefers-contrast: more` |
| `--bb2-warm-amber` | `#ffcc00` | Warning legibility on black |

Wire as `[data-theme="high-contrast"]` + Rust `ThemeVariant::Backbone2HighContrast` when implemented.

## Dashboard hex audit steps (L105 evidence)

The embedded dashboard (`src/dashboard.html`) still uses inline hex. This checklist maps each value to the Backbone-2 SoT and records drift for brand-cohesion scoring.

### 1. Extract hex inventory

```bash
rg -o '#[0-9a-fA-F]{3,8}' src/dashboard.html | sort -u
```

### 2. Map to token SoT

| Dashboard hex | Role | Nearest token | Match? |
|---------------|------|---------------|:------:|
| `#1a1a2e` | `body` background | `--bb2-graphite` `#0a0d12` | drift |
| `#e2e8f0` | body text | — (no text token yet) | drift |
| `#a78bfa` | `h1` title | `--bb2-sync-violet` `#a371f7` | near |
| `#161b22` | table header bg | `--bb2-panel` | **match** |
| `#a371f7` | table header text | `--bb2-sync-violet` | **match** |
| `#22c55e` | connected / running | `--bb2-pulse-green` `#3fb950` | near |
| `#f59e0b` | memory column | `--bb2-warm-amber` `#d29922` | near |
| `#ef4444` | disconnected / error | — (no error token) | drift |
| `#94a3b8` | status label, pid | — | drift |
| `#cbd5e1` | table cells | — | drift |
| `#16213e` | row hover, focus bg | — | drift |
| `#2d2d4e` / `#1e1e3a` | borders | — | drift |
| `#4a5568` | empty-state copy | — | drift |

### 3. Contrast spot-check

Cross-check pairs against [`contrast.md`](./contrast.md). All listed dashboard pairs pass WCAG 2.2 AA normal text on the default dark chrome; re-run after token alignment.

### 4. Manual forced-colors pass (when implemented)

1. Windows: **Settings → Accessibility → Contrast themes** → enable a high-contrast theme.
2. Open `sharecli serve` dashboard at `http://127.0.0.1:PORT/`.
3. Confirm: table headers readable, focus ring visible, status dot has text label (`#status-label`).
4. Record pass/fail in release checklist; until then L105 hex drift remains partial.

### 5. CI / sync guards

| Check | Path |
|-------|------|
| Rust ↔ CSS hex lock | `theme::backbone2_constants_match_tokens_css` |
| WCAG ratios | `docs/a11y/contrast.md` |
| axe Level A | `.github/workflows/a11y.yml` |

## Related docs

- [Color contrast ratios](./contrast.md) — WCAG pairs for Backbone-2 tokens
- [Theming](../visual/theming.md) — dark/light CLI + CSS contract
- [C10 lane evidence](../../audit/.lane-c10/C10.md) — L104/L105 rubric scores

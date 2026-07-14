# Color contrast — Backbone-2 tokens

Documented contrast ratios for sharecli design tokens. Ratios computed per WCAG 2.2 relative luminance (1.4.3). **AA** requires ≥4.5:1 for normal text, ≥3:1 for large text (≥18pt or ≥14pt bold) and UI components (1.4.11).

## CSS / Rust token source

| Token | Hex | Defined in |
|-------|-----|------------|
| `--bb2-graphite` / `graphite` | `#0a0d12` | `assets/tokens.css`, `src/theme.rs:69` |
| `--bb2-panel` / `panel` | `#161b22` | `assets/tokens.css`, `src/theme.rs:70` |
| `--bb2-pulse-green` / `pulse_green` | `#3fb950` | `assets/tokens.css`, `src/theme.rs:71` |
| `--bb2-sync-violet` / `sync_violet` | `#a371f7` | `assets/tokens.css`, `src/theme.rs:72` |
| `--bb2-warm-amber` / `warm_amber` | `#d29922` | `assets/tokens.css`, `src/theme.rs:73` |

## Backbone-2 accent on backgrounds

| Foreground | Background | Ratio | AA normal | AA large / UI |
|------------|------------|------:|:---------:|:-------------:|
| `#3fb950` pulse green | `#0a0d12` graphite | **7.66:1** | pass | pass |
| `#3fb950` pulse green | `#161b22` panel | **6.81:1** | pass | pass |
| `#a371f7` sync violet | `#161b22` panel | **5.16:1** | pass | pass |
| `#d29922` warm amber | `#161b22` panel | **6.85:1** | pass | pass |

TUI thermal labels pair ratatui named colors with text (`GREEN` / `YELLOW` / `RED`) so state is not conveyed by color alone.

## Dashboard (`src/dashboard.html`) pairs

| Pair | Ratio | AA normal | Notes |
|------|------:|:---------:|-------|
| Body `#e2e8f0` on `#1a1a2e` | **13.84:1** | pass | Primary body copy |
| Cells `#cbd5e1` on `#1a1a2e` | **11.49:1** | pass | Table data |
| H1 `#a78bfa` on `#1a1a2e` | **6.27:1** | pass | Page title |
| Status `#94a3b8` on `#1a1a2e` | **6.65:1** | pass | Connection label |
| Running `#22c55e` on `#1a1a2e` | **7.49:1** | pass | Status column |
| Table header `#7c3aed` on `#16213e` | **2.79:1** | **fail** | Decorative uppercase 11px — tracked for token alignment |

### Known gap

Table header violet-on-panel (**2.79:1**) is below the 3:1 UI-component threshold. Remediation: align dashboard chrome with Backbone-2 `--bb2-sync-violet` on `--bb2-panel` (5.16:1) in a future visual pass (C10).

## Verification

Recompute ratios after token changes:

```bash
python -c "..."  # see audit/.lane-c09/C09.md evidence for the formula used 2026-07-13
```

Unit test `theme::backbone2_constants_match_tokens_css` keeps Rust hex literals in sync with `assets/tokens.css`.

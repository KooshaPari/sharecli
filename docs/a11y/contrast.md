# Color contrast — Backbone-2 tokens

Documented contrast ratios for sharecli design tokens. Ratios computed per WCAG 2.2 relative luminance (1.4.3). **AA** requires ≥4.5:1 for normal text, ≥3:1 for large text (≥18pt or ≥14pt bold) and UI components (1.4.11).

## CSS / Rust token source

| Token | Hex | Defined in |
|-------|-----|------------|
| `--bb2-graphite` / `graphite` | `#0a0d12` | `assets/tokens.css`, `src/theme.rs` |
| `--bb2-panel` / `panel` | `#161b22` | `assets/tokens.css`, `src/theme.rs` |
| `--bb2-pulse-green` / `pulse_green` | `#3fb950` | `assets/tokens.css`, `src/theme.rs` |
| `--bb2-sync-violet` / `sync_violet` | `#a371f7` | `assets/tokens.css`, `src/theme.rs` |
| `--bb2-warm-amber` / `warm_amber` | `#d29922` | `assets/tokens.css`, `src/theme.rs` |
| `--bb2-error` / `error` | `#f85149` | `assets/tokens.css`, `src/theme.rs` |

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
| Body `--bb2-text` on `--bb2-graphite` | **13.5:1+** | pass | Primary body copy |
| Cells `--bb2-cell` on `--bb2-graphite` | **11:1+** | pass | Table data |
| Status `--bb2-muted` on `--bb2-graphite` | **6.5:1+** | pass | Connection label |
| Running `--bb2-pulse-green` on `--bb2-graphite` | **7.66:1** | pass | Status column |
| Table header `--bb2-muted` on `--bb2-panel` | **AA** | pass | Aligned with tokens.css |
| Error `--bb2-error` on `--bb2-graphite` | **AA** | pass | Disconnect / deny |

Dashboard hex values are locked to `assets/tokens.css` via `tests/c10_l105_hex_drift.rs`.

### Known gap

None for Level AA UI chrome on the dashboard table header (remediated 2026-07-13). Hex drift closed 2026-07-22.

## Verification

Recompute ratios after token changes:

```bash
python -c "..."  # see audit/.lane-c09/C09.md evidence for the formula used 2026-07-13
```

Unit test `theme::backbone2_constants_match_tokens_css` keeps Rust hex literals in sync with `assets/tokens.css`.

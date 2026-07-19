# Golden visual tests

Audit-v38 **C10 L107** — screenshot / visual-regression plan. Text goldens for CLI/TUI are **shipped** (T-250); the dashboard PNG diff is a **blocking gate** (T-600), while hex-drift closure remains soft.

## Scope split

| Layer | Status | Path | CI |
|-------|--------|------|-----|
| CLI / TUI text goldens | **Shipped** | `tests/golden/*.txt` + `tests/golden_snapshots.rs` | `cargo test --test golden_snapshots` |
| Splash string regression | **Shipped** | `tests/integration_cli.rs` | default test suite |
| Rust ↔ CSS hex lock | **Shipped** | `src/theme.rs` (`backbone2_constants_match_tokens_css`) | unit tests |
| Dashboard PNG snapshots | **Hard** | `tests/visual/dashboard/` (committed Ubuntu baselines + manifest bytes) + `artifacts/playwright/` capture | `visual-soft.yml` (blocking diff) |
| Dashboard hex → token alignment | **Soft** | `src/dashboard.html` → `assets/tokens.css` | manual + future computed-style assert |

Eval-corpus JSON goldens (`docs/ops/eval-corpus.md`) are **not** screenshot tests — keep them separate from C10 visual gates.

## Existing text goldens (`tests/golden/`)

Harness: `tests/golden_snapshots.rs`. Regenerate:

```bash
UPDATE_GOLDENS=1 cargo test --test golden_snapshots
```

| Fixture | Surface | Asserts |
|---------|---------|---------|
| `cli_help.txt` | `sharecli --help` | Command inventory + `--theme` flag |
| `help.txt` | `sharecli --help` | Canonical help golden (L81.10; mirrors `cli_help.txt`) |
| `cli_ps_help.txt` | `sharecli ps --help` | FR-001 list surface documents `--project` |
| `thermal_green.txt` | TUI headless (ratatui `TestBackend` 80×24) | GREEN / ADMIT |
| `thermal_yellow.txt` | same | YELLOW / ADMIT |
| `thermal_red.txt` | same | RED / DENY |

Cross-refs: `docs/ops/governance/GAP-QA-MATRIX.md` (L30.7 / T-250), `audit/.lane-c10/C10.md` (L107).

## Token SoT (`assets/tokens.css`)

Palette and motion tokens are the CSS source of truth; Rust mirrors in `src/theme.rs`.

| Concern | Where | Guard |
|---------|-------|-------|
| Dark pair | `:root` `--bb2-*` | `backbone2_constants_match_tokens_css` |
| Light pair | `[data-theme="light"]`, `prefers-color-scheme: light` | CLI `--theme light`; extend unit test when light lock added |
| Type / motion | `--type-*`, `--motion-*` | `docs/visual/typography.md`, `docs/visual/motion.md` |
| Dashboard drift | inline hex in `src/dashboard.html` | see [Dashboard hex drift](#dashboard-hex-drift-l105) |

Do not add one-off hex on new surfaces; wire through tokens and mirror in Rust.

## Dashboard hex drift (L105)

The embedded dashboard still uses inline hex instead of `var(--bb2-*)`. Full inventory and mapping live in [`docs/a11y/high-contrast.md`](../a11y/high-contrast.md#dashboard-hex-audit-steps-l105-evidence).

Quick extract:

```bash
rg -o '#[0-9a-fA-F]{3,8}' src/dashboard.html | sort -u
```

| Dashboard hex | Nearest token | Match? |
|---------------|---------------|:------:|
| `#161b22` | `--bb2-panel` | **match** |
| `#a371f7` | `--bb2-sync-violet` | **match** |
| `#1a1a2e` | `--bb2-graphite` `#0a0d12` | drift |
| `#22c55e` | `--bb2-pulse-green` `#3fb950` | near |
| `#f59e0b` | `--bb2-warm-amber` `#d29922` | near |
| `#ef4444` | — (no error token) | drift |

**Hex-alignment sequence**

1. Refactor `dashboard.html` to `@import` or inline `assets/tokens.css` and replace hard-coded hex with `var(--bb2-*)`.
2. Add `--bb2-error` (or map errors to warm-amber + label) in `tokens.css` + `theme.rs`.
3. Re-run contrast spot-check (`docs/a11y/contrast.md`).
4. Capture new Playwright baselines (below).

## Dashboard PNG gate (Playwright)

`.github/workflows/visual-soft.yml` captures and compares all three committed baselines on `ubuntu-24.04`. The job has no `continue-on-error`; a missing capture, byte-lock mismatch, dimension mismatch, or over-threshold pixel delta blocks the change. `.github/workflows/playwright-soft.yml` remains the separate accessibility artifact capture. See `docs/a11y/playwright-viewports.md`.

### Phase A — baseline capture (current)

| Viewport | PNG | Surface |
|----------|-----|---------|
| `mobile-375` | `artifacts/playwright/mobile-375.png` | `src/dashboard.html` @ 375×812 |
| `tablet-768` | `artifacts/playwright/tablet-768.png` | same @ 768×1024 |
| `desktop-1280` | `artifacts/playwright/desktop-1280.png` | same @ 1280×800 |

Local:

```bash
cargo build --release -p sharecli
# terminal 1: sharecli serve (default :9000)
npx --yes playwright@1.49.0 install chromium
SHARECLI_DASH_URL=http://127.0.0.1:9000/ node scripts/a11y/playwright_viewports.mjs
```

### Phase B — committed baselines (scaffold → seed)

| Step | Status | Action |
|------|--------|--------|
| B1 | **Done** | `tests/visual/dashboard/` — `manifest.json` + README stub paths (#327) |
| B1b | **Done** | Committed `{mobile,tablet,desktop}.png` with manifest `bytes` lock; Ubuntu hard-gate reseed @ `91448b2` |
| B2 | **Done** | `scripts/visual/compare_screenshots.mjs` — pixelmatch diff vs manifest thresholds |
| B3 | **Done** | `visual-soft.yml` — blocking T-600 gate with deterministic Ubuntu capture |
| B4 | **Done** | `UPDATE_VISUALS=1` regen path in compare script + dashboard README |

Baseline contract (committed):

| Viewport | Baseline path | Artifact source |
|----------|---------------|-----------------|
| 375×812 | `tests/visual/dashboard/mobile.png` | `artifacts/playwright/mobile-375.png` |
| 768×1024 | `tests/visual/dashboard/tablet.png` | `artifacts/playwright/tablet-768.png` |
| 1280×800 | `tests/visual/dashboard/desktop.png` | `artifacts/playwright/desktop-1280.png` |

See `tests/visual/dashboard/README.md` + `manifest.json`. Keep names aligned with
`docs/a11y/playwright-viewports.md` committed-baseline policy.

### Deterministic capture contract

The blocking job uses locked npm dependencies and Playwright Chromium 1.49.0. It fixes locale (`en-US`), timezone (`UTC`), dark color scheme, reduced motion, CSS scale, and device scale factor 1. `SHARECLI_VISUAL_FIXTURE=1` replaces the live WebSocket with an empty-pool fixture before dashboard code runs, waits for the fixed `connected` state and fonts, and disables animations for the screenshot. Baselines must be regenerated on the same `ubuntu-24.04` job; Windows/macOS captures are diagnostic only.

### Phase C — theme matrix (planned)

After dashboard imports `tokens.css`:

| Theme | PNG set |
|-------|---------|
| dark (default) | 3 viewports |
| `data-theme="light"` | 3 viewports |
| `prefers-reduced-motion: reduce` | 1 desktop smoke |

Tray / Win32 surfaces stay out of scope until L112 signing hardens (C11).

## Pass criteria

| Gate | Current contract | Follow-up |
|------|------------------|-----------|
| Text goldens | 5/5 fixtures green in CI | unchanged |
| Hex lock | dark pair unit test | + light pair lock |
| Dashboard PNG | blocking manifest byte/dimension checks; pixel diff ≤ 0.1% @ 768/1280 and ≤ 0.2% @ 375 | theme matrix |
| Hex drift | documented in high-contrast.md | zero unmatched hex in `dashboard.html` |
| axe Level A | `a11y.yml` | unchanged |

L107 remains **3**: the hard promotion strengthens existing evidence but does not justify a score above the rubric maximum or change the C10 cluster total. T-600 closes the visual-gate remediation; dashboard hex drift remains separate.

## Related docs

- [VISUAL_SPEC.md](./VISUAL_SPEC.md) — acceptance contract (§7 golden follow-up)
- [theming.md](./theming.md) — dark/light CLI + CSS contract
- [high-contrast.md](../a11y/high-contrast.md) — hex audit + forced-colors posture
- [playwright-viewports.md](../a11y/playwright-viewports.md) — C09 viewport matrix
- [eval-corpus.md](../ops/eval-corpus.md) — JSON fixture goldens (separate from C10 screenshots)

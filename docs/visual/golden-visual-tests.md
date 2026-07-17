# Golden visual tests (soft)

Audit-v38 **C10 L107** — screenshot / visual-regression plan. Text goldens for CLI/TUI are **shipped** (T-250); dashboard PNG gates and hex-drift closure remain **soft**.

## Scope split

| Layer | Status | Path | CI |
|-------|--------|------|-----|
| CLI / TUI text goldens | **Shipped** | `tests/golden/*.txt` + `tests/golden_snapshots.rs` | `cargo test --test golden_snapshots` |
| Splash string regression | **Shipped** | `tests/integration_cli.rs` | default test suite |
| Rust ↔ CSS hex lock | **Shipped** | `src/theme.rs` (`backbone2_constants_match_tokens_css`) | unit tests |
| Dashboard PNG snapshots | **Soft** | planned `tests/visual/` or `artifacts/playwright/` baselines | `playwright-soft.yml` (artifact-only) |
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

**Soft gate sequence**

1. Refactor `dashboard.html` to `@import` or inline `assets/tokens.css` and replace hard-coded hex with `var(--bb2-*)`.
2. Add `--bb2-error` (or map errors to warm-amber + label) in `tokens.css` + `theme.rs`.
3. Re-run contrast spot-check (`docs/a11y/contrast.md`).
4. Capture new Playwright baselines (below).

## Soft PNG plan (Playwright)

Today: `.github/workflows/playwright-soft.yml` + `scripts/a11y/playwright_viewports.mjs` capture **artifacts only** (`continue-on-error`). See `docs/a11y/playwright-viewports.md`.

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

### Phase B — committed baselines (planned)

| Step | Action |
|------|--------|
| B1 | Seed `tests/visual/dashboard/{mobile,tablet,desktop}.png` from stable fixture data (empty + populated rows) |
| B2 | Add `scripts/visual/compare_screenshots.mjs` (pixel diff, maxDiffPixels threshold) or Playwright `toHaveScreenshot()` |
| B3 | Wire `visual-soft.yml` — soft required check; promote to hard after two green release cycles |
| B4 | Document `UPDATE_VISUALS=1` regen path (mirror `UPDATE_GOLDENS=1`) |

### Phase C — theme matrix (planned)

After dashboard imports `tokens.css`:

| Theme | PNG set |
|-------|---------|
| dark (default) | 3 viewports |
| `data-theme="light"` | 3 viewports |
| `prefers-reduced-motion: reduce` | 1 desktop smoke |

Tray / Win32 surfaces stay out of scope until L112 signing hardens (C11).

## Pass criteria (soft → hard)

| Gate | Soft (now) | Hard (target) |
|------|------------|---------------|
| Text goldens | 5/5 fixtures green in CI | unchanged |
| Hex lock | dark pair unit test | + light pair lock |
| Dashboard PNG | artifacts uploaded, no diff | pixel diff ≤ 0.1% @ 1280; ≤ 0.2% @ 375 |
| Hex drift | documented in high-contrast.md | zero unmatched hex in `dashboard.html` |
| axe Level A | `a11y.yml` | unchanged |

L107 stays **2** until Phase B baselines land in-repo; cluster top gap “golden visual tests” closes when PNG diff blocks merge.

## Related docs

- [VISUAL_SPEC.md](./VISUAL_SPEC.md) — acceptance contract (§7 golden follow-up)
- [theming.md](./theming.md) — dark/light CLI + CSS contract
- [high-contrast.md](../a11y/high-contrast.md) — hex audit + forced-colors posture
- [playwright-viewports.md](../a11y/playwright-viewports.md) — C09 viewport matrix
- [eval-corpus.md](../ops/eval-corpus.md) — JSON fixture goldens (separate from C10 screenshots)

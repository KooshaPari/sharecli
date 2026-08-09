# Playwright viewport soft gate

Audit-v38 **C09 L81.11**. Soft — `continue-on-error`.

Related: [`sr-checklist.md`](sr-checklist.md) (SR companion) ·
[`golden-visual-tests.md`](../visual/golden-visual-tests.md) (C10 L107 PNG plan) ·
[`responsive.md`](responsive.md) (TUI breakpoints).

## Matrix

| Viewport | Width | Height | PNG name | Surface |
|----------|------:|-------:|----------|---------|
| mobile | 375 | 812 | `mobile-375.png` | `src/dashboard.html` |
| tablet | 768 | 1024 | `tablet-768.png` | same |
| desktop | 1280 | 800 | `desktop-1280.png` | same |

Breakpoints already in CSS (`@media` 768 / 375). TUI compact width = 80 cols.

## Soft CI

`.github/workflows/playwright-soft.yml` boots `sharecli serve`, installs Playwright
Chromium, captures PNGs for the three widths, uploads artifacts. Failures do not
block merge.

| Control | Path | Gate strength |
|---------|------|---------------|
| Capture script | `scripts/a11y/playwright_viewports.mjs` | **Soft** |
| Workflow | `.github/workflows/playwright-soft.yml` | `continue-on-error: true` |
| Triggers | PR/push path filter on `dashboard.html` + a11y docs | Partial |
| Artifact | `playwright-viewports-<sha>` | 14-day retention |

## Baseline commit & artifact policy

**Today:** CI uploads ephemeral PNGs only. **No** committed baselines under
`tests/visual/` yet — pixel diff does not run on PRs.

### Artifact contract (phase A — current)

| Field | Value |
|-------|-------|
| Output dir | `artifacts/playwright/` (override: `SHARECLI_PW_OUT`) |
| Files | `mobile-375.png`, `tablet-768.png`, `desktop-1280.png` |
| CI artifact name | <span v-pre>`playwright-viewports-${{ github.sha }}`</span> |
| Retention | **14 days** (`upload-artifact` in `playwright-soft.yml`) |
| Serve URL | `SHARECLI_DASH_URL` default `http://127.0.0.1:9000/` |
| Playwright pin | `playwright@1.49.0` (workflow + local) |
| Fixture data | Empty pool on fresh `sharecli serve` (deterministic smoke) |

Artifacts are for **human triage** and release evidence — not merge gates until
phase B lands.

### Committed baseline policy (phase B — planned)

When promoting from artifact-only to in-repo baselines:

| Rule | Policy |
|------|--------|
| **SoT path** | `tests/visual/dashboard/{mobile,tablet,desktop}.png` |
| **Seed commit** | One PR that copies stable CI artifacts from `main` after visual change freeze |
| **Regen** | `UPDATE_VISUALS=1` env (mirror `UPDATE_GOLDENS=1` for text goldens) |
| **Diff tool** | `scripts/visual/compare_screenshots.mjs` or Playwright `toHaveScreenshot()` |
| **Threshold** | ≤ 0.1% pixel delta @ 1280; ≤ 0.2% @ 375 (see `golden-visual-tests.md`) |
| **Platform** | `ubuntu-24.04` + Playwright Chromium only (no macOS/Win baselines) |
| **Theme** | Dark default first; light matrix deferred until `tokens.css` import |

**Do not** commit `artifacts/playwright/` from local runs with ad-hoc pool data.
Seed from CI artifact on `main` HEAD or from a documented empty-pool fixture.

### Soft phases (no hard gate yet)

| Phase | Deliverable | Committed baselines? | Hard gate? |
|-------|-------------|----------------------|------------|
| **0 — today** | `playwright-soft.yml` + this doc + `playwright_viewports.mjs` | No | No |
| **1 — plan** | Baseline policy section + scorecard/worklog (FR-003) | No | No |
| **2 — seed** | Copy CI artifacts → `tests/visual/dashboard/*.png` | **Yes** | No (soft diff job) |
| **3 — soak** | Two green release cycles with soft pixel diff | Yes | No |
| **4 — hard** | Remove `continue-on-error`; wire `ci-success` / branch protection | Yes | **Yes — deferred** |

L81.11 stays **2** until phase 4 (or SR manual pass per `sr-checklist.md`).
C10 L107 PNG closure tracks the same baseline set — keep paths aligned with
`golden-visual-tests.md`.

### Maintainer checklist (phase 2 seed)

- [ ] Dashboard visual change merged and frozen for one `main` push.
- [ ] Download `playwright-viewports-<sha>` artifact from green `playwright-soft` run.
- [ ] Copy three PNGs into `tests/visual/dashboard/` with names above.
- [ ] Open PR titled `chore(visual): seed Playwright dashboard baselines (FR-003)`.
- [ ] Add soft diff step (`visual-soft.yml`) before promoting to required check.

### When to refresh baselines

| Trigger | Action |
|---------|--------|
| Intentional dashboard layout/CSS change | Regen all three viewports; note in PR body |
| Token/hex migration (`assets/tokens.css`) | Regen after contrast spot-check |
| Playwright major bump | Re-seed on `ubuntu-24.04`; pin version in workflow + doc |
| Flaky CI only | **Do not** refresh — fix serve health wait or font rendering first |

## Local

```bash
cargo build --release -p sharecli
npx --yes playwright@1.49.0 install chromium
# terminal 1:
./target/release/sharecli serve --bind 127.0.0.1:9000
# terminal 2:
SHARECLI_DASH_URL=http://127.0.0.1:9000/ node scripts/a11y/playwright_viewports.mjs
ls artifacts/playwright/
```

Optional: `just playwright-soft` when the recipe is present in `justfile`.

**Status:** soft plan (phase 1) · **FR:** FR-003 traceability · **Last sync:** 2026-07-17

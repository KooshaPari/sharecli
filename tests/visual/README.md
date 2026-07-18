# Visual regression baselines (soft)

Audit-v38 **C10 L107** — committed PNG baselines for dashboard screenshot diff.

| Path | Purpose |
|------|---------|
| [`dashboard/`](dashboard/) | Playwright viewport baselines (`mobile`, `tablet`, `desktop`) |
| [`dashboard/manifest.json`](dashboard/manifest.json) | Expected baseline filenames + viewport sizes |

Capture flow (phase A) writes ephemeral PNGs to `artifacts/playwright/` via
`scripts/a11y/playwright_viewports.mjs`. Phase B1b seeds stable copies into
`tests/visual/dashboard/` — see [`docs/visual/golden-visual-tests.md`](../docs/visual/golden-visual-tests.md).

Regen: `UPDATE_VISUALS=1 node scripts/visual/compare_screenshots.mjs` after capture.
Soft CI: `.github/workflows/visual-soft.yml`.

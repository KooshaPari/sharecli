# Dashboard PNG baselines

Committed screenshot baselines for `src/dashboard.html` at three viewports.
Aligned with C09 `playwright-viewports.md` and C10 `golden-visual-tests.md`.

## Baselines

| Baseline | Viewport | Committed path | Bytes | Phase-A artifact |
|----------|----------|----------------|------:|------------------|
| mobile | 375×812 | `mobile.png` | 11272 | `artifacts/playwright/mobile-375.png` |
| tablet | 768×1024 | `tablet.png` | 14145 | `artifacts/playwright/tablet-768.png` |
| desktop | 1280×800 | `desktop.png` | 13905 | `artifacts/playwright/desktop-1280.png` |

`manifest.json` lists the same contract (including `bytes` lock) for tooling.

## Blocking diff

`scripts/visual/compare_screenshots.mjs` compares fresh Playwright captures against
these baselines (pixelmatch + manifest byte check). CI: `.github/workflows/visual-soft.yml`
(hard/non-`continue-on-error`).

CI is the baseline authority: capture on `ubuntu-24.04` with
`SHARECLI_VISUAL_FIXTURE=1`. The fixture fixes browser locale/timezone/color/motion,
uses an empty-pool WebSocket state, waits for fonts, and disables screenshot animations.
Do not commit Windows or macOS captures as baselines.

## Regen

```bash
cargo build --release -p sharecli
# terminal 1: sharecli serve --bind 127.0.0.1:9000
npx --yes playwright@1.49.0 install chromium
SHARECLI_DASH_URL=http://127.0.0.1:9000/ SHARECLI_VISUAL_FIXTURE=1 node scripts/a11y/playwright_viewports.mjs
UPDATE_VISUALS=1 node scripts/visual/compare_screenshots.mjs
# refresh manifest.json bytes, then:
git add -f tests/visual/dashboard/*.png tests/visual/dashboard/manifest.json
```

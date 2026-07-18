# Dashboard PNG baselines (phase B scaffold)

Committed screenshot baselines for `src/dashboard.html` at three viewports.
Aligned with C09 `playwright-viewports.md` and C10 `golden-visual-tests.md`.

## Expected files (stub paths)

| Baseline | Viewport | Committed path | Phase-A artifact |
|----------|----------|----------------|------------------|
| mobile | 375×812 | `mobile.png` | `artifacts/playwright/mobile-375.png` |
| tablet | 768×1024 | `tablet.png` | `artifacts/playwright/tablet-768.png` |
| desktop | 1280×800 | `desktop.png` | `artifacts/playwright/desktop-1280.png |

`manifest.json` lists the same contract for tooling.

## Git policy

`*.png` in this directory is **gitignored** until a maintainer seeds from a green
`playwright-soft` CI artifact on `main`. Without a local browser / serve stack,
only the manifest + README ship in-repo (stub path scaffold).

To seed (after visual freeze):

```bash
# download playwright-viewports-<sha> artifact, then:
cp mobile-375.png tests/visual/dashboard/mobile.png
cp tablet-768.png tests/visual/dashboard/tablet.png
cp desktop-1280.png tests/visual/dashboard/desktop.png
git add -f tests/visual/dashboard/*.png
```

Soft pixel diff (`scripts/visual/compare_screenshots.mjs`, phase B2) is not wired yet.

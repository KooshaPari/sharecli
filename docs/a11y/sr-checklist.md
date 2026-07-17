# Screen reader checklist (soft)

Audit-v38 **C09 L81.11** — Playwright viewports companion.

## Dashboard (`serve` + `/dashboard`)

- [ ] Page `lang` attribute present (`dashboard.html`)
- [ ] Landmark regions: `main`, `nav` or `header` (`tests/a11y/dashboard_landmarks.rs`)
- [ ] Table headers associated with cells
- [ ] Status badges have text, not color-only (`thermal_*` golden strings)
- [ ] Focus order: skip link or logical tab order documented

## CLI / TUI

- [ ] `NO_COLOR` honored (`src/main.rs`)
- [ ] Quit keys documented (`q`, Ctrl-C) in `docs/a11y/README.md`
- [ ] Error messages plain-language (no glyph-only failures)

## CI evidence

| Check | Path |
|-------|------|
| axe Level A | `.github/workflows/a11y.yml` |
| Playwright viewports (soft) | `playwright-soft.yml` + `scripts/a11y/playwright_viewports.mjs` |
| Landmark unit tests | `tests/a11y/dashboard_landmarks.rs` |

Soft goal: L81.11 stays **2** until SR manual pass recorded in release checklist.

# Screen reader checklist (soft → acceptance evidence)

Audit-v38 **C09 L81.4** — Screen-Reader Compatibility (FR-004 NFR).
Companion to Playwright viewports (`L81.11`) and axe CI (`L81.1` / `L81.5`).

**Procedure date:** 2026-07-18  
**Operator:** agent (automated + static verification; no live VoiceOver/NVDA session)  
**Full evidence log:** [`sr-pass-evidence.md`](./sr-pass-evidence.md)

## Dashboard (`serve` + `/dashboard`)

- [x] Page `lang` attribute present (`src/dashboard.html` `<html lang="en">`)
- [x] Landmark regions: `main`, `nav` + labels (`tests/a11y/dashboard_landmarks.rs`)
- [x] Table headers associated with cells (`scope="col"` ×5 + `aria-label="Managed processes"`)
- [x] Status badges have text, not color-only (`aria-live` status/thermal strings; TUI `GREEN`/`YELLOW`/`RED` goldens)
- [x] Focus order: skip link documented (`docs/a11y/keyboard.md`) + present in HTML

## CLI / TUI

- [x] `NO_COLOR` honored (`src/main.rs` `is_no_color` + unit tests)
- [x] Quit keys documented (`q`, Ctrl-C) in `docs/a11y/README.md` / `keyboard.md`
- [x] Error messages plain-language (no glyph-only failures); machine modes: `list --json`, `report --format json`

## CI / automated evidence (this pass)

| Check | Path | Result (2026-07-18) |
|-------|------|---------------------|
| axe Level A | `npm run a11y:dashboard` → `scripts/a11y/axe-dashboard.mjs` | **PASS** — 0 violations (`wcag2a`/`wcag21a`/`wcag22a`) |
| Landmark + SR structure unit tests | `cargo test --test a11y` | **PASS** — lang/landmarks/live/`dashboard_sr_table_and_skip_link` |
| Keyboard docs | `docs/a11y/keyboard.md` | Present (skip link + TUI quit matrix) |
| Tray / brand SR labels | tray `accessibilityDescription`; SVG `role="img"` + `aria-label` | Present (static) |

## Remaining manual gaps (soft goal — not score-blocking)

Per rubric soft-optimizing goal for L81.4:

- [ ] Live **VoiceOver** (macOS) pass on `sharecli serve` dashboard: nav announces connection; table headers read in order
- [ ] Live **NVDA** (Windows) or **Orca** (Linux) pass when those surfaces are release-gated
- [ ] Per-release checkbox in release checklist that records VO/NVDA operator + date

**READY follow-up:** W9.3 — record live VoiceOver/NVDA results in release checklist (effort: M).

## Score posture

- **Acceptance (rubric L81.4):** met via alt/label/live-region/JSON evidence + documented SR procedure + automated gates → score **3**.
- **Soft goal:** live AT tool checklist per release remains open (above).

# Screen-reader procedure pass — evidence (C09 L81.4)

**FR:** FR-004 NFR (dashboard health cockpit accessibility)  
**Pillar:** audit-v38 C09 L81.4 — Screen-Reader Compatibility  
**Date:** 2026-07-18  
**Method:** Structured procedure against `docs/a11y/sr-checklist.md` using automated gates + static source verification.  
**Live AT tools:** VoiceOver / NVDA / Orca **not available** in this agent environment — **no fabricated AT transcripts**.

## Rubric mapping

| Acceptance item (X-ax-L81-L95 L81.4) | Evidence |
|-------------------------------------|----------|
| Images: non-empty `alt` or `aria-hidden` | No `<img>` in `src/dashboard.html`; decorative status-dot uses `aria-hidden="true"` |
| Form inputs labeled | Dashboard is read-only (no form controls) |
| Icon-only buttons `aria-label` | No `<button>` elements on dashboard |
| Live regions for dynamic updates | `aria-live="polite"` on status, thermal, last-update |
| Decorative SVGs / named graphics | Brand SVG: `role="img"` + `aria-label` + `<title>`/`<desc>` |
| CLI/TUI structured + JSON modes | `list --json`; `report --format json\|text`; TUI text level labels |
| Docs reference SR procedure | `docs/a11y/README.md` + this file + `sr-checklist.md` |

## Commands run (verbatim results)

### axe-core (WCAG 2.x Level A)

```text
$ npm run a11y:dashboard
axe dashboard scan — tags: wcag2a, wcag21a, wcag22a; violations: 0

PASS: zero axe violations for WCAG 2.x Level A tags
```

### Landmark / SR structure unit tests

```text
$ cargo test --test a11y
running 4 tests
test dashboard_landmarks::dashboard_announces_live_status ... ok
test dashboard_landmarks::dashboard_has_responsive_breakpoints ... ok
test dashboard_landmarks::dashboard_has_lang_and_landmarks ... ok
test dashboard_landmarks::dashboard_sr_table_and_skip_link ... ok

test result: ok. 4 passed; 0 failed; …
```

*(Fourth test added in this pass: table `aria-label`, `aria-labelledby`, `scope="col"`, skip link, no bare `<img>`.)*

### Static greps (evidence model)

- `aria-label` / `aria-live` / `aria-hidden` / `aria-labelledby` present on dashboard + brand SVG (≥5 hits).
- Zero `<img ` without `alt` under `src/`.
- Zero icon-only `<button>` on dashboard.
- CLI `--json` / `--format json` present in `src/main.rs`.

## Checklist completion

See [`sr-checklist.md`](./sr-checklist.md) — all automated/static rows checked; live VoiceOver/NVDA rows remain open under soft goal.

## Remaining gaps (honest)

1. **No live VoiceOver/NVDA session** recorded for this release train.
2. Thermal status still prefixes emoji (`🌡️` / `⚠️`); adjacent text labels mitigate SR risk but emoji noise is a soft polish item.
3. Per-release AT checkbox not yet wired into a published release checklist template.

**READY:** W9.3 — operator VoiceOver (macOS) + optional NVDA pass; attach dated notes to release checklist.

## Score decision

Bump **L81.4 2 → 3**: acceptance criterion + evidence model satisfied without inventing live AT output. Soft-optimizing “VO/NVDA per release” stays tracked as READY (does not block score 3 per WORKER-SPEC).

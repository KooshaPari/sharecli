## Summary

Wave17 **Plan 796 (T-910)** ships C09 L81.12 (history command) and L81.15 (CTA token system), lifting C09 from **42/45 93% A** to **44/45 98% A**.

| Field | Value |
|-------|-------|
| Source | `e770132f` |
| Base | `0509a52` (main post #788) |
| C09 cluster | **42/45 93% A → 44/45 98% A** |
| Overall weighted | **92.6% A → 93.1% A** |
| Overall unweighted | **91.5% A → 92.6% A** |
| Tier-1 | **92.6% A → 93.4% A** |

## What changed

### C09 L81.12 — Recognition Over Recall (`src/commands/history.rs`)

JSONL-backed CLI invocation history with:
- `append_to()` — write entry to explicit path
- `read_recent()` — tail N entries
- `clear()` — truncate file
- `format_entry()` — human-readable timestamp + status
- XDG_STATE_HOME compliant (`$XDG_STATE_HOME/sharecli/history.jsonl`)
- Self-contained epoch→civil-date converter (no chrono dependency)

CLI wiring: `Commands::History { limit, json, clear }` with `--limit 20`, `--json`, `--clear`.

### C09 L81.15 — CTA Token System (`assets/tokens.css` + `src/dashboard.html`)

Four new CSS custom properties across dark + light + prefers-color-scheme:
- `--bb2-cta-primary: #3fb950` (pulse-green) + `--bb2-cta-primary-text: #0a0d12`
- `--bb2-cta-secondary: #a371f7` (sync-violet) + `--bb2-cta-secondary-text: #ffffff`

Dashboard button classes: `.cta-primary` / `.cta-secondary` with hover opacity transition.

### FR-003 acceptance gates (`tests/c09_l81_recognition_cta.rs`, 10/10 PASS)

5 history gates + 5 CTA gates covering module existence, roundtrip, limit, Serialize/Deserialize, append_to API, CSS token presence, dark+light coverage, hex values, and dashboard class wiring.

## Files (12, +536 / -19)

- `src/commands/history.rs` — NEW (241 lines)
- `src/commands/mod.rs` — pub mod history
- `src/main.rs` — Commands::History variant + handler
- `assets/tokens.css` — CTA tokens (dark + light + media query)
- `src/dashboard.html` — .cta-primary / .cta-secondary classes
- `tests/c09_l81_recognition_cta.rs` — NEW (189 lines, 10 tests)
- `audit/.lane-c09/C09.md` — L81.12 2→3, L81.15 2→3, 44/45 98%
- `audit/SCORECARD-v38.md` — weighted 93.1%, Pin 0509a52
- `docs/ops/governance/WBS-PHASED.md` — C09 98% DONE
- `docs/ops/governance/GAP-QA-MATRIX.md` — L81.12 + L81.15 Closed
- `docs/ops/governance/RC-audit-v38-80B.md` — Pin 0509a52, 93.1%
- `WORK_DAG.md` — T-910 DONE

## Verification

- `cargo test --locked --test c09_l81_recognition_cta`: **10/10 PASS**
- `cargo check --tests --locked`: clean (warnings only)
- All governance files claim-locked to pin `e770132`

## FR-003 / C09 L81.12 + L81.15

Traces to FR-003 (coverage/traceability, Wave17 thesis residual). C09 moves from 93% to 98% A. Weighted overall reaches **93.1% A**.

# Friction log — sharecli

Known user/agent friction. Link new entries to an FR ID and optionally a
`WORK_DAG.md` task. Prefer filing a GitHub issue with label `friction:UX`
when the fix is non-trivial.

| Date | Surface | Friction | FR | Severity | Status | Follow-up |
|------|---------|----------|----|----------|--------|-----------|
| 2026-07-10 | Spec | Root `FUNCTIONAL_REQUIREMENTS.md` used FR-PROC/SESSION grammar; agents could not claim FR-NNN tasks | FR-001..005 / L30.1 | High | Mitigated this PR | Keep root + `docs/specs/FR.md` in sync |
| 2026-07-10 | Backlog | `PLAN.md` was multi-week phases only — no ≤1-day claimable DAG | L30.2 | High | Mitigated (`WORK_DAG.md`) | Claim T-200+ |
| 2026-07-10 | Tests | TRACEABILITY lists `fr002`–`fr005` files that are not on disk | FR-002..005 | High | Open | T-200..T-230 |
| 2026-07-10 | Journeys | `docs/journeys/quick-start.md` still shows library `Client` snippet, not CLI | FR-001 / FR-002 | Med | Open | T-240 + journey rewrite |
| 2026-07-10 | Agent index | Missing `llms.txt` / thin `AGENTS.md` ops checklist | L30.4 / L30.11 | Med | Mitigated this PR | — |
| 2026-07-10 | Toolchain | No `rust-toolchain.toml`; agents could drift from CI stable | L30.5 | Low | Mitigated this PR | — |
| 2026-07-10 | PR quality | Template asked for FR but CI did not enforce | L30.8 | Med | Mitigated (`pr-lint.yml`) | Branch protection may require the check |

## How to add an entry

1. Reproduce the friction (CLI command + expected vs actual).
2. Map to nearest `FR-NNN` (or open a new FR if none fits).
3. Add a row above; open issue or claim a `WORK_DAG.md` task.
4. Prefer an unhappy-path test (`*_invalid_*` / `*_missing_*`) when fixing.

# AgilePlus worklog — sharecli

Machine-oriented append-only log. Prefer Status tokens matching WORK_DAG / GAP-QA.

## 2026-07-13 (C08 jwt bench-gate)

- Status: DONE — `jwt_auth_validate` in soft/`bench-gate`/nightly; baseline `jwt_validate_rs256` (BENCH-4)
- Score: C08 stays 60% C (18/30); overall stays 68% C
- FR: FR-012
- Next: hyperfine JSON CI artifact; axe CI (C09)

## 2026-07-13 (C08 eval lift)

- Status: DONE — `jwt_auth_validate` Criterion bench (FR-012); `docs/eval/GOVERNANCE.md`; L71+L80→3
- Score: C08 → 60% C (18/30); overall → 67% C
- Next: wire jwt bench into `bench-gate`; C09 AX lift

## 2026-07-13 (C06 supply-chain lift)

- Status: DONE — L52 repro-check (`scripts/repro-check.sh`, `repro-check.yml`, `just repro-check`); L55 deny sources + audit.toml sync; L56 cosign roadmap in `docs/slsa.md`
- Score: C06 → 67% C (20/30); overall → 67% C
- FR: FR-002 (config/build determinism) for repro gate
- Next: SLSA L3 / hermetic builds; GHCR+cosign when publish lands

## 2026-07-13 (C09 a11y lift)

- Status: DONE — W7.1–W7.4: dashboard landmarks, `docs/a11y/*`, TUI `is_quit_key` tests
- Score: C09 → 67% C (30/45); L81.1/L81.2/L81.3/L81.5 1→2; overall → 68% C
- FR: FR-004 status/recovery cited in `docs/a11y/status-and-recovery.md`
- Next: axe CI for dashboard; responsive TUI (L81.11)

## 2026-07-13 (W5.3 threat review)

- Status: DONE — post-federation STRIDE refresh; SECURITY.md aligned; checklist signed
- Score: C04 L39 remains 3; C04 60% C unchanged
- Next READY: C08 eval / C09 a11y; W4.3 signing Blocked

## 2026-07-13 (W5.2 retention + burn)

- Status: DONE — audit JSONL size rotation; AuthN unauthorized counter + burn alert
- Score: C02 → 80% B (L23+L27); overall → 66% C
- Next: W5.3 threat review; C08/C09 lifts

## 2026-07-13 (W5.1 JWT AuthN)

- Status: DONE — FR-012 JWT/JWKS for `serve`; L21 2→3
- Score: C02 → 73% C; overall → 66% C
- Next: W5.2 audit retention; W5.3 threat review; C08/C09 lifts

## 2026-07-13 (W4.2 brew)

- Status: DONE — v0.3.0 darwin/linux assets attached; Formula sha256 filled
- Score: C11 → 67% C; overall 65% C
- Next: W5.1 federated AuthN; L112 signing Blocked on Apple secrets

## 2026-07-13 (C06 release pin)

- Status: DONE — fixed broken `upload-artifact` SHA; SOURCE_DATE_EPOCH on release; audit.toml yanked=warn
- Score: C06 → 60% C (L60 2→3); overall 65% C
- Next: re-dispatch Release → fill brew sha; W5.1 federated AuthN

## 2026-07-13 (T-300)

- Status: DONE — T-300 unhappy-path (`tests/fr_invalid_missing_friction.rs`)
- Score: C03 → 92% A; W3.5 DONE; overall 65% C
- Next READY: T-310 C03 polish / Wave4 brew; brew sha still Blocked

## 2026-07-13 (T-250)

- Status: DONE — T-250 golden fixtures (`tests/golden/` ×5 + `golden_snapshots.rs`)
- Score: C03 → 89% B; overall → 65% C
- Next READY: T-300 unhappy-path; brew sha still Blocked

## 2026-07-13 (T-240)

- Status: DONE — T-240 outside-in journey (`tests/quick_start_journey.rs`)
- Next READY: T-250 golden / T-300 unhappy-path; brew sha still Blocked

## 2026-07-13 (T-230)

- Status: DONE — T-230 FR-005 acceptance (`tests/fr005_*.rs`)
- Next READY: T-240 journey / T-300 unhappy-path; brew sha still Blocked

## 2026-07-12 (T-220)

- Status: DONE — T-220 FR-004 acceptance (`tests/fr004_*.rs`)
- Next READY: T-230 FR-005; brew sha still Blocked

## 2026-07-12 (T-210)

- Status: DONE — T-210 FR-003 acceptance (`tests/fr003_*.rs`)
- Status: DONE — T-260 claim-lock protocol in AGENTS.md
- Status: DONE — T-270 local loop budgets (`docs/ops/LOCAL_LOOP_BUDGETS.md`)
- Next READY: T-220 FR-004; W4.2 brew sha still Blocked

## 2026-07-12

- Status: DONE — T-200 FR-002 acceptance tests (`tests/fr002_*.rs`)
- Status: DONE — THREAT_MODEL.md (C04 L39 / C02 L20)
- Status: DONE — release.yml unsigned GH Release attach + SBOM in-archive (C11 L118 / C04 L32)
- Status: DONE — Cargo.toml rust-version 1.85 (C11 L119)
- Status: DONE — WBS-PHASED + GAP-QA-MATRIX under docs/ops/governance/
- Status: DONE — WORK_DAG T-100..T-160 + T-200 flipped DONE; audit_scorecard.json synced to 64% C
- Next READY: T-210 FR-003; W4.2 brew sha (Blocked on first tagged attach)

## 2026-04

- Stub worklog created.

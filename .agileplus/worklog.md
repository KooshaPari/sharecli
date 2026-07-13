# AgilePlus worklog — sharecli

Machine-oriented append-only log. Prefer Status tokens matching WORK_DAG / GAP-QA.

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

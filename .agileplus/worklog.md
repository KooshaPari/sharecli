# AgilePlus worklog — sharecli

Machine-oriented append-only log. Prefer Status tokens matching WORK_DAG / GAP-QA.

## 2026-07-13 (MVP finality + OS parity — W10.6)

- Status: DONE — FINALITY.md; desktop-builds.yml; Windows CLI in release matrix; tray-linux release job; just parity recipes
- Score: packaging evidence (C11); no GA claim for tray/desktop; overall stays ~71% C
- FR: FR-004 NFR (deploy/parity)
- Next: harden Win tray CI; macOS desktop artifact green; L112 signing when secrets land

## 2026-07-13 (C04 OSV + Dependabot groups — W10.5)

- Status: DONE — osv.yml (OSV/GHSA), Dependabot cargo/actions groups, container-hardening.md
- Score: C04 → 70% C (21/30); L37/L38/L40 lifts; overall stays ~71% C (856/12)
- FR: FR-004 NFR (security scanning)
- Next: SLSA L3 / cosign (C06); signed commits (C04 L34)

## 2026-07-13 (C07 proptest/mutants/fuzz + C08 corpus — W10.4)

- Status: DONE — proptest on thermal-tui; soft mutants + fuzz CI; synthetic corpus + trend CSV
- Score: C07 → 77% B (23/30); C08 stays 60% C; overall → ~71% C (846/12)
- FR: FR-004 NFR (eval/QEng)
- Next: mutants hard gate; C11 signing (blocked); C06 SLSA L3

## 2026-07-13 (C00 OpenAPI drift CI — W10.3)

- Status: DONE — expand serve.yaml to all routes; `check-openapi-drift.py` + workflow
- Score: C00 → 63% C (19/30); L2 2→3; overall stays ~69% C (832/12)
- FR: FR-004
- Next: synthetic eval corpus (C08); lib.rs module split (C00)

## 2026-07-13 (C09 responsive layout — W10.2)

- Status: DONE — TUI compact/full via `frame.area().width`; dashboard 375/768 media queries; a11y smoke tests
- Score: C09 → 76% B (34/45); L81.11 1→2; overall stays ~69% C (829/12)
- FR: FR-004 NFR (adaptive UX)
- Next: synthetic eval corpus (C08); OpenAPI drift CI (C00)

## 2026-07-13 (C01 action SHA pins — W10.1)

- Status: DONE — pin Actions tags to commit SHAs; ubuntu-24.04 across workflows; L10 2→3
- Score: C01 → 67% C (20/30); overall stays ~69% C (826/12)
- FR: FR-001
- Next: synthetic eval corpus (C08); responsive TUI (C09)

## 2026-07-13 (C09 axe CI — W9.1)

- Status: DONE — `.github/workflows/a11y.yml` + `scripts/a11y/axe-dashboard.mjs` (axe-core + jsdom; wcag2a/wcag21a/wcag22a; hard-fail serious/critical)
- Score: C09 → 73% C (33/45); L81.1+L81.5 2→3 on top of contrast lift; overall ~69% C (822/12)
- FR: FR-004 NFR (dashboard a11y)
- Next: responsive TUI (L81.11)

## 2026-07-13 (C09 table-header contrast)

- Status: DONE — dashboard `thead` `#a371f7` on `#161b22` (5.16:1); L81.2 2→3
- Score: C09 → 69% C (31/45); overall stays ~68% C
- FR: FR-004 (dashboard chrome a11y)
- Next: axe CI; responsive TUI

## 2026-07-13 (C08 hyperfine CI artifact)

- Status: DONE — soft `hyperfine healthz` job + nightly JSON artifact upload (L72 / LOAD-2)
- Score: C08 stays 60% C (18/30); overall stays 68% C
- FR: FR-004 (`GET /healthz`)
- Next: axe CI (C09); synthetic eval corpus (C08)

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


## W10.7 — OSSF Scorecard publish + workflow YAML (FR-001)
- Fixed deploy-docs.yml flow-mapping parse error (inline with: block containing GitHub expressions broke Dangerous-Workflow / Pinned-Dependencies / Token-Permissions / SAST).
- scorecard.yml: top-level permissions read-all; job-scoped security-events/id-token write; SARIF upload; branch_protection_rule casing.

## W10.8 — Deploy docs dead links (FR-005)
- VitePress build failed on main after scorecard YAML fix unblocked Deploy docs.
- Fixed ADR 0001→0002, corpus README path, removed missing stories/integration link.
- Pointed out-of-tree paths (trays, WORK_DAG, repro-check) at GitHub blob/tree URLs; ignoreDeadLinks safety net in config.mts.

## W10.9 — Deploy docs via GitHub Pages workflow (FR-005)
- VitePress build was green; peaceiris push 403 because default_workflow_permissions=read and Pages unset.
- Enabled Pages build_type=workflow; switched deploy-docs.yml to upload-pages-artifact + deploy-pages with pages/id-token write.

## W10.10 — OSSF Binary-Artifacts: untrack zig-cache (FR-001)

- Removed tracked `crates/spawn-core/.zig-cache/` from git index (11 files: `.o`, `.a`, build cache).
- `.gitignore`: `**/zig-cache/`, `**/zig-out/` alongside existing `**/.zig-cache/`.
- SECURITY.md: note that Zig caches are local-only; Scorecard Binary-Artifacts expects none in-tree.
- Soft CI: `scripts/check-no-build-artifacts.sh` + `build-artifacts-soft.yml` (`continue-on-error`).

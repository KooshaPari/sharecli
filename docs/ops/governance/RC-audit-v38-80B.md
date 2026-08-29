# Release Candidate — audit-v38 grade A (~91.9% tier-1)

**Status:** SOFT RC (evidence stack; not a product GA claim)
**Pin commit:** `70893b3` (`main` after #785 governance lock + #784 Wave17 Plan 794 + #783 governance lock post #782 Wave17 Plan 793 + #781 Wave17 Plan 782 + #780 Wave17 Plan 776 attempt 2 + #777 Wave17 Plan 778b SLSA L3 re-pin + #776 Wave17 Plan 777 SLSA L3 generator + #775 Wave17 T-810 `--lib` coverage pin; **Plan 794 (T-890, C02 L26) follow-up** with `tests/c02_l26_resilience.rs` 10 FR-003 acceptance gates + `src/retry.rs` + `src/backoff.rs` u128 overflow fix; **Plan 795 (T-900, C07 L68) follow-up** with `scripts/flake_tracker.py` + `audit/.flake-tracker/` operations runbook + `.github/workflows/flake-tracker.yml` + `tests/c07_l68_flake_tracker.rs` 6/6 PASS)
**Scorecard:** `audit/SCORECARD-v38.md` — weighted **92.6% A**, unweighted **91.5% A** (sum 1098 / 12), tier-1 **92.6% A** (post Plans 776 attempt 2 + 777 + 778b + 782 + 793 + 794 + **795**; pre-Plan 776 attempt 2: unweighted 91.3%, weighted 91.2%; pre-Plan 793: unweighted 91.75%, weighted 91.8%, tier-1 91.9%; pre-Plan 794: weighted 92.0%, unweighted 91.0%, tier-1 92.0%; pre-Plan 795: weighted 92.3%, unweighted 91.25%, tier-1 92.4%). `TEST_COVERAGE_MATRIX.md` lib pin **77.34%** @ `fa887e9` shipped; workspace pin **80.51%** @ `5d8dc08` retained as historical evidence. C04 L34 2→3 on verified merge commit evidence (3 squash-merge commits on `main` `verified: true` via GitHub web-flow signing). C05 L49 2→3 on Grafana provisioning as code (Plan 782). C06 L53 2→3 SLSA L3 generator landed in `release-attestation.yml`; **Wave17 Plan 778b** re-pins generator from `@v2` to commit SHA `5a775b367a56d5bd118a224a811bba288150a563` for digest-pinned L3 hardening. **C11 L111 1→2** on soft upgrade probe (Plan 793, no install path, no network egress). **C02 L26 2→3** on resilience overflow fix + FR-003 acceptance gates (Plan 794, fixed real u64-overflow at saturation; C02 cluster now 28/30 93% A; tier-1 C02 contributes to weighted bump). **C07 L68 2→3** on flake-tracker dashboard (Plan 795; pure-stdlib JUnit parser + CI workflow + ops runbook + 6 FR-003 tests; C07 cluster now 28/30 93% A; **C07 IS in tier-1**; second tier-1 lift in Wave17).

## RC scope (what shipped Jul 14–19)

| Wave | PR range | Theme |
|------|----------|-------|
| Hotfix | #290 | `tracing-subscriber` json feature |
| Reconcile | #279, #320 | SCORECARD table sync |
| Soft docs + CI | #292–#319 | C01–C11 runbooks, soft workflows |
| Code + scripts | #323–#324 | spawn audit JSONL, chaos restart |
| Wave12 lifts | #326–#330 | error envelope, proptest, trace inject, PNG scaffold, Harbor Phase 3 plan |
| Wave13 lifts | #332–#336 | OpenAPI component, soak scaffold, IPC/tray trace, PNG bytes + soft diff, governance sync |
| Wave14 hard gates | #337–#340 | chaos ci-success, coverage snapshot, visual hard, tray HTTP trace |
| Cluster lifts | #364–#391 | proptest expand, C09/C01/C03/C04/C10/C05/C06/C07/C11 score lifts, Harbor N/A rescore, product fixes |
| Wave15 lifts | #396, #399 | C10 L99 skeletons; C01 coverage-lift tests (#399) — pin refresh pending green `coverage.yml` |
| Feb recovery mesh | #397, #400 | GPG L34 operator guide; CoW/worktree recovery docs |

## Cluster snapshot

| Cluster | Pct | Grade | RC note |
|---------|:---:|:-----:|---------|
| C03 | 100% | A | Agent readiness — hold |
| C00 | 97% | A | L4/L6 async shutdown + perf budgets (#373) |
| C01 | 93% | A | coverage pin 83.48% (#338/#399 pending refresh) + FR SSOT (#368) |
| C09/C10 | 93–97% | A | keyboard/Vale/FAQ + skeleton loading states |
| C02/C07 | 90% → **93%** | **A** | rate limit + e2e tier (#384); **Wave17 Plan 794 (T-890)**: L26 2→3 on overflow fix + FR-003 acceptance gates; **Wave17 Plan 795 (T-900)**: L68 2→3 on flake-tracker dashboard |
| C04 | **90%** | **A** | **Wave17 Plan 776 attempt 2 (T-860)**: L34 2→3 on verified merge commits (3 squash-merges on `main`); ruleset 19181236 evidence removed (stale) |
| C05 | **90%** | **A** | **Wave17 Plan 782 (T-870)**: L49 2→3 — Grafana provisioning as code (1 datasource + 1 provider + 3 dashboards + 1 audit manifest). C05 was 87% B pre-Plan 782; now 90% A. |
| C11 | **89%** | **B** | **Wave17 Plan 793 (T-880)**: L111 1→2 — soft auto-update probe (`src/commands/upgrade.rs` + `Commands::Upgrade` + 6 FR-003 tests, no network egress, no install path). C11 was 87% B pre-Plan 793; now 89% B. L112 codesign/notarize still Blocked on org secrets. |
| C06 | 90% | A | **Wave17 Plan 777** L53 SLSA L2 → L3 generator landed; **Plan 778b** re-pinned to commit SHA `5a775b367...` (v2.0.0) |
| C08 | 73% | C | Harbor L76 EXTRACTED/N/A (ADR 0002/0005) |

## RC blockers (hard — out of RC scope)

- **C11 L112** codesign/notarize org secrets (zero `APPLE_*` / `WINDOWS_CERT_*` present)
- ~~**C05 L49** Dashboard coverage provisioning~~ — **CLOSED** via Plan 782 (T-870): L49 2→3 on Grafana provisioning as code (1 Prometheus datasource + 1 dashboard provider + 3 dashboards + 1 audit manifest + README runbook). C05 26/30 87% B → 27/30 90% A. Lane-level provisioned (`sharecli` folder); org-wide folder promotion deferred (out of repo scope).
- ~~**C04 L34** Verified commit evidence lift~~ — **CLOSED** via Plan 776 attempt 2 (T-860): L34 2→3 on 3 verified squash-merge commits on `main`; ruleset `19181236` evidence **removed** from claim (ruleset no longer present at repo level; `gh api repos/KooshaPari/sharecli/rulesets` → `[]`); operator-side gpg/SSH signing on PR commits remains as separate follow-up (private-key delivery out-of-band)
- ~~**C06** SLSA L3 full provenance~~ — **CLOSED** via Plan 777 (L53 2→3); remaining L53 hardening: ~~re-pin `@v2` → `@<commit-sha>`~~ — **CLOSED** via Plan 778b (commit SHA `5a775b367a56d5bd118a224a811bba288150a563`); remaining: hermetic flags (L52 path)
- ~~**C02 L26** Resilience FR-003 acceptance gates~~ — **CLOSED** via Plan 794 (T-890): L26 2→3 on overflow fix + FR-003 acceptance gates (`tests/c02_l26_resilience.rs` 10/10 pass + `src/retry.rs` u128 saturating_mul + `src/backoff.rs` u128 saturating_mul; saturation overflow at extreme attempts now clamps correctly). C02 27/30 90% A → 28/30 93% A.
- ~~**C07 L68** Flake-tracker dashboard~~ — **CLOSED** via Plan 795 (T-900): L68 2→3 on `scripts/flake_tracker.py` (pure-stdlib JUnit parser, classifies testcase as `flaky | regression | stable | skipped`, emits JSON with `baseline_diff` introduced/resolved/persistent counts) + `scripts/comment_flake_tracker.py` (PR commenter) + `audit/.flake-tracker/` operations runbook + `.github/workflows/flake-tracker.yml` (paths-filtered advisory `continue-on-error: true`; uploads `flake-report.json` artifact) + `tests/c07_l68_flake_tracker.rs` (6/6 PASS — flake classification, regression classification, baseline diff, output path, `--fail-on-flake` exit code, `NO_COLOR` respected). C07 27/30 90% A → 28/30 93% A. **C07 IS in tier-1**; second tier-1 lift in Wave17.

## RC verification checklist

- [x] `cargo build` matrix green on `main`
- [x] FR PR bodies contain `FR-NNN` (pr-lint)
- [x] SCORECARD remediation log matches merged PRs (post #391)
- [x] Error envelope unified on serve (T-400 · #330)
- [x] OpenAPI `ErrorEnvelope` component (T-500 · #332)
- [x] PNG bytes committed + soft diff gate (T-510 · #335)
- [x] Harbor Phase 3 soak execution scaffold (T-520 · #333) — **EXTRACTED** to benchora `harbor-soft` (FR-003)
- [x] IPC/tray traceparent inject (T-530 · #334)
- [x] Harbor Phase 3 soak evidence plan (T-440 · #326) — **EXTRACTED** / N/A (sharecli; ADR 0005)
- [ ] ~~Seven-day Harbor soak log completion (Wave14)~~ — **EXTRACTED / N/A (sharecli)**; tracked in benchora/`portage-temp` (T-650; FR-003)
- [x] Visual-soft hard promote (Wave14 · T-600)
- [x] Chaos restart ci-success hard gate (T-630 · #337)
- [x] Coverage llvm-cov snapshot + numeric pin (T-620/T-625 · #338)
- [x] Visual-soft hard promote (T-600 · #339)
- [x] Tray dashboard HTTP traceparent inject (T-610 · #340)
- [x] Harbor soak EXTRACTED/N/A — not sharecli product scope (T-675 · ADR 0002/0005)

## Supersedes

Informal "64% C" / "~80% B pre-Wave12" / "~81% B pre-Wave13" / "~82% B pre-Wave14" references in stale headers — use this RC + live SCORECARD instead.

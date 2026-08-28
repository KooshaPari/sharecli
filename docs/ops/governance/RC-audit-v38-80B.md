# Release Candidate — audit-v38 grade A (~91.9% tier-1)

**Status:** SOFT RC (evidence stack; not a product GA claim)
**Pin commit:** `8f1990d` (`main` after #780 Wave17 Plan 776 attempt 2 + #777 Wave17 Plan 778b SLSA L3 re-pin + #776 Wave17 Plan 777 SLSA L3 generator + #775 Wave17 T-810 `--lib` coverage pin; governance sync chain #774/#773/#771/Plan782)
**Scorecard:** `audit/SCORECARD-v38.md` — weighted **91.8% A**, unweighted **90.8% A** (sum 1090 / 12), tier-1 **91.9% A** (post Plans 776 attempt 2 + 777 + 778b + 782; pre-Plan 776 attempt 2: unweighted 91.3%, weighted 91.2%). `TEST_COVERAGE_MATRIX.md` lib pin **77.34%** @ `fa887e9` shipped; workspace pin **80.51%** @ `5d8dc08` retained as historical evidence. C04 L34 2→3 on verified merge commit evidence (3 squash-merge commits on `main` `verified: true` via GitHub web-flow signing). C05 L49 2→3 on Grafana provisioning as code (Plan 782). C06 L53 2→3 SLSA L3 generator landed in `release-attestation.yml`; **Wave17 Plan 778b** re-pins generator from `@v2` to commit SHA `5a775b367a56d5bd118a224a811bba288150a563` for digest-pinned L3 hardening.

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
| C02/C07 | 90% | A | rate limit + e2e tier (#384) |
| C04 | **90%** | **A** | **Wave17 Plan 776 attempt 2 (T-860)**: L34 2→3 on verified merge commits (3 squash-merges on `main`); ruleset 19181236 evidence removed (stale) |
| C05 | **90%** | **A** | **Wave17 Plan 782 (T-870)**: L49 2→3 — Grafana provisioning as code (1 datasource + 1 provider + 3 dashboards + 1 audit manifest). C05 was 87% B pre-Plan 782; now 90% A. |
| C05/C11 | 87% | B | OSV hard, chaos ci-success, systemd `.deb` (L112 deferred — external secrets; **C05 now 90% A on Plan 782**, only C11 remains B) |
| C06 | 90% | A | **Wave17 Plan 777** L53 SLSA L2 → L3 generator landed; **Plan 778b** re-pinned to commit SHA `5a775b367...` (v2.0.0) |
| C08 | 73% | C | Harbor L76 EXTRACTED/N/A (ADR 0002/0005) |

## RC blockers (hard — out of RC scope)

- **C11 L112** codesign/notarize org secrets (zero `APPLE_*` / `WINDOWS_CERT_*` present)
- ~~**C05 L49** Dashboard coverage provisioning~~ — **CLOSED** via Plan 782 (T-870): L49 2→3 on Grafana provisioning as code (1 Prometheus datasource + 1 dashboard provider + 3 dashboards + 1 audit manifest + README runbook). C05 26/30 87% B → 27/30 90% A. Lane-level provisioned (`sharecli` folder); org-wide folder promotion deferred (out of repo scope).
- ~~**C04 L34** Verified commit evidence lift~~ — **CLOSED** via Plan 776 attempt 2 (T-860): L34 2→3 on 3 verified squash-merge commits on `main`; ruleset `19181236` evidence **removed** from claim (ruleset no longer present at repo level; `gh api repos/KooshaPari/sharecli/rulesets` → `[]`); operator-side gpg/SSH signing on PR commits remains as separate follow-up (private-key delivery out-of-band)
- ~~**C06** SLSA L3 full provenance~~ — **CLOSED** via Plan 777 (L53 2→3); remaining L53 hardening: ~~re-pin `@v2` → `@<commit-sha>`~~ — **CLOSED** via Plan 778b (commit SHA `5a775b367a56d5bd118a224a811bba288150a563`); remaining: hermetic flags (L52 path)

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

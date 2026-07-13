# audit-v38 Scorecard — sharecli

**Repo:** KooshaPari/sharecli
**Date:** 2026-07-12
**Repo-type profile:** CLI+daemon
**Auditor:** cursor-agent cluster-fleet (C00–C11); T-200 FR-002 + threat/release lifts
**Commit audited:** (pending merge of feat/sharecli-t200-fr002-threat-release)

> Scoring: each sub-pillar 0=absent / 1=seeded / 2=partial / 3=complete, evidence-mandatory (`file:line`).
> Cluster score = sum / (sub-pillars × 3). Grade: A≥90% · B≥75% · C≥60% · D≥40% · F<40%.
> Lane evidence: `audit/.lane-c00` … `audit/.lane-c11`. Rubric pin: `audit/rubric/` (from phenotype-org-audits audit-v38).

## Category Scores

| Cluster | Category | Pillars | Score (sum/max) | Pct | Grade | Top-3 gaps |
|---------|----------|---------|:---------------:|:---:|:-----:|------------|
| C00 | Architecture + Module | L0–L9 | 18/30 | 60% | C | lib.rs sprawl; OpenAPI drift CI; tight perf budgets |
| C01 | CI, DX, Observability | L10–L19 | 19/30 | 63% | C | echo coverage; i18n; action SHA pins |
| C02 | Error handling, API, Governance | L20–L29 | 21/30 | 70% | C | federated IdP; audit retention; burn alerts |
| C03 | Agent Readiness | L30 | 30/36 | 83% | B | journey e2e; golden fixtures; unhappy-path |
| C04 | Security | L31–L40 | 18/30 | 60% | C | signed commits; residual AuthZ; sandbox harden |
| C05 | Observability (deep) | L41–L50 | 21/30 | 70% | C | Pyroscope push; multi-hop traces; live PD secrets |
| C06 | Supply Chain | L51–L60 | 17/30 | 57% | D | provenance upgrade; lock/deny gaps |
| C07 | DX, QEng, Portability | L61–L70 | 19/30 | 63% | C | proptest; mutants CI gate; freebsd/wasm |
| C08 | Eval Coverage | L71–L80 | 16/30 | 53% | D | Harbor N/A by ADR; tighter bench thresholds |
| C09 | Accessibility + UX | L81–L95 | 26/45 | 58% | D | degraded-mode AX; recovery docs |
| C10 | Visual Identity | L96–L107 | 24/36 | 67% | C | visual docs; light theme; type scale |
| C11 | Packaging + Distribution | L108–L122 | 29/45 | 64% | C | Homebrew sha; signing; native installers |

## Overall

**Weighted overall score:** 64% · **Overall grade:** C

(Unweighted mean of cluster pcts: (60+63+70+83+60+70+57+63+53+58+67+64)/12 = 768/12 = **64%**.)

**Tier-1 double-weight (C00–C03):** (60+63+70+83)×2 + (60+70+57+63+53+58+67+64) = 552 + 492 = 1044 / 16 = **65.25%** (C).

## Headline Findings

- **Strongest:** C03 Agent Readiness (83% B) — FR-NNN + WORK_DAG + llms.txt + pr-lint after Wave1.
- **Wave2 lifts:** C00 50%→60% C (OpenAPI stub + Criterion/bench-gate); C04 47%→53% D (CycloneDX SBOM CI); C08 40%→47% D (per-PR hard-ish gate + baselines).
- **Evidence-only (no pct change):** C07 L69 now PR-matrix ubuntu+macos (rubric score-1 bar; Windows still needed for 2); C11 Formula `head do` + OpenAPI deploy row (brew sha still PLACEHOLDER).
- **SBOM scored under C04 L32** (not C06 — supply-chain pillars have no SBOM slot).
- **Highest-leverage remaining:** brew digest after first `v*` attach (C11), journey/golden (T-240/T-250), federated AuthN, C06 provenance.
- **Governance:** `docs/ops/governance/WBS-PHASED.md` + `GAP-QA-MATRIX.md` (machine Status tokens) + `WORK_DAG.md`.
- **Agent-readiness verdict (C03):** Agents can claim WORK_DAG tasks with FR refs; FR-001..005 Covered; journey/golden still open.
- **Time-2 verdict (C11):** OCI+uninstall+mobile decision + brew --HEAD solid; bottle PLACEHOLDER and unsigned archives remain.

## Supersedes

Root `audit_scorecard.json` tracks this v38 card. Do not use the legacy Python 30-pillar auto-scan for fleet ranking.

## Post-audit remediations

### 2026-07-09
- `ci-success` now fails when any required job fails (false-green closed).
- `coverage.yml` replaced echo stub with llvm-cov + test-count > 0 guard.
- quality-gate coverage threshold no longer `continue-on-error`.
- **C05 re-scored 9/30 (30% F) → 12/30 (40% D):** `/readyz` + health JSON unit tests; `docs/ops/SLO.md` draft SLOs.
- **C10 re-scored 23/36 (64% C) → 24/36 (67% C):** `assets/tokens.css` present; L96 2→3.

### 2026-07-10 (Wave1 lift re-score)
- **C03 22/36 (61% D*) → 30/36 (83% B):** FR-NNN root, WORK_DAG, llms.txt, rust-toolchain, pr-lint, friction-log. (*prior card mislabeled 61% as D; rubric C≥60%.)
- **C07 10/30 (33% F) → 18/30 (60% C):** `.devcontainer/`, nextest CI retries + flake-policy, fuzz toml_lite, `just dev`/`mutants`.
- **C08 0/30 (0% F) → 12/30 (40% D):** Criterion benches, load script, REPRO pins, soft `bench.yml`, ADR 0002 eval scope.
- **C11 19/45 (42% D) → 27/45 (60% C):** Containerfile USER+HEALTHCHECK, `docs/deploy.md`, ADR 0001 no-mobile, README uninstall.

### 2026-07-10 (Wave2 score-lift re-score)
- **C00 15/30 (50% D) → 18/30 (60% C):** L2 1→2 (`docs/openapi/serve.yaml`); L6 0→2 (Criterion + `bench-gate` + baselines).
- **C04 14/30 (47% D) → 16/30 (53% D):** L32 0→2 (`.github/workflows/sbom.yml` CycloneDX artifact on main).
- **C07 18/30 (60% C) unchanged:** L69 evidence refreshed — PR CI macos matrix; still score 1 until Windows (rubric).
- **C08 12/30 (40% D) → 14/30 (47% D):** L73 2→3 (3-tier+SLO+CI assert); L74 1→2 (per-PR `bench-gate`, 50% threshold).
- **C11 27/45 (60% C) unchanged:** Formula `head do` + OpenAPI/deploy evidence; brew sha PLACEHOLDER remains.
- **C06 unchanged:** SBOM lives under C04 L32.

### 2026-07-11 (Windows CI lane)
- **C07 18/30 (60% C) → 19/30 (63% C):** L69 1→2 — PR CI matrix adds `windows-latest` (Zig skipped; spawn-core-sys Rust stub).
- Wave2 macOS Zig path: `zig build-obj` + `ar` on Darwin; stopwatch `best_lap` de-flaked.

### 2026-07-11 (C05 OTel + RED + Grafana)
- **C05 12/30 (40% D) → 18/30 (60% C):** L42 0→2 (OTLP/HTTP + tracing-opentelemetry), L44 0→2 (`traceparent` middleware), L43 2→3 (HTTP RED series), L49 1→2 (Grafana JSON).
- **Overall 58% D → 60% C.**
- Docs: `docs/ops/otel.md`, `docs/ops/grafana/sharecli-serve.json`.

### 2026-07-11 (C02 AuthN + C08 measured baselines)
- **C02 15/30 (50% D) → 20/30 (67% C):** L21 Bearer AuthN, L23 JSONL audit log, L27 SLO/AUTH docs.
- **C08 14/30 (47% D) → 16/30 (53% D):** measured Criterion baselines, hyperfine script, bench flake quarantine.
- **Overall 60% C → 62% C.**

### 2026-07-11 (C05 pprof + OTel 0.32.1 security)
- **C05 18/30 (60% C) → 20/30 (67% C):** L45 0→2 (`/debug/pprof/profile` + `docs/ops/profiling.md`).
- **Security:** coordinated bump `opentelemetry`/`sdk`/`otlp` 0.30→0.32.1 + `tracing-opentelemetry` 0.33 (closes Dependabot alert on unbounded W3C Baggage).
- **Overall 62% C → 63% C.**

### 2026-07-11 (C05 Alertmanager + C08 nightly trends)
- **C05 20/30 (67% C) → 21/30 (70% C):** L48 1→2 (Alertmanager rule pack + severity routing + runbooks).
- **C08 evidence:** nightly `bench-nightly` cron + `export-trend.py` artifacts (`docs/eval/TRENDS.md`); L74 remains 3.
- **Overall stays 63% C** (C05 lift ≈ +0.3pp unweighted).

### 2026-07-12 (T-200 FR-002 + threat/release + governance)
- **C04 16/30 (53% D) → 18/30 (60% C):** L32 2→3 (SBOM in-archive), L39 2→3 (`THREAT_MODEL.md`).
- **C02 20/30 (67% C) → 21/30 (70% C):** L20 2→3 (STRIDE artifact).
- **C11 27/45 (60% C) → 29/45 (64% C):** L118 2→3 (unsigned GH Release attach), L119 2→3 (`rust-version`).
- **C03 evidence:** T-200 FR-002 acceptance on disk; WORK_DAG T-100..160+T-200 → DONE.
- **Governance:** `docs/ops/governance/WBS-PHASED.md` + `GAP-QA-MATRIX.md`; `audit_scorecard.json` synced.
- **Overall 63% C → 64% C.**

### 2026-07-12 (T-210 FR-003 + T-260/T-270)
- **C03 evidence:** FR-003 acceptance on disk (`tests/fr003_*.rs`); T-210/T-260/T-270 → DONE.
- Cluster pct unchanged (already 83% B); gaps now FR-004..005 + journey/golden.
- Docs: `AGENTS.md` claim-lock; `docs/ops/LOCAL_LOOP_BUDGETS.md`.

### 2026-07-12 (T-220 FR-004)
- **C03 evidence:** FR-004 acceptance on disk (`tests/fr004_*.rs`); T-220 → DONE.
- Remaining FR gap: FR-005 (T-230) + journey/golden (T-240/T-250).

### 2026-07-13 (T-230 FR-005)
- **C03 evidence:** FR-005 acceptance on disk (`tests/fr005_*.rs`); T-230 → DONE.
- Remaining C03 gaps: journey/golden (T-240/T-250) + unhappy-path (T-300); T-310 still BLOCKED on re-score.

## Spine links

- Rubric: [phenotype-org-audits/audit-v38](https://github.com/KooshaPari/phenotype-org-audits/tree/main/audit-v38)
- Spine index: [docs/SPINE-INDEX.md](https://github.com/KooshaPari/phenotype-org-audits/blob/main/docs/SPINE-INDEX.md)
- CI truth notes: `audit/CI_TRUTH_FINDINGS.md`
- Boundary: `audit/BOUNDARY_VERIFY_2026-07-10.md`

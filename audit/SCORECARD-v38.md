# audit-v38 Scorecard — sharecli

**Repo:** KooshaPari/sharecli
**Date:** 2026-07-11
**Repo-type profile:** CLI+daemon
**Auditor:** cursor-agent cluster-fleet (C00–C11); C05 pprof + OTel 0.32.1 security
**Commit audited:** (pending merge of lift/c05-pprof-otel-0.32)

> Scoring: each sub-pillar 0=absent / 1=seeded / 2=partial / 3=complete, evidence-mandatory (`file:line`).
> Cluster score = sum / (sub-pillars × 3). Grade: A≥90% · B≥75% · C≥60% · D≥40% · F<40%.
> Lane evidence: `audit/.lane-c00` … `audit/.lane-c11`. Rubric pin: `audit/rubric/` (from phenotype-org-audits audit-v38).

## Category Scores

| Cluster | Category | Pillars | Score (sum/max) | Pct | Grade | Top-3 gaps |
|---------|----------|---------|:---------------:|:---:|:-----:|------------|
| C00 | Architecture + Module | L0–L9 | 18/30 | 60% | C | lib.rs sprawl; OpenAPI drift CI; tight perf budgets |
| C01 | CI, DX, Observability | L10–L19 | 19/30 | 63% | C | echo coverage; i18n; action SHA pins |
| C02 | Error handling, API, Governance | L20–L29 | 20/30 | 67% | C | federated IdP; audit retention; burn alerts |
| C03 | Agent Readiness | L30 | 30/36 | 83% | B | FR-002..005 tests; journey e2e; golden fixtures |
| C04 | Security | L31–L40 | 16/30 | 53% | D | SBOM in-tarball; residual auth/threat gaps |
| C05 | Observability (deep) | L41–L50 | 20/30 | 67% | C | Pyroscope push; multi-hop traces; on-call alerts |
| C06 | Supply Chain | L51–L60 | 17/30 | 57% | D | provenance upgrade; lock/deny gaps |
| C07 | DX, QEng, Portability | L61–L70 | 19/30 | 63% | C | proptest; mutants CI gate; freebsd/wasm |
| C08 | Eval Coverage | L71–L80 | 16/30 | 53% | D | Harbor N/A by ADR; nightly bench trends |
| C09 | Accessibility + UX | L81–L95 | 26/45 | 58% | D | degraded-mode AX; recovery docs |
| C10 | Visual Identity | L96–L107 | 24/36 | 67% | C | visual docs; light theme; type scale |
| C11 | Packaging + Distribution | L108–L122 | 27/45 | 60% | C | Homebrew sha; signing; native installers |

## Overall

**Weighted overall score:** 63% · **Overall grade:** C

(Unweighted mean of cluster pcts: (60+63+67+83+53+67+57+63+53+58+67+60)/12 = 751/12 ≈ **62.6%** → **63%**.)

**Tier-1 double-weight (C00–C03):** (60+63+67+83)×2 + (53+67+57+63+53+58+67+60) = 546 + 478 = 1024 / 16 = **64%** (C).

## Headline Findings

- **Strongest:** C03 Agent Readiness (83% B) — FR-NNN + WORK_DAG + llms.txt + pr-lint after Wave1.
- **Wave2 lifts:** C00 50%→60% C (OpenAPI stub + Criterion/bench-gate); C04 47%→53% D (CycloneDX SBOM CI); C08 40%→47% D (per-PR hard-ish gate + baselines).
- **Evidence-only (no pct change):** C07 L69 now PR-matrix ubuntu+macos (rubric score-1 bar; Windows still needed for 2); C11 Formula `head do` + OpenAPI deploy row (brew sha still PLACEHOLDER).
- **SBOM scored under C04 L32** (not C06 — supply-chain pillars have no SBOM slot).
- **Highest-leverage remaining:** brew digest + signing (C11), Pyroscope/on-call (C05), federated AuthN, C08 nightly trends.
- **Agent-readiness verdict (C03):** Agents can claim WORK_DAG tasks with FR refs; FR-002..005 acceptance suites still open.
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

## Spine links

- Rubric: [phenotype-org-audits/audit-v38](https://github.com/KooshaPari/phenotype-org-audits/tree/main/audit-v38)
- Spine index: [docs/SPINE-INDEX.md](https://github.com/KooshaPari/phenotype-org-audits/blob/main/docs/SPINE-INDEX.md)
- CI truth notes: `audit/CI_TRUTH_FINDINGS.md`
- Boundary: `audit/BOUNDARY_VERIFY_2026-07-10.md`

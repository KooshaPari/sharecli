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
| C02 | Error handling, API, Governance | L20–L29 | 24/30 | 80% | B | residual OAuth/SAML; spawn audit events |
| C03 | Agent Readiness | L30 | 33/36 | 92% | A | optional polish; brew still Blocked |
| C04 | Security | L31–L40 | 18/30 | 60% | C | signed commits; residual AuthZ; sandbox harden |
| C05 | Observability (deep) | L41–L50 | 21/30 | 70% | C | Pyroscope push; multi-hop traces; live PD secrets |
| C06 | Supply Chain | L51–L60 | 20/30 | 67% | C | SLSA L3; hermetic builds; container cosign |
| C07 | DX, QEng, Portability | L61–L70 | 19/30 | 63% | C | proptest; mutants CI gate; freebsd/wasm |
| C08 | Eval Coverage | L71–L80 | 18/30 | 60% | C | synthetic eval corpus; tighter multi-week trend CSV |
| C09 | Accessibility + UX | L81–L95 | 33/45 | 73% | C | responsive TUI; SR checklist |
| C10 | Visual Identity | L96–L107 | 24/36 | 67% | C | visual docs; light theme; type scale |
| C11 | Packaging + Distribution | L108–L122 | 30/45 | 67% | C | signing/notarize; native dmg/msi |

## Overall

**Weighted overall score:** 69% · **Overall grade:** C

(Unweighted mean of cluster pcts: (60+63+80+92+60+70+67+63+60+73+67+67)/12 = 822/12 = **68.5% ≈ 69%**.)

**Tier-1 double-weight (C00–C03):** (60+63+80+92)×2 + (60+70+67+63+60+73+67+67) = 590 + 527 = 1117 / 16 = **69.8% ≈ 70%** (C).

## Headline Findings

- **Strongest:** C03 Agent Readiness (92% A); C02 now **80% B** (W5.1 JWT + W5.2 retention/burn).
- **W5.2:** audit JSONL size rotation + AuthN burn metric/alert (`sharecli_http_unauthorized_total`).
- **Highest-leverage remaining:** codesign/notarize (C11 L112), responsive TUI (C09), synthetic eval corpus (C08), SLSA L3 / cosign (C06).
- **Governance:** `docs/ops/governance/WBS-PHASED.md` + `GAP-QA-MATRIX.md` + `WORK_DAG.md`.
- **Packaging (C11):** brew bottle sha filled (W4.2); L112 signing still Blocked.

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

### 2026-07-13 (T-240 journey)
- **C03 30/36 (83% B) → 31/36 (86% B):** L30.6 2→3 (`tests/quick_start_journey.rs`).
- Overall stays **64% C** (mean 771/12).
- Remaining C03 gaps: golden (T-250) + unhappy-path (T-300).

### 2026-07-13 (T-250 goldens)
- **C03 31/36 (86% B) → 32/36 (89% B):** L30.7 2→3 (`tests/golden/` ×5 + `golden_snapshots.rs`).
- **Overall 64% C → 65% C** (mean 774/12).
- Remaining C03 gap: unhappy-path (T-300); T-310 re-score still optional.

### 2026-07-13 (T-300 unhappy-path)
- **C03 32/36 (89% B) → 33/36 (92% A):** L30.12 2→3 (`tests/fr_invalid_missing_friction.rs`).
- Overall stays **65% C** (mean 777/12).
- W3.5 Journey+golden+friction → DONE.

### 2026-07-13 (C06 release pin + L60)
- Fix `actions/upload-artifact` pin (broken SHA blocked Release attach / brew).
- Seed `SOURCE_DATE_EPOCH=0` on release builds; add `audit.toml` yanked=warn.
- **C06 17/30 (57% D) → 18/30 (60% C):** L60 2→3 (`rust-toolchain.toml` evidence).
- Overall stays **65% C** (mean 780/12).

### 2026-07-13 (C06 supply-chain lift)
- **C06 18/30 (60% C) → 20/30 (67% C):** L52 2→3 (`scripts/repro-check.sh` + `repro-check.yml` + `just repro-check`); L55 2→3 (`unknown-registry=deny` + post-W5.1 deny/audit.toml alignment).
- Docs: `docs/slsa.md` repro + cosign/GHCR roadmap (L56 documented-only).
- **Overall 66% C → 67% C** (mean 800/12).
- FR-002 (config/build determinism) traceability for repro gate.

### 2026-07-13 (W4.2 brew sha)
- Attached linux+darwin tarballs to `v0.3.0`; Formula sha256 filled.
- Release: Zig setup + cyclonedx filename + attach no longer blocked on attest.
- **C11 29/45 (64% C) → 30/45 (67% C):** L109 2→3.
- Overall stays **65% C** (mean 783/12).

### 2026-07-13 (C08 eval lift)
- **C08 16/30 (53% D) → 18/30 (60% C):** L71 2→3 (`jwt_auth_validate` bench, FR-012); L80 2→3 (`docs/eval/GOVERNANCE.md`).
- **Overall 66% C → 67% C** (mean 793/12 → 800/12).
- Remaining C08 gaps: wire jwt bench into `bench-gate`; hyperfine JSON CI artifact.

### 2026-07-13 (C09 a11y lift)
- **C09 26/45 (58% D) → 30/45 (67% C):** L81.1/L81.5 1→2 (dashboard landmarks + `docs/a11y/README.md` + `tests/a11y/`); L81.2 1→2 (`docs/a11y/contrast.md`); L81.3 1→2 (`is_quit_key` tests + `docs/a11y/keyboard.md`).
- Docs: `docs/a11y/status-and-recovery.md` (FR-004 status matrix); `--help` after_long_help cites a11y + degraded mode.
- **Overall 67% C → 68% C** (mean 816/12; C06+C08+C09 at 67%/60%/67%).

### 2026-07-13 (C08 jwt bench-gate)
- Wired `jwt_auth_validate` into soft/`bench-gate`/nightly Criterion jobs; baseline `jwt_validate_rs256` (BENCH-4, 10 ms).
- C08 score unchanged (18/30 60% C); closes L71 follow-up gap. Remaining: hyperfine JSON CI artifact.

### 2026-07-13 (C08 hyperfine CI artifact)
- Soft PR/push job + nightly upload of `hyperfine-healthz-<sha>.json` (LOAD-2 / L72).
- C08 score unchanged (18/30 60% C); closes L72 hyperfine artifact gap.

### 2026-07-13 (C09 table-header contrast)
- **C09 30/45 (67% C) → 31/45 (69% C):** L81.2 2→3 — dashboard `thead` uses `#a371f7` on `#161b22` (5.16:1).
- Overall stays **~68% C** (mean 818/12).


### 2026-07-13 (C09 axe CI — W9.1)
- **C09 31/45 (69% C) → 33/45 (73% C):** L81.1 2→3 + L81.5 2→3 (`.github/workflows/a11y.yml` + `scripts/a11y/axe-dashboard.mjs`; WCAG 2.x Level A; zero violations).
- **Overall 68% C → 69% C** (mean 822/12).
- FR-004 NFR: dashboard axe gate.

## Spine links

- Rubric: [phenotype-org-audits/audit-v38](https://github.com/KooshaPari/phenotype-org-audits/tree/main/audit-v38)
- Spine index: [docs/SPINE-INDEX.md](https://github.com/KooshaPari/phenotype-org-audits/blob/main/docs/SPINE-INDEX.md)
- CI truth notes: `audit/CI_TRUTH_FINDINGS.md`
- Boundary: `audit/BOUNDARY_VERIFY_2026-07-10.md`

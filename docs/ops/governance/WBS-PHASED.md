# WBS-PHASED — sharecli + org spine

**Status:** ACTIVE  
**Target overall:** ~80% B (tier-1 weighted) · **Pinned card:** `audit/SCORECARD-v38.md`  
**Spine:** [phenotype-org-audits SPINE-INDEX](https://github.com/KooshaPari/phenotype-org-audits/blob/main/docs/SPINE-INDEX.md) · rubric `audit-v38`  
**DAG:** [`WORK_DAG.md`](https://github.com/KooshaPari/sharecli/blob/main/WORK_DAG.md) · [`PERT-DAG-W12.md`](./PERT-DAG-W12.md) · **RC:** [`RC-audit-v38-80B.md`](./RC-audit-v38-80B.md)  
**FRs:** [`FUNCTIONAL_REQUIREMENTS.md`](https://github.com/KooshaPari/sharecli/blob/main/FUNCTIONAL_REQUIREMENTS.md)  
**Machine tokens:** `Status: DONE` | `READY` | `BLOCKED` | `IN_PROGRESS`  
**Last sync:** 2026-07-19 (Wave14 #337–#340 + cluster lifts through #391; overall ~91% A weighted / ~89.8% B unweighted)

> Agents: flip only the `Status:` token and Evidence cell; keep ID columns stable.

## Org ↔ project map

| Org spine | Project artifact | Clusters | Status |
|-----------|------------------|----------|--------|
| audit-v38 rubric | `audit/rubric/` | C00–C11 | Status: DONE |
| SPINE-INDEX | this WBS + GAP-QA | fleet | Status: IN_PROGRESS |
| Lane evidence | `audit/.lane-c00`…`c11` | C00–C11 | Status: DONE |
| Scorecard | `audit/SCORECARD-v38.md` | all | Status: DONE |
| Claimable DAG | `WORK_DAG.md` T-IDs | C03 / FR | Status: IN_PROGRESS |

## Cluster rollup (audit-v38)

| Cluster | Focus | Pct | Grade | Phase anchor | Status |
|---------|-------|:---:|:-----:|--------------|--------|
| C00 | Architecture + Module | 97% | A | Wave2 + W11–W14 | Status: DONE |
| C01 | CI / DX / Obs | 93% | A | Wave1–2 + W10–W14 | Status: IN_PROGRESS |
| C02 | Error / API / Governance | 90% | A | Wave2 + W5 + W11–W14 | Status: IN_PROGRESS |
| C03 | Agent Readiness | 100% | A | Wave1 + Wave3 + W11–W14 | Status: DONE |
| C04 | Security | 87% | B | Wave2 + W10–W14 | Status: IN_PROGRESS |
| C05 | Observability (deep) | 87% | B | Wave2 + W11–W14 | Status: IN_PROGRESS |
| C06 | Supply Chain | 87% | B | Wave2 + W6 + W11–W14 | Status: IN_PROGRESS |
| C07 | DX / QEng / Portability | 90% | A | Wave1–2 + W10–W14 | Status: IN_PROGRESS |
| C08 | Eval Coverage | 73% | C | Wave1–2 + W11–W14; L76 N/A=1 (ADR 0002/0005) | Status: IN_PROGRESS |
| C09 | Accessibility + UX | 93% | A | Wave7 + W9–W14 | Status: IN_PROGRESS |
| C10 | Visual Identity | 94% | A | Wave1 + W11–W14 | Status: IN_PROGRESS |
| C11 | Packaging + Distribution | 87% | B | Wave4 + W11–W14 | Status: IN_PROGRESS |

## Phased WBS

### Wave0–2 — DONE (baseline, agent readiness, score-lift fleet)

See SCORECARD post-audit remediations through 2026-07-11. C05 closed at **70%**.

### Wave3 — FR acceptance suites (current)

| WBS | Work | Links | Status |
|-----|------|-------|--------|
| W3.1 | FR-002 config acceptance | T-200 · `tests/fr002_*.rs` · C03 | Status: DONE |
| W3.2 | FR-003 project registry | T-210 · `tests/fr003_*.rs` · C03 | Status: DONE |
| W3.3 | FR-004 status/health | T-220 · `tests/fr004_*.rs` · C03 | Status: DONE |
| W3.4 | FR-005 limits | T-230 · C03 | Status: DONE |
| W3.5 | Journey + golden + friction | T-240..T-300 · C03 | Status: DONE |
| W3.6 | C03 re-score | T-310 | Status: DONE |
| W3.7 | Claim-lock + loop budgets | T-260 · T-270 | Status: DONE |

Pred: W3.3←W3.2←W3.1; W3.4←W3.3; W3.6←W3.4.

### Wave4 — Packaging / signing

| WBS | Work | Links | Status |
|-----|------|-------|--------|
| W4.1 | Unsigned GH Release attach + SBOM in-archive | C11 L118 · C04 L32 · `release.yml` | Status: DONE |
| W4.2 | Homebrew bottle sha (replace PLACEHOLDER) | C11 · `Formula/sharecli.rb` | Status: DONE |
| W4.3 | Codesign / notarize | C11 L112 | Status: BLOCKED — zero repo secrets (`gh secret list` empty 2026-07-19) |
| W4.4 | Declare MSRV (`rust-version`) | C11 L119 | Status: DONE |

### Wave5 — AuthN federation

| WBS | Work | Links | Status |
|-----|------|-------|--------|
| W5.1 | Federated IdP for `serve` | C02 L21 | Status: DONE |
| W5.2 | Audit retention + burn alerts | C02 | Status: DONE |
| W5.3 | Threat-model review post-federation | C04 L39 · `THREAT_MODEL.md` | Status: DONE |

### Wave6 — Supply chain (C06)

| WBS | Work | Links | Status |
|-----|------|-------|--------|
| W6.1 | Repro-check script + CI (L52) | FR-002 · `scripts/repro-check.sh` · `repro-check.yml` | Status: DONE |
| W6.2 | Deny sources tighten + audit.toml sync (L55) | `deny.toml` · `audit.toml` | Status: DONE |
| W6.3 | Container cosign hard publish (L56 / T-660) | `container-cosign.yml` · `docs/slsa.md` | Status: DONE |
| W6.4 | C06 re-score | `audit/.lane-c06/C06.md` | Status: DONE |

### Wave7 — Accessibility / UX (C09)

| WBS | Work | Links | Status |
|-----|------|-------|--------|
| W7.1 | Dashboard landmarks + Level A docs | L81.1 · L81.5 · `docs/a11y/README.md` · `tests/a11y/` | Status: DONE |
| W7.2 | Contrast token documentation | L81.2 · `docs/a11y/contrast.md` | Status: DONE |
| W7.3 | TUI keyboard matrix + quit-key tests | L81.3 · `docs/a11y/keyboard.md` | Status: DONE |
| W7.4 | Status/recovery + degraded-mode docs | L81.6 · L81.7 · FR-004 · `docs/a11y/status-and-recovery.md` | Status: DONE |
| W7.5 | Table-header contrast AA UI | L81.2 · `src/dashboard.html` · `docs/a11y/contrast.md` | Status: DONE |

### Wave8 — Eval / perf gate polish (C08)

| WBS | Work | Links | Status |
|-----|------|-------|--------|
| W8.1 | Wire `jwt_auth_validate` into Criterion CI | L71 · FR-012 · `bench.yml` · `jwt_validate_rs256` baseline | Status: DONE |
| W8.2 | Hyperfine JSON CI artifact | L73 · `scripts/bench/` | Status: DONE |

### Wave9 — Accessibility CI (C09)

| WBS | Work | Links | Status |
|-----|------|-------|--------|
| W9.1 | axe-core CI for dashboard (WCAG 2.x Level A) | L81.1 · L81.5 · FR-004 NFR · `.github/workflows/a11y.yml` · `scripts/a11y/axe-dashboard.mjs` | Status: DONE |
| W9.2 | C09 re-score after axe CI | L81.1 · L81.5 · `audit/.lane-c09/C09.md` | Status: DONE |
| W9.3 | SR procedure pass (axe+landmarks) + live VO/NVDA soft | L81.4 · FR-004 NFR · `docs/a11y/sr-pass-evidence.md` | Status: DONE (acceptance); live AT Status: READY |

### Wave10 — CI hygiene (C01) + C09 adaptive

| WBS | Work | Links | Status |
|-----|------|-------|--------|
| W10.1 | Pin Actions to SHAs + ubuntu-24.04 | L10 · `.github/workflows/*` | Status: DONE |
| W10.2 | Responsive TUI + dashboard breakpoints | L81.11 · `sharecli-thermal-tui` · `docs/a11y/responsive.md` | Status: DONE |
| W10.3 | OpenAPI ↔ serve route drift CI | L2 · `docs/openapi/serve.yaml` · `scripts/check-openapi-drift.py` | Status: DONE |
| W10.4 | proptest + soft mutants/fuzz + C08 corpus/CSV | L65–L67 · L71/L74 · `mutants.toml` · `docs/eval/corpus/` | Status: DONE |
| W10.5 | OSV scanner + Dependabot groups + container hardening | L37 · L38 · L40 · `osv.yml` · `docs/ops/container-hardening.md` | Status: DONE |
| W10.6 | MVP finality + OS parity (macOS/Windows/Linux/WSL) | L108–L110 · `docs/deploy/FINALITY.md` · `desktop-builds.yml` | Status: DONE |

### Wave11 — Continuous audit ship (Jul 17)

| WBS | Work | Links | Status |
|-----|------|-------|--------|
| W11.1 | JSON logging hotfix + rustfmt | #290 · C05 | Status: DONE |
| W11.2 | SCORECARD reconcile v1/v2 | #279 · #320 · C05 L47 1→2 | Status: DONE |
| W11.3 | Soft runbook fleet (C01–C11 docs) | #292–#318 | Status: DONE |
| W11.4 | Soft CI fleet (soak, netblock, harbor, live-pool, playwright) | #319–#322 · #321 | Status: DONE |
| W11.5 | Spawn audit JSONL (code) | #323 · C02 L28 partial | Status: DONE |
| W11.6 | Chaos restart script | #324 · C05 L48 evidence | Status: DONE |
| W11.7 | Governance sync (WBS/GAP/DAG/RC/PERT) | T-450 · #325 | Status: DONE |

Pred: W11.7←W11.6; Wave12 T-400..T-440 parallel after W11.7.

### Wave12 — Code lifts toward 82% B (DONE)

| WBS | Work | Links | Status |
|-----|------|-------|--------|
| W12.1 | C00 error envelope unify | T-400 · `docs/ops/error-envelope.md` · #330 | Status: DONE |
| W12.2 | C07 proptest config roundtrip | T-410 · `docs/ops/config-proptest.md` · #329 | Status: DONE |
| W12.3 | C05 traceparent CLI inject | T-420 · `docs/ops/trace-multihop.md` · #328 | Status: DONE |
| W12.4 | C10 PNG dashboard baseline | T-430 · `docs/visual/golden-visual-tests.md` · #327 | Status: DONE |
| W12.5 | C08 Harbor Phase 3 soak | T-440 · ADR 0005 · #326 | Status: EXTRACTED → benchora `harbor-soft` |
| W12.4b | W4.3 codesign / notarize | C11 L112 | Status: BLOCKED — zero repo secrets |

### Wave13 — Hard gates toward 82% B (DONE)

| WBS | Work | Links | Status |
|-----|------|-------|--------|
| W13.1 | OpenAPI `ErrorEnvelope` component | C00 L2 · `docs/openapi/serve.yaml` · #332 | Status: DONE |
| W13.2 | PNG baseline commit + soft diff | C10 L107 · `tests/visual/dashboard/` · #335 | Status: DONE |
| W13.3 | Harbor Phase 3 soak execution (7d) | C08 L76 · EXTRACTED → benchora `harbor-soft` / `portage-temp` · #333 | Status: EXTRACTED |
| W13.4 | Trace IPC + tray injectors | C05 L44 · `docs/ops/trace-multihop.md` · T-530 · #334 | Status: DONE |
| W13.5 | Governance sync (WBS/GAP/DAG/RC) | T-550 · #336 | Status: DONE |

### Wave14 — Remaining hard gates (DONE)

| WBS | Work | Links | Status |
|-----|------|-------|--------|
| W14.1 | Deterministic dashboard visual hard gate | FR-003 · C10 L107 · T-600 · #339 | Status: DONE |
| W14.2 | Seven-day Harbor soak log completion | C08 L76 · T-675 · EXTRACTED → benchora `harbor-soft` / `portage-temp` | Status: EXTRACTED |
| W14.3 | Codesign / notarize | C11 L112 | Status: BLOCKED — zero repo secrets |

### Wave14 — Evidence hardening (DONE)

| WBS | Work | Links | Status |
|-----|------|-------|--------|
| W14.1 | Coverage percentage pin + llvm-cov snapshot artifact | C01 L11 · T-620/T-625 · #338 · `TEST_COVERAGE_MATRIX.md` · `coverage.yml` | Status: DONE |

### Wave14 — Residual hardening (DONE)

| WBS | Work | Links | Status |
|-----|------|-------|--------|
| W14.1 | Tray dashboard HTTP traceparent inject | C05 L44 · T-610 · FR-003 · #340 · `tests/c05_trace_tray_http_inject.rs` | Status: DONE |
| W14.2 | Chaos restart ci-success hard gate | C05 L50 · T-630 · FR-003 · #337 | Status: DONE |
| W14.3 | OSV/GHSA hard gate | C04 L38 · T-655 · FR-003 | Status: DONE |
| W14.4 | Mutants + cosign hard gates | C07 L65 · C06 L56 · T-640 · T-660 | Status: DONE |
| W14.5 | Cluster score lifts (#364–#391) | C00–C11 lane evidence | Status: DONE |
| W14.6 | Governance sync (WBS/GAP/DAG/RC/SCORECARD) | T-680 | Status: DONE |

## Sync protocol

1. After merge: update matching `Status:` here + row in `GAP-QA-MATRIX.md`.
2. Re-score lane MD → bump SCORECARD pct → adjust cluster rollup.
3. Cite FR / T-ID / Cxx in PR body (pr-lint).
4. Flip `WORK_DAG.md` task Status READY→DONE when Done-when passes.

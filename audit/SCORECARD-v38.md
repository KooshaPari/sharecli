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
| C00 | Architecture + Module | L0–L9 | 21/30 | 70% | C | lib.rs sprawl; error envelope; tight perf budgets |
| C01 | CI, DX, Observability | L10–L19 | 24/30 | 80% | B | fluent catalogs deferred; gitleaks polish; advisory hard-fail |
| C02 | Error handling, API, Governance | L20–L29 | 26/30 | 87% | B | residual OAuth/SAML; spawn audit events |
| C03 | Agent Readiness | L30 | 33/36 | 92% | A | optional polish; brew still Blocked |
| C04 | Security | L31–L40 | 24/30 | 80% | B | require signed commits ruleset; org 2FA enforce; OSV hard-fail |
| C05 | Observability (deep) | L41–L50 | 23/30 | 77% | B | multi-hop traces; live PD; chaos restart hard gate |
| C06 | Supply Chain | L51–L60 | 24/30 | 80% | B | SLSA L3; network-blocked hermetic; GHCR publish default |
| C07 | DX, QEng, Portability | L61–L70 | 23/30 | 77% | B | mutants hard gate; config proptest; freebsd/wasm |
| C08 | Eval Coverage | L71–L80 | 22/30 | 73% | C | agent-eval Phase 4 harness; bench-gate hard; thermal gate corpus |
| C09 | Accessibility + UX | L81–L95 | 34/45 | 76% | B | Playwright committed baselines; manual SR pass; axe hard required |
| C10 | Visual Identity | L96–L107 | 31/36 | 86% | B | golden visual tests; high-contrast; dashboard hex drift |
| C11 | Packaging + Distribution | L108–L122 | 35/45 | 78% | B | hard codesign/notarize; dmg/msi; harden Win tray; in-binary updater |

## Overall

**Weighted overall score:** 80% · **Overall grade:** B

(Unweighted mean of cluster pcts: (70+80+80+92+80+77+80+77+73+76+86+78)/12 = 949/12 = **79.1% ≈ 79%**.)

**Tier-1 double-weight (C00–C03):** (70+80+87+92)×2 + (80+77+80+77+73+76+86+78) = 658 + 627 = 1285 / 16 = **80.3% ≈ 80%** (B).

## Headline Findings

- **Strongest:** C03 Agent Readiness (92% A); C10 **86% B**; C01/C02/C04/C06 **80% B**.
- **W5.2:** audit JSONL size rotation + AuthN burn metric/alert (`sharecli_http_unauthorized_total`).
- **Highest-leverage remaining:** hard codesign/notarize secrets (C11 L112), SLSA L3 network-block (C06), mutants hard required check (C07 L65), ruleset “Require signed commits” (C04 L34).
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

### 2026-07-13 (C01 action SHA pins — W10.1)
- **C01 19/30 (63% C) → 20/30 (67% C):** L10 2→3 — pin floating Actions tags to commit SHAs; `ubuntu-latest` → `ubuntu-24.04`.
- Overall stays **~69% C** (mean 826/12).

### 2026-07-13 (C09 responsive layout — W10.2)
- **C09 33/45 (73% C) → 34/45 (76% B):** L81.11 1→2 — TUI `is_compact`/`COLUMNS`-adaptive render + Resize; dashboard `@media` 375/768 + landmark smoke.
- Overall stays **~69% C** (mean 829/12).

### 2026-07-13 (C00 OpenAPI drift CI — W10.3)
- **C00 18/30 (60% C) → 19/30 (63% C):** L2 2→3 — full path coverage + `scripts/check-openapi-drift.py` + `openapi-drift.yml`.
- Overall stays **~69% C** (mean 832/12).

### 2026-07-13 (C07 proptest/mutants/fuzz + C08 corpus — W10.4)
- **C07 19/30 (63% C) → 23/30 (77% B):** L65 1→2, L66 0→2, L67 1→2 (proptest + soft mutants/fuzz CI).
- **C08** stays 60% C; adds synthetic corpus fixtures + `criterion-trends.csv`.
- **Overall 69% → 71% C** (mean 846/12).

### 2026-07-13 (C04 OSV + Dependabot groups — W10.5)
- **C04 18/30 (60% C) → 21/30 (70% C):** L37 2→3, L38 1→2, L40 2→3 (OSV workflow, Dependabot groups, container hardening docs).
- Overall stays **~71% C** (mean 856/12).

### 2026-07-14 (C06 container cosign soft — L56)
- **C06 20/30 (67% C) → 21/30 (70% C):** L56 1→2 (`container-cosign-soft.yml` + `scripts/container-cosign-soft.sh`; keyless sign-blob on main).
- Docs: `docs/slsa.md` verify-blob + opt-in GHCR cosign verify commands.
- **Overall 71% C → 72% C** (mean 859/12).
- FR-002 supply-chain traceability for container digest signing.

## Spine links

- Rubric: [phenotype-org-audits/audit-v38](https://github.com/KooshaPari/phenotype-org-audits/tree/main/audit-v38)
- Spine index: [docs/SPINE-INDEX.md](https://github.com/KooshaPari/phenotype-org-audits/blob/main/docs/SPINE-INDEX.md)
- CI truth notes: `audit/CI_TRUTH_FINDINGS.md`
- Boundary: `audit/BOUNDARY_VERIFY_2026-07-10.md`

### 2026-07-14 (C05 Pyroscope soft)
- Docs/recipe for pprof→Pyroscope ingest; L45 stays 2 (in-process agent still soft).

### 2026-07-14 (C01 i18n ADR + C08 corpus harness)
- **C01 20/30 (67% C) → 21/30 (70% C):** L16 0→1 (docs/adr/0003-cli-english-primary.md).
- **C08** stays 18/30 (60% C); closes corpus runner soft gap (scripts/eval/run-corpus.sh + soft CI).
- **Overall 71% → 72% C** (mean 862/12).

### 2026-07-14 (C00 concurrency/memory soft)
- **C00 19/30 (63% C) → 21/30 (70% C):** L7/L8 1→2 (docs/ops/concurrency.md, docs/ops/memory.md, soft miri-soft.yml).
- Overall ~72% → **~73% C** (mean 869/12).

### 2026-07-14 (C08 live corpus health assertions)
- Soft live path: unit test maps `expect.health` fixtures → `healthz_json`; optional `SHARECLI_CORPUS_LIVE=1` curl probe.
- C08 stays 18/30 (60% C); closes “live corpus assertions” soft gap for health fixtures.

### 2026-07-14 (C04 DCO signed-commits soft)
- **C04 21/30 (70% C) → 22/30 (73% C):** L34 0→1 (CONTRIBUTING DCO, docs/ops/signed-commits.md, dco-soft.yml).
- Overall mean **~73% C** (872/12); weighted **~74% C**.

### 2026-07-14 (C10 docs/visual soft)
- **C10 24/36 (67% C) → 28/36 (78% B):** L97 1→2, L105 2→3, L106 2→3, L107 1→2 (`docs/visual/*`).
- Overall mean **~74% C** (883/12); weighted **~75% B**.

### 2026-07-14 (C04 2FA + C10 motion/type tokens)
- **C04 22/30 (73% C) → 23/30 (77% B):** L36 0→1 (maintainer-2fa.md + SECURITY.md).
- **C10 28/36 (78% B) → 30/36 (83% B):** L97 2→3, L102 2→3 (tokens.css type/motion + dashboard reduced-motion).
- Overall mean **~74%** (892/12); weighted **~75% B**.

### 2026-07-14 (C08 ADR seed + thermal gate corpus + C06 DCO provenance)
- **C08 18/30 (60% C) → 22/30 (73% C):** L75–L78 0→1 (ADR-0002 seeded N/A); thermal/gate fixture unit test.
- **C06 21/30 (70% C) → 22/30 (73% C):** L59 1→2 (DCO soft provenance).
- Overall mean **~76% B** (908/12); weighted **~76% B**.

### 2026-07-14 (C11 codesign soft + C06 MCP ADR)
- **C11 30/45 (67% C) → 33/45 (73% C):** L112 0→1 (codesign-notarize.md + soft CI); L114/L115 0→1 (deploy N/A seed).
- **C06 22/30 (73% C) → 23/30 (77% B):** L57 0→1 (ADR-0004 no MCP server).
- Overall mean **~77% B** (918/12); weighted **~77% B**.

### 2026-07-14 (C06 hermetic soft + C10 light theme)
- **C06 23/30 (77% B) → 24/30 (80% B):** L54 1→2 (hermetic-soft.yml + just hermetic).
- **C10 30/36 (83% B) → 31/36 (86% B):** L104 2→3 (Backbone2Light + tokens.css light).
- Overall mean **~77% B** (924/12); weighted **~77% B**.

### 2026-07-14 (C01 a11y checklist + SBOM evidence sync)
- **C01 21/30 (70% C) → 24/30 (80% B):** L17 1→3 (cli-tui-checklist + existing a11y suite); L19 2→3 (CycloneDX SBOM in sbom.yml/release).
- Soft mutants threshold doc (C07 L65 stays 2).
- Overall mean **~78% B** (934/12); weighted **~79% B**.

### 2026-07-14 (MVP finality + OS parity refresh)
- FINALITY host×capability matrix; README four-host install blocks; `release.yml` tray-macos + tray-windows attach (Win soft).
- C11 L108/L110/L118 evidence updated; no GA overclaim for tray/desktop (still beta until L112).

### 2026-07-14 (C05 soft load burst — L50)
- **C05 21/30 (70% C) → 22/30 (73% C):** L50 1→2 (`load-soft.yml` + `scripts/load/healthz_burst.sh` CI wiring + `just load-soft`).
- Overall mean **~78% B** (937/12); weighted **~79% B**.

### 2026-07-14 (C11 auto-update soft — L111)
- **C11 33/45 (73% C) → 34/45 (76% B):** L111 0→1 (`docs/ops/auto-update.md` + deploy.md link).
- Overall mean **~78% B** (940/12); weighted **~79% B**.

### 2026-07-14 (C00 alloc profiling + soft RSS — L8 evidence)
- docs/ops/alloc-profiling.md + soft rss-soft.yml.
- C00 L8 stays 2/3; cluster 21/30 (70% C) unchanged.


### 2026-07-14 (C07 mutants soft threshold harden — L65)
- Soft fail-on-survivors + JSON artifact; L65 stays 2.

### 2026-07-14 (C04 GPG/SSH verified commits soft — L34)
- **C04 23/30 (77% B) → 24/30 (80% B):** L34 1→2 (`gpg-soft.yml` + signed-commits GPG/SSH + ruleset checklist).
- Overall mean **~79% B** (943/12); weighted **~79% B**.

### 2026-07-14 (C11 systemd/Caddy soft samples — L115)
- **C11 34/45 (76% B) → 35/45 (78% B):** L115 1→2 (sample systemd unit + Caddyfile).
- Overall mean **~79% B** (945/12); weighted **~79% B**.

### 2026-07-14 (C02 crypto/privacy soft — L22/L24)
- **C02 24/30 (80% B) → 26/30 (87% B):** L22/L24 1→2 (crypto-keys.md + privacy-tenant.md).
- Overall mean **~79% B** (952/12); weighted **~80% B**.

### 2026-07-14 (C08 live pool soft probe)
- Soft `live-pool-soft.yml` probes `/healthz` + `/health/processes`; C08 cluster stays 22/30 pending agent-eval ADR supersede.

### 2026-07-14 (C05 multi-hop soft — L44 evidence)
- `docs/ops/trace-multihop.md`; L44 stays 2 (CLI/IPC/tray injectors still soft).

### 2026-07-14 (C01 secrets soft — L18 evidence)
- ``docs/ops/secrets.md``; L18 stays 2 (OS keyring still deferred).

### 2026-07-14 (C07 config proptest soft — L66 evidence)
- `docs/ops/config-proptest.md`; L66 stays 2 until root proptest dep lands.

### 2026-07-17 (C09 SR checklist soft — L81.11 evidence)
- `docs/a11y/sr-checklist.md`; L81.11 stays 2 until manual SR pass logged.

### 2026-07-17 (C02 spawn audit soft — L28 evidence)
- `docs/ops/spawn-audit.md`; L28 stays 2 until JSONL spawn rows ship.

### 2026-07-17 (C06 GHCR publish soft — L58 evidence)
- `docs/ops/ghcr-publish.md`; L58 1→2 (release push still manual/soft).

### 2026-07-17 (C05 soak/chaos soft — L47 evidence)
- `docs/ops/soak-chaos.md`; L47 plan seeded (script + CI follow).

### 2026-07-17 (C01 gitleaks polish soft — L19 evidence)
- `docs/ops/gitleaks.md`; L19 stays 3, gitleaks polish documented.

### 2026-07-17 (C04 ruleset checklist soft — L34 evidence)
- `docs/ops/ruleset-checklist.md`; L34 stays 2 until ruleset applied.

### 2026-07-17 (C08 eval corpus soft — L72 evidence)
- `docs/ops/eval-corpus.md`; L72 stays 2.

### 2026-07-17 (C07 portability soft — L70 evidence)
- docs/ops/portability-freebsd-wasm.md; L70 stays 1.

### 2026-07-17 (C10 high-contrast soft — L104 evidence)
- docs/a11y/high-contrast.md; L104 stays 3, L105 1→2.

### 2026-07-17 (C00 perf budgets soft — L6 evidence)
- `docs/ops/perf-budgets.md`; L6 stays 2.

### 2026-07-17 (C02 OAuth/SAML roadmap soft — L21 evidence)
- `docs/ops/oauth-saml-roadmap.md`; L21 stays 3 (JWT resource-server); residual OAuth Code / SAML SP deferred.
### 2026-07-17 (C05 soak healthz script — L47 evidence)
- `scripts/load/soak_healthz.sh` + `just load-soak`; L47 stays 1 until CI soak gate lands.
### 2026-07-17 (C01 advisory hard-fail soft — L19 evidence)
- `docs/ops/advisory-hard-fail.md`; L19 stays 3; hard gate deferred until RustSec backlog cleared.
### 2026-07-17 (C00 lib-sprawl plan soft — L0/L1 evidence)
- `docs/ops/lib-sprawl-plan.md`; L0/L1 stays 2 until crate split lands; cross-ref `error-envelope.md`.
### 2026-07-17 (C11 Win tray hardening soft — L110/L118 evidence)
- `docs/ops/win-tray-hardening.md` + `deploy.md` link; L110/L118 stay 3 (Win CI `continue-on-error`; L112 signing cross-ref only).
### 2026-07-17 (C04 OSV hard-fail soft — L38 evidence)
- `docs/ops/osv-hard-fail.md`; L38 stays 2 until hard gate; cross-ref `advisory-hard-fail.md`.
### 2026-07-17 (C10 golden visual soft — L107 evidence)
- `docs/visual/golden-visual-tests.md`; cross-ref `tests/golden/*`, `assets/tokens.css`, dashboard hex drift; L107 stays 2 until PNG baselines commit.
### 2026-07-17 (C01 i18n/fluent roadmap soft — L16 evidence)
- `docs/ops/i18n-fluent.md`; L16 stays 1 (ADR 0003 English-primary); fluent/gettext catalogs deferred.
### 2026-07-17 (C07 mutants hard-gate soft — L65 evidence)
- `docs/ops/mutants-hard-gate.md`; cross-ref `mutants-threshold.md`; L65 stays 2 until `continue-on-error` removed + branch protection.
### 2026-07-17 (C06 network-block build soft — L54 evidence)
- `docs/ops/network-block-build.md`; `scripts/ci/netblock_check.sh`; L54 stays 2 (offline + `CARGO_NET_OFFLINE` plan; cross-ref `hermetic-builds.md`).
### 2026-07-17 (C06 netblock soft CI — L54 evidence)
- `.github/workflows/netblock-soft.yml` + `scripts/ci/netblock_check.sh` CI wiring (`continue-on-error`); L54 stays 2; cross-ref `hermetic-soft.yml`.
### 2026-07-17 (C08 agent-eval ADR supersede soft — L71/L80 evidence)
- `docs/adr/0005-agent-eval-supersede.md`; L71/L76/L80 supersede when/how documented; ADR 0002 remains authoritative until Phase 4 harness; C08 cluster stays 22/30.
### 2026-07-17 (C03 brew bottle soft — L30 / C11 L109 evidence)
- `docs/ops/brew-bottle.md`; sha placeholder policy + `release.yml` cross-ref + homebrew tap sketch; L109 stays 3 (in-repo digest); tap publish still soft.


### 2026-07-17 (C09 Playwright baseline policy soft — L81.11 evidence)
- `docs/a11y/playwright-viewports.md` baseline commit/artifact policy; cross-ref `golden-visual-tests.md`; L81.11 stays 2 until committed baselines + hard diff.
### 2026-07-17 (C11 in-binary updater soft — L111 evidence)
- `docs/ops/in-binary-updater.md`; cross-ref `auto-update.md`, `deploy.md`; TUF metadata sketch; L111 stays 1 until `self-update` or signed appcast ships (L112).
### 2026-07-17 (C05 soak soft CI — L47 evidence)
- `.github/workflows/soak-soft.yml` + `scripts/load/soak_healthz.sh` CI wiring (60s soft soak, `continue-on-error`); L47 1→2; chaos restart still planned.


### 2026-07-17 (scorecard reconcile v2 — soak CI merged #319)
- **C05 22/30 (73% C) → 23/30 (77% B):** L47 1→2 (`soak-soft.yml` + `soak_healthz.sh` on main).
- Top-3 gaps refreshed: removed completed soak script, live pool, and SR checklist doc items from C05/C08/C09 rows.
- Overall unweighted **~79%** (949/12); weighted **~80% B** (1285/16).

### 2026-07-17 (C08 Harbor Phase 3 soak plan — L76 evidence)
- `docs/ops/harbor-phase3-soak.md`; ADR 0005 Phase 3 checklist + portage/pheno-harness pin table; cross-ref `harbor-eval-stub.md`; L76 stays 1 until seven-day soak completes; C08 cluster stays 22/30.

### 2026-07-17 (C08 Harbor eval stub soft — L71 evidence)
- `docs/ops/harbor-eval-stub.md` + `scripts/eval/harbor_stub.sh` + `harbor-eval-stub-soft.yml`; corpus preflight + stub pass; cross-ref ADR 0005 Phase 2; L71 stays 3; L76 stays 1 until Phase 3 soak; C08 cluster stays 22/30.


### 2026-07-17 (C02 spawn audit JSONL soft — L28 evidence)
- `src/runtime.rs` + `audit_log::emit_if_configured`; `tests/spawn_audit.rs`; `docs/ops/spawn-audit.md` status wired; L28 stays 2 (partial — env-gated spawn/stop rows; signed envelopes + SIEM deferred).

### 2026-07-17 (governance Wave12 planning — W11.7)
- WBS/GAP/DAG/RC/PERT artifacts synced to ~80% B reality; Wave12 T-400..T-440 READY for parallel tick.

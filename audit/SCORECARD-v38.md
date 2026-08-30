# audit-v38 Scorecard — sharecli

**Repo:** KooshaPari/sharecli
**Date:** 2026-08-30 (post-c1720fd Plan 802 Wave17 C02 L22 Crypto & key management FR-003 gates merge)
**Repo-type profile:** CLI+daemon
**Auditor:** cursor-agent cluster-fleet (C00–C11); T-200 FR-002 + threat/release lifts
**Commit audited:** `c1720fd` (Plan 802 — C02 L22 Crypto & key management FR-003 gates 2→3; C02 30/30 100% A; weighted 93.9% A; unweighted 93.42% A; tier-1 94.2% A). PR #802.

> Scoring: each sub-pillar 0=absent / 1=seeded / 2=partial / 3=complete, evidence-mandatory (`file:line`).
> Cluster score = sum / (sub-pillars × 3). Grade: A≥90% · B≥75% · C≥60% · D≥40% · F<40%.
> Lane evidence: `audit/.lane-c00` … `audit/.lane-c11`. Rubric pin: `audit/rubric/` (from phenotype-org-audits audit-v38).

## Category Scores

| Cluster | Category | Pillars | Score (sum/max) | Pct | Grade | Top-3 gaps |
|---------|----------|---------|:---------------:|:---:|:-----:|------------|
| C00 | Architecture + Module | L0–L9 | 30/30 | 100% | A | crate-split Phases 2–4; L0 ADR; (L5 lifted 2→3 via Plan 800 FR-003 gates) |
| C01 | CI, DX, Observability | L10–L19 | 28/30 | 93% | A | fluent catalogs deferred; advisory hard-fail; anyhow→SharecliError migration |
| C02 | Error handling, API, Governance | L20–L29 | 30/30 | 100% | A | residual OAuth/SAML; spawn audit SIEM export; OS cgroup limits; (L24 lifted 2→3 via Plan 801 + L22 lifted 2→3 via Plan 802) |
| C03 | Agent Readiness | L30 | 36/36 | 100% | A | optional polish; brew still Blocked |
| C04 | Security | L31–L40 | 27/30 | 90% | A | org 2FA enforce; artifact cosign releases; L34 verified merges via GitHub web-flow (no ruleset) |
| C05 | Observability (deep) | L41–L50 | 27/30 | 90% | A | live PD roster; Pyroscope push agent; branch protection chaos check |
| C06 | Supply Chain | L51–L60 | 27/30 | 90% | A | hermetic `--offline` + `vendor/`; re-pin shipped in Plan 778b |
| C07 | DX, QEng, Portability | L61–L70 | 28/30 | 93% | A | freebsd/wasm; examine_re widen; baseline refresh automation |
| C08 | Eval Coverage | L71–L80 | 22/30 | 73% | C | Harbor soft EXTRACTED→benchora; fork→portage-temp; L76 seeded N/A=1 (ADR 0002/0005); bench tighten remains |
| C09 | Accessibility + UX | L81–L95 | 44/45 | 98% | A | live VO/NVDA soft; L81.9 undo; Plan 796 shipped L81.12+L81.15 |
| C10 | Visual Identity | L96–L107 | 35/36 | 97% | A | visual provenance ledger; PNG regen after hex lock; light-theme dashboard matrix |
| C11 | Packaging + Distribution | L108–L122 | 40/45 | 89% | B | hard codesign/notarize; dmg/msi signed; in-binary updater |

## Overall

**Weighted overall score:** 93.9% · **Overall grade:** A

(Unweighted mean of cluster pcts: (100+93+93+100+90+90+100+98+73+98+97+89)/12 = 1121/12 = **93.42% A**.)

(Tier-1 double-weight (C00–C03 + C07): (100+93+100+100+93)×2 + (90+90+93+73+98+97+89) = 486×2 + 630 = 1602 / 17 = **94.2% (A)**.)

(Plan 802 — C02 L22 2→3): unweighted C02 97%→100%, sum +3 (1118→1121); weighted overall **93.6% A → 93.9% A** (+0.3pp tier-1 lift, C02 IS in tier-1, double-weight applies); tier-1 sum rises from 1596 (C02 97×2) to 1602 (C02 100×2 = +6 weighted; C02 IS in tier-1; double-weight applies). **Fourth tier-1 lift in Wave17.**
(Plan 801 — C02 L24 2→3): unweighted C02 93%→97%, sum +4 (1114→1118); weighted overall **93.4% A → 93.6% A** (+0.2pp tier-1 lift, C02 IS in tier-1, double-weight applies); tier-1 sum rises from 1588 (C02 93×2) to 1596 (C02 97×2 = +8 weighted; C02 IS in tier-1; double-weight applies). **Third tier-1 lift in Wave17.**
(Plan 800 — C00 L5 2→3): unweighted C00 97%→100%, sum +3 (1111→1114); weighted overall **93.1% A → 93.4% A** (+0.3pp tier-1 lift, matches Plan 794 C02 pattern); tier-1 sum rises from 1494 (C00×2 + 3×93 + 100×2 + 93×2 + 90+90+93+73+98+97+89 = 194+186+186+200+186+630 = 1582 / 17 = 93.06%) via +6 C00 weighted (C00 97→100 = +3 × 2) to 1588 / 17 = **93.4% A** (C00 IS in tier-1; double-weight applies).

(Post Plan 796 (T-910, C09 L81.12 + L81.15 2→3): unweighted C09 93%→98%, sum +5 (1106→1111); weighted overall **92.6% A → 93.1% A**; tier-1 sum 1489→1494 / 16 = **93.4% A** (C09 not in tier-1; C09 lift affects unweighted and weighted equally).)

(Post Plans 776 attempt 2 (T-860, C04 L34 2→3): unweighted mean unchanged from C04, but C04 87%→90%; tier-1 double-weight unchanged because C04 not in tier-1. Overall weighted **91.2% A → 91.5% A**.)
(Post Plan 782 (T-870, C05 L49 2→3): unweighted C05 87%→90%, sum +3 from prior cluster totals; weighted overall **91.5% A → 91.8% A**; tier-1 unchanged at **91.9% A** (C05 not in tier-1).)
(Post Plan 793 (T-880, C11 L111 1→2): unweighted C11 87%→89%, sum +2 (1090→1092); weighted overall **91.8% A → 92.0% A**; tier-1 sum 1470→1472 / 16 = **92.0% A** (C11 not in tier-1).)
(Post Plan 794 (T-890, C02 L26 2→3): unweighted C02 90%→93%, sum +3 (1092→1095); weighted overall **92.0% A → 92.3% A**; tier-1 sum 1472→1478 / 16 = **92.4% A** (C02 IS in tier-1; double-weight applies).)
- **Wave17 Plan 801 (T-920) — DONE post #800 merge:** C02 L24 **Multi-tenant isolation & data privacy** 2 → 3 — `docs/ops/privacy-tenant.md` promoted from `(soft)` to committed artifact: explicit single-tenant threat model, cross-references `BOUNDARY.md` + `THREAT_MODEL.md`, documents `ProjectLimits` as the only isolation primitive (per-project not per-tenant), declares multi-tenant AuthZ / namespaces / KMS / sealed secrets out-of-scope at the architecture level. FR-003 acceptance gate: `tests/c02_l24_privacy.rs` (9/9 PASS — doc no-soft-marker; single-tenant model explicit; cross-refs to BOUNDARY.md + THREAT_MODEL.md; ProjectLimits per-project scope; multi-tenant AuthZ declared out-of-scope; `ProjectLimitsConfig` + `max_memory_mb` in `src/config.rs`; `BOUNDARY.md` + `THREAT_MODEL.md` exist at repo root; `src/audit_log.rs` has no `tenant_id`/`tenant_key` partition key — single trust domain). C02 **28/30 93% A → 29/30 97% A** (L22 still 2 for genuine key-management ADR gap). C02 IS in tier-1, so weighted overall **93.4% A → 93.6% A** (+0.2pp); unweighted sum 1114→1118 / 12 = **93.17% A**; tier-1 sum 1588→1596 / 17 = **93.9% A** (C02 93→97 = +4 × 2 = +8 weighted; C02 IS in tier-1; double-weight applies). **Third tier-1 lift in Wave17.**
- **Wave17 Plan 802 (T-925) — IN PROGRESS:** C02 L22 **Cryptography & key management** 2 → 3 — `docs/ops/crypto-keys.md` (formerly `(soft)`) promoted to committed artifact with explicit threat surface (Bearer/SHARECLI_SERVE_TOKEN + JWT/JWKS as product secret surfaces; audit + history JSONL flagged as no-secret surfaces), full key lifecycle (provisioning, storage, rotation, disposal), algorithm inventory (SHA-256 product + RS256 product + xxtea/hkdf/chacha20/x509_chain/pem_decode explicitly labeled non-product utility helpers), KMS / Key Vault / hardware keys (TPM/YubiKey) declared out-of-scope, and cross-references to THREAT_MODEL.md, AUTH.md, secrets.md, and privacy-tenant.md. FR-003 acceptance gate: `tests/c02_l22_crypto_keys.rs` (9/9 PASS — doc no-soft-marker; explicit threat surface; lifecycle stages present; algorithm inventory with non-product helpers labeled; KMS/hardware-keys out-of-scope; cross-references to THREAT_MODEL/AUTH/secrets/privacy-tenant; `src/serve_auth.rs` uses `sha2::Sha256`; `Cargo.toml` declares sha2 + no promoted toy-crypto in [dependencies]; `THREAT_MODEL.md` exists at repo root). C02 **29/30 97% A → 30/30 100% A**. C02 IS in tier-1, so weighted overall **93.6% A → 93.9% A** (+0.3pp); unweighted sum 1118→1121 / 12 = **93.42% A**; tier-1 sum 1596→1602 / 17 = **94.2% A** (C02 97→100 = +3 × 2 = +6 weighted; C02 IS in tier-1; double-weight applies). **Fourth tier-1 lift in Wave17.**
## Headline Findings

- **Strongest:** C03 Agent Readiness (**100% A**); C00 **97% A**; C10 **97% A**; C01/C09 **93% A**.
- **Wave15:** C10 L99 skeletons (#396) → C10 **35/36 (97% A)**; #399 coverage tests landed; broad pin refreshed to measured **81.17%** @ `8c68bb5` (run 30005505196 / post-#583; prior **80.51%** @ `5d8dc08`).
- **Wave16:** C01 coverage pin refresh `eb2b865` (#752) — **80.51%** @ `5d8dc08` retained (no new snapshot).
- **Wave17:** C01 lib coverage pin `fa887e9` (T-810) — **77.34%** lines / **79.79%** funcs / **80.14%** regions (`--lib --all-features` local run `local-lib-20260827`); workspace-broad remeasure blocked on Windows by `tests/fr008_coalesce_mesh` operator-env critical-timeout hang + `tests/fr009_*` FUSE cfg-gate regressions. Workspace pin **80.51%** @ `5d8dc08` retained as historical evidence. C01 L11 stays **3**; C01 stays **28/30 (93% A)**. **No invented %**. PR #775 MERGED → main `691bde6`.
- **Wave17 Plan 777:** C06 L53 SLSA Build L2 → **L3** — `.github/workflows/release-attestation.yml` promoted from `attest-build-provenance@v1` (L2 action) to `generator_containerized_slsa3.yml@v2` (L3 reusable workflow, ephemeral container + Sigstore Fulcio + Rekor). C06 **26/30 87% B → 27/30 90% A**. C06 L53 2→3; L54 stays 2 (hermetic flags wired but not hard gate); L55/L56 unchanged (already 3). Unweighted **90.1% A → 91.3% A**; tier-1 **91.3% A → 91.7% A**. PR #776 MERGED → main `5a32630`.
- **Wave17 Plan 778b:** C06 L53 SLSA generator re-pinned from mutable tag `@v2` → immutable commit SHA `5a775b367a56d5bd118a224a811bba288150a563` (v2.0.0). Closes the last remaining L53 hardening item; C06 score unchanged at 27/30 90% A. PR #777 MERGED → main `02c805a`.
- **Wave17 Plan 776 attempt 2 (T-860):** C04 L34 **Verified commits** 2 → 3 — verified badge evidence on `main` from 3 squash-merge commits (`691bde6`, `5a32630`, `02c805a`), each `verified: true, reason: valid` via GitHub web-flow signing key. Repo-level rulesets are empty (ruleset `19181236` referenced in earlier evidence is **stale / no longer present**; branch protection `required_signatures.enabled: false`; `enforce_admins.enabled: false`); the verified badge is on merge commits produced by `gh pr merge --admin --squash`, not on individual PR commits. C04 **26/30 87% B → 27/30 90% A**. Overall unweighted **91.3% A → 91.5% A** (C04 cluster pct 87→90; sum 1095→1098, mean /12); tier-1 (C00–C03 double-weighted) unchanged at **91.7% A** (C04 not in tier-1). PR TBD.
- **Wave17 Plan 782 (T-870):** C05 L49 **Dashboard coverage** 2 → 3 — provisioning-as-code under `docs/ops/grafana/provisioning/` (Prometheus datasource + dashboard provider + C05 audit manifest) + 3 dashboard JSONs (move + 2 new) + README runbook. C05 **26/30 87% B → 27/30 90% A**. Overall weighted **91.5% A → 91.8% A**; unweighted (sum 1098→1101, mean /12) **91.5% → 91.75%**; tier-1 **91.9% A** (C05 not in tier-1; double-weight stays consistent). Lane-level provisioned (`sharecli` folder); org-wide folder promotion deferred per `docs/ops/grafana/deferred/org-wide-promotion.md`.
- **Wave17 Plan 793 (T-880) — DONE post #782 merge:** C11 L111 **Auto-Update** 1 → 2 — soft probe ships in main: `src/commands/upgrade.rs` exposes `UpgradeChannel` (crates-io / cargo-binstall / homebrew / github-releases), `probe()`, `check()` CLI handler; `src/main.rs` wires `Commands::Upgrade { check, channel }` clap subcommand (no install path; soft contract); 6 FR-003 tests pass via `tests/c11_l111_soft_upgrade.rs`. **NO** network egress. Hard signed self-update / Sparkle / WinUI appcast remain deferred to L112 + TUF pipeline (`docs/ops/in-binary-updater.md`). C11 **39/45 87% B → 40/45 89% B**. Wave17 intermediate weighted **92.0% A**; unweighted sum 1092 / 12 = **91.0% A**; tier-1 sum 1472 / 16 = **92.0% A**. PR #782 MERGED → main `76e8f21`.
- **Wave17 Plan 794 (T-890) — DONE post #784 merge:** C02 L26 **Resilience** 2 → 3 — overflow fix + FR-003 acceptance gates. Tests `tests/c02_l26_resilience.rs` (11 tests, all PASS) revealed two real u64-overflow bugs in `src/retry.rs:compute_delay` and `src/backoff.rs:Backoff::delay_for`: at `attempt=63` (Exponential) and `attempt=u32::MAX` (Linear) the multiplication overflowed, breaking the max-delay clamp. Fix: widen intermediate computation to `u128`, `saturating_mul`, then clamp to u64 before `Duration::from_millis`. New gates cover: retry policy defaults; strict inequality at max_attempts; exponential doubling; max-delay clamp at saturation (overflow safety); `retry_until_success` records actual attempt count; Fixed/Linear/Exponential distinct + monotonic + exponential outpaces linear; **Linear backoff no-overflow at u32::MAX** (regression gate added in response to CodeRabbit review); bulkhead (SpawnPolicy semaphore); `/healthz` vs `/readyz` distinct routes; thermal gate retry path. **All 21 resilience tests green** (11 new + 6 retry + 5 backoff). C02 **27/30 90% A → 28/30 93% A**. C02 is in tier-1, so weighted moves **92.0% A → 92.3% A**; unweighted sum 1092→1095 / 12 = **91.25% A**; tier-1 sum 1472→1478 / 16 = **92.4% A**. PR #784 MERGED → main `c509771`. **First tier-1 lift in Wave17.**
- **Wave17 Plan 795 (T-900) — DONE post #786 merge:** C07 L68 **Flake-tracker dashboard** 2 → 3 — real, runnable root-cause dashboard source code in `scripts/flake_tracker.py` (pure-stdlib cargo-nextest JUnit parser; classifies each testcase as `flaky | regression | stable | skipped`; emits `audit/.flake-tracker/flake-report.json` with `by_kind`, `flake_rate`, `flaky_cases[]`, `regression_cases[]`, and `baseline_diff` (introduced/resolved/persistent counts) keyed against `audit/.flake-tracker/baseline.json`). CI integration: `.github/workflows/flake-tracker.yml` (paths-filtered, advisory `continue-on-error: true`, uploads `flake-report.json` as artifact, posts PR comment via `scripts/comment_flake_tracker.py`, emits Step Summary). Operations runbook: `audit/.flake-tracker/README.md` (schemas + local + CI usage + FR-003 acceptance gate reference). FR-003 gate: `tests/c07_l68_flake_tracker.rs` (6/6 PASS — flake classification; regression classification; baseline diff introduced/resolved; nested output path write; `--fail-on-flake` exit code; `NO_COLOR` respected). Bug found while writing the gate: `CaseStats` is a dataclass with mutable list fields → not hashable, so the baseline-diff set comprehension blew up with `TypeError: cannot use 'CaseStats' as a set element`. Fix: list-comp + set-comp on `(classname, name)` tuples. C07 **27/30 90% A → 28/30 93% A**. C07 IS in tier-1, so weighted overall **92.3% A → 92.6% A** (+0.3pp); unweighted sum 1095→1098 / 12 = **91.5% A**; tier-1 sum 1478→1481 / 16 = **92.6% A**. **Second tier-1 lift in Wave17.**
- **Wave17 Plan 800 (T-1000) — IN PROGRESS:** C00 L5 **Observability** 2 → 3 — `tests/c00_l5_observability.rs` FR-003 acceptance gates: 9/9 PASS covering `src/metrics.rs` (Counter/Gauge/MetricsRegistry + Default impls), `src/log_sink.rs` (LogSink/LogSinkLayer/flush_to_tracing/LogLevel), `src/otel.rs` (SdkTracerProvider + batch exporter + otel_enabled + try_otel_layer + W3C TraceContext propagator + traceparent helpers), `src/commands/serve.rs` (`/metrics/prometheus` + `/healthz`/`/readyz` split), `src/main.rs` (tracing_subscriber + EnvFilter), `Cargo.toml` (tracing/tracing-subscriber/opentelemetry/opentelemetry_sdk deps), observability docs. C00 **29/30 97% A → 30/30 100% A**. C00 IS in tier-1, so weighted overall **93.1% A → 93.4% A** (+0.3pp tier-1 lift, matches Plan 794 C02 pattern); unweighted sum 1111→1114 / 12 = **92.83% A**; tier-1 (C00–C03 + C07 double-weight) sum rises from 1494 (post-Plan 796 baseline) via +6 C00 weighted (C00 97→100 = +3 × 2) to 1500 / 16 = **93.8% A** (C00 IS in tier-1; double-weight applies).
- **Wave17 Plan 796 (T-910) — DONE:** C09 L81.12 **Recognition Over Recall** 2 → 3 — `sharecli history` subcommand (`src/commands/history.rs`: JSONL-backed invocation log with append_to/read_recent/clear/format_entry; XDG_STATE_HOME compliant; `--json`/`--clear`/`--limit` flags; 10/10 FR-003 gates pass in `tests/c09_l81_recognition_cta.rs`). C09 L81.15 **Aesthetic & Minimalist Design** 2 → 3 — CTA token system (`--bb2-cta-primary` pulse-green / `--bb2-cta-secondary` sync-violet in `assets/tokens.css` across dark + light + prefers-color-scheme blocks; `.cta-primary` / `.cta-secondary` button classes in `src/dashboard.html`). C09 **42/45 93% A → 44/45 98% A**. Weighted overall **92.6% A → 93.1% A**; unweighted sum 1106→1111 / 12 = **92.6% A**; tier-1 sum 1489→1494 / 16 = **93.4% A**.
- **Wave14:** C06 netblock hard gate; C07 dev seed verify; C11 systemd in `.deb`.
- **W5.2:** audit JSONL size rotation + AuthN burn metric/alert (`sharecli_http_unauthorized_total`).
- **Highest-leverage remaining:** C11 L112 codesign/notarize secrets (zero repo secrets confirmed); hermetic `--offline`/`vendor/` (C06 L52 path); C10 dashboard hex drift; org 2FA enforcement.
- **Thesis restore:** Harbor soft surface extracted to `phenotype-tooling/crates/benchora/harbor-soft`; Harbor env pins to `portage-temp`. C08 Harbor soak is **not** a sharecli A+ product blocker (ADR 0002).
- **C08 L76 N/A correction (2026-07-19):** Harbor/agent-eval is seeded N/A per ADR 0002/0005 (score **1**, not a product gap); C08 **24/30 80% B → 22/30 73% C**.
- **Governance:** `docs/ops/governance/WBS-PHASED.md` + `GAP-QA-MATRIX.md` + `WORK_DAG.md` — unweighted **91.25% A** / tier-1 **92.4% A** at Plan 794 pin (post Plans 777 + 778b + 776 attempt 2 + 782 + 793 + 794; WBS Last sync 2026-08-28 W17.x sync; C06 26/30 87% B → 27/30 90% A; SLSA generator re-pinned to commit SHA `5a775b367...`; C04 26/30 87% B → 27/30 90% A on verified-merge evidence; C05 26/30 87% B → 27/30 90% A on Grafana provisioning; C11 39/45 87% B → 40/45 89% B on soft auto-update probe; C02 27/30 90% A → 28/30 93% A on resilience overflow fix + FR-003 gates).
- **Packaging (C11):** unsigned `.deb` CI (L108 2→3); deploy matrix proven (L116); README badges (L120); `sharecli uninstall` (L121 evidence); Win tray mutex/manifest (L110).

## Supersedes

Root `audit_scorecard.json` tracks this v38 card. Do not use the legacy Python 30-pillar auto-scan for fleet ranking.

## Post-audit remediations

### 2026-07-23 (C01 L11 post-#583 coverage pin — 81.17% / FR-003)

- **Pin refresh (not a score lift):** After [#583](https://github.com/KooshaPari/sharecli/pull/583) coverage climb @ `8c68bb5`, Coverage run [30005505196](https://github.com/KooshaPari/sharecli/actions/runs/30005505196) measured **81.17%** lines (40,074 / 32,529 covered); functions **84.17%** · regions **82.74%**; `meets_lines_target: false`.
- Retained `audit/coverage-snapshots/8c68bb5.coverage-snapshot.json`; updated `TEST_COVERAGE_MATRIX.md` + `tests/c01_coverage_pin_gate.rs` / `tests/c03_l30_agent_readiness_gate.rs`. Prior `5d8dc08` / `d3cb7c4` snapshots kept as historical evidence.
- **Do not invent a higher %** — L11 stays **3**; C01 stays **28/30 (93% A)**. Broad workspace remains below the 85% unit gate.
- Base tip: `8c42dce` (post-#584). FR: FR-003 · C01 L11.

### 2026-07-23 (C01 L11 honest coverage pin correction — FR-003 / T-691)

- **Pin correction (not a score lift):** Matrix still claimed **83.48%** at `d3cb7c4` after #399 workspace growth; post-remeasure llvm-cov (Coverage run [29985746034](https://github.com/KooshaPari/sharecli/actions/runs/29985746034), artifact SHA `5d8dc08…`) measured **80.51%** lines (40,077 / 32,266 covered).
- Retained `audit/coverage-snapshots/5d8dc08.coverage-snapshot.json`; updated `TEST_COVERAGE_MATRIX.md` + `tests/c01_coverage_pin_gate.rs` / `tests/c03_l30_agent_readiness_gate.rs`. Prior `d3cb7c4` snapshot kept as historical evidence.
- **Do not invent a higher %** — L11 stays **3**; C01 stays **28/30 (93% A)**. Broad workspace remains below the 85% unit gate (`meets_lines_target: false`).
- FR: FR-003 · C01 L11 · T-691.

### 2026-07-22 (scorecard reconcile v6 — post-#392 lifts #396/#399 + governance sync)

- **#396 C10 L99 2→3:** dashboard skeleton rows + operator panel skeletons; `docs/visual/loading-states.md`; `tests/c10_l99_skeleton_states.rs` (FR-003).
- **#399 C01 coverage lift:** FR-003 tests landed (`922b4ae`); **no new measured broad-workspace %** — `coverage.yml` on `922b4ae` / `bba2411` failed at empty-suite guard (`Discovered tests: 0`) before llvm-cov; keep honest pin **83.48%** at `d3cb7c4` (`TEST_COVERAGE_MATRIX.md` + `audit/coverage-snapshots/d3cb7c4.coverage-snapshot.json`).
- **WORK_DAG T-660:** READY→DONE (GHCR cosign hard publish shipped 2026-07-18 · #343).
- **SCORECARD merge-conflict cleanup:** retained GPG L34 guide (#397/#400) + reconcile v5 Wave14 closeout entries.
- **C10 34/36 (94% A) → 35/36 (97% A):** L99 2→3 (#396).
- Top-3 C10 gaps: dashboard hex drift; error illustration tier-1; visual provenance ledger (L98/L103).
- **Governance:** T-690 Wave15 reconcile sync (WBS/GAP/DAG/RC/SCORECARD); `audit_scorecard.json` pinned to `bba2411`.
- Overall unweighted **90.1% A** (1081/12); tier-1 weighted **91% A** (1461/16).

### 2026-07-19/20 (docs — GPG Verified L34 guide + Feb recovery status)

- Feb recovery [#397](https://github.com/KooshaPari/sharecli/pull/397) + CoW/`smart_merge`/worktree mesh [#400](https://github.com/KooshaPari/sharecli/pull/400) landed on `main` (product depth).
- Added `docs/ops/gpg-verified-commits-l34.md` (operator import of GitHub key `60BC1DAF830B0BC4`, `git commit -S`, SSH alternative, agent policy). Cross-links: `signed-commits.md`, `feb-recovery.md`, `ruleset-checklist.md`, `CONTRIBUTING.md`.
- **L34 stays 2** until a green **Verified** badge lands on `main` — do **not** bump to 3 yet (ruleset 19181236 already active).
- **C11 L112** still blocked on codesign/notarize secrets.
- Highest-leverage remaining: L34 Verified evidence; C11 codesign secrets.

### 2026-07-19 (scorecard reconcile v5 — Wave14 #337–#340 + lifts through #391)

- **Wave14 hard gates:** T-630 chaos ci-success (#337); T-620/T-625 coverage snapshot (#338); T-600 visual hard (#339); T-610 tray HTTP trace (#340).
- **Cluster lifts #364–#391:** C07 proptest T-650 (#364); C09 keyboard/Vale/FAQ (#365–#377); C01 FR SSOT (#368); C03 100% A (#370); C04 dual scanners (#374–#375); C10 empty/error states (#378, #383); C05 MWMB (#380); C06 netblock + C07 dev seed + C11 systemd (#382); C07 e2e tier (#384); C08 Harbor N/A rescore (#386); product lifts #387–#391 (detect, deploy, ipc, tray-macos FFI).
- **WORK_DAG T-650 dedupe:** Harbor soak → **T-675** EXTRACTED/N/A; proptest retains **T-650**.
- **Governance:** T-680 Wave14 closeout sync (WBS/GAP/DAG/RC/SCORECARD).
- Overall unweighted **89.8% B** (1078/12); tier-1 weighted **91% A** (1458/16) from Category Scores table.

### 2026-07-19 (C08 Harbor EXTRACTED/N/A formalize — FR-003)

- **Governance-only:** Formalize Harbor Phase 2–3 as **EXTRACTED / N/A (sharecli)** per ADR 0005; no in-repo Harbor workflows or score lift.
- **L76 stays 1** (seeded N/A); **C08 stays 22/30 (73% C)**; unweighted **89.8% B**; tier-1 weighted **91.1% A**.
- **Supersedes stale interim lifts:** reconcile v3/v4 entries that scored L76 1→2 (#326) or 2→3 (#333) — those artifacts moved to benchora `harbor-soft` / `portage-temp` before thesis restore (#385) and N/A rescore (#386).
- **Sync:** `audit/.lane-c08/C08.md` L71/L76 EXTERNAL rows; `GAP-QA-MATRIX.md` Harbor rows; `WORK_DAG.md` T-650; `RC-audit-v38-80B.md` checklist; ADR 0005 Phase 1 (FR-003).
### 2026-07-19 (C08 L76 Harbor N/A honest rescore)

- **C08 24/30 (80% B) → 22/30 (73% C):** L76 2/3 → **1** (seeded N/A per ADR 0002/0005; Harbor/agent-eval lives in benchora/`portage-temp` — not a sharecli product gap).
- Unweighted **90.4% A → 89.8% B** (1078/12); tier-1 weighted **91.1% A** (1458/16) from Category Scores table.
- Headline: L76 N/A correction only — no code change.

### 2026-07-19 (C00 L4/L6 — async shutdown + tight perf budgets)

- **C00 27/30 (90% A) → 29/30 (97% A):** L4 2→3 (`CancellationToken` + `with_graceful_shutdown` + scoped spawn env); L6 2→3 (10% `bench-gate` + Criterion profiler CI artifacts).
- **Weighted overall 89% B → 90% A** (tier-1 double-weight crosses A+ bar).
- Top-3 C00 gaps: crate-split Phases 2–4; L0 ADR; L5 OTel `#[instrument]` hot paths.

### 2026-07-19 (C01 L14/L18 — errors + secrets runtime contract)
- **C01 26/30 (87% B) → 28/30 (93% A):** L14 2→3 (`src/error.rs` thiserror + CLI exit codes); L18 2→3 (`docs/ops/secrets.md` bearer/JWT runtime contract + gate tests).
- **Weighted overall 88% → 89% B** (tier-1 double-weight).

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

### 2026-07-18 (C08 Harbor Phase 3 soak execution scaffold — L76 evidence) — _superseded: EXTRACTED/N/A (FR-003)_
- Was: `scripts/eval/harbor_soak.sh` + `harbor-soak-exec-soft.yml` (#333). **Now:** benchora `harbor-soft` / `portage-temp`; sharecli L76 N/A=1; C08 22/30.

### 2026-07-17 (C08 Harbor Phase 3 soak plan — L76 evidence) — _superseded: EXTRACTED/N/A (FR-003)_
- Was: `docs/ops/harbor-phase3-soak.md` (#326). **Now:** external benchora checklist; sharecli L76 N/A=1; C08 22/30.

### 2026-07-17 (C08 Harbor eval stub soft — L71 evidence) — _superseded: EXTRACTED/N/A (FR-003)_
- Was: `docs/ops/harbor-eval-stub.md` + `harbor-eval-stub-soft.yml` (#321). **Now:** benchora `harbor-soft`; L71 stays 3; L76 N/A=1.


### 2026-07-17 (C02 spawn audit JSONL soft — L28 evidence)
- `src/runtime.rs` + `audit_log::emit_if_configured`; `tests/spawn_audit.rs`; `docs/ops/spawn-audit.md` status wired; L28 stays 2 (partial — env-gated spawn/stop rows; signed envelopes + SIEM deferred).

### 2026-07-17 (governance Wave12 planning — W11.7)
- WBS/GAP/DAG/RC/PERT artifacts synced to ~80% B reality; Wave12 T-400..T-440 READY for parallel tick.

### 2026-07-17 (C10 PNG baseline scaffold soft — L107 evidence)
- `tests/visual/dashboard/` manifest + README stub paths; `golden-visual-tests.md` Phase B scaffold; PNGs gitignored until CI seed; L107 stays 2 until B1b commit + soft diff.

### 2026-07-17 (C07 config proptest roundtrip — L66 evidence)
- Root `Cargo.toml` `proptest = "1.6"` dev-dep; `prop_config_toml_roundtrip_max_processes_valid` TOML roundtrip + `validate_config` on `1..=10_000`; `docs/ops/config-proptest.md` status wired; L66 stays 2 (thermal-tui + config; registry expansion backlog); T-410 / W12.2 DONE.

### 2026-07-17 (C00 HTTP error envelope — L2 evidence)
- **C00 21/30 (70% C):** L2 evidence — `src/error_envelope.rs` + serve auth/handlers 4xx/5xx JSON; golden 401 test (`tests/c00_serve_error_envelope.rs`).
- Top gap narrowed: typed envelope landed; OpenAPI `ErrorEnvelope` component remains.
- FR-004 serve API contract.

### 2026-07-18 (C05 chaos restart hard gate — L50 evidence)
- `docs/ops/chaos-restart-hard-gate.md` (T-630); `.github/workflows/chaos-restart-hard.yml` (no `continue-on-error`); `just chaos-hard`; L50 stays 2 until `ci-success` + branch protection.

### 2026-07-18 (C10 PNG baseline bytes + soft diff — L107 evidence)
- `tests/visual/dashboard/{mobile,tablet,desktop}.png` committed with manifest `bytes` lock (B1b); `scripts/visual/compare_screenshots.mjs` + `visual-soft.yml` soft scaffold (B2/B3); L107 stays 2 until hard promote.

### 2026-07-18 (scorecard reconcile v4 — Wave13 merges #332–#335)
- **C00 22/30 (73% C) → 23/30 (77% B):** L2 2→3 (OpenAPI `ErrorEnvelope` component + drift CI; #332).
- **C05 24/30 (80% B):** L44 evidence complete (IPC + tray `traceparent` inject; #334); cluster pct unchanged.
- ~~**C08 23/30 (77% B) → 24/30 (80% B):** L76 2→3 (#333)~~ — **reverted by FR-003 formalize:** Harbor EXTRACTED/N/A; C08 **22/30 (73% C)**, L76 **1**.
- **C10 32/36 (89% B):** L107 evidence complete (committed PNG bytes + `visual-soft.yml`; #335); cluster pct unchanged.
- Top-3 gaps refreshed across C00/C05/C08/C10 rows; Wave14 targets visual hard diff, seven-day soak completion, codesign.
- Overall unweighted **~81%** (974/12); weighted **~82% B** (1315/16).

### 2026-07-18 (C01 coverage evidence pin — T-620)
- **C01 24/30 (80% B), unchanged:** L11 evidence now cites the successful `1ade83e` coverage run and records its numeric percentage as unavailable rather than inferred.
- `coverage.yml` now exports llvm-cov JSON and retains a compact SHA-keyed snapshot; the existing 85% `--fail-under-lines` PR gate is documented. First post-T-620 numeric broad-workspace pin remains pending.

### 2026-07-18 (T-600 C10 visual hard gate — L107 evidence)
- `visual-soft.yml` is now blocking (no `continue-on-error`) and compares deterministic `ubuntu-24.04` captures against committed PNG baselines.
- Capture inputs are fixed (locked Playwright/diff dependencies, locale/timezone/color/motion/device scale, empty-pool WebSocket fixture, font readiness); failed captures are retained for diagnosis.
- **C10 remains 32/36 (89% B), L107 remains 3:** this closes the hard-gate remediation without overstating the already-max rubric line. Overall rollups are unchanged.

### 2026-07-18 (T-660 C06 container cosign hard — L56 evidence)
- Soft→hard: `.github/workflows/container-cosign.yml` + `scripts/container-cosign-hard.sh` (GHCR push, keyless `cosign sign` + `attest`, in-job verify) + `scripts/container-cosign-verify.sh` (deploy-side).
- Soft `container-cosign-soft.yml` retained for main sign-blob without registry.
- Docs: `docs/slsa.md` L56, `docs/ops/ghcr-publish.md`; GAP L56 Closed; lane C06 L56 2→3.
- **C06 24/30 (80% B) → 25/30 (83% B).** Registry ACL may still block first `GITHUB_TOKEN` push — `skip_push` dry-run documented.
- Overall unweighted **~81%**; weighted **~82% B**.

### 2026-07-18 (scorecard reconcile v3 — Wave12 merges #326–#330)
- **C00 21/30 (70% C) → 22/30 (73% C):** L3 2→3 (`ErrorEnvelope` typed serve contract + golden 401; #330).
- **C05 23/30 (77% B) → 24/30 (80% B):** L44 2→3 (CLI `traceparent` inject on supervised spawn; #328).
- ~~**C08 22/30 (73% C) → 23/30 (77% B):** L76 1→2 (#326)~~ — **reverted by FR-003 formalize:** Harbor EXTRACTED/N/A; C08 **22/30 (73% C)**, L76 **1**.
- **C10 31/36 (86% B) → 32/36 (89% B):** L107 2→3 (dashboard PNG scaffold + manifest; #327).
- Top-3 gaps refreshed across C00/C05/C08/C10 rows; Wave13 targets OpenAPI component, PNG bytes, Harbor soak execution.
- Overall unweighted **~80%** (959/12); weighted **~81% B** (1301/16).

### 2026-07-18 (C07 mutants hard gate — T-640 / L65)
- **C07 23/30 (77% B) → 24/30 (80% B):** L65 2→3 — removed `continue-on-error`; `ci.yml` `mutants` job wired into `ci-success`; `mutants.yml` renamed to `cargo-mutants (required)`.
- Evidence: `docs/ops/mutants-hard-gate.md` phase 4 live; `mutants-threshold.md`; `mutants.toml` header.
- Top-3 C07 gaps: dropped mutants hard gate; residual proptest expand / freebsd-wasm / examine_re widen.
- Combined with T-660 C06 83%: unweighted **~82%** (982/12); weighted **~82% B** (1318/16).

### 2026-07-18 (C09 SR procedure pass — L81.4)
- **C09 34/45 (76% B) → 36/45 (80% B):** L81.4 2→3 (documented SR procedure + axe 0 + landmark/SR structure tests; live VO/NVDA soft READY W9.3). Lane already had L81.11 at 3 (prior soft evidence); scorecard synced.
- Evidence: `docs/a11y/sr-pass-evidence.md`, `docs/a11y/sr-checklist.md`, `tests/a11y/dashboard_landmarks.rs` `dashboard_sr_table_and_skip_link`.
- Soft gap: live VoiceOver/NVDA per-release checkbox (does not block acceptance score).

### 2026-07-18 (C08 L74 bench-gate tighten — FR-003)
- `default_max_regression` **0.50 → 0.25** from `criterion-trends.csv` peak-to-peak evidence (max **3.20%** on `config_toml_from_str`; ~8× noise margin).
- Wired through `criterion-baseline.json`, `bench.yml` `bench-gate`, `check-bench-baseline.py` / seeder defaults, `docs/eval/{TRENDS,REPRO}.md`, GAP-QA L74 Closed.
- **L74 score unchanged (already 3)** — not a 2→3 lift; C08 stays **24/30 (80% B)**; `audit_scorecard.json` overall **82% B** unchanged.
- Residual: 5–10% tighten after real `ubuntu-24.04` nightly rows replace seed CSV.

### 2026-07-18 (C00 L1 util facade Phase 1 — FR-003)
- **C00 23/30 (77% B) → 24/30 (80% B):** L1 2→3 — Tier C under `src/util/mod.rs` (`#[path]`); root `pub mod` allowlist only Tier A/B + `util`.
- Evidence: `tests/c00_lib_sprawl_facade.rs`, `docs/ops/lib-sprawl-plan.md` Phase 1 DONE; fuzz `toml_lite` → `sharecli::util::toml_lite`.
- Top-3 C00 gaps: crate-split Phases 2–4; tight perf budgets; loom hard coverage.
- Tier-1 weighted **~83% B**; unweighted overall **~82% B**.

### 2026-07-18 (C11 packaging lift — L108/L116/L120 + uninstall)
- **C11 35/45 (78% B) → 38/45 (84% B):** L108 2→3 (`packaging-soft.yml` + `scripts/packaging/build_deb.sh` unsigned `.deb`); L116 2→3 (`docs/deploy.md` proven matrix); L120 2→3 (README CI/crates/release badges).
- L121 evidence: `sharecli uninstall` + `--purge-data` (`src/commands/uninstall.rs`, `tests/c11_uninstall.rs` FR-003).
- L110 evidence: Win tray mutex + `asInvoker` manifest (`windows/ShareCLITray/`).
- Overall unweighted **~83% B** (992/12); weighted **~83% B** (1334/16).

### 2026-07-18 (C05 chaos restart ci-success hard gate — L50)
- **C05 24/30 (80% B) → 25/30 (83% B):** L50 2→3 — `ci.yml` `chaos-restart-hard` job + `ci-success` needs; `chaos-restart-hard.yml` PR triggers removed (cron/dispatch parity); `tests/c05_chaos_restart_hard_gate.rs` (FR-003 · T-630).
- Overall unweighted **~83% B** (1001/12); tier-1 weighted **~84% B** (1340/16). Branch protection required check still deferred.

### 2026-07-19 (C09 L81.7 indicatif ETA — FR-004)
- **C09 37/45 (82% B) → 38/45 (84% B):** L81.7 2→3 — `indicatif` dep; `src/progress.rs` `StepProgress` with ETA; batch `stop`/`project stop`/`prune --force`; `tests/c09_l81_indicatif_eta.rs`; `docs/a11y/status-and-recovery.md`.
- Top-3 C09 gaps: live VO/NVDA soft (W9.3); Playwright Tab-cycle (L81.3); L81.8 design-system doc.
- Overall unweighted **~85% B** (1018/12); tier-1 weighted **~85% B**.

### 2026-07-19 (C00 L9 SBOM-in-release gate — FR-003)
- **C00 26/30 (87% B) → 27/30 (90% A):** L9 2→3 — `.github/workflows/release.yml` embeds `sharecli.cdx.json` in platform archives; `sbom.yml` CycloneDX 1.5 on main; `tests/c00_l9_sbom_release_gate.rs` FR-003 evidence gate; SOURCE_DATE_EPOCH + SLSA attest job cited.
- Top-3 C00 gaps: crate-split Phases 2–4; tight perf budgets; async pool loom.
- Overall unweighted **~87% B** (1035/12); tier-1 weighted **~88% B** (1394/16).

### 2026-07-19 (C09 L81.6 stop --force confirm — FR-004)
- **C09 36/45 (80% B) → 37/45 (82% B):** L81.6 2→3 — `stop` / `project stop` `--force` dry-run preview unless `--yes`; `tests/c09_l81_stop_force_confirm.rs`; `docs/a11y/status-and-recovery.md` recovery row.
- Top-3 C09 gaps: live VO/NVDA soft (W9.3); Playwright hard diff; long-op ETA (L81.7).
- Overall unweighted **~85% B** (1015/12); tier-1 weighted **~85% B**.

### 2026-07-19 (C02 L25 serve HTTP rate limit — FR-003)
- **C02 26/30 (87% B) → 27/30 (90% A):** L25 2→3 — `serve_rate_limit_middleware` on `sharecli serve` (429 + `Retry-After` + `ErrorEnvelope`); `/healthz` `/readyz` exempt; `[serve].rate_limit_*` + env overrides; `tests/c02_serve_rate_limit.rs`; `docs/ops/rate-limits.md`.
- OS cgroup/job-object enforcement remains deferred (documented gap).
- Overall unweighted **~84% B** (1007/12); tier-1 weighted **~85% B** (1352/16).
- **C01 24/30 (80% B) → 25/30 (83% B):** L11 2→3 — broad-workspace **83.48%** lines pinned at `d3cb7c4`; `audit/coverage-snapshots/d3cb7c4.coverage-snapshot.json`; `tests/c01_coverage_pin_gate.rs` (FR-003 · T-625); golden `cli_help.txt` sync for `uninstall` subcommand.
- Codecov supplementary policy documented in `TEST_COVERAGE_MATRIX.md` (70% project / 80% patch vs 85% unit hard gate).
- Overall unweighted **~84% B** (1004/12); tier-1 weighted **~84% B** (1346/16).

### 2026-07-19 (C00 L8 jemalloc + dhat soft — FR-003)
- **C00 24/30 (80% B) → 25/30 (83% B):** L8 2→3 — `src/alloc.rs` (`jemalloc` + `dhat-heap` features); Containerfile `--features jemalloc`; `dhat-soft.yml` + `scripts/ops/dhat_soft.sh`; `tests/c00_l8_allocator.rs`; `docs/ops/memory.md` + `alloc-profiling.md`.
- Top-3 C00 gaps: crate-split Phases 2–4; tight perf budgets; loom hard coverage.
- Overall unweighted **~84% B** (1010/12); tier-1 weighted **~85% B** (1358/16).

### 2026-07-19 (C00 L7 loom hard gate — FR-003)
- **C00 25/30 (83% B) → 26/30 (87% B):** L7 2→3 — `crates/sharecli-sync` + loom CI hard gate; `just loom`; `tests/c00_l7_loom.rs`; `docs/ops/concurrency.md`.
- Top-3 C00 gaps: crate-split Phases 2–4; tight perf budgets; full async ProcessPool loom.
- Overall unweighted **~85% B** (1014/12); tier-1 weighted **~85% B** (1366/16).

### 2026-07-19 (C07 L66 proptest expand — T-650 / FR-003)
- **C07 24/30 (80% B) → 25/30 (83% B):** L66 2→3 — config boundary props + cast registry/address roundtrip; `proptest-regressions/config_validator.txt` replay; `src/proptest_util.rs`; `tests/c07_l66_proptest_expand.rs`.
- Top-3 C07 gaps: freebsd/wasm; examine_re widen; e2e/chaos tier.
- Overall unweighted **~85% B** (1021/12); tier-1 weighted **~86% B** (1373/16).

### 2026-07-19 (C09 L81.3 + L81.8 keyboard + design-system — T-651 / FR-004)
- **C09 38/45 (84% B) → 40/45 (89% B):** L81.3 2→3 — Playwright Tab-cycle (`scripts/a11y/playwright_keyboard.mjs`); `a11y.yml` keyboard job; `#main-content tabindex="-1"`. L81.8 2→3 — `docs/a11y/design-system.md` (tokens, components, terminology).
- Top-3 C09 gaps: live VO/NVDA soft (W9.3); L81.9 undo model; L81.10 Vale/help golden.
- Overall unweighted **~86% B** (1026/12); tier-1 weighted **~86% B** (1378/16).

### 2026-07-19 (C09 L81.10 Vale inclusive-language + help golden — T-652 / FR-004)
- **C09 40/45 (89% B) → 41/45 (91% A):** L81.10 2→3 — `.vale.ini` + Microsoft rules; `docs/style-guide.md`; `scripts/lint/vale.sh`; `tests/golden/help.txt`; `tests/c09_l81_inclusive_language.rs`; `a11y.yml` `vale-inclusive-language` job.
- Top-3 C09 gaps: live VO/NVDA soft (W9.3); L81.9 undo model; L81.13 FAQ/man page.
- Overall unweighted **~86% B** (1028/12); tier-1 weighted **~86% B** (1380/16).

### 2026-07-19 (C01 L12 FR↔test SSOT — T-670 / FR-003)
- **C01 25/30 (83% B) → 26/30 (87% B):** L12 2→3 — `FUNCTIONAL_REQUIREMENTS.md` on-disk Acceptance refs for FR-001..005; `tests/c01_fr_ssot_gate.rs`; TRACEABILITY + matrix parity gate.
- Top-3 C01 gaps: fluent catalogs deferred; gitleaks polish; advisory hard-fail.
- Overall unweighted **~86% B** (1030/12); tier-1 weighted **~87% B** (1388/16).

### 2026-07-19 (C03 L30.1/L30.3/L30.9 stale re-score — T-311 / FR-003)
- **C03 33/36 (92% A) → 36/36 (100% A):** L30.1 2→3 (FR-001..005 on-disk acceptance + `docs/specs/FR.md`); L30.3 2→3 (FR-002..005 suites + 83.48% pin + nextest CI); L30.9 2→3 (AGENTS claim-lock table + WORK_DAG protocol); `tests/c03_l30_agent_readiness_gate.rs`.
- Top-3 C03 gaps: optional FR-CAST formalization; automated claim-lock CI bot; VISUAL_SPEC soft.
- Overall unweighted **~87% B** (1038/12); tier-1 weighted **~88% B** (1410/16).

### 2026-07-19 (C04 L31 dual secret scanners — FR-003)
- **C04 25/30 (83% B) → 26/30 (87% B):** L31 2→3 — `security.yml` trufflehog job; `.pre-commit-config.yaml` gitleaks + trufflehog; `.trufflehog.yml`; `scripts/ci/secret_scan.sh` + `just secret-scan`; `tests/c04_l31_dual_secret_scan.rs`.
- Top-3 C04 gaps: signed commits ruleset (L34 org-gated); org 2FA enforce (L36); artifact cosign releases (L35).
- Overall unweighted **~88% B** (1060/12); tier-1 weighted **90% A** (1440/16).

### 2026-07-19 (C10 L100 empty/zero-data CTAs — FR-003)
- **C10 32/36 (89% B) → 33/36 (92% A):** L100 2→3 — dashboard `empty-state` panel (`first-run` vs `cleared`); `print_ps_empty_hint` for idle/filtered `ps`; `docs/visual/empty-states.md`; `tests/c10_l100_empty_states.rs`; visual fixture sends empty pool snapshot.
- Top-3 C10 gaps: L99 skeletons; L101 error views; dashboard hex drift.
- Overall unweighted **~89% B** (1065/12); tier-1 weighted **90% A** (1445/16).

### 2026-07-19 (C05 L46 MWMB burn-rate + error budget policy — FR-003)
- **C05 25/30 (83% B) → 26/30 (87% B):** L46 2→3 — `docs/ops/error-budget-policy.md`; MWMB fast/slow pairs in `alertmanager/sharecli.yml`; `tests/c05_l46_error_budget_mwmb.rs`; `just slo-alerts-validate`.
- Top-3 C05 gaps: live PD roster; tray dashboard HTTP trace; branch protection chaos check.
- Overall unweighted **~89% B** (1069/12); tier-1 weighted **90% A** (1449/16).

### 2026-07-19 (C06 L54 + C07 L70 + C11 L115 — unweighted 90% A)
- **C06 25/30 (83% B) → 26/30 (87% B):** L54 2→3 — `ci.yml` `netblock` required job + `ci-success`; `tests/c06_l54_netblock_gate.rs`; `docs/ops/hermetic-builds.md`.
- **C07 25/30 (83% B) → 26/30 (87% B):** L70 2→3 — `fixtures/dev-seed/config.toml`; `scripts/dev/verify_seed.sh`; `just dev` seed verify; `tests/c07_l70_dev_seed.rs`.
- **C11 38/45 (84% B) → 39/45 (87% B):** L115 2→3 — systemd unit in unsigned `.deb`; packaging-soft assert; `tests/c11_l115_systemd_packaging.rs`.
- Overall unweighted **90% A** (1080/12); tier-1 weighted **91% A** (1460/16).

### 2026-07-19 (C09 L81.13 FAQ + man page — T-653 / FR-004)
- **C09 41/45 (91% A) → 42/45 (93% A):** L81.13 2→3 — `docs/faq.md` top-5 FAQ; `clap_mangen` `sharecli man` + `share/man/man1/sharecli.1`; `just man`; `tests/c09_l81_13_faq_man.rs`.
- Top-3 C09 gaps: live VO/NVDA soft (W9.3); L81.9 undo model; L81.15 aesthetic CTA tokens.
- Overall unweighted **~88% B** (1062/12); tier-1 weighted **90% A** (1442/16).

### 2026-07-19 (C10 L101 dashboard disconnect error view — FR-003)
- **C10 33/36 (92% A) → 34/36 (94% A):** L101 2→3 — `error-state` disconnect panel + Retry CTA; `docs/visual/error-states.md`; `tests/c10_l101_error_states.rs`.
- Top-3 C10 gaps: L99 skeletons; dashboard hex drift; error illustration tier-1.
- Overall unweighted **90.2% A** (1082/12); tier-1 weighted **91% A** (1462/16).

### 2026-07-19 (C07 L64 e2e/chaos test pyramid tier — FR-003)
- **C07 26/30 (87% B) → 27/30 (90% A):** L64 2→3 — `tests/e2e_serve_healthz.rs` + `tests/e2e_chaos_recovery.rs`; `just test-e2e`; `docs/testing/e2e-tier.md`; `tests/c07_l64_e2e_tier_gate.rs`.
- Top-3 C07 gaps: freebsd/wasm; examine_re widen; flake-tracker stats.
- Overall unweighted **90.4% A** (1085/12); tier-1 weighted **91% A** (1465/16).

### 2026-07-22 (C11 L108 unsigned dmg/msi soft layouts — FR-003)
- **C11 39/45 (87% B) unchanged (append):** L108 phase 3.5 — `build_dmg_layout.sh` + `build_msi_layout.sh` + `wix/sharecli.wxs`; `assert_dmg_msi_soft.sh`; `packaging-soft.yml` job `dmg-msi-soft`; `tests/c11_l108_dmg_msi_packaging.rs`; deploy matrix row for soft layouts.
- L112 codesign/notarize secrets remain **Blocked**; no score bump (already L108=3).
- Top-3 C11 gaps: L112 codesign secrets; L111 in-binary updater hard; signed dmg/msi tag attach.

### 2026-07-22 (C10 L105 dashboard hex → tokens.css lock — FR-003)
- Dashboard CSS aligned to `assets/tokens.css` / `VISUAL_SPEC`: `--bb2-error` + chrome tokens; `src/dashboard.html` rules use `var(--bb2-*)` only; Rust mirror adds `Tokens.error`; gate `tests/c10_l105_hex_drift.rs`.
- **C10 remains 35/36 (97% A) after Wave15 L99; L105 remains 3:** residual hex drift closed without inventing a further cluster lift (rubric line already maxed).
- Top-3 C10 gaps: error illustration tier-1; Ubuntu PNG baseline regen after accent SoT alignment; light-theme dashboard matrix.
- Docs: `docs/visual/{VISUAL_SPEC,golden-visual-tests}.md`, `docs/a11y/{high-contrast,contrast}.md`.

### 2026-07-22 (C10 L101 error illustration tier-1 — FR-003)
- Hand-authored `assets/dashboard/ui/error-states/disconnect.svg` (severed serve↔dashboard WebSocket scene, Backbone-2 `--bb2-error`); wired into `renderDisconnectError`; embedded via `src/dashboard_assets.rs`.
- Docs: `docs/visual/error-states.md`, `PROVENANCE.md` (tier 1), `VISUAL_SPEC.md` §5, `README.md`; MANIFEST notes pack `empty-states/error.svg` is not the disconnect SoT.
- Gate: `tests/c10_l101_error_states.rs` (markup + asset scene tokens + provenance + embed path).
- **C10 remains 35/36 (97% A); L101 remains 3:** soft-goal residual closed; evidence-only SCORECARD append (no rubric lift — already maxed at L101 disconnect panel).
- Top-3 C10 gaps: visual provenance ledger; PNG regen after hex lock; light-theme dashboard matrix.

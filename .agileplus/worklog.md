# AgilePlus worklog — sharecli

## 2026-07-23 (C01 L11 honest coverage pin correction — T-691 / FR-003)
- Status: DONE — corrected stale **83.48%** @ `d3cb7c4` claim to measured **80.51%** @ `5d8dc08` from Coverage run 29985746034 (`coverage-snapshot-5d8dc08…` artifact)
- Artifacts: `audit/coverage-snapshots/5d8dc08.coverage-snapshot.json`; `TEST_COVERAGE_MATRIX.md` v1.6; pin gates updated; prior `d3cb7c4` snapshot retained
- Score: L11 stays **3**; C01 **28/30 (93% A)** unchanged (documentation honesty, not a lift)
- Note: do **not** invent a higher percentage — broad workspace still below 85% unit gate
- FR: FR-003 · C01 L11 · T-691

## 2026-07-22 (C11 L108 unsigned dmg/msi soft layouts — FR-003)
- Status: DONE — phase 3.5 soft scaffolds (`build_dmg_layout.sh`, `build_msi_layout.sh`, `wix/sharecli.wxs`, `assert_dmg_msi_soft.sh`, packaging-soft `dmg-msi-soft`, FR-003 tests)
- Score: C11 **39/45 (87% B)** unchanged (append); L112 still Blocked
- FR: FR-003 · C11 L108

## 2026-07-22 (Wave15 governance reconcile — T-690 / FR-003)
- Status: DONE — reconcile SCORECARD v6 after #392 merges (#396 C10 L99 2→3; #399 coverage tests at `922b4ae`); resolve SCORECARD merge conflict; WORK_DAG T-660 READY→DONE + Wave15 stub (T-685 DONE / T-691 READY / T-692 READY); pin `audit_scorecard.json` at `bba2411`
- Coverage: measured post-#399 % **unavailable** (`coverage.yml` empty-suite on `922b4ae`/`bba2411`); honest pin remains **83.48%** @ `d3cb7c4`
- Score: C10 34/36 (94% A) → 35/36 (97% A); unweighted **90.1% A**; tier-1 weighted **91% A**
- FR: FR-003 · T-690

## 2026-07-19 (C08 Harbor EXTRACTED/N/A formalize — FR-003)
- Status: DONE — ADR 0005 Phase 1; Harbor Phase 2–3 **EXTRACTED** to benchora `harbor-soft` / `portage-temp`; sharecli hosts no Harbor workflows
- Score: L76 **1** (seeded N/A); C08 **22/30 (73% C)** unchanged; reconcile v3/v4 L76 interim lifts superseded
- Sync: `audit/.lane-c08/C08.md` · `SCORECARD-v38.md` · `GAP-QA-MATRIX.md` · `WORK_DAG.md` T-650 · `RC-audit-v38-80B.md` · `GOVERNANCE.md`
- FR: FR-003 · C08 L71/L76 · ADR 0005
## 2026-07-19 (Wave14 governance closeout — T-680 / FR-003)
- Status: DONE — reconcile WBS/GAP/DAG/RC/SCORECARD after Wave14 #337–#340 + cluster lifts through #391; WORK_DAG T-650 dedupe (Harbor → T-675 EXTRACTED)
- Score: unweighted **89.8% B**; tier-1 weighted **91% A** (pin `0fa1fd0`)
- FR: FR-003 · T-680

## 2026-07-19 (C03 L30.1/L30.3/L30.9 stale re-score — T-311 / FR-003)
- Status: DONE — L30.1/L30.3/L30.9 2→3; `tests/c03_l30_agent_readiness_gate.rs`; lane + SCORECARD sync
- Score: C03 33/36 (92% A) → 36/36 (100% A); unweighted overall ~87% B; tier-1 weighted ~88% B
- FR: FR-003 · C03 L30.1/L30.3/L30.9 · T-311

## 2026-07-19 (C01 L12 FR↔test SSOT — T-670 / FR-003)
- Status: DONE — `FUNCTIONAL_REQUIREMENTS.md` on-disk Acceptance refs FR-001..005; `tests/c01_fr_ssot_gate.rs`; TRACEABILITY + matrix parity
- Score: C01 25/30 (83% B) → 26/30 (87% B); L12 2→3; unweighted overall ~86% B; tier-1 weighted ~87% B
- FR: FR-003 · C01 L12 · T-670

## 2026-07-19 (C09 L81.10 Vale inclusive-language + help golden — FR-004)
- Status: DONE — `.vale.ini` + Microsoft rules; `docs/style-guide.md`; `scripts/lint/vale.sh`; `tests/golden/help.txt`; `tests/c09_l81_inclusive_language.rs`; `a11y.yml` vale job
- Score: C09 40/45 (89% B) → 41/45 (91% A); L81.10 2→3; unweighted overall ~86% B
- FR: FR-004 · C09 L81.10

## 2026-07-19 (C09 L81.7 indicatif ETA — FR-004)
- Status: DONE — `indicatif` dep; `src/progress.rs` StepProgress ETA; batch stop/project stop/prune; `tests/c09_l81_indicatif_eta.rs`
- Score: C09 37/45 (82% B) → 38/45 (84% B); L81.7 2→3; unweighted overall ~85% B
- FR: FR-004 · C09 L81.7

## 2026-07-19 (C00 L8 jemalloc + dhat soft — FR-003)
- Status: DONE — `src/alloc.rs` jemalloc/dhat-heap features; Containerfile jemalloc build; `dhat-soft.yml`; `tests/c00_l8_allocator.rs`
- Score: C00 24/30 (80% B) → 25/30 (83% B); L8 2→3; unweighted overall ~84% B; tier-1 weighted ~85% B
- FR: FR-003 · C00 L8

## 2026-07-19 (C01 coverage numeric pin — T-625 / FR-003)
- Status: DONE — pinned broad-workspace **83.48%** lines at `d3cb7c4`; `audit/coverage-snapshots/d3cb7c4.coverage-snapshot.json`; `tests/c01_coverage_pin_gate.rs`; golden `cli_help.txt` `uninstall` sync
- Score: C01 24/30 (80% B) → 25/30 (83% B); L11 2→3; unweighted overall ~84% B; tier-1 weighted ~84% B
- FR: FR-003 · C01 L11 · T-625

## 2026-07-18 (C05 chaos ci-success hard gate — T-630 / FR-003)
- Status: DONE — `ci.yml` `chaos-restart-hard` job + `ci-success` needs; `chaos-restart-hard.yml` cron/dispatch parity; `tests/c05_chaos_restart_hard_gate.rs`
- Score: C05 24/30 (80% B) → 25/30 (83% B); L50 2→3; unweighted overall ~83% B; tier-1 weighted ~84% B
- FR: FR-003 · C05 L50 · T-630

## 2026-07-18 (C04 OSV hard gate — T-655 / FR-003)
- Status: DONE — removed `continue-on-error` + soft pass shim; `ci.yml` `osv` + `ci-success` needs; `scripts/ci/osv_scan.sh` + `just osv-scan`; `tests/c04_osv_hard_gate.rs`
- Score: C04 24/30 (80% B) → 25/30 (83% B); L38 2→3; unweighted overall ~83% B; tier-1 weighted ~84% B
- FR: FR-003 · C04 L38 · T-655

## 2026-07-18 (C09 SR procedure pass — L81.4 / FR-004)
- Status: DONE — documented SR checklist + evidence (`docs/a11y/sr-checklist.md`, `sr-pass-evidence.md`); axe 0 violations; `dashboard_sr_table_and_skip_link` test; L81.4 2→3; C09 34→36/45 (80% B)
- Soft READY: live VoiceOver/NVDA per-release checkbox (W9.3)
- FR: FR-004 NFR · C09 L81.4

## 2026-07-18 (C07 mutants hard gate — T-640 / FR-003)
- Status: DONE — removed `continue-on-error`; `ci.yml` `mutants` + `ci-success` needs; `mutants.yml` → `cargo-mutants (required)`; examine_globs + `-p sharecli-thermal-tui`; L65 2→3; C07 23→24/30 (80% B)
- FR: FR-003 · C07 L65 · T-640

## 2026-07-18 (T-660 C06 container cosign hard — L56 / FR-003)
- Status: DONE — soft→hard GHCR keyless cosign sign+attest+verify (`container-cosign.yml`, `container-cosign-hard.sh`, `container-cosign-verify.sh`); soft sign-blob retained; docs/slsa.md + ghcr-publish.md; GAP L56 Closed
- Score: C06 24/30 (80% B) → 25/30 (83% B); L56 2→3
- Note: first live GHCR push may need org package ACL for `GITHUB_TOKEN`; `skip_push` dry-run documented
- FR: FR-003

## 2026-07-18 (C05 chaos restart hard gate — FR-003)
- Status: DONE — `docs/ops/chaos-restart-hard-gate.md` (T-630); `chaos-restart-hard.yml` (no `continue-on-error`); `just chaos-hard`; soak-chaos.md + load README cross-refs; L50 stays 2 until ci-success + branch protection
- FR: FR-003

## 2026-07-18 (C01 coverage evidence pin — T-620 / FR-003)
- Status: DONE — successful base-SHA Coverage run 29640723234 verified; numeric percentage explicitly recorded unknown because the run retained no percentage-bearing report.
- Automation: `coverage.yml` now emits a compact, SHA-keyed llvm-cov snapshot artifact via `scripts/coverage_snapshot.py`; `TEST_COVERAGE_MATRIX.md` documents the 85% hard PR gate.
- Score: C01 remains 24/30 (80% B); first post-T-620 numeric broad-workspace pin is pending.
- FR: FR-003

## 2026-07-18 (governance Wave13 closeout — FR-003)
- Status: IN_PROGRESS — WBS W13.1–W13.4 DONE; GAP-QA T-500..T-530 closed; WORK_DAG T-500..T-530 completed; RC @ 6466b4e ~82% B; SCORECARD reconcile v4 (#332–#335)
- Score: C00 77% B · C05 80% B · C08 80% B · C10 89% B · weighted ~82% B
- FR: FR-003

## 2026-07-18 (C08 Harbor Phase 3 soak execution scaffold — FR-003)
- Status: EXTRACTED / N/A (sharecli) — superseded by benchora `harbor-soft` + FR-003 formalize; was #333 in-repo scaffold
- FR: FR-003

## 2026-07-18 (C05 trace IPC + tray injectors — FR-003)
- Status: DONE — `apply_traceparent_spawn_env` in `src/otel.rs`; tray FFI sidecar spawn in `crates/sharecli-ffi`; `tests/c05_trace_ipc_tray_inject.rs`; `docs/ops/trace-multihop.md` IPC/tray rows wired; T-530 / W13.4

## 2026-07-18 (C10 PNG baseline bytes + soft diff — FR-003)
- Status: DONE — committed `tests/visual/dashboard/*.png` with manifest `bytes`; `compare_screenshots.mjs` + `visual-soft.yml` scaffold; golden-visual-tests Phase B1b/B2/B3
- FR: FR-003

## 2026-07-18 (governance Wave12 closeout — FR-003)
- Status: DONE — WBS Wave11.7+Wave12 DONE; GAP-QA T-400..T-440 closed; WORK_DAG completed table; PERT-DAG-W12 DONE + Wave13 stub; RC @ 03391c4 ~81% B; SCORECARD reconcile v3 (#326–#330)
- Score: C00 73% C · C05 80% B · C08 77% B · C10 89% B · weighted ~81% B
- FR: FR-003

## 2026-07-17 (governance Wave12 sync — FR-003)
- Status: DONE — WBS-PHASED Wave11+12, GAP-QA Jul-17 closures, WORK_DAG T-400..T-450, PERT-DAG-W12, RC-audit-v38-80B (#325)
- FR: FR-003

## 2026-07-17 (C08 Harbor Phase 3 soak plan — FR-003)
- Status: EXTRACTED / N/A (sharecli) — superseded by benchora checklist + FR-003 formalize; was `docs/ops/harbor-phase3-soak.md` (#326)
- FR: FR-003

## 2026-07-17 (C10 PNG baseline scaffold soft — FR-003)
- Status: DONE — `tests/visual/dashboard/` placeholder + `golden-visual-tests.md` Phase B scaffold; PNG stub paths gitignored until browser seed
- FR: FR-003

## 2026-07-17 (C07 config proptest roundtrip — FR-003)
- Status: DONE — root `proptest = "1.6"` dev-dep; `prop_config_toml_roundtrip_max_processes_valid` in `config_validator.rs`; T-410 / W12.2; L66 stays 2 (thermal-tui + config; registry backlog)
- FR: FR-003

## 2026-07-17 (C00 HTTP error envelope — FR-004)
- Status: DONE — `src/error_envelope.rs`; serve_auth 401 + handler 4xx/5xx JSON unified; `tests/c00_serve_error_envelope.rs`
- Score: C00 L2 evidence; cluster 21/30 (70% C) unchanged
- FR: FR-004 serve API contract
- Next: OpenAPI `ErrorEnvelope` component; request-id middleware

## 2026-07-17 (C05 chaos restart soft — FR-003)
- Status: DONE — `scripts/load/chaos_restart.sh`, `just chaos-soft`, soak-chaos.md CI-skip; L47 stays 2; L48 stays 2
- FR: FR-003

## 2026-07-17 (C02 spawn audit JSONL soft — FR-004)
- Status: DONE — `src/runtime.rs` + `audit_log::emit_if_configured`; `tests/spawn_audit.rs`; spawn-audit.md status wired; L28 stays 2
- FR: FR-004

## 2026-07-17 (C06 netblock soft CI — FR-003)
- Status: DONE — netblock-soft.yml runs netblock_check.sh; network-block-build.md workflow link; L54 stays 2
- FR: FR-003

## 2026-07-17 (C08 Harbor eval stub soft — FR-003)
- Status: EXTRACTED / N/A (sharecli) — superseded by benchora `harbor-soft` + FR-003 formalize; was #321 in-repo stub
- FR: FR-003

## 2026-07-17 (scorecard reconcile v2 — FR-003)
- Status: DONE — SCORECARD-v38.md C05 22→23/30 (73%→77% B); Top-3 gaps trimmed; fixed corrupted `` `n `` tail bullets; overall weighted ~80% B
- Score: C05 L47 1→2 after soak-soft.yml (#319); unweighted ~79%; weighted ~80% B
- FR: FR-003

## 2026-07-17 (C05 soak soft CI — FR-003)
- Status: DONE — soak-soft.yml starts serve + soak_healthz (60s soft); soak-chaos.md link; L47 1→2
- FR: FR-003

## 2026-07-17 (C11 in-binary updater soft — FR-003)
- Status: DONE — docs/ops/in-binary-updater.md; cross-ref auto-update.md + deploy.md; TUF sketch; L111 stays 1; hard self-update deferred until L112
- FR: FR-003

## 2026-07-17 (C09 Playwright baseline policy soft — FR-003)
- Status: DONE — docs/a11y/playwright-viewports.md baseline commit/artifact policy; L81.11 stays 2

## 2026-07-17 (C03 brew bottle soft — FR-003)
- Status: DONE — docs/ops/brew-bottle.md; sha placeholder policy; release.yml cross-ref; homebrew tap sketch; L109 stays 3 until tap lands
- FR: FR-003

## 2026-07-17 (C08 agent-eval ADR supersede soft — FR-003)
- Status: DONE — `docs/adr/0005-agent-eval-supersede.md`; L71/L80 supersede triggers + phases; ADR 0002 stays in force until Phase 4
- FR: FR-003

## 2026-07-17 (C06 network-block build soft — FR-003)
- Status: DONE — docs/ops/network-block-build.md; scripts/ci/netblock_check.sh; hermetic-builds.md cross-ref; L54 stays 2
- FR: FR-003
## 2026-07-17 (C07 mutants hard-gate soft — FR-003)
- Status: DONE — docs/ops/mutants-hard-gate.md; cross-ref mutants-threshold.md; L65 stays 2
- FR: FR-003
## 2026-07-17 (C01 i18n/fluent roadmap soft — FR-004)
- Status: DONE — docs/ops/i18n-fluent.md; L16 stays 1; ADR 0003 cross-ref; fluent/gettext deferred
- FR: FR-004
## 2026-07-17 (C10 golden visual soft — FR-003)
- Status: DONE — docs/visual/golden-visual-tests.md; L107 stays 2
- FR: FR-003
## 2026-07-17 (C04 OSV hard-fail soft — FR-004)
- Status: DONE — docs/ops/osv-hard-fail.md; L38 stays 2; hard gate deferred; cross-ref advisory-hard-fail.md
- FR: FR-004

## 2026-07-17 (C11 Win tray hardening soft — FR-003)
- Status: DONE — docs/ops/win-tray-hardening.md; deploy.md link; L110/L118 evidence; Win CI still soft
- FR: FR-003
## 2026-07-17 (C00 lib-sprawl plan soft — FR-003)
- Status: DONE — docs/ops/lib-sprawl-plan.md; L0/L1 stays 2
- FR: FR-003
## 2026-07-17 (C01 advisory hard-fail soft — FR-004)
- Status: DONE — docs/ops/advisory-hard-fail.md; L19 stays 3; hard gate deferred
- FR: FR-004

## 2026-07-17 (C05 soak healthz script — FR-003)
- Status: DONE — `scripts/load/soak_healthz.sh`, `just load-soak`, soak-chaos.md link; L47 stays 1
- FR: FR-003
## 2026-07-17 (C02 OAuth/SAML roadmap soft — FR-004)
- Status: DONE — docs/ops/oauth-saml-roadmap.md; L21 stays 3; residual OAuth Code / SAML SP deferred
- FR: FR-004

## 2026-07-17 (C00 perf budgets soft — FR-003)
- Status: DONE — docs/ops/perf-budgets.md; L6 stays 2
- FR: FR-003
## 2026-07-17 (C10 high-contrast soft — FR-003)
- Status: DONE — docs/a11y/high-contrast.md; L104 stays 3, L105 1→2
- FR: FR-003
## 2026-07-17 (C07 portability soft — FR-003)
- Status: DONE — portability-freebsd-wasm.md; L69/L70 evidence (L70 stays 1)
- FR: FR-003
## 2026-07-17 (C08 eval corpus soft — FR-003)
- Status: DONE — eval-corpus.md expansion doc; L72 stays 2
- FR: FR-003
## 2026-07-17 (C04 ruleset checklist — FR-004)
- Status: DONE — docs/ops/ruleset-checklist.md; L34/L36 org-gated steps; L34 stays 2
- FR: FR-004


## 2026-07-17 (C01 gitleaks polish — FR-004) DONE.
- Status: DONE — docs/ops/gitleaks.md; L18/L19 evidence; L19 stays 3
- FR: FR-004
## 2026-07-17 (C05 soak/chaos soft — FR-003)
- Status: DONE — soak-chaos.md plan; L47 stays 1
- FR: FR-003
## 2026-07-17 (C06 GHCR publish soft — FR-003)
- Status: DONE — ghcr-publish.md; L58 1→2
- FR: FR-003
## 2026-07-17 (C02 spawn audit soft — FR-004)
- Status: DONE — spawn-audit.md; L28 stays 2
- FR: FR-004

## 2026-07-17 (C02 spawn audit JSONL code — FR-004)
- Status: DONE — `ProcessPool` spawn/stop rows via `SHARECLI_AUDIT_LOG`; `tests/spawn_audit.rs`
- FR: FR-004

## 2026-07-17 (C09 SR checklist soft — FR-003)
- Status: DONE — docs/a11y/sr-checklist.md; L81.11 stays 2
- FR: FR-003

## 2026-07-14 (C07 config proptest soft — FR-003)
- Status: DONE — config-proptest.md plan; L66 stays 2
- FR: FR-003
## 2026-07-14 (C01 secrets soft — FR-004)
- Status: DONE — docs/ops/secrets.md; L18 stays 2
- FR: FR-004
## 2026-07-14 (C05 multi-hop soft — FR-003)
- Status: DONE — trace-multihop.md; L44 stays 2
- FR: FR-003
## 2026-07-14 (C08 live pool soft — FR-003)
- Status: DONE — live-pool-soft.yml + live_pool_probe.sh
- FR: FR-003

## 2026-07-14 (C02 crypto/privacy soft — FR-004)
- Status: DONE — crypto-keys.md + privacy-tenant.md; L22/L24 1→2
- Score: C02 80%→87% B
- FR: FR-004

## 2026-07-14 (C11 systemd/Caddy soft samples — FR-004)

- Status: DONE — docs/deploy/systemd/sharecli.service + Caddyfile.sample; L115 1→2
- Score: C11 76%→78% B; overall ~79% / weighted ~79% B
- FR: FR-004 packaging / traditional server sample
- Next: hard codesign secrets; dmg/msi; Win tray harden

## 2026-07-14 (C04 GPG/SSH verified commits soft — FR-004)

- Status: DONE — gpg-soft.yml verified-commit check; signed-commits.md GPG/SSH + ruleset checklist; L34 1→2
- Score: C04 77%→80% B; overall ~79% / weighted ~79% B
- FR: FR-004 NFR (commit integrity)
- Next: enable “Require signed commits” ruleset when bots/maintainers ready

## 2026-07-14 (C07 mutants soft threshold — FR-003)
- Status: DONE — mutants soft fail-on-survivors; L65 stays 2
- FR: FR-003
## 2026-07-14 (C00 alloc profiling + soft RSS — FR-003)
- Status: DONE — alloc-profiling.md; rss-soft.yml; L8 stays 2
- FR: FR-003
## 2026-07-14 (C11 auto-update soft — FR-004)

- Status: DONE — docs/ops/auto-update.md channel matrix; deploy.md link; L111 0→1
- Score: C11 73%→76% B; overall ~78% / weighted ~79% B
- FR: FR-004 packaging / update channels
- Next: in-binary updater after L112; dmg/msi; harden Win tray CI
## 2026-07-14 (C05 soft load burst — FR-003)

- Status: DONE — load-soft.yml starts serve + healthz_burst; just load-soft; L50 1→2
- Score: C05 70%→73% C; overall ~78% / weighted ~79% B
- FR: FR-003 observability / load harness
- Next: soak/chaos hard gate; multi-hop traces; live PD

## 2026-07-14 (MVP finality + OS parity refresh — FR-004)

- Status: DONE — FINALITY host×capability matrix; README four install blocks; release.yml tray-macos + tray-windows attach; just build-cli-windows; C11 L108/L110 evidence
- Score: packaging parity (C11); no GA claim for tray/desktop; Win tray remains soft `continue-on-error`
- FR: FR-004 NFR (deploy/parity)
- Next: harden Win tray CI; L112 signing when secrets land; native dmg/msi

## 2026-07-14 (C01 a11y checklist + SBOM sync — FR-003)

- Status: DONE — docs/a11y/cli-tui-checklist.md; L17/L19 evidence sync; mutants-threshold.md (soft)
- Score: C01 70%→80% B; overall ~78% / weighted ~79% B
- FR: FR-003 a11y / supply-chain evidence
- Next: mutants hard required check; C00 70% lifts; hard codesign secrets

## 2026-07-14 (C06 hermetic soft + C10 light theme — FR-003/FR-004)

- Status: DONE — hermetic-soft.yml + just hermetic; Backbone2Light + tokens.css light; docs/visual/theming.md
- Score: C06 77%→80% B; C10 83%→86% B; overall ~77% B
- FR: FR-004 hermetic builds + FR-003 visual theming
- Next: mutants hard gate; SLSA L3 network-block; hard codesign secrets

## 2026-07-14 (C11 codesign soft + C06 MCP ADR — FR-004)

- Status: DONE — docs/ops/codesign-notarize.md; codesign-soft.yml; ADR-0004 no MCP; deploy.md N/A seed
- Score: C11 67%→73% C; C06 73%→77% B; overall ~77% B
- FR: FR-004 NFR (packaging / supply-chain scope)
- Next: hard signing secrets; hermetic builds; mutants hard gate

## 2026-07-14 (C08 thermal/gate corpus + ADR seed + C06 L59 — FR-003)

- Status: DONE — corpus_thermal_gate_fixtures_match_gate_decision; L75–L78 seeded via ADR-0002; C06 L59 DCO evidence
- Score: C08 60%→73% C; C06 70%→73% C; overall ~76% B
- FR: FR-003 eval corpus / thermal gate assertions
- Next: C11 signing docs; mutants hard gate; hermetic builds

## 2026-07-14 (C04 2FA + C10 motion/type — FR-004 / FR-003)

- Status: DONE — maintainer-2fa.md; tokens.css type+motion; dashboard prefers-reduced-motion; docs/visual/motion.md
- Score: C04 73%→77% B (L36 0→1); C10 78%→83% B (L97/L102→3); overall ~74% / weighted ~75% B
- FR: FR-004 (authN policy) + FR-003 (visual/a11y motion)
- Next: org 2FA enforce; golden visual tests; C11 signing; C08 live corpus

## 2026-07-14 (C10 docs/visual soft — FR-003)

- Status: DONE — docs/visual/{README,IDENTITY,VISUAL_SPEC,PROVENANCE,typography}.md
- Score: C10 24/36 → 28/36 (78% B); L97/L105/L106/L107 lifts; overall ~74% / weighted ~75%
- FR: FR-003 (visual acceptance / agent-detectable polish)
- Next: tests/golden visual gates; light theme; wire type tokens into tokens.css

## 2026-07-14 (C04 DCO signed-commits soft — FR-004)

- Status: DONE — CONTRIBUTING DCO; docs/ops/signed-commits.md; dco-soft.yml (continue-on-error)
- Score: C04 L34 0→1; C04 21/30 → 22/30 (73% C); overall ~73% / weighted ~74% C
- FR: FR-004 NFR (supply-chain / commit integrity)
- Next: GPG+branch protection require signed commits; 2FA evidence (L36)

## 2026-07-14 (C05 Pyroscope soft — FR-003)

- Status: DONE — L45 external Pyroscope push recipe (`docs/ops/pyroscope.md`, `just pyro-push-sample`, `.env.example` SHARECLI_PYROSCOPE_*)
- Status: DONE — `serve` logs hint when `SHARECLI_PYROSCOPE_URL` set (no in-process agent)
- Re-score: C05 L45 stays 2/3 (70% C); soft goal met for push docs without Grafana Cloud secrets
- Next READY: in-process always-on agent (effort L); multi-hop traces; live PD secrets

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

## W10.12 — C05 Pyroscope soft push path (FR-003)
- docs/ops/pyroscope.md + just pyro-push-sample; links from profiling.md.

## W10.13 — C01 i18n ADR + C08 corpus harness (FR-003)
- ADR 0003 English-primary (L16 seeded); soft `just eval-corpus` + eval-corpus-soft.yml.

## W10.14 — C00 concurrency/memory soft (FR-001)
- docs/ops/concurrency.md + memory.md; soft miri-soft.yml; C00 → 70% C.

## W10.15 — C08 live corpus health assertions (FR-003)
- serve.rs corpus_health_fixtures_match_healthz + optional SHARECLI_CORPUS_LIVE in run-corpus.sh.

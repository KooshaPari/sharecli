# audit-v38 Scorecard — sharecli

**Repo:** KooshaPari/sharecli
**Date:** 2026-07-10
**Repo-type profile:** CLI+daemon
**Auditor:** cursor-agent cluster-fleet (C00–C11); Wave1 re-score C03/C07/C08/C11
**Commit audited:** 8aa2df12441c0aa14b9407615e6e363ddbc1d0c2

> Scoring: each sub-pillar 0=absent / 1=seeded / 2=partial / 3=complete, evidence-mandatory (`file:line`).
> Cluster score = sum / (sub-pillars × 3). Grade: A≥90% · B≥75% · C≥60% · D≥40% · F<40%.
> Lane evidence: `audit/.lane-c00` … `audit/.lane-c11`. Rubric pin: `audit/rubric/` (from phenotype-org-audits audit-v38).

## Category Scores

| Cluster | Category | Pillars | Score (sum/max) | Pct | Grade | Top-3 gaps |
|---------|----------|---------|:---------------:|:---:|:-----:|------------|
| C00 | Architecture + Module | L0–L9 | 15/30 | 50% | D | benches; OpenAPI for serve; lib.rs sprawl |
| C01 | CI, DX, Observability | L10–L19 | 19/30 | 63% | C | echo coverage; i18n; action SHA pins |
| C02 | Error handling, API, Governance | L20–L29 | 15/30 | 50% | D | serve AuthN; audit log; SLOs |
| C03 | Agent Readiness | L30 | 30/36 | 83% | B | FR-002..005 tests; journey e2e; golden fixtures |
| C04 | Security | L31–L40 | 14/30 | 47% | D | SBOM publish; residual auth/threat gaps |
| C05 | Observability (deep) | L41–L50 | 12/30 | 40% | D | OTel/traces; RED/USE; ops dashboards |
| C06 | Supply Chain | L51–L60 | 17/30 | 57% | D | provenance upgrade; lock/deny gaps |
| C07 | DX, QEng, Portability | L61–L70 | 18/30 | 60% | C | proptest; CI OS matrix; mutants CI gate |
| C08 | Eval Coverage | L71–L80 | 12/30 | 40% | D | hard perf gate; baselines; Harbor N/A by ADR |
| C09 | Accessibility + UX | L81–L95 | 26/45 | 58% | D | degraded-mode AX; recovery docs |
| C10 | Visual Identity | L96–L107 | 24/36 | 67% | C | visual docs; light theme; type scale |
| C11 | Packaging + Distribution | L108–L122 | 27/45 | 60% | C | Homebrew sha; signing; native installers |

## Overall

**Weighted overall score:** 56% · **Overall grade:** D

(Unweighted mean of cluster pcts: (50+63+50+83+47+40+57+60+40+58+67+60)/12 = 675/12 = **56.25%**.)

**Tier-1 double-weight (C00–C03):** (50+63+50+83)×2 + (47+40+57+60+40+58+67+60) = 492 + 429 = 921 / (4×2 + 8) = 921/16 ≈ **57.6%** (still D).

## Headline Findings

- **Strongest:** C03 Agent Readiness (83% B) — FR-NNN + WORK_DAG + llms.txt + pr-lint after Wave1.
- **Off F:** C07 (33%→60% C) via devcontainer/nextest flake policy/fuzz seed; C08 (0%→40% D) via Criterion+REPRO+eval ADR.
- **Toward C:** C11 (42%→60% C) via Containerfile harden, deploy matrix, no-mobile ADR, uninstall docs.
- **Highest-leverage remaining:** hard perf gate (C08), proptest + CI OS matrix (C07), Homebrew digest + signing (C11).
- **Agent-readiness verdict (C03):** Agents can claim WORK_DAG tasks with FR refs; FR-002..005 acceptance suites still open.
- **Time-2 verdict (C11):** OCI+uninstall+mobile decision solid; brew PLACEHOLDER and unsigned archives remain.

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

## Spine links

- Rubric: [phenotype-org-audits/audit-v38](https://github.com/KooshaPari/phenotype-org-audits/tree/main/audit-v38)
- Spine index: [docs/SPINE-INDEX.md](https://github.com/KooshaPari/phenotype-org-audits/blob/main/docs/SPINE-INDEX.md)
- CI truth notes: `audit/CI_TRUTH_FINDINGS.md`
- Boundary: `audit/BOUNDARY_VERIFY_2026-07-10.md`

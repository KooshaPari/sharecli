# audit-v38 Scorecard — sharecli

**Repo:** KooshaPari/sharecli
**Date:** 2026-07-09
**Repo-type profile:** CLI+daemon
**Auditor:** cursor-agent cluster-fleet (C00–C11)
**Commit audited:** 31cf3166ce5744d3e08c91f2ed434d3a34ebff03 (+ same-branch remediations re-scored)

> Scoring: each sub-pillar 0=absent / 1=seeded / 2=partial / 3=complete, evidence-mandatory (`file:line`).
> Cluster score = sum / (sub-pillars × 3). Grade: A≥90% · B≥75% · C≥60% · D≥40% · F<40%.
> Lane evidence: `audit/.lane-c00` … `audit/.lane-c11`. Rubric pin: `audit/rubric/` (from phenotype-org-audits audit-v38).

## Category Scores

| Cluster | Category | Pillars | Score (sum/max) | Pct | Grade | Top-3 gaps |
|---------|----------|---------|:---------------:|:---:|:-----:|------------|
| C00 | Architecture + Module | L0–L9 | 15/30 | 50% | D | benches; OpenAPI for serve; lib.rs sprawl |
| C01 | CI, DX, Observability | L10–L19 | 19/30 | 63% | C | echo coverage; i18n; action SHA pins |
| C02 | Error handling, API, Governance | L20–L29 | 15/30 | 50% | D | serve AuthN; audit log; SLOs |
| C03 | Agent Readiness | L30 | 22/36 | 61% | D | FR-NNN grammar; WORK_DAG; llms.txt |
| C04 | Security | L31–L40 | 14/30 | 47% | D | SBOM publish; residual auth/threat gaps |
| C05 | Observability (deep) | L41–L50 | 12/30 | 40% | D | OTel/traces; RED/USE; ops dashboards |
| C06 | Supply Chain | L51–L60 | 17/30 | 57% | D | provenance upgrade; lock/deny gaps |
| C07 | DX, QEng, Portability | L61–L70 | 10/30 | 33% | F | Windows Zig path; QEng depth |
| C08 | Eval Coverage | L71–L80 | 0/30 | 0% | F | no formal eval harness (re-check if over-strict) |
| C09 | Accessibility + UX | L81–L95 | 26/45 | 58% | D | degraded-mode AX; recovery docs |
| C10 | Visual Identity | L96–L107 | 24/36 | 67% | C | visual docs; light theme; type scale |
| C11 | Packaging + Distribution | L108–L122 | 19/45 | 42% | D | Homebrew sha; tray parity; installers |

## Overall

**Weighted overall score:** 47% · **Overall grade:** D

(Unweighted mean of cluster pcts: (50+63+50+61+47+40+57+33+0+58+67+42)/12 ≈ 47.3%.)

**Tier-1 double-weight (C00–C03):** (50+63+50+61)×2 + (47+40+57+33+0+58+67+42) = 448 + 344 = 792 / (4×2 + 8) = 792/16 ≈ **49.5%** (still D).

## Headline Findings

- **Strongest:** C10 Visual Identity (67% C) — `assets/tokens.css` SoT + iconset, demo media, theme.rs, README hero present.
- **Weakest:** C08 Eval Coverage (0% F); next C07 DX/QEng/Portability (33% F).
- **Highest-leverage fix:** Formal eval harness (C08), then Windows Zig/QEng (C07); C05 lifted off F via `/readyz`, draft SLOs, and serve tracing/tests.
- **Agent-readiness verdict (C03):** Partial — AGENTS/CLAUDE/just/CI exist, but FR grammar and claimable WORK_DAG are incomplete; agent can navigate but not autonomously close FRs.
- **Time-2 verdict (C11):** Partial — crates.io/binstall/releases exist (v0.3.0); Homebrew formula stub; tray surfaces incomplete across OS.

## Supersedes

Root `audit_scorecard.json` (Python 30-pillar auto-scan, overall 48, “No source files found”) is **superseded** by this v38 card. Do not use it for fleet ranking.

## Post-audit remediations (re-scored 2026-07-09)

- `ci-success` now fails when any required job fails (false-green closed).
- `coverage.yml` replaced echo stub with llvm-cov + test-count > 0 guard.
- quality-gate coverage threshold no longer `continue-on-error`.
- Build blockers: unused `PI` allowed with note; unread blake2 `last` fields removed.
- **C05 re-scored 9/30 (30% F) → 12/30 (40% D):** `/readyz` + health JSON unit tests; `docs/ops/SLO.md` draft SLOs; `#[instrument]` / lifecycle `tracing` on serve handlers; existing Prometheus unit tests counted.
- **C10 re-scored 23/36 (64% C) → 24/36 (67% C):** `assets/tokens.css` present; L96 2→3.

## Spine links

- Rubric: [phenotype-org-audits/audit-v38](https://github.com/KooshaPari/phenotype-org-audits/tree/main/audit-v38)
- Spine index: [docs/SPINE-INDEX.md](https://github.com/KooshaPari/phenotype-org-audits/blob/main/docs/SPINE-INDEX.md)
- CI truth notes: `audit/CI_TRUTH_FINDINGS.md`

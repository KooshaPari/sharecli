# audit-v38 Scorecard — sharecli

**Repo:** KooshaPari/sharecli
**Date:** 2026-07-08
**Repo-type profile:** CLI+daemon
**Auditor:** cursor-agent cluster-fleet (C00–C11)
**Commit audited:** 31cf3166ce5744d3e08c91f2ed434d3a34ebff03

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
| C05 | Observability (deep) | L41–L50 | 9/30 | 30% | F | OTel/traces; /readyz; SLOs; RED/USE |
| C06 | Supply Chain | L51–L60 | 17/30 | 57% | D | provenance upgrade; lock/deny gaps |
| C07 | DX, QEng, Portability | L61–L70 | 10/30 | 33% | F | Windows Zig path; QEng depth |
| C08 | Eval Coverage | L71–L80 | 0/30 | 0% | F | no formal eval harness (re-check if over-strict) |
| C09 | Accessibility + UX | L81–L95 | 26/45 | 58% | D | degraded-mode AX; recovery docs |
| C10 | Visual Identity | L96–L107 | 23/36 | 64% | C | tokens.css file; visual docs; light theme |
| C11 | Packaging + Distribution | L108–L122 | 19/45 | 42% | D | Homebrew sha; tray parity; installers |

## Overall

**Weighted overall score:** 46% · **Overall grade:** D

(Unweighted mean of cluster pcts: (50+63+50+61+47+30+57+33+0+58+64+42)/12 ≈ 46.3%.)

**Tier-1 double-weight (C00–C03):** (50+63+50+61)×2 + (47+30+57+33+0+58+64+42) = 448 + 331 = 779 / (4×2 + 8) = 779/16 ≈ **48.7%** (still D).

## Headline Findings

- **Strongest:** C10 Visual Identity (64% C) — iconset, demo media, theme.rs, README hero present; CYCLE_CLOSE T1–7 only partially true (tokens.css still missing at audit time).
- **Weakest:** C08 Eval Coverage (0% F) and C05 Observability deep (30% F).
- **Highest-leverage fix:** CI truth (false-green `CI Success` + echo `coverage.yml`) then lift C05 (`/readyz`, traces) and ship `assets/tokens.css`.
- **Agent-readiness verdict (C03):** Partial — AGENTS/CLAUDE/just/CI exist, but FR grammar and claimable WORK_DAG are incomplete; agent can navigate but not autonomously close FRs.
- **Time-2 verdict (C11):** Partial — crates.io/binstall/releases exist (v0.3.0); Homebrew formula stub; tray surfaces incomplete across OS.

## Supersedes

Root `audit_scorecard.json` (Python 30-pillar auto-scan, overall 48, “No source files found”) is **superseded** by this v38 card. Do not use it for fleet ranking.

## Post-audit remediations (same branch, not yet re-scored)

- `ci-success` now fails when any required job fails (false-green closed).
- `coverage.yml` replaced echo stub with llvm-cov + test-count > 0 guard.
- quality-gate coverage threshold no longer `continue-on-error`.
- Build blockers: unused `PI` allowed with note; unread blake2 `last` fields removed.
- C05 lift: `GET /readyz` added beside `/healthz`.
- C10 lift: `assets/tokens.css` Backbone-2 tokens added (mirrors `src/theme.rs`).

Re-score C05/C10 after merge for updated pcts.

## Spine links

- Rubric: [phenotype-org-audits/audit-v38](https://github.com/KooshaPari/phenotype-org-audits/tree/main/audit-v38)
- Spine index: [docs/SPINE-INDEX.md](https://github.com/KooshaPari/phenotype-org-audits/blob/main/docs/SPINE-INDEX.md)
- CI truth notes: `audit/CI_TRUTH_FINDINGS.md`

# ADR 0005 — Agent-eval harness supersede pathway (soft plan)

**Status:** Accepted (supersede plan only — [ADR 0002](0002-eval-surface-out-of-scope.md) remains authoritative)  
**Date:** 2026-07-17  
**Deciders:** sharecli maintainers  
**Supersedes:** — (does not supersede ADR 0002 until Phase 4)  
**Traceability:** audit-v38 C08 L71, L80; `docs/eval/GOVERNANCE.md`

---

## Context

[ADR 0002](0002-eval-surface-out-of-scope.md) correctly scores Harbor / Terminal-Bench,
SWE-bench task corpora, cross-language parity, compression benches, and LLM token-burn
tracking as **N/A** for sharecli's process-supervisor profile. C08 is **22/30 (73% C)**
with L71–L74 and L79–L80 at score 3 and L75–L78 seeded at 1 via ADR 0002.

The scorecard Top-3 gap for C08 still lists **“supersede ADR if agent-eval lands.”**
Without a written supersede contract, auditors cannot tell whether missing Harbor wiring
is still N/A or a product gap. This ADR closes that governance gap while **keeping
ADR 0002 in force** until an agent-eval harness actually ships **and** sharecli claims
it via FR / product scope.

## Decision

1. **Today (Phase 0–1):** ADR 0002 remains the eval policy. No Harbor/SWE-bench
   corpora under `docs/eval/corpus/`; supervisor JSON fixtures only
   ([`docs/ops/eval-corpus.md`](../ops/eval-corpus.md)). sharecli hosts **no**
   Harbor workflows.
2. **Supersede trigger (any one):**
   - A merged **FR** explicitly requires agent-eval or SWE-bench-style task runs
     against sharecli.
   - sharecli **product scope** claims an agent-eval or coding-agent benchmark runner
     surface (README, `llms.txt`, or registry profile).
   - A **soft CI harness** in the dedicated Harbor home (benchora `harbor-soft` /
     `portage-temp`) runs ≥1 Harbor / Terminal-Bench task on `main` for seven
     consecutive days (soak) **and** an FR ties that soak to sharecli product scope.
3. **Supersede process (Phase 4 — when a trigger fires):**
   | Step | Action |
   |------|--------|
   | 1 | Set ADR 0002 **Status → Superseded**; link to this ADR |
   | 2 | Update [`docs/eval/GOVERNANCE.md`](../eval/GOVERNANCE.md) N/A table — flip
        affected L75–L78 rows from N/A to in-scope |
   | 3 | Keep Harbor / portage wiring in **benchora `harbor-soft` + `portage-temp`**
        (not the supervisor binary — per ADR 0002) |
   | 4 | Cross-link soft workflow + pheno-harness tasks from sharecli docs only;
        do not re-vendor Harbor into this repo |
   | 5 | Refresh `audit/.lane-c08/C08.md` evidence; re-score L71/L76/L80 in
        `audit/SCORECARD-v38.md` |
4. **Until Phase 4:** auditors score L75–L78 as **N/A / deferred** per ADR 0002, not
   as absent product gaps. Phase 2–3 soft artifacts are **EXTRACTED** (see below).

## Soft phases

| Phase | Deliverable | ADR 0002 in force? | Hard gate? |
|-------|-------------|--------------------|------------|
| **0 — today** | Supervisor corpus + Criterion + REPRO + ADR 0002 | **Yes** | bench-gate only |
| **1 — plan** | This ADR + scorecard/worklog (FR-003) | **Yes** | No |
| **2 — stub** | **EXTRACTED** → `phenotype-tooling/crates/benchora/harbor-soft` | **Yes** | No (`continue-on-error`) |
| **3 — soak** | **EXTRACTED** → benchora `harbor-soft` + Harbor env in `KooshaPari/portage-temp` (awaiting GH restore of `portage`) | **Yes** | No |
| **4 — supersede** | ADR 0002 marked superseded; GOVERNANCE + lane re-score; **requires FR / product claim** | **No** | Per new harness |

Phases 2–3 soft CI/soak no longer live in sharecli. Phase 4 still needs maintainer
sign-off after an explicit FR or product-scope claim.

## Phase 1 formalization (FR-003)

**Status:** DONE (2026-07-19) — governance-only; ADR 0002 remains authoritative.

| Artifact | Action |
|----------|--------|
| `audit/.lane-c08/C08.md` | L71/L76 EXTERNAL + GAP-QA cross-ref; L76 score **1** |
| `audit/SCORECARD-v38.md` | Reconcile v3/v4 L76 interim lifts superseded; EXTRACTED/N/A formalize entry |
| `docs/ops/governance/GAP-QA-MATRIX.md` | Harbor stub/soak rows → `Status: EXTRACTED / N/A (sharecli)` |
| `WORK_DAG.md` T-650 | Seven-day soak → tracked in benchora/`portage-temp`, not sharecli `main` |
| `docs/eval/GOVERNANCE.md` | L76 N/A + ADR 0005 EXTRACTED destination table |

**Auditor rule:** Missing in-repo Harbor paths are **not** product gaps. Score L76 as **1**
until Phase 4 supersede trigger fires with an explicit FR or product-scope claim.

A read-only **visibility mirror** (`docs/eval/harbor-7d.log`) mirrors the honest
`0/7` soak state from `benchora/harbor-soft` into sharecli docs for auditor
visibility only. It does not re-vendor the soak and does not count toward L76.

## Consequences

- C08 agent-eval lifts come only after Phase 4 and lane re-score (L76 target 1→2+);
  in-repo Harbor stubs are not required evidence.
- [`docs/ops/eval-corpus.md`](../ops/eval-corpus.md) rule stands: do not add
  agent-eval task corpora without completing the supersede process above.
- Phenotype org Harbor soft surface: **benchora `harbor-soft`**; Harbor fork/env:
  **`KooshaPari/portage-temp`**. sharecli docs point outward only.

## Audit evidence (C08 L71 / L80)

| Pillar | Evidence (today) | Score | After Phase 4 (planned) |
|--------|------------------|-------|-------------------------|
| **L71** Eval corpus | `docs/eval/corpus/` supervisor scenarios; ADR 0002 agent corpora N/A; [`eval-corpus.md`](../ops/eval-corpus.md); Harbor stub **external** (benchora) | **3** | Agent-task family added; SWE-bench row flips from N/A |
| **L76** Agent-eval pipeline | ADR 0002 Harbor/Terminal-Bench N/A; Phase 2–3 **EXTRACTED** to benchora/`portage-temp`; this ADR supersede trigger §Decision.2 | **1** (seeded) | **2+** when Harbor soft soak + FR claim |
| **L80** Eval governance | `docs/eval/GOVERNANCE.md`; ADR 0002 eval policy; **this ADR** supersede contract | **3** | GOVERNANCE N/A map updated post-supersede |

**Soft follow-up**

| Item | Status |
|------|--------|
| Agent-eval supersede pathway ADR | Done (this file) |
| Phase 2 Harbor stub soft CI | **EXTRACTED** → benchora `harbor-soft` |
| Phase 3 Harbor soak plan / soft CI | **EXTRACTED** → benchora `harbor-soft` + `portage-temp` |
| Seven-day Harbor soft soak | Tracked externally (not sharecli `main`) |
| Mark ADR 0002 superseded | Deferred (Phase 4 — needs FR claim) |

## References

- Superseded policy (current): [`0002-eval-surface-out-of-scope.md`](0002-eval-surface-out-of-scope.md)
- Lane evidence: `audit/.lane-c08/C08.md`
- Rubric: `audit/rubric/audit-30-pillar/audit-30-pillar-L71-L80-eval-coverage.md`
- Corpus ops: [`../ops/eval-corpus.md`](../ops/eval-corpus.md)
- Harbor soft CI / soak (Phase 2–3): `phenotype-tooling/crates/benchora/harbor-soft`
- Harbor fork/env: `KooshaPari/portage-temp` (awaiting GH restore of `portage`)
- Related org: pheno-harness (SWE-bench tasks) — cross-repo only

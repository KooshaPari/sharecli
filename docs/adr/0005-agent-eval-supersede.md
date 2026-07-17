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
ADR 0002 in force** until an agent-eval harness actually ships.

## Decision

1. **Today (Phase 0–1):** ADR 0002 remains the eval policy. No Harbor/SWE-bench
   corpora under `docs/eval/corpus/`; supervisor JSON fixtures only
   ([`docs/ops/eval-corpus.md`](../ops/eval-corpus.md)).
2. **Supersede trigger (any one):**
   - A merged **FR** explicitly requires agent-eval or SWE-bench-style task runs
     against sharecli.
   - sharecli **product scope** claims an agent-eval or coding-agent benchmark runner
     surface (README, `llms.txt`, or registry profile).
   - A **soft CI harness** under `scripts/eval/` or a sibling crate runs ≥1 Harbor /
     Terminal-Bench task on `main` for seven consecutive days (soak).
3. **Supersede process (Phase 4 — when a trigger fires):**
   | Step | Action |
   |------|--------|
   | 1 | Set ADR 0002 **Status → Superseded**; link to this ADR |
   | 2 | Update [`docs/eval/GOVERNANCE.md`](../eval/GOVERNANCE.md) N/A table — flip
        affected L75–L78 rows from N/A to in-scope |
   | 3 | Land Harbor / portage wiring in a **dedicated crate or repo** (not the
        supervisor binary — per ADR 0002 consequences) |
   | 4 | Add agent-task corpus paths + soft workflow (e.g. `agent-eval-soft.yml`);
        cross-link pheno-harness tasks if applicable |
   | 5 | Refresh `audit/.lane-c08/C08.md` evidence; re-score L71/L76/L80 in
        `audit/SCORECARD-v38.md` |
4. **Until Phase 4:** auditors score L75–L78 as **N/A / deferred** per ADR 0002, not
   as absent product gaps.

## Soft phases (no harness yet)

| Phase | Deliverable | ADR 0002 in force? | Hard gate? |
|-------|-------------|--------------------|------------|
| **0 — today** | Supervisor corpus + Criterion + REPRO + ADR 0002 | **Yes** | bench-gate only |
| **1 — plan** | This ADR + scorecard/worklog (FR-003) | **Yes** | No |
| **2 — stub** | [`docs/ops/harbor-eval-stub.md`](../ops/harbor-eval-stub.md) + `scripts/eval/harbor_stub.sh` | **Yes** | No (`continue-on-error`) |
| **3 — soak** | Soft workflow green on `main` 7 days; Harbor env pin documented | **Yes** | No |
| **4 — supersede** | ADR 0002 marked superseded; GOVERNANCE + lane re-score | **No** | Per new harness |

Phases 0–3 are **documentation + soft CI** only. Phase 4 needs maintainer sign-off
after phase-3 soak and an explicit FR or product-scope claim.

## Consequences

- C08 cluster score stays **22/30** through phases 0–3; lifts from agent-eval come
  only after Phase 4 and lane re-score (L76 target 1→2+).
- [`docs/ops/eval-corpus.md`](../ops/eval-corpus.md) rule stands: do not add
  agent-eval task corpora without completing the supersede process above.
- Phenotype org eval assets (portage, pheno-harness) remain **out of repo** until
  Phase 2+; cross-repo pins are documented, not vendored.

## Audit evidence (C08 L71 / L80)

| Pillar | Evidence (today) | Score | After Phase 4 (planned) |
|--------|------------------|-------|-------------------------|
| **L71** Eval corpus | `docs/eval/corpus/` supervisor scenarios; ADR 0002:26 agent corpora N/A; [`eval-corpus.md`](../ops/eval-corpus.md) | **3** | Agent-task family added; SWE-bench row flips from N/A |
| **L76** Agent-eval pipeline | ADR 0002:27 Harbor/Terminal-Bench N/A; this ADR supersede trigger §Decision.2 | **1** (seeded) | **2+** when Harbor soft workflow soaks |
| **L80** Eval governance | `docs/eval/GOVERNANCE.md`; ADR 0002:32 eval policy; **this ADR** supersede contract | **3** | GOVERNANCE N/A map updated post-supersede |

**Soft follow-up**

| Item | Status |
|------|--------|
| Agent-eval supersede pathway ADR | Done (this file) |
| [`docs/ops/harbor-eval-stub.md`](../ops/harbor-eval-stub.md) + `harbor_stub.sh` | Done (Phase 2 soft) |
| Seven-day Harbor soft soak on `main` | Open (Phase 3) |
| Mark ADR 0002 superseded | Deferred (Phase 4) |

## References

- Superseded policy (current): [`0002-eval-surface-out-of-scope.md`](0002-eval-surface-out-of-scope.md)
- Lane evidence: `audit/.lane-c08/C08.md`
- Rubric: `audit/rubric/audit-30-pillar/audit-30-pillar-L71-L80-eval-coverage.md`
- Corpus ops: [`../ops/eval-corpus.md`](../ops/eval-corpus.md)
- Harbor stub (Phase 2): [`../ops/harbor-eval-stub.md`](../ops/harbor-eval-stub.md)
- Related org repos: portage (Harbor), pheno-harness (SWE-bench tasks) — cross-repo only

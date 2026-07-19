# ADR 0001 — Eval surface scope for sharecli (Harbor / SWE-bench out of scope)

**Status:** Accepted  
**Date:** 2026-07-10  
**Deciders:** sharecli maintainers  
**Supersedes:** —  
**Traceability:** audit-v38 C08 (L71–L80), `docs/eval/REPRO.md`

---

## Context

audit-v38 cluster **C08 Eval Coverage** scores agent-eval frameworks (Harbor /
Terminal-Bench, SWE-bench / SWE-RL), cross-language parity harnesses, compression
benches, and LLM token-burn tracking alongside ordinary Criterion / load benches.

sharecli is an **OS-adjacent agent runtime** (detect, watch, coalesce, mesh)
with a supervisor/serve operator surface. It is not an agent-eval product, a
coding-agent benchmark runner, or an LLM gateway. Treating missing
Harbor/SWE-bench wiring as a product defect mis-scores the repo against the
wrong profile.

## Decision

1. **In scope (process-supervisor eval):** Criterion microbenches under
   `benches/`, load scripts under `scripts/load/`, reproducibility contract in
   `docs/eval/REPRO.md`, soft CI in `.github/workflows/bench.yml`, and SLO rows
   in `docs/ops/SLO.md` that link those benches.
2. **Out of scope (explicit N/A until product scope expands):**
   - Harbor / portage / Terminal-Bench agent-eval pipelines (**L76**)
   - SWE-bench / SWE-RL task corpora (**L71** agent-task families)
   - Cross-language (Py/Go/TS) eval parity suites (**L75**)
   - Compression / spec-extraction benchmark suites (**L77**)
   - Per-eval LLM cost / token-burn tracking (**L78**)
3. **Harbor soft surface ownership (external):** Harbor soft CI and soak live in
   `phenotype-tooling/crates/benchora/harbor-soft`. Harbor fork/env live in
   `KooshaPari/portage-temp` (awaiting GitHub restore of `portage`). **sharecli
   does not host Harbor workflows**, stub scripts, or soak logs.
4. **Governance:** This ADR is the eval policy for sharecli (**L80**). Revisit
   only if sharecli claims an agent-eval or multi-lang harness product surface.

## Consequences

- Auditors should score L75–L78 as **N/A / deferred** (or seeded-only via this
  ADR), not as absent product gaps.
- C08 lifts come from Criterion + load + REPRO + soft bench workflow + this ADR,
  not from adopting Harbor in-repo.
- Missing Harbor soft CI in sharecli is **correct** — look at benchora
  `harbor-soft` and `portage-temp`, not this repo.
- If a future FR requires agent-eval as a sharecli product claim, supersede this
  ADR (see [ADR 0005](0005-agent-eval-supersede.md) Phase 4); keep Harbor wiring
  in the dedicated crate/repo, not the supervisor binary.

## References

- Rubric: `audit/rubric/audit-30-pillar/audit-30-pillar-L71-L80-eval-coverage.md`
- Lane baseline: `audit/.lane-c08/C08.md` (pre-remediation 0/30 F)
- Repro contract: [`../eval/REPRO.md`](../eval/REPRO.md)

# ADR 0006 — Feb harness recovery lineage (A+ path)

**Status:** Accepted  
**Date:** 2026-07-19  
**Deciders:** sharecli maintainers  
**Supersedes:** —  
**Related:** [ADR 0002](0002-eval-surface-out-of-scope.md) (Harbor / agent-eval surface remains out of scope)

---

## Context

sharecli's product lineage is discontinuous across three eras. Without a written
recovery map, agents treat the current tray/serve UX as the Feb harness and
either rewrite mesh from scratch or chase Harbor soft CI that [ADR 0002](0002-eval-surface-out-of-scope.md)
explicitly excludes.

Historical spine:

| Era | What landed | Ownership |
|-----|-------------|-----------|
| **Feb** | `agent-harness` absorbed into thegent as mesh substrate | thegent (`src/thegent/mesh/`) |
| **Mar 25** | Greenfield Rust `sharecli` + `thegent-sharecli`; orphan root is a **new DAG**, not a wipe of this remote | New sharecli remote / DAG |
| **Jun 21** | Boundary flip — **canonical sharecli**; mesh/harness move out of thegent | This repo (canonical) |

Recovery sources still exist off the current main tip:

1. `~/Downloads/files/` — archived Feb/Mar artifacts
2. `thegent/src/thegent/mesh/` — mesh code absorbed in Feb
3. Live sharecli crates — current supervisor/serve/tray surface

Current UX is **tray + serve**, not the Feb TUI. The A+ recovery path is finishing
**strategy + FUSE + mesh** recovery into this repo — not restoring Harbor eval
pipelines (see ADR 0002) and not re-homing mesh under thegent.

## Decision

1. **Canonical home:** sharecli (this repo) owns supervisor, serve, tray, FUSE,
   strategy, and mesh recovery targets after the Jun 21 boundary flip.
2. **Lineage is recovery, not rewrite:** Prefer extracting and porting from the
   ranked recovery sources in [`docs/ops/feb-recovery.md`](../ops/feb-recovery.md)
   over greenfield reimplementation of Feb harness behavior.
3. **Mar 25 orphan root:** Treat as a new DAG for history/audit; do not interpret
   it as deletion of Feb mesh content that still lives in thegent / Downloads.
4. **UX boundary:** Tray/serve is the current operator surface. Feb TUI is
   historical reference only unless an FR reclaims it.
5. **A+ completion criteria:** Strategy crate paths, FUSE intercept, and mesh
   substrate recovered/integrated sufficiently to close Feb→sharecli lineage —
   documented against this ADR.
6. **Harbor remains out of scope:** [ADR 0002](0002-eval-surface-out-of-scope.md)
   stays authoritative for Harbor / Terminal-Bench / SWE-bench. This ADR does
   **not** reopen agent-eval product scope.

## Consequences

- Auditors and implementers follow [`docs/ops/feb-recovery.md`](../ops/feb-recovery.md)
  for ranked source paths before inventing new mesh/FUSE designs.
- thegent mesh paths are **recovery donors**, not the long-term canonical home.
- Missing Harbor soft CI in sharecli remains correct (ADR 0002 / ADR 0005).
- Product claims stay supervisor/OS-adjacent runtime; Feb harness recovery does
  not imply agent-eval product scope.

## References

- Eval out of scope: [`0002-eval-surface-out-of-scope.md`](0002-eval-surface-out-of-scope.md)
- Agent-eval supersede plan (still deferred): [`0005-agent-eval-supersede.md`](0005-agent-eval-supersede.md)
- Ops recovery pointer: [`../ops/feb-recovery.md`](../ops/feb-recovery.md)
- Recovery donors: `~/Downloads/files/`, `thegent/src/thegent/mesh/`, live sharecli crates

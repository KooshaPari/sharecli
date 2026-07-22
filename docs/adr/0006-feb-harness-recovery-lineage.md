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
| **Feb** | `agent-harness` absorbed into thegent as mesh substrate; **FUSE was part of the Feb product** (`agent-harness fuse` CLI / `harness-fuse` binary, intercept + coalesce behavior) | thegent (`src/thegent/mesh/`) + Feb harness artifacts |
| **Mar 25** | Greenfield Rust `sharecli` + `thegent-sharecli` on GitHub; orphan root is a **new DAG for this remote**, not a wipe of Feb mesh/FUSE content that still lives in thegent / Downloads | New sharecli remote / DAG |
| **Jun 21** | Boundary flip — **canonical ownership moves to sharecli**; mesh/harness/FUSE recovery targets land here, not back under thegent | This repo (canonical) |

**February included FUSE.** Do not treat FUSE as post-Feb scope. The Feb harness
shipped `harness-fuse` and the `agent-harness fuse` subcommand alongside mesh
substrate; recovery must port that behavior into `sharecli-fuse` / `sharecli fuse`,
not reinvent it as a greenfield feature.

Recovery sources still exist off the current main tip. Prefer them in rank order
(see [`docs/ops/feb-recovery.md`](../ops/feb-recovery.md)):

| Rank | Source | Use for |
|------|--------|---------|
| **1** | Live sharecli crates (this repo) | Current tray/serve UX, supervisor surface, already-ported strategy/FUSE/mesh |
| **2** | `thegent/src/thegent/mesh/` | Feb mesh substrate absorbed into thegent (donor after Jun 21 flip) |
| **3** | `recovery/feb-2026-agent-harness/` (in-repo vault) | Durable copy of Feb Downloads dump — **`harness-fuse.elf`**, install tar (no `.git` / no `fuse/` sources), `agents.conf`, `rules.conf`, plans. Prefer this over `~/Downloads/files/`. |
| **3b** | `KooshaPari/thegent-sharecli` (restored) | Mar 25 sharecli twin → Python absorb stub; **not** Feb FUSE donor |

Current UX is **tray + serve**, **not** the Feb harness TUI. The Feb operator
surface was a harness **TUI dashboard with metrics** (thermal/coalesce/mesh meters).
Tray/serve is the accepted current deploy UX; optional later port of harness
dashboard metrics does not block lineage closure.

The A+ recovery path is finishing **strategy + FUSE + mesh** recovery into this
repo — not restoring Harbor eval pipelines (see ADR 0002) and not re-homing mesh
under thegent.

## Decision

1. **Canonical home:** sharecli (this repo) owns supervisor, serve, tray, FUSE,
   strategy, and mesh recovery targets after the **Jun 21 boundary flip** (canonical
   ownership → sharecli).
2. **Lineage is recovery, not rewrite:** Prefer extracting and porting from the
   ranked recovery sources in [`docs/ops/feb-recovery.md`](../ops/feb-recovery.md)
   over greenfield reimplementation of Feb harness behavior — including Feb FUSE
   (`harness-fuse`, `core.sh`, `rules.conf` in Downloads).
3. **Mar 25 orphan root:** Treat as a **new DAG for GitHub sharecli** for
   history/audit; do **not** interpret it as deletion of Feb mesh/FUSE content that
   still lives in thegent / Downloads.
4. **UX boundary:** Tray/serve is the current operator surface. Feb harness TUI
   (dashboard metrics) is historical reference only unless an FR reclaims it.
5. **A+ completion criteria:** Strategy crate paths, FUSE intercept, and mesh
   substrate recovered/integrated sufficiently to close Feb→sharecli lineage —
   documented against this ADR and [`docs/ops/feb-recovery.md`](../ops/feb-recovery.md).
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
- Agents must not claim "FUSE wasn't February" — Feb shipped FUSE; sharecli recovery
  completes the port.

## References

- Eval out of scope: [`0002-eval-surface-out-of-scope.md`](0002-eval-surface-out-of-scope.md)
- Agent-eval supersede plan (still deferred): [`0005-agent-eval-supersede.md`](0005-agent-eval-supersede.md)
- Ops recovery pointer: [`../ops/feb-recovery.md`](../ops/feb-recovery.md)
- L34 operator guide: [`../ops/gpg-verified-commits-l34.md`](../ops/gpg-verified-commits-l34.md)
- Recovery donors: live sharecli crates → `thegent/src/thegent/mesh/` → `recovery/feb-2026-agent-harness/` (Downloads vault; `agent-harness` remote unrestorable past 90d)

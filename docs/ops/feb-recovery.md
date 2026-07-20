# Feb harness recovery (A+ path)

Pointer for agents recovering Feb→sharecli lineage. Policy: [ADR 0006](../adr/0006-feb-harness-recovery-lineage.md). Harbor stays out of scope ([ADR 0002](../adr/0002-eval-surface-out-of-scope.md)).

## Ranked recovery paths

Prefer higher ranks before inventing new mesh/FUSE/strategy code:

| Rank | Source | Use for |
|------|--------|---------|
| **1** | Live sharecli crates (this repo) | Current tray/serve UX, supervisor surface, any already-ported strategy/FUSE/mesh |
| **2** | `thegent/src/thegent/mesh/` | Feb mesh substrate absorbed into thegent (donor after Jun 21 boundary flip) |
| **3** | `~/Downloads/files/` | Archived Feb/Mar harness and related artifacts |

## Notes

- **Mar 25** greenfield orphan root is a new DAG — not a wipe of recovery donors above.
- **Jun 21** made sharecli canonical; finish strategy + FUSE + mesh recovery here.
- Tray/serve is current UX; Feb TUI is historical reference only.
